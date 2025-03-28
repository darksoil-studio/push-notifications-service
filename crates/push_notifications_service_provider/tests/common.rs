use std::path::PathBuf;
use std::{io::Write, time::Duration};

use env_logger::Builder;
use holochain::prelude::{DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties};
use holochain_client::{AgentPubKey, AppInfo, AppWebsocket};
use holochain_runtime::{vec_to_locked, HolochainRuntime, HolochainRuntimeConfig};
use log::Level;
use push_notifications_service_provider::fcm_client::MockFcmClient;
use push_notifications_service_provider::{read_from_file, run};
use roles_types::Properties;

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

pub fn infra_provider_happ_path() -> PathBuf {
    std::option_env!("INFRA_PROVIDER_HAPP")
        .expect("Failed to find INFRA_PROVIDER_HAPP")
        .into()
}

pub fn end_user_happ_path() -> PathBuf {
    std::option_env!("END_USER_HAPP")
        .expect("Failed to find END_USER_HAPP")
        .into()
}

pub async fn launch_infra_provider() -> (AppWebsocket, HolochainRuntime) {
    let infra_provider = HolochainRuntime::launch(
        vec_to_locked(vec![]).expect("Could not build passphrase"),
        HolochainRuntimeConfig::new(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            None,
        ),
    )
    .await
    .expect("Could not launch holochain runtime");

    let admin_ws = infra_provider
        .admin_websocket()
        .await
        .expect("Failed to connect AdminWebsocket");

    let infra_provider_pub_key = admin_ws
        .generate_agent_pub_key()
        .await
        .expect("Failed to generate pubkey");

    let roles_properties = Properties {
        progenitors: vec![infra_provider_pub_key.clone().into()],
    };
    let value = serde_yaml::to_value(roles_properties).unwrap();
    let properties_bytes = YamlProperties::new(value);
    let modifiers = DnaModifiersOpt {
        properties: Some(properties_bytes),
        ..Default::default()
    };

    let mut roles_settings = RoleSettingsMap::new();
    roles_settings.insert(
        String::from("push_notifications_service"),
        RoleSettings::Provisioned {
            membrane_proof: None,
            modifiers: Some(modifiers),
        },
    );
    let app_id = String::from("infra-provider");
    let app_info = infra_provider
        .install_app(
            app_id.clone(),
            read_from_file(&infra_provider_happ_path())
                .await
                .expect("Failed to read infra provider happ"),
            Some(roles_settings),
            Some(infra_provider_pub_key.clone()),
            None,
        )
        .await
        .expect("Failed to install infra provider happ");

    let app_ws = infra_provider
        .app_websocket(app_id, holochain_client::AllowedOrigins::Any)
        .await
        .unwrap();

    (app_ws, infra_provider)
}

pub async fn launch(
    infra_provider_pub_key: AgentPubKey,
    roles: Vec<String>,
    happ_path: PathBuf,
) -> (AppWebsocket, HolochainRuntime) {
    let runtime = HolochainRuntime::launch(
        vec_to_locked(vec![]).expect("Could not build passphrase"),
        HolochainRuntimeConfig::new(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            None,
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
                    ..Default::default()
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
    pub infra_provider: (AppWebsocket, HolochainRuntime),
    // pub service_provider: (AppWebsocket, HolochainRuntime),
    pub happ_developer: (AppWebsocket, HolochainRuntime),
    pub sender: (AppWebsocket, HolochainRuntime),
    pub recipient: (AppWebsocket, HolochainRuntime),
}

pub async fn setup() -> Scenario {
    Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(env_logger::Target::Stdout)
        .filter(None, Level::Info.to_level_filter())
        .filter_module("holochain_sqlite", log::LevelFilter::Off)
        .filter_module("tracing::span", log::LevelFilter::Off)
        .init();

    let infra_provider = launch_infra_provider().await;
    let infra_provider_pubkey = infra_provider.0.my_pub_key.clone();
    tokio::spawn(async move {
        run::<MockFcmClient>(
            tempdir::TempDir::new("test")
                .expect("Could not make tempdir")
                .into_path(),
            None,
            service_provider_happ_path(),
            vec![infra_provider_pubkey],
        )
        .await
        .unwrap();
    });
    let happ_developer = launch(
        infra_provider.0.my_pub_key.clone(),
        vec![String::from("service_providers")],
        happ_developer_happ_path(),
    )
    .await;
    let sender = launch(
        infra_provider.0.my_pub_key.clone(),
        vec![String::from("service_providers")],
        end_user_happ_path(),
    )
    .await;
    let recipient = launch(
        infra_provider.0.my_pub_key.clone(),
        vec![String::from("service_providers")],
        end_user_happ_path(),
    )
    .await;

    std::thread::sleep(Duration::from_secs(15));

    Scenario {
        infra_provider,
        // service_provider,
        happ_developer,
        sender,
        recipient,
    }
}
