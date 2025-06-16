use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{io::Write, time::Duration};

use env_logger::Builder;
use holo_hash::fixt::AgentPubKeyFixturator;
use holo_hash::DnaHash;
use holochain::prelude::{DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties};
use holochain_client::{AdminWebsocket, AgentPubKey, AppWebsocket};
use holochain_runtime::{vec_to_locked, HolochainRuntime, HolochainRuntimeConfig, NetworkConfig};
use log::Level;
use push_notifications_service_provider::fcm_client::MockFcmClient;
use push_notifications_service_provider::read_from_file;
use roles_types::Properties;
use url2::url2;

pub fn happ_developer_happ_path() -> PathBuf {
    std::option_env!("HAPP_DEVELOPER_HAPP")
        .expect("Failed to find HAPP_DEVELOPER_HAPP")
        .into()
}

pub fn service_provider_happ_path() -> PathBuf {
    std::option_env!("SERVICE_PROVIDER_HAPP")
        .expect("Failed to find SERVICE_PROVIDER_HAPP")
        .into()
}

pub fn client_happ_path() -> PathBuf {
    std::option_env!("CLIENT_HAPP")
        .expect("Failed to find INFRA_PROVIDER_HAPP")
        .into()
}

pub fn end_user_happ_path() -> PathBuf {
    std::option_env!("END_USER_HAPP")
        .expect("Failed to find END_USER_HAPP")
        .into()
}

pub fn network_config() -> NetworkConfig {
    let mut network_config = NetworkConfig::default();
    network_config.bootstrap_url = url2!("http://bad");
    network_config.signal_url = url2!("ws://bad");
    network_config
}

pub async fn launch(
    infra_provider_pub_key: AgentPubKey,
    roles: Vec<String>,
    happ_path: PathBuf,
    network_seed: String,
) -> (AppWebsocket, HolochainRuntime) {
    let runtime = HolochainRuntime::launch(
        vec_to_locked(vec![]),
        HolochainRuntimeConfig::new(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            network_config(),
        ),
    )
    .await
    .expect("Could not launch holochain runtime");

    let roles_properties = Properties {
        progenitors: vec![infra_provider_pub_key.clone().into()],
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

    let app_info = runtime
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
    pub happ_developer: (AppWebsocket, HolochainRuntime),
    pub sender: (AppWebsocket, HolochainRuntime),
    pub recipient: (AppWebsocket, HolochainRuntime),
    pub progenitor: AgentPubKey,
    pub network_seed: String,
}

pub async fn setup() -> Scenario {
    Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(env_logger::Target::Stdout)
        .filter(None, Level::Info.to_level_filter())
        .filter_module("holochain_sqlite", log::LevelFilter::Off)
        .filter_module("tracing::span", log::LevelFilter::Off)
        .filter_module("kitsune2", log::LevelFilter::Warn)
        .filter_module("iroh", log::LevelFilter::Error)
        .init();

    let network_seed = String::from("test");

    let infra_provider_pubkey = fixt::fixt!(AgentPubKey);
    let pubkey = infra_provider_pubkey.clone();

    let tmp = tempdir::TempDir::new("test").unwrap();
    let path = tmp.path().to_path_buf();
    // We spawn two nodes to make gossip work between them
    tokio::spawn(async move {
        push_notifications_service_provider::run::<MockFcmClient>(
            path,
            network_config(),
            String::from("test-app"),
            service_provider_happ_path(),
            vec![pubkey.clone()],
        )
        .await
        .unwrap();
    });
    let tmp = tempdir::TempDir::new("test").unwrap();
    let path = tmp.path().to_path_buf();
    let pubkey = infra_provider_pubkey.clone();
    tokio::spawn(async move {
        push_notifications_service_provider::run::<MockFcmClient>(
            path,
            network_config(),
            String::from("test-app"),
            service_provider_happ_path(),
            vec![pubkey.clone()],
        )
        .await
        .unwrap();
    });
    let happ_developer = launch(
        infra_provider_pubkey.clone(),
        vec![String::from("service_providers")],
        happ_developer_happ_path(),
        network_seed.clone(),
    )
    .await;
    let sender = launch(
        infra_provider_pubkey.clone(),
        vec![String::from("service_providers")],
        end_user_happ_path(),
        network_seed.clone(),
    )
    .await;
    let recipient = launch(
        infra_provider_pubkey.clone(),
        vec![String::from("service_providers")],
        end_user_happ_path(),
        network_seed.clone(),
    )
    .await;

    std::thread::sleep(Duration::from_secs(20));

    Scenario {
        happ_developer,
        sender,
        recipient,
        progenitor: infra_provider_pubkey.clone(),
        network_seed,
    }
}

pub async fn consistency(admins_wss: Vec<AdminWebsocket>) -> anyhow::Result<()> {
    let mut retry_count = 0;
    loop {
        let dna_hashes: BTreeSet<DnaHash> =
            futures::future::try_join_all(admins_wss.iter().map(|admin| admin.list_dnas()))
                .await
                .unwrap()
                .into_iter()
                .flatten()
                .collect();

        let consistencied = futures::future::try_join_all(
            dna_hashes
                .into_iter()
                .map(|dna| are_conductors_consistencied(&admins_wss, dna)),
        )
        .await?
        .iter()
        .all(|c| c.clone());

        if consistencied {
            return Ok(());
        }

        retry_count += 1;

        if retry_count > 200 {
            return Err(anyhow::anyhow!("Timeout"));
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

async fn are_conductors_consistencied(
    admins_wss: &Vec<AdminWebsocket>,
    dna_hash: DnaHash,
) -> anyhow::Result<bool> {
    let states = futures::future::try_join_all(admins_wss.iter().map(|admin_ws| async {
        let cells = admin_ws.list_cell_ids().await?;
        let Some(cell_id) = cells.into_iter().find(|cell| cell.dna_hash().eq(&dna_hash)) else {
            return Err(anyhow::anyhow!("Cell not found for dna: {dna_hash}."));
        };
        let dump = admin_ws.dump_full_state(cell_id, None).await?;
        Ok(dump)
    }))
    .await?;

    if states.iter().any(|s| {
        s.integration_dump.validation_limbo.len() > 0
            || s.integration_dump.integration_limbo.len() > 0
    }) {
        return Ok(false);
    }

    if !states
        .windows(2)
        .all(|w| w[0].integration_dump.integrated.len() == w[1].integration_dump.integrated.len())
    {
        return Ok(false);
    }

    Ok(true)
}
