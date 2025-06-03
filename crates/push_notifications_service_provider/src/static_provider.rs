use anyhow::anyhow;
use holochain::conductor::{
    conductor::hdk::prelude::holochain_zome_types::clone, manager::handle_shutdown,
};
use holochain_client::{CellInfo, ZomeCallTarget};
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::{
    PublishServiceAccountKeyInput, SendPushNotificationSignal, ServiceAccountKey,
};
use roles_types::Properties;
use std::path::PathBuf;

use crate::{fcm_client::FcmClient, read_from_file, SERVICE_PROVIDERS_ROLE_NAME};

pub async fn run_singleton_provider<T: FcmClient>(
    data_dir: PathBuf,
    network_config: NetworkConfig,
    app_id: String,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
    service_account_key: fcm_v1::auth::ServiceAccountKey,
    fcm_project_id: String,
    supported_network_seeds: Vec<NetworkSeed>,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), network_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![]), config).await?;
    setup(
        &runtime,
        &app_id,
        &push_notifications_service_provider_happ_path,
        progenitors,
        service_account_key,
        fcm_project_id,
        supported_network_seeds,
    )
    .await?;

    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;

    app_ws
        .on_signal(move |signal| {
            let Signal::App { signal, .. } = signal else {
                return ();
            };

            holochain_util::tokio_helper::run_on(async move {
                if let Err(err) = handle_signal::<T>(signal).await {
                    log::error!("Failed to handle signal: {err:?}");
                }
            });
        })
        .await;

    log::info!("Starting push notifications service provider.");

    // wait for a unix signal or ctrl-c instruction to
    // shutdown holochain
    tokio::signal::ctrl_c()
        .await
        .unwrap_or_else(|e| log::error!("Could not handle termination signal: {:?}", e));
    log::info!("Gracefully shutting down conductor...");
    let shutdown_result = runtime.conductor_handle.shutdown().await;
    handle_shutdown(shutdown_result);

    Ok(())
}

async fn handle_signal<T: FcmClient>(signal: AppSignal) -> anyhow::Result<()> {
    if let Ok(send_push_notification_signal) = signal
        .clone()
        .into_inner()
        .decode::<SendPushNotificationSignal>()
    {
        T::send_push_notification(
            send_push_notification_signal.fcm_project_id,
            crate::into(send_push_notification_signal.service_account_key),
            send_push_notification_signal.token,
            send_push_notification_signal.notification,
        )
        .await?;
    }
    Ok(())
}

async fn setup(
    runtime: &HolochainRuntime,
    app_id: &String,
    push_notifications_service_provider_happ_path: &PathBuf,
    progenitors: Vec<AgentPubKey>,
    service_account_key: fcm_v1::auth::ServiceAccountKey,
    fcm_project_id: String,
    supported_network_seeds: Vec<NetworkSeed>,
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

    for supported_network_seed in supported_network_seeds.clone() {
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
        if !supported_network_seeds
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

    Ok(())
}
