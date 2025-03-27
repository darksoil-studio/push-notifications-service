use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Builder;
use holochain_runtime::WANNetworkConfig;
use log::Level;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

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

    push_notifications_service_provider::run::<push_notifications_service_provider::RealFcmClient>(
        data_dir,
        wan_network_config(),
        args.push_notifications_service_provider_happ,
    )
    .await
}
