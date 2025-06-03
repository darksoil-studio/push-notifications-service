use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Builder;
use fcm_v1::auth::ServiceAccountKey;
use holochain::core::AgentPubKeyB64;
use holochain::prelude::NetworkSeed;
use holochain_client::InstalledAppId;
use holochain_runtime::NetworkConfig;
use log::Level;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use push_notifications_service_provider::fcm_client::RealFcmClient;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    push_notifications_service_provider_happ: PathBuf,

    #[arg(long)]
    app_id: InstalledAppId,

    /// Directory to store all holochain data
    #[arg(long)]
    data_dir: PathBuf,

    #[arg(long)]
    progenitors: Vec<AgentPubKeyB64>,

    #[arg(long)]
    bootstrap_url: String,

    #[arg(long)]
    signal_url: String,

    #[arg(long)]
    static_network_seeds: Vec<NetworkSeed>,

    #[arg(long)]
    static_fcm_project_id: Option<String>,

    #[arg(long)]
    static_service_account_key_path: Option<PathBuf>,
}

fn network_config(bootstrap_url: String, signal_url: String) -> NetworkConfig {
    let mut network_config = NetworkConfig::default();

    network_config.bootstrap_url = url2::Url2::parse(bootstrap_url);
    network_config.signal_url = url2::Url2::parse(signal_url);
    network_config.webrtc_config = Some(serde_json::json!({
        "ice_servers": {
            "urls": ["stun://stun.l.google.com:19302"]
        },
    }));

    network_config
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

    let mut static_service_account_key: BTreeMap<String, ServiceAccountKey> = BTreeMap::new();

    match (
        args.static_fcm_project_id,
        args.static_service_account_key_path,
    ) {
        (Some(fcm_project_id), Some(service_account_key_path)) => {
            let service_account_str = std::fs::read_to_string(service_account_key_path)?;
            let service_account_key: ServiceAccountKey =
                serde_json::from_str(&service_account_str.as_str())?;
            static_service_account_key.insert(fcm_project_id, service_account_key);
        }
        (None, None) => {}
        (Some(_), None) => Err(anyhow!(
            "--static-service-account-key-path must be defined if --static-fcm-project is given."
        ))?,
        (None, Some(_)) => Err(anyhow!(
            "--static-fcm-project must be defined if --static-service-account-key-path is given."
        ))?,
    }

    push_notifications_service_provider::run::<RealFcmClient>(
        data_dir,
        network_config(args.bootstrap_url, args.signal_url),
        args.app_id,
        args.push_notifications_service_provider_happ,
        args.progenitors.into_iter().map(|p| p.into()).collect(),
        static_service_account_key,
        args.static_network_seeds,
    )
    .await
}
