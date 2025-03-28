use hc_zome_traits::*;
use hdk::prelude::*;
pub use push_notifications_types::SendPushNotificationToAgentInput;

#[zome_trait]
pub trait PushNotificationsService {
    fn send_push_notification(input: SendPushNotificationToAgentInput) -> ExternResult<()>;
}
