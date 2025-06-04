use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use env_logger::Builder;
use fcm_v1::auth::ServiceAccountKey;
use holochain::core::AgentPubKeyB64;
use holochain::prelude::NetworkSeed;
use holochain_runtime::NetworkConfig;
use holochain_util::ffs::read_to_string;
use log::Level;
use push_notifications_service_client::PushNotificationsServiceClient;
use std::env::temp_dir;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    push_notifications_service_provider_happ: PathBuf,

    #[arg(long)]
    progenitors: Vec<AgentPubKeyB64>,

    #[arg(long)]
    bootstrap_url: String,

    #[arg(long)]
    signal_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Publishes a service account key
    PublishServiceAccountKey {
        #[arg(long)]
        fcm_project_id: String,

        #[arg(long)]
        service_account_key_path: PathBuf,
    },
    /// Create a clone request for the service providers DNA
    CreateCloneRequest {
        #[arg(long)]
        network_seed: NetworkSeed,
    },
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
    network_config.target_arc_factor = 0;

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

    let data_dir = temp_dir();
    if data_dir.exists() {
        if !std::fs::read_dir(&data_dir).is_ok() {
            return Err(anyhow!("The given data dir is not a directory."));
        };
    } else {
        std::fs::create_dir_all(data_dir.clone())?;
    }

    let client = PushNotificationsServiceClient::create(
        data_dir.clone(),
        network_config(args.bootstrap_url, args.signal_url),
        String::from("temporary-client-app"),
        args.push_notifications_service_provider_happ,
        args.progenitors.into_iter().map(|p| p.into()).collect(),
    )
    .await?;

    match args.command {
        Commands::PublishServiceAccountKey {
            fcm_project_id,
            service_account_key_path,
        } => {
            let service_account_str = read_to_string(service_account_key_path).await?;
            let service_account_key: ServiceAccountKey =
                serde_json::from_str(&service_account_str)?;

            client
                .publish_service_account_key(fcm_project_id, service_account_key)
                .await?;
        }
        Commands::CreateCloneRequest { network_seed } => {
            client.create_clone_request(network_seed).await?;
        }
    }

    std::fs::remove_dir_all(data_dir)?;

    Ok(())
}
