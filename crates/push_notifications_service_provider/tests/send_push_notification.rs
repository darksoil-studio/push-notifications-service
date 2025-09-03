use std::time::Duration;

mod common;
use anyhow::anyhow;
use common::*;
use holochain_client::{AgentPubKey, ExternIO, ZomeCallTarget};
use push_notifications_service_client::{into, PushNotificationsServiceClient};
use push_notifications_service_provider::{fcm_client::MockFcmClient, SERVICES_ROLE_NAME};
use push_notifications_types::{
    PushNotification, RegisterFcmTokenInput, SendPushNotificationToAgentInput, ServiceAccountKey,
};
use service_providers_utils::make_service_request;
use tempdir::TempDir;

#[tokio::test(flavor = "multi_thread")]
async fn send_push_notification() {
    let Scenario {
        network_seed,
        bootstrap_srv,
        progenitors,
        sender,
        recipient,
    } = setup().await;

    let fcm_project_id = String::from("FCM_PROJECT_1");

    let service_account_key = ServiceAccountKey {
        key_type: None,
        project_id: Some(fcm_project_id.clone()),
        private_key_id: None,
        client_id: None,
        auth_uri: None,
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
        private_key: String::from("private_key_1"),
        client_email: String::from("random@email.com"),
        token_uri: String::from("random://token.uri"),
    };

    let tmp = TempDir::new("pns").unwrap();

    let client = PushNotificationsServiceClient::create(
        tmp.path().to_path_buf(),
        network_config(&bootstrap_srv),
        "client-happ".into(),
        client_happ_path(),
        progenitors,
        false,
    )
    .await
    .unwrap();

    with_retries(
        async || {
            client
                .publish_service_account_key(into(service_account_key.clone()))
                .await
                .unwrap();
            Ok(())
        },
        5,
    )
    .await
    .unwrap();

    client.create_clone_request(network_seed).await.unwrap();

    let push_notifications_service_trait_service_id =
        push_notifications_service_trait::PUSH_NOTIFICATIONS_SERVICE_HASH.to_vec();

    with_retries(
        async || {
            let service_providers: Vec<AgentPubKey> = recipient
                .0
                .call_zome(
                    ZomeCallTarget::RoleName(SERVICES_ROLE_NAME.into()),
                    "service_providers".into(),
                    "get_providers_for_service".into(),
                    ExternIO::encode(push_notifications_service_trait_service_id.clone()).unwrap(),
                )
                .await?
                .decode()?;
            if service_providers.is_empty() {
                return Err(anyhow!("No service providers yet"));
            }
            Ok(())
        },
        60,
    )
    .await
    .unwrap();

    let token = String::from("myfcmtoken");

    let _response: () = make_service_request(
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

    std::thread::sleep(Duration::from_secs(5));

    let ctx = MockFcmClient::send_push_notification_context();
    ctx.expect().once().returning(
        |_fcm_project_id, _service_account_key, _token, _push_notification| {
            Box::pin(async { Ok(()) })
        },
    );

    with_retries(
        async || {
            let service_providers: Vec<AgentPubKey> = sender
                .0
                .call_zome(
                    ZomeCallTarget::RoleName(SERVICES_ROLE_NAME.into()),
                    "service_providers".into(),
                    "get_providers_for_service".into(),
                    ExternIO::encode(push_notifications_service_trait_service_id.clone()).unwrap(),
                )
                .await?
                .decode()?;
            if service_providers.is_empty() {
                return Err(anyhow!("No service providers yet"));
            }
            Ok(())
        },
        20,
    )
    .await
    .unwrap();

    let _response: () = make_service_request(
        &sender.0,
        push_notifications_service_trait_service_id,
        "send_push_notifications".into(),
        vec![SendPushNotificationToAgentInput {
            agent: recipient.0.my_pub_key.clone(),
            notification: PushNotification {
                title: String::from("Hey"),
                body: String::from("there"),
            },
        }],
    )
    .await
    .unwrap();

    std::thread::sleep(Duration::from_secs(5));
    ctx.checkpoint();
}
