use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use fcm_v1::auth::ServiceAccountKey;
use holochain::core::AgentPubKeyB64;
use holochain::prelude::NetworkSeed;
use holochain_runtime::NetworkConfig;
use holochain_util::ffs::read_to_string;
use log::Level;
use push_notifications_service_client::PushNotificationsServiceClient;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tempdir::TempDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    push_notifications_service_provider_happ: PathBuf,

    #[arg(long, required = true, num_args = 1)]
    progenitors: Vec<AgentPubKeyB64>,

    #[arg(long)]
    bootstrap_url: Option<String>,

    #[arg(long)]
    signal_url: Option<String>,

    #[arg(long)]
    mdns_discovery: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Publishes a service account key
    PublishServiceAccountKey {
        #[arg(long)]
        service_account_key_path: PathBuf,
    },
    /// Create a clone request for the service providers DNA
    CreateCloneRequest {
        #[arg(long)]
        network_seed: NetworkSeed,
    },
}

fn network_config(bootstrap_url: Option<String>, signal_url: Option<String>) -> NetworkConfig {
    let mut network_config = NetworkConfig::default();

    if let Some(bootstrap_url) = bootstrap_url {
        network_config.bootstrap_url = url2::Url2::parse(bootstrap_url);
    }
    if let Some(signal_url) = signal_url {
        network_config.signal_url = url2::Url2::parse(signal_url);
    }
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
        .filter_module("iroh", log::LevelFilter::Warn)
        .init();
    set_wasm_level();

    let tempdir = TempDir::new("push-notifications-service-client")?;
    let data_dir = tempdir.path().to_path_buf();

    let client = PushNotificationsServiceClient::create(
        data_dir.clone(),
        network_config(args.bootstrap_url, args.signal_url),
        String::from("temporary-client-app"),
        args.push_notifications_service_provider_happ,
        args.progenitors.into_iter().map(|p| p.into()).collect(),
        args.mdns_discovery,
    )
    .await?;

    match args.command {
        Commands::PublishServiceAccountKey {
            service_account_key_path,
        } => {
            let service_account_str = read_to_string(service_account_key_path).await?;
            let service_account_key: ServiceAccountKey =
                serde_json::from_str(&service_account_str)?;

            client
                .publish_service_account_key(service_account_key)
                .await?;
        }
        Commands::CreateCloneRequest { network_seed } => {
            client.create_clone_request(network_seed).await?;
        }
    }

    client.runtime.shutdown().await?;

    Ok(())
}
