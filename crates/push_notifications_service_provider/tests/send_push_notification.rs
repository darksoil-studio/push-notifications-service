use std::time::Duration;

mod common;
use common::*;
use holochain::prelude::DnaModifiers;
use holochain_client::{AgentPubKey, ExternIO, SerializedBytes, Timestamp, ZomeCallTarget};
use push_notifications_types::{
    CloneServiceRequest, PublishServiceAccountKeyInput, ServiceAccountKey,
};
use roles_types::Properties;

#[tokio::test(flavor = "multi_thread")]
async fn send_push_notification() {
    let Scenario {
        infra_provider,
        // service_provider,
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

    let input = PublishServiceAccountKeyInput {
        fcm_project_id: String::from("FCM_PROJECT_1"),
        service_account_key: ServiceAccountKey {
            key_type: None,
            project_id: Some(String::from("FCM_PROJECT_1")),
            private_key_id: None,
            client_id: None,
            auth_uri: None,
            auth_provider_x509_cert_url: None,
            client_x509_cert_url: None,
            private_key: String::from("private_key_1"),
            client_email: String::from("random@email.com"),
            token_uri: String::from("random://token.uri"),
        },
    };

    infra_provider
        .0
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            "push_notifications_service".into(),
            "publish_service_account_key".into(),
            ExternIO::encode(input).unwrap(),
        )
        .await
        .unwrap();

    infra_provider
        .0
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            "push_notifications_service_providers_manager".into(),
            "create_clone_service_request".into(),
            ExternIO::encode(CloneServiceRequest {
                dna_modifiers: modifiers,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    std::thread::sleep(Duration::from_secs(3));

    let push_notifications_service_trait_service_id =
        push_notifications_service_trait::PUSH_NOTIFICATIONS_SERVICE_HASH.to_vec();

    let service_providers: Vec<AgentPubKey> = happ_developer
        .0
        .call_zome(
            ZomeCallTarget::RoleName("service_providers".into()),
            "service_providers".into(),
            "get_providers_for_service".into(),
            ExternIO::encode(push_notifications_service_trait_service_id.clone()).unwrap(),
        )
        .await
        .unwrap()
        .decode()
        .unwrap();

    let service_providers: Vec<AgentPubKey> = happ_developer
        .0
        .call_zome(
            ZomeCallTarget::RoleName("service_providers".into()),
            "service_providers".into(),
            "get_providers_for_service".into(),
            ExternIO::encode(push_notifications_service_trait_service_id).unwrap(),
        )
        .await
        .unwrap()
        .decode()
        .unwrap();

    assert_eq!(service_providers.len(), 1)
}
