use hdk::prelude::*;
use push_notifications_service_trait::PUSH_NOTIFICATIONS_SERVICE_HASH;

mod push_notifications_service;

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    error!("[init] Running init");

    let mut fns: BTreeSet<GrantedFunction> = BTreeSet::new();
    fns.insert((zome_info()?.name, FunctionName::from("register_fcm_token")));
    fns.insert((
        zome_info()?.name,
        FunctionName::from("send_push_notifications"),
    ));
    let functions = GrantedFunctions::Listed(fns);
    let cap_grant = ZomeCallCapGrant {
        tag: String::from("send_push_notification"),
        access: CapAccess::Unrestricted,
        functions,
    };
    create_cap_grant(cap_grant)?;
    error!("[init] Created cap grant");

    let response = call(
        CallTargetCell::Local,
        ZomeName::from("service_providers"),
        "announce_as_provider".into(),
        None,
        PUSH_NOTIFICATIONS_SERVICE_HASH,
    )?;
    error!("[init] Response: {response:?}");
    let ZomeCallResponse::Ok(_) = response else {
        error!("[init] Init failed.");
        return Ok(InitCallbackResult::Fail(format!(
            "Failed to announce as provider: {response:?}"
        )));
    };
    error!("[init] Init passed.");

    Ok(InitCallbackResult::Pass)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Signal {}

#[hdk_extern(infallible)]
pub fn post_commit(committed_actions: Vec<SignedActionHashed>) {
    for action in committed_actions {
        if let Err(err) = signal_action(action) {
            error!("Error signaling new action: {:?}", err);
        }
    }
}
fn signal_action(_action: SignedActionHashed) -> ExternResult<()> {
    Ok(())
}
