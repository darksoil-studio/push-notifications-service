use push_notifications_types::PushNotification;
use serde_json::{Map, Value};
use std::{collections::HashMap, time::Duration};

use fcm_v1::{
    android::AndroidConfig, apns::ApnsConfig, auth::Authenticator, message::Message, Client,
};
use yup_oauth2::ServiceAccountKey;

pub async fn validate_fcm_project(
    fcm_project_id: String,
    service_account_key: push_notifications_types::ServiceAccountKey,
) -> anyhow::Result<()> {
    let auth = Authenticator::service_account::<String>(into(service_account_key)).await?;

    let client = Client::new(auth, fcm_project_id, false, Duration::from_secs(2));

    let mut message = Message::default();

    let mut map = HashMap::new();
    map.insert(
        "title".to_string(),
        Value::String(String::from("This is a test notification")),
    );
    map.insert(
        "body".to_string(),
        Value::String(String::from("This is a test notification")),
    );
    message.data = Some(map.clone());
    let mut apns_config = ApnsConfig::default();

    let mut alert_data = Map::new();
    alert_data.insert(
        "title".to_string(),
        Value::String(String::from("This is a test notification")),
    );
    alert_data.insert(
        "body".to_string(),
        Value::String(String::from("This is a test notification")),
    );

    let mut aps_data = Map::new();
    aps_data.insert("alert".to_string(), Value::Object(alert_data.clone()));
    aps_data.insert("mutable-content".to_string(), Value::Number(1.into()));
    let mut apns_data = HashMap::new();
    apns_data.insert("aps".to_string(), Value::Object(aps_data));
    apns_config.payload = Some(apns_data);

    message.apns = Some(apns_config);

    let mut android_config = AndroidConfig::default();
    android_config.data = Some(map);

    message.android = Some(android_config);
    message.topic = Some(String::from("test"));

    client.send(&message).await?;

    Ok(())
}

pub async fn send_push_notification(
    fcm_project_id: String,
    service_account_key: push_notifications_types::ServiceAccountKey,
    token: String,
    push_notification: PushNotification,
) -> anyhow::Result<()> {
    let auth = Authenticator::service_account::<String>(into(service_account_key)).await?;

    let client = Client::new(auth, fcm_project_id, false, Duration::from_secs(2));

    let mut message = Message::default();

    let mut map = HashMap::new();
    map.insert(
        "title".to_string(),
        Value::String(push_notification.title.clone()),
    );
    map.insert(
        "body".to_string(),
        Value::String(push_notification.body.clone()),
    );
    message.data = Some(map.clone());
    let mut apns_config = ApnsConfig::default();

    let mut alert_data = Map::new();
    alert_data.insert(
        "title".to_string(),
        Value::String(push_notification.title.clone()),
    );
    alert_data.insert(
        "body".to_string(),
        Value::String(push_notification.body.clone()),
    );

    let mut aps_data = Map::new();
    aps_data.insert("alert".to_string(), Value::Object(alert_data.clone()));
    aps_data.insert("mutable-content".to_string(), Value::Number(1.into()));
    let mut apns_data = HashMap::new();
    apns_data.insert("aps".to_string(), Value::Object(aps_data));
    apns_config.payload = Some(apns_data);

    message.apns = Some(apns_config);

    let mut android_config = AndroidConfig::default();
    android_config.data = Some(map);

    message.android = Some(android_config);
    message.token = Some(token);

    client.send(&message).await?;

    Ok(())
}

fn into(key: push_notifications_types::ServiceAccountKey) -> ServiceAccountKey {
    ServiceAccountKey {
        key_type: key.key_type,
        project_id: key.project_id,
        private_key_id: key.private_key_id,
        private_key: key.private_key,
        client_email: key.client_email,
        client_id: key.client_id,
        auth_uri: key.auth_uri,
        token_uri: key.token_uri,
        auth_provider_x509_cert_url: key.auth_provider_x509_cert_url,
        client_x509_cert_url: key.client_x509_cert_url,
    }
}
