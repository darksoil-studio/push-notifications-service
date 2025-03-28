use hc_zome_traits::*;
use hdk::prelude::*;
use push_notifications_types::ConfigurePushNotificationsInput;
pub use push_notifications_types::SendPushNotificationToAgentInput;

#[zome_trait]
pub trait PushNotificationsService {
    fn send_push_notification(input: SendPushNotificationToAgentInput) -> ExternResult<()>;

    fn configure_push_notifications(input: ConfigurePushNotificationsInput) -> ExternResult<()>;
}
