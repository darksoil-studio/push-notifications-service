use std::path::PathBuf;

use anyhow::anyhow;
use holochain::prelude::{DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties};
use holochain_client::{AgentPubKey, ExternIO, ZomeCallTarget};
use holochain_runtime::HolochainRuntime;
use roles_types::Properties;

use crate::read_from_file;

pub async fn setup(
    runtime: &HolochainRuntime,
    app_id: &String,
    push_notifications_service_provider_happ_path: &PathBuf,
    progenitors: Vec<AgentPubKey>,
) -> anyhow::Result<()> {
    let admin_ws = runtime.admin_websocket().await?;
    let installed_apps = admin_ws
        .list_apps(None)
        .await
        .map_err(|err| anyhow!("{err:?}"))?;
    let happ_bundle = read_from_file(push_notifications_service_provider_happ_path).await?;

    if installed_apps
        .iter()
        .find(|app| app.installed_app_id.eq(app_id))
        .is_none()
    {
        let roles_properties = Properties {
            progenitors: progenitors.into_iter().map(|p| p.into()).collect(),
        };
        let value = serde_yaml::to_value(roles_properties).unwrap();
        let properties_bytes = YamlProperties::new(value);

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
            .await
            .map_err(|err| anyhow!("{:?}", err))?;

        log::info!("Installed app {app_info:?}");
    }
    Ok(())
}
