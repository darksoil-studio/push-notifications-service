use anyhow::anyhow;
use clone_manager_types::NewCloneRequest;
use clone_manager_utils::{clone_cell, reconcile_cloned_cells};
use holochain_client::{AdminWebsocket, AppWebsocket, ZomeCallTarget};
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::SendPushNotificationSignal;
use roles_types::Properties;
use std::{path::PathBuf, time::Duration};

use crate::{fcm_client::FcmClient, read_from_file, SERVICE_PROVIDERS_ROLE_NAME};

pub async fn run_dynamic_provider<T: FcmClient>(
    data_dir: PathBuf,
    network_config: NetworkConfig,
    app_id: String,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), network_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![]), config).await?;
    setup(
        &runtime,
        &app_id,
        &push_notifications_service_provider_happ_path,
        progenitors,
    )
    .await?;

    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;
    let app_clone = app_ws.clone();
    let admin_ws = runtime.admin_websocket().await?;

    app_ws
        .on_signal(move |signal| {
            let Signal::App { signal, .. } = signal else {
                return ();
            };

            let app_ws = &app_clone;
            let admin_ws = &admin_ws;

            holochain_util::tokio_helper::run_on(async move {
                if let Err(err) = handle_signal::<T>(admin_ws, app_ws, signal).await {
                    log::error!("Failed to handle signal: {err:?}");
                }
            });
        })
        .await;

    log::info!("Starting push notifications service provider.");

    loop {
        let app_ws = runtime
            .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
            .await?;
        let admin_ws = runtime.admin_websocket().await?;
        if let Err(err) = reconcile_cloned_cells(
            &admin_ws,
            &app_ws,
            "push_notifications_service".into(),
            SERVICE_PROVIDERS_ROLE_NAME.into(),
        )
        .await
        {
            log::error!("Failed to reconcile cloned services: {err}");
        }

        std::thread::sleep(Duration::from_secs(30));
    }

    // // wait for a unix signal or ctrl-c instruction to
    // // shutdown holochain
    // tokio::signal::ctrl_c()
    //     .await
    //     .unwrap_or_else(|e| log::error!("Could not handle termination signal: {:?}", e));
    // log::info!("Gracefully shutting down conductor...");
    // let shutdown_result = runtime.conductor_handle.shutdown().await;
    // handle_shutdown(shutdown_result);

    // Ok(())
}

pub async fn handle_signal<T: FcmClient>(
    admin_ws: &AdminWebsocket,
    app_ws: &AppWebsocket,
    signal: AppSignal,
) -> anyhow::Result<()> {
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
    if let Ok(new_clone_request) = signal.into_inner().decode::<NewCloneRequest>() {
        clone_cell(
            &admin_ws,
            &app_ws,
            SERVICE_PROVIDERS_ROLE_NAME.into(),
            new_clone_request.clone_request,
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
