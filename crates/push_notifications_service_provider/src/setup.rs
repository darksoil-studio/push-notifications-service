use std::{collections::BTreeMap, path::PathBuf};

use anyhow::anyhow;
use holochain::prelude::{
    CloneCellId, CreateCloneCellPayload, DisableCloneCellPayload, DnaModifiersOpt,
    EnableCloneCellPayload, NetworkSeed, RoleSettings, RoleSettingsMap, YamlProperties,
};
use holochain_client::{AgentPubKey, CellInfo, ClonedCell, ExternIO, ZomeCallTarget};
use holochain_runtime::HolochainRuntime;
use push_notifications_types::{PublishServiceAccountKeyInput, ServiceAccountKey};
use roles_types::Properties;

use crate::{read_from_file, SERVICE_PROVIDERS_ROLE_NAME};

pub async fn setup(
    runtime: &HolochainRuntime,
    app_id: &String,
    push_notifications_service_provider_happ_path: &PathBuf,
    progenitors: Vec<AgentPubKey>,
    static_service_account_keys: BTreeMap<String, fcm_v1::auth::ServiceAccountKey>,
    static_network_seeds: Vec<NetworkSeed>,
) -> anyhow::Result<()> {
    let admin_ws = runtime.admin_websocket().await?;
    let installed_apps = admin_ws
        .list_apps(None)
        .await
        .map_err(|err| anyhow!("{err:?}"))?;
    let happ_bundle = read_from_file(push_notifications_service_provider_happ_path).await?;
    let roles_properties = Properties {
        progenitors: progenitors.into_iter().map(|p| p.into()).collect(),
    };
    let value = serde_yaml::to_value(roles_properties).unwrap();
    let properties_bytes = YamlProperties::new(value);

    if installed_apps
        .iter()
        .find(|app| app.installed_app_id.eq(app_id))
        .is_none()
    {
        let mut roles_settings = RoleSettingsMap::new();
        roles_settings.insert(
            String::from("push_notifications_service"),
            RoleSettings::Provisioned {
                membrane_proof: None,
                modifiers: Some(DnaModifiersOpt {
                    properties: Some(properties_bytes.clone()),
                    ..Default::default()
                }),
            },
        );
        roles_settings.insert(
            String::from("service_providers"),
            RoleSettings::Provisioned {
                membrane_proof: None,
                modifiers: Some(DnaModifiersOpt {
                    properties: Some(properties_bytes.clone()),
                    network_seed: Some("throwaway".into()),
                }),
            },
        );

        let app_info = runtime
            .install_app(
                app_id.clone(),
                happ_bundle,
                Some(roles_settings),
                None,
                None,
            )
            .await?;
        let app_ws = runtime
            .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
            .await?;

        app_ws
            .call_zome(
                ZomeCallTarget::RoleName("push_notifications_service".into()),
                "clone_manager".into(),
                "init".into(),
                ExternIO::encode(())?,
            )
            .await?;

        log::info!("Installed app {app_info:?}");
    }

    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;
    let Some(app_info) = app_ws.app_info().await? else {
        return Err(anyhow!("App Info returned None"));
    };

    let enabled_service_providers = app_info
        .cell_info
        .get("service_providers")
        .cloned()
        .unwrap_or(vec![]);

    let cloned_cells: Vec<ClonedCell> = enabled_service_providers
        .into_iter()
        .filter_map(|c| match c {
            CellInfo::Cloned(c) => Some(c.clone()),
            _ => None,
        })
        .collect();

    for supported_network_seed in static_network_seeds.clone() {
        let cloned_cell = cloned_cells
            .iter()
            .find(|cell| cell.dna_modifiers.network_seed.eq(&supported_network_seed));
        if let Some(cloned_cell) = cloned_cell {
            if !cloned_cell.enabled {
                app_ws
                    .enable_clone_cell(EnableCloneCellPayload {
                        clone_cell_id: CloneCellId::CloneId(cloned_cell.clone_id.clone()),
                    })
                    .await?;
            }
        } else {
            let modifiers = DnaModifiersOpt {
                properties: Some(properties_bytes.clone()),
                network_seed: Some(supported_network_seed),
            };

            app_ws
                .create_clone_cell(CreateCloneCellPayload {
                    role_name: SERVICE_PROVIDERS_ROLE_NAME.into(),
                    modifiers,
                    membrane_proof: None,
                    name: None,
                })
                .await?;
        }
    }

    for cloned_cell in cloned_cells {
        if !static_network_seeds
            .iter()
            .any(|network_seed| cloned_cell.dna_modifiers.network_seed.eq(network_seed))
        {
            app_ws
                .disable_clone_cell(DisableCloneCellPayload {
                    clone_cell_id: CloneCellId::CloneId(cloned_cell.clone_id),
                })
                .await?;
        }
    }

    for (fcm_project_id, service_account_key) in static_service_account_keys {
        let existing_service_account_key: Option<ServiceAccountKey> = app_ws
            .call_zome(
                ZomeCallTarget::RoleName("push_notifications_service".into()),
                "push_notifications_service".into(),
                "get_current_service_account_key".into(),
                ExternIO::encode(fcm_project_id.clone())?,
            )
            .await?
            .decode()?;

        let should_publish = match existing_service_account_key {
            None => true,
            Some(existing_key) => existing_key.ne(&crate::from(service_account_key.clone())),
        };
        if should_publish {
            app_ws
                .call_zome(
                    ZomeCallTarget::RoleName("push_notifications_service".into()),
                    "push_notifications_service".into(),
                    "publish_service_account_key".into(),
                    ExternIO::encode(PublishServiceAccountKeyInput {
                        fcm_project_id,
                        service_account_key: crate::from(service_account_key),
                    })?,
                )
                .await?;
        }
    }

    Ok(())
}
