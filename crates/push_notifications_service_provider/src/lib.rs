use anyhow::{anyhow, Result};
use cloned_services::{clone_service, reconcile_cloned_services};
use holochain_client::{AppWebsocket, ZomeCallTarget};
use holochain_conductor_api::CellInfo;
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::{NewCloneServiceRequest, SendPushNotificationSignal};
use roles_types::Properties;
use std::{path::PathBuf, time::Duration};

mod cloned_services;
pub mod fcm_client;
use fcm_client::FcmClient;

pub async fn run<T: FcmClient>(
    data_dir: PathBuf,
    wan_config: Option<WANNetworkConfig>,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), wan_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![])?, config).await?;
    let app_ws = setup(
        &runtime,
        &push_notifications_service_provider_happ_path,
        progenitors,
    )
    .await?;

    let app_clone = app_ws.clone();

    app_ws
        .on_signal(move |signal| {
            let Signal::App { signal, .. } = signal else {
                return ();
            };

            let app_clone = app_clone.clone();

            tokio::spawn(async move {
                if let Err(err) = handle_signal::<T>(app_clone, signal).await {
                    log::error!("Failed to handle signal: {err:?}");
                }
            });
        })
        .await;

    log::info!("Starting push notifications service provider.");

    loop {
        if let Err(err) = reconcile_cloned_services(&app_ws).await {
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

async fn handle_signal<T: FcmClient>(
    app_ws: AppWebsocket,
    signal: AppSignal,
) -> anyhow::Result<()> {
    if let Ok(send_push_notification_signal) = signal
        .clone()
        .into_inner()
        .decode::<SendPushNotificationSignal>()
    {
        T::send_push_notification(
            send_push_notification_signal.fcm_project_id,
            into(send_push_notification_signal.service_account_key),
            send_push_notification_signal.token,
            send_push_notification_signal.notification,
        )
        .await?;
    }
    if let Ok(new_clone_service_request) = signal.into_inner().decode::<NewCloneServiceRequest>() {
        clone_service(app_ws, new_clone_service_request.clone_service_request).await?;
    }
    Ok(())
}

async fn setup(
    runtime: &HolochainRuntime,
    push_notifications_service_provider_happ_path: &PathBuf,
    progenitors: Vec<AgentPubKey>,
) -> Result<AppWebsocket> {
    let admin_ws = runtime.admin_websocket().await?;
    let installed_apps = admin_ws
        .list_apps(None)
        .await
        .map_err(|err| anyhow!("{err:?}"))?;
    let happ_bundle = read_from_file(push_notifications_service_provider_happ_path).await?;

    let app_id = happ_bundle.manifest().app_name().to_string();

    if installed_apps
        .iter()
        .find(|app| app.installed_app_id.eq(&app_id))
        .is_none()
    {
        let roles_properties = Properties {
            progenitors: progenitors.into_iter().map(|p| p.into()).collect(),
        };
        let value = serde_yaml::to_value(roles_properties).unwrap();
        let properties_bytes = YamlProperties::new(value);

        let mut roles_settings = RoleSettingsMap::new();
        roles_settings.insert(
            String::from("push_notifications_service_providers_manager"),
            RoleSettings::Provisioned {
                membrane_proof: None,
                modifiers: Some(DnaModifiersOpt {
                    properties: Some(properties_bytes.clone()),
                    ..Default::default()
                }),
            },
        );

        let _app_info = runtime
            .install_app(app_id.clone(), happ_bundle, None, None, None)
            .await?;
        let app_ws = runtime
            .app_websocket(
                app_id.clone(),
                holochain_types::websocket::AllowedOrigins::Any,
            )
            .await?;

        app_ws
            .call_zome(
                ZomeCallTarget::RoleName("push_notifications_service_providers_manager".into()),
                "push_notifications_service_providers_manager".into(),
                "announce_as_provider".into(),
                ExternIO::encode(())?,
            )
            .await
            .map_err(|err| anyhow!("{:?}", err))?;

        log::info!("Installed app for hApp {}", app_id);
    }
    let app_ws = runtime
        .app_websocket(
            app_id.clone(),
            holochain_types::websocket::AllowedOrigins::Any,
        )
        .await?;
    Ok(app_ws)
}

pub async fn read_from_file(happ_bundle_path: &PathBuf) -> Result<AppBundle> {
    mr_bundle::Bundle::read_from_file(happ_bundle_path)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

fn cell_id(cell_info: &CellInfo) -> Option<CellId> {
    match cell_info {
        CellInfo::Provisioned(provisioned) => Some(provisioned.cell_id.clone()),
        CellInfo::Cloned(cloned) => Some(cloned.cell_id.clone()),
        CellInfo::Stem(_) => None,
    }
}

fn into(key: push_notifications_types::ServiceAccountKey) -> yup_oauth2::ServiceAccountKey {
    yup_oauth2::ServiceAccountKey {
        key_type: key.key_type,
        project_id: key.project_id,
        private_key_id: key.private_key_id,
        private_key: key.private_key,
        client_email: key.client_email,
        client_id: key.client_id,
        auth_uri: key.auth_uri,
        token_uri: key.token_uri,
        auth_provider_x509_cert_url: key.auth_provider_x509_cert_url,
        client_x509_cert_url: key.client_x509_cert_url,
    }
}
