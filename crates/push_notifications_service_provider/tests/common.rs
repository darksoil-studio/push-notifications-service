use std::path::PathBuf;
use std::{io::Write, time::Duration};

use anyhow::anyhow;
use env_logger::Builder;
use fixt::fixt;
use holo_hash::fixt::AgentPubKeyFixturator;
use holochain::prelude::{DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties};
use holochain_client::{AgentPubKey, AppWebsocket};
use holochain_runtime::{vec_to_locked, HolochainRuntime, HolochainRuntimeConfig, NetworkConfig};
use kitsune2_bootstrap_srv::BootstrapSrv;
use log::Level;
use push_notifications_service_provider::fcm_client::MockFcmClient;
use push_notifications_service_provider::{read_from_file, run};
use roles_types::Properties;
use url2::url2;

pub fn service_provider_happ_path() -> PathBuf {
    std::option_env!("SERVICE_PROVIDER_HAPP")
        .expect("Failed to find SERVICE_PROVIDER_HAPP")
        .into()
}

pub fn client_happ_path() -> PathBuf {
    std::option_env!("CLIENT_HAPP")
        .expect("Failed to find CLIENT_HAPP")
        .into()
}

pub fn end_user_happ_path() -> PathBuf {
    std::option_env!("END_USER_HAPP")
        .expect("Failed to find END_USER_HAPP")
        .into()
}

pub fn network_config(bootstrap_srv: &BootstrapSrv) -> NetworkConfig {
    let address = bootstrap_srv.listen_addrs()[0].clone();

    let mut network_config = NetworkConfig::default();
    network_config.bootstrap_url = url2!("http://{}", address);
    network_config.signal_url = url2!("ws://{}", address);
    network_config
}

pub async fn run_bootstrap_server() -> BootstrapSrv {
    tokio::task::spawn_blocking(|| {
        let config = kitsune2_bootstrap_srv::Config::testing();
        let server = kitsune2_bootstrap_srv::BootstrapSrv::new(config).unwrap();
        server
    })
    .await
    .unwrap()
}

pub async fn launch(
    progenitors: Vec<AgentPubKey>,
    roles: Vec<String>,
    happ_path: PathBuf,
    network_seed: String,
    network_config: NetworkConfig,
) -> (AppWebsocket, HolochainRuntime) {
    let runtime = HolochainRuntime::launch(
        vec_to_locked(vec![]),
        HolochainRuntimeConfig::new(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            network_config,
        ),
    )
    .await
    .expect("Could not launch holochain runtime");

    let roles_properties = Properties {
        progenitors: progenitors.into_iter().map(Into::into).collect(),
    };
    let value = serde_yaml::to_value(roles_properties).unwrap();
    let properties_bytes = YamlProperties::new(value);

    let mut roles_settings = RoleSettingsMap::new();
    for role in roles {
        roles_settings.insert(
            role.clone(),
            RoleSettings::Provisioned {
                membrane_proof: None,
                modifiers: Some(DnaModifiersOpt {
                    properties: Some(properties_bytes.clone()),
                    network_seed: Some(network_seed.clone()),
                }),
            },
        );
    }

    let app_id = String::from("push-notifications-test");

    let _app_info = runtime
        .install_app(
            app_id.clone(),
            read_from_file(&happ_path).await.unwrap(),
            Some(roles_settings),
            None,
            None,
        )
        .await
        .unwrap();

    let app_ws = runtime
        .app_websocket(app_id, holochain_client::AllowedOrigins::Any)
        .await
        .unwrap();
    (app_ws, runtime)
}

pub struct Scenario {
    pub network_seed: String,
    pub progenitors: Vec<AgentPubKey>,
    pub sender: (AppWebsocket, HolochainRuntime),
    pub recipient: (AppWebsocket, HolochainRuntime),
    pub bootstrap_srv: BootstrapSrv,
}

pub async fn setup() -> Scenario {
    Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(env_logger::Target::Stdout)
        .filter(None, Level::Info.to_level_filter())
        .filter_module("holochain_sqlite", log::LevelFilter::Off)
        .filter_module("kitsune2_bootstrap_srv", log::LevelFilter::Debug)
        .filter_module("tracing::span", log::LevelFilter::Off)
        .filter_module("kitsune2", log::LevelFilter::Warn)
        .filter_module("iroh", log::LevelFilter::Error)
        .init();

    let network_seed = String::from("somesecret");
    let progenitors = vec![fixt!(AgentPubKey)];
    let bootstrap_srv = run_bootstrap_server().await;

    let p = progenitors.clone();
    let nc = network_config(&bootstrap_srv);
    tokio::spawn(async move {
        run::<MockFcmClient>(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            nc,
            String::from("test-app"),
            service_provider_happ_path(),
            p.clone(),
        )
        .await
        .unwrap();
    });
    let p = progenitors.clone();
    let nc = network_config(&bootstrap_srv);
    tokio::spawn(async move {
        run::<MockFcmClient>(
            tempdir::TempDir::new("test2")
                .expect("Could not make tempdir")
                .into_path(),
            nc,
            String::from("test-app"),
            service_provider_happ_path(),
            p.clone(),
        )
        .await
        .unwrap();
    });
    let sender = launch(
        progenitors.clone(),
        vec![String::from("services")],
        end_user_happ_path(),
        network_seed.clone(),
        network_config(&bootstrap_srv),
    )
    .await;
    let recipient = launch(
        progenitors.clone(),
        vec![String::from("services")],
        end_user_happ_path(),
        network_seed.clone(),
        network_config(&bootstrap_srv),
    )
    .await;

    Scenario {
        network_seed,
        progenitors,
        sender,
        recipient,
        bootstrap_srv,
    }
}

pub async fn with_retries<T>(
    condition: impl AsyncFn() -> anyhow::Result<T>,
    retries: usize,
) -> anyhow::Result<T> {
    let mut retry_count = 0;
    loop {
        let response = condition().await;

        match response {
            Ok(r) => {
                return Ok(r);
            }
            Err(err) => {
                log::warn!("Condition not met yet: {err:?} Retrying in 1s.");
                std::thread::sleep(Duration::from_secs(1));

                retry_count += 1;
                if retry_count == retries {
                    return Err(anyhow!("Timeout. Last error: {err:?}"));
                }
            }
        }
    }
}
