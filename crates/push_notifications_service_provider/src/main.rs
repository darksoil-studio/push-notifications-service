use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Builder;
use holochain::conductor::manager::handle_shutdown;
use holochain_client::ZomeCallTarget;
use holochain_conductor_api::CellInfo;
use holochain_runtime::*;
use holochain_types::prelude::*;
use log::Level;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use url2::Url2;

const SIGNAL_URL: &'static str = "wss://sbd.holo.host";
const BOOTSTRAP_URL: &'static str = "https://bootstrap.holo.host";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    push_notifications_service_provider_happ: PathBuf,

    /// Directory to store all holochain data
    #[arg(long)]
    data_dir: PathBuf,
}

fn wan_network_config() -> Option<WANNetworkConfig> {
    Some(WANNetworkConfig {
        signal_url: url2::url2!("{}", SIGNAL_URL),
        bootstrap_url: url2::url2!("{}", BOOTSTRAP_URL),
        ice_servers_urls: vec![],
    })
}

fn log_level() -> Level {
    match std::env::var("RUST_LOG") {
        Ok(s) => Level::from_str(s.as_str()).expect("Invalid RUST_LOG level"),
        _ => Level::Info,
    }
}

fn set_wasm_level() {
    match std::env::var("WASM_LOG") {
        Ok(_s) => {}
        _ => {
            std::env::set_var("WASM_LOG", "info");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(env_logger::Target::Stdout)
        .filter(None, log_level().to_level_filter())
        .filter_module("holochain_sqlite", log::LevelFilter::Off)
        .filter_module("tracing::span", log::LevelFilter::Off)
        .init();
    set_wasm_level();

    let data_dir = args.data_dir;
    if data_dir.exists() {
        if !std::fs::read_dir(&data_dir).is_ok() {
            return Err(anyhow!("The given data dir is not a directory."));
        };
    } else {
        std::fs::create_dir_all(data_dir.clone())?;
    }

    let wan_config = wan_network_config();

    let config = HolochainRuntimeConfig::new(data_dir.clone(), wan_config);

    let mut runtime = HolochainRuntime::launch(vec_to_locked(vec![])?, config).await?;
    let admin_ws = runtime.admin_websocket().await?;

    let installed_apps = admin_ws
        .list_apps(None)
        .await
        .map_err(|err| anyhow!("{err:?}"))?;

    let mut app_ids: Vec<String> = installed_apps
        .iter()
        .map(|app| app.installed_app_id.clone())
        .collect();
    let happ_bundle = read_from_file(&args.push_notifications_service_provider_happ).await?;

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

        app_ids.push(app_id.clone());

        log::info!("Installed app for hApp {}", app_id);
    }

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

async fn read_from_file(happ_bundle_path: &PathBuf) -> Result<AppBundle> {
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
