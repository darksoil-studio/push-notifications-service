use anyhow::{anyhow, Result};
use clone_manager_types::{CloneRequest, NewCloneRequest};
use clone_manager_utils::reconcile_cloned_cells;
use holochain_client::{AdminWebsocket, AppWebsocket};
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::SendPushNotificationSignal;
use setup::setup;
use std::{fs, path::PathBuf, time::Duration};
use utils::with_retries;

pub mod fcm_client;
mod utils;
use fcm_client::FcmClient;
mod setup;

pub const SERVICES_ROLE_NAME: &'static str = "services";

pub async fn run<T: FcmClient>(
    data_dir: PathBuf,
    network_config: NetworkConfig,
    app_id: String,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
    mdns_discovery: bool
) -> anyhow::Result<()> {
    let mut config = HolochainRuntimeConfig::new(data_dir.clone(), network_config);
    config.mdns_discovery = mdns_discovery;

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

    let r = runtime.clone();

    tokio::spawn(async move {
        loop {
            let Ok(app_ws) = r
                .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
                .await
            else {
                log::error!("Failed to connect to the app websocket");
                continue;
            };
            let Ok(admin_ws) = r.admin_websocket().await else {
                log::error!("Failed to connect to the admin websocket");
                continue;
            };
            if let Err(err) = reconcile_cloned_cells(
                &admin_ws,
                &app_ws,
                "push_notifications_service".into(),
                SERVICES_ROLE_NAME.into(),
            )
            .await
            {
                log::error!("Failed to reconcile cloned services: {err}");
            }

            std::thread::sleep(Duration::from_secs(60));
        }
    });

    // wait for a unix signal or ctrl-c instruction to
    // shutdown holochain
    tokio::signal::ctrl_c()
        .await
        .unwrap_or_else(|e| log::error!("Could not handle termination signal: {:?}", e));
    log::info!("Gracefully shutting down conductor...");
    runtime.shutdown().await?;

    Ok(())
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
        handle_new_clone_request_signal(admin_ws, app_ws, new_clone_request).await?;
    }
    Ok(())
}

async fn handle_new_clone_request_signal(
    admin_ws: &AdminWebsocket,
    app_ws: &AppWebsocket,
    new_clone_request: NewCloneRequest,
) -> Result<()> {
    let a = app_ws.clone();
    with_retries(
        async move || {
            let clone_request: Option<CloneRequest> = a
                .call_zome(
                    holochain_client::ZomeCallTarget::RoleName(String::from(
                        "push_notifications_service",
                    )),
                    "clone_manager".into(),
                    "get_clone_request".into(),
                    ExternIO::encode(new_clone_request.clone_request_hash.clone())?,
                )
                .await?
                .decode()?;
            let Some(_) = clone_request else {
                return Err(anyhow!("CloneRequest not found."));
            };

            Ok(())
        },
        20,
    )
    .await?;

    reconcile_cloned_cells(
        &admin_ws,
        &app_ws,
        "push_notifications_service".into(),
        SERVICES_ROLE_NAME.into(),
    )
    .await?;

    Ok(())
}

pub async fn read_from_file(happ_bundle_path: &PathBuf) -> Result<AppBundle> {
    let bytes = fs::read(happ_bundle_path)?;
    Ok(AppBundle::decode(bytes.as_slice())?)
}

pub fn into(key: push_notifications_types::ServiceAccountKey) -> fcm_v1::auth::ServiceAccountKey {
    fcm_v1::auth::ServiceAccountKey {
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

pub fn from(key: fcm_v1::auth::ServiceAccountKey) -> push_notifications_types::ServiceAccountKey {
    push_notifications_types::ServiceAccountKey {
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
