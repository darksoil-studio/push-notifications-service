use std::time::Duration;

mod common;
use clone_manager_types::CloneRequest;
use clone_manager_utils::dna_modifiers;
use common::*;
use holochain::prelude::DnaModifiers;
use holochain_client::{AgentPubKey, ExternIO, SerializedBytes, ZomeCallTarget};
use push_notifications_service_provider::fcm_client::MockFcmClient;
use push_notifications_types::{
    PublishServiceAccountKeyInput, PushNotification, RegisterFcmTokenInput,
    SendPushNotificationToAgentInput, ServiceAccountKey,
};
use roles_types::Properties;
use service_providers_utils::make_service_request;

#[tokio::test(flavor = "multi_thread")]
async fn send_push_notification() {
    let Scenario {
        infra_provider,
        // service_provider,
        happ_developer,
        sender,
        recipient,
    } = setup().await;

    let app_info = happ_developer.0.app_info().await.unwrap().unwrap();
    let cell = app_info
        .cell_info
        .get("service_providers")
        .unwrap()
        .first()
        .unwrap();
    let origin_time = dna_modifiers(cell).origin_time;

    let roles_properties = Properties {
        progenitors: vec![infra_provider.0.my_pub_key.clone().into()],
    };
    let properties_bytes = SerializedBytes::try_from(roles_properties).unwrap();
    let modifiers = DnaModifiers {
        properties: properties_bytes,
        network_seed: String::from(""),
        origin_time,
        quantum_time: Duration::from_secs(60 * 5),
    };

    let fcm_project_id = String::from("FCM_PROJECT_1");

    let input = PublishServiceAccountKeyInput {
        fcm_project_id: fcm_project_id.clone(),
        service_account_key: ServiceAccountKey {
            key_type: None,
            project_id: Some(String::from("GOOGLE_CLOUD_PROJECT_1")),
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

    let service_providers: Vec<AgentPubKey> = infra_provider
        .0
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            "clone_manager".into(),
            "get_clone_providers".into(),
            ExternIO::encode(()).unwrap(),
        )
        .await
        .unwrap()
        .decode()
        .unwrap();

    assert_eq!(service_providers.len(), 1);

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
            "clone_manager".into(),
            "create_clone_request".into(),
            ExternIO::encode(CloneRequest {
                dna_modifiers: modifiers,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    std::thread::sleep(Duration::from_secs(15));

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

    assert_eq!(service_providers.len(), 1);

    let token = String::from("myfcmtoken");

    let response: () = make_service_request(
        &recipient.0,
        push_notifications_service_trait_service_id.clone(),
        "register_fcm_token".into(),
        RegisterFcmTokenInput {
            fcm_project_id: fcm_project_id.clone(),
            token: token.clone(),
        },
    )
    .await
    .unwrap();

    let ctx = MockFcmClient::send_push_notification_context();
    ctx.expect().once().returning(
        |fcm_project_id, service_account_key, token, push_notification| Box::pin(async { Ok(()) }),
    );

    let response: () = make_service_request(
        &sender.0,
        push_notifications_service_trait_service_id,
        "send_push_notification".into(),
        SendPushNotificationToAgentInput {
            agent: recipient.0.my_pub_key.clone(),
            notification: PushNotification {
                title: String::from("Hey"),
                body: String::from("there"),
            },
        },
    )
    .await
    .unwrap();

    std::thread::sleep(Duration::from_secs(1));
    ctx.checkpoint();
}
