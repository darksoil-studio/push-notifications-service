use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Builder;
use holochain::core::AgentPubKeyB64;
use holochain_client::InstalledAppId;
use holochain_runtime::NetworkConfig;
use log::Level;
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

    #[arg(long, required = true, num_args = 1)]
    progenitors: Vec<AgentPubKeyB64>,

    #[arg(long)]
    bootstrap_url: Option<String>,

    #[arg(long)]
    signal_url: Option<String>,
}

fn network_config(bootstrap_url: Option<String>, signal_url: Option<String>) -> NetworkConfig {
    let mut network_config = NetworkConfig::default();

    if let Some(bootstrap_url) = bootstrap_url {
        network_config.bootstrap_url = url2::Url2::parse(bootstrap_url);
    }
    if let Some(signal_url) = signal_url {
        network_config.signal_url = url2::Url2::parse(signal_url);
    }

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
        .filter_module("iroh", log::LevelFilter::Warn)
        .filter_module("kitsune2", log::LevelFilter::Warn)
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

    push_notifications_service_provider::run::<RealFcmClient>(
        data_dir,
        network_config(args.bootstrap_url, args.signal_url),
        args.app_id,
        args.push_notifications_service_provider_happ,
        args.progenitors.into_iter().map(|p| p.into()).collect(),
    )
    .await
}
