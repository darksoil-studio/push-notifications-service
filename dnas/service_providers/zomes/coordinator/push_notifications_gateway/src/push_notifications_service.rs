use hc_zome_traits::{implement_zome_trait_as_externs, implemented_zome_traits};
use hdk::prelude::*;
use push_notifications_service_trait::{
    PushNotificationsService, SendPushNotificationToAgentInput,
};

#[implemented_zome_traits]
pub enum ZomeTraits {
    PushNotifications(PushNotificationsGateway),
}

pub struct PushNotificationsGateway;

#[implement_zome_trait_as_externs]
impl PushNotificationsService for PushNotificationsGateway {
    fn send_push_notification(input: SendPushNotificationToAgentInput) -> ExternResult<()> {
        Ok(())
    }
}
