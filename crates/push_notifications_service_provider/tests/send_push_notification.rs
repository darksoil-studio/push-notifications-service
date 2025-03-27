use std::{collections::HashMap, path::PathBuf, time::Duration};

mod common;
use common::*;
use holochain::{
    conductor::conductor::hdk::prelude::holochain_zome_types::properties,
    prelude::{DnaModifiers, DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties},
};
use holochain_client::{ExternIO, SerializedBytes, Timestamp, ZomeCallTarget};
use holochain_runtime::{vec_to_locked, HolochainRuntime, HolochainRuntimeConfig};
use push_notifications_service_provider::read_from_file;
use push_notifications_types::CloneServiceRequest;
use roles_types::Properties;

#[tokio::test(flavor = "multi_thread")]
async fn send_push_notification() {
    let Scenario {
        infra_provider,
        service_provider,
        happ_developer,
        sender,
        recipient,
    } = setup().await;

    let roles_properties = Properties {
        progenitors: vec![infra_provider.0.my_pub_key.clone().into()],
    };
    let properties_bytes = SerializedBytes::try_from(roles_properties).unwrap();
    let modifiers = DnaModifiers {
        properties: properties_bytes,
        network_seed: String::from(""),
        origin_time: Timestamp::now(),
        quantum_time: Duration::from_secs(60 * 5),
    };

    infra_provider
        .0
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service_providers_manager".into()),
            "push_notifications_service_providers_manager".into(),
            "create_clone_service_request".into(),
            ExternIO::encode(CloneServiceRequest {
                dna_modifiers: modifiers,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    std::thread::sleep(Duration::from_secs(1));

    let app_info = service_provider.0.app_info().await.unwrap().unwrap();

    let cells = app_info.cell_info.get("service_providers").unwrap();
    assert_eq!(cells.len(), 2)
}
