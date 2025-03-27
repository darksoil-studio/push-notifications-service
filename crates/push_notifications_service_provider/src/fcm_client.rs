use push_notifications_types::PushNotification;
use serde_json::{Map, Value};
use std::{collections::HashMap, time::Duration};

use fcm_v1::{
    android::AndroidConfig, apns::ApnsConfig, auth::Authenticator, message::Message, Client,
};

use mockall::predicate::*;
use mockall::*;

// We extract the actual calls to FCM to make our code testable
#[automock]
pub trait FcmClient {
    fn validate_fcm_project(
        fcm_project_id: String,
        service_account_key: yup_oauth2::ServiceAccountKey,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn send_push_notification(
        fcm_project_id: String,
        service_account_key: yup_oauth2::ServiceAccountKey,
        token: String,
        push_notification: PushNotification,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

pub struct RealFcmClient;

impl FcmClient for RealFcmClient {
    async fn validate_fcm_project(
        fcm_project_id: String,
        service_account_key: yup_oauth2::ServiceAccountKey,
    ) -> anyhow::Result<()> {
        let auth = Authenticator::service_account::<String>(service_account_key).await?;

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

    async fn send_push_notification(
        fcm_project_id: String,
        service_account_key: yup_oauth2::ServiceAccountKey,
        token: String,
        push_notification: PushNotification,
    ) -> anyhow::Result<()> {
        let auth = Authenticator::service_account::<String>(service_account_key).await?;

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
}
