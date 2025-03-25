use hdk::prelude::*;
use push_notifications_types::{SendPushNotificationSignal, SendPushNotificationToAgentInput};

use crate::{
    fcm_token::get_fcm_token_for_agent, service_account_key::get_current_service_account_key,
};

#[hdk_extern]
pub fn send_push_notification_to_agent(
    input: SendPushNotificationToAgentInput,
) -> ExternResult<()> {
    let Some(token_tag) = get_fcm_token_for_agent(input.agent)? else {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "Agent hasn't registered their FCM token yet"
        ))));
    };

    let Some(service_account_key) = get_current_service_account_key(&token_tag.fcm_project_id)?
    else {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "FCM authority hasn't registered a service account key yet"
        ))));
    };

    let signal = SendPushNotificationSignal {
        token: token_tag.token,
        fcm_project_id: token_tag.fcm_project_id,
        notification: input.notification,
        service_account_key,
    };

    emit_signal(signal)?;

    Ok(())
}
