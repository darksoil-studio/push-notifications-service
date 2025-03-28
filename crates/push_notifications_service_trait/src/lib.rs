use hc_zome_traits::*;
use hdk::prelude::*;
pub use push_notifications_types::{RegisterFcmTokenInput, SendPushNotificationToAgentInput};

#[zome_trait]
pub trait PushNotificationsService {
    fn register_fcm_token(input: RegisterFcmTokenInput) -> ExternResult<()>;

    fn send_push_notification(input: SendPushNotificationToAgentInput) -> ExternResult<()>;
}
