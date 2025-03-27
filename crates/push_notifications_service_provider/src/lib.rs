use anyhow::{anyhow, Result};
use fcm_client::FcmClient;
use holochain::conductor::manager::handle_shutdown;
use holochain_client::{AppWebsocket, ZomeCallTarget};
use holochain_conductor_api::CellInfo;
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::SendPushNotificationSignal;
use std::path::PathBuf;

mod fcm_client;
pub use fcm_client::RealFcmClient;

pub async fn run<T: FcmClient>(
    data_dir: PathBuf,
    wan_config: Option<WANNetworkConfig>,
    push_notifications_service_provider_happ_path: PathBuf,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), wan_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![])?, config).await?;
    let app_ws = setup(&runtime, &push_notifications_service_provider_happ_path).await?;

    app_ws
        .on_signal(|signal| {
            let Signal::App { signal, .. } = signal else {
                return ();
            };

            tokio::spawn(async move {
                if let Err(err) = handle_signal::<T>(signal)
                .await
                {
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
    let Ok(send_push_notification_signal) =
        signal.into_inner().decode::<SendPushNotificationSignal>()
    else {
        return Ok(());
    };
    T::send_push_notification(
        send_push_notification_signal.fcm_project_id,
        into(send_push_notification_signal.service_account_key),
        send_push_notification_signal.token,
        send_push_notification_signal.notification,
    ).await?;

    Ok(())
}

async fn setup(
    runtime: &HolochainRuntime,
    push_notifications_service_provider_happ_path: &PathBuf,
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
        let app_info = runtime
            .install_app(app_id.clone(), happ_bundle, None, None, None)
            .await?;
        let app_ws = runtime
            .app_websocket(
                app_id.clone(),
                holochain_types::websocket::AllowedOrigins::Any,
            )
            .await?;

        for (_role, cell_infos) in app_info.cell_info {
            for cell_info in cell_infos {
                let Some(cell_id) = cell_id(&cell_info) else {
                    continue;
                };
                let dna_def = admin_ws
                    .get_dna_definition(cell_id.dna_hash().clone())
                    .await
                    .map_err(|err| anyhow!("{err:?}"))?;

                let Some(first_zome) = dna_def.coordinator_zomes.first() else {
                    continue;
                };

                app_ws
                    .call_zome(
                        ZomeCallTarget::CellId(cell_id),
                        first_zome.0.clone(),
                        "init".into(),
                        ExternIO::encode(())?,
                    )
                    .await
                    .map_err(|err| anyhow!("{:?}", err))?;
            }
        }

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
