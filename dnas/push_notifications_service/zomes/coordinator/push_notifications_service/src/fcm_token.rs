use hdk::prelude::*;
use push_notifications_service_integrity::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFcmTokenForAgentInput {
    pub agent: AgentPubKey,
    pub fcm_token: String,
}

#[hdk_extern]
pub fn add_fcm_token_for_agent(input: AddFcmTokenForAgentInput) -> ExternResult<()> {
    create_link(
        input.agent.clone(),
        input.agent,
        LinkTypes::FcmToken,
        input.fcm_token,
    )?;
    Ok(())
}

#[hdk_extern]
pub fn get_fcm_tokens_for_agent(agent: AgentPubKey) -> ExternResult<Vec<String>> {
    let links = get_links(GetLinksInputBuilder::try_new(agent, LinkTypes::FcmToken)?.build())?;
    let fcm_token = links
        .into_iter()
        .map(|link| {
            String::from_utf8(link.tag.into_inner()).map_err(|e| {
                wasm_error!(WasmErrorInner::Guest(format!(
                    "Error converting link tag to string: {:?}",
                    e
                )))
            })
        })
        .collect::<ExternResult<Vec<String>>>()?;
    Ok(fcm_token)
}
