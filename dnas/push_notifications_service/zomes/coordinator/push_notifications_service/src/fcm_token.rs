use hdk::prelude::*;
use push_notifications_service_integrity::*;
use push_notifications_types::RegisterFcmTokenForAgentInput;

#[derive(Serialize, Deserialize, Debug, SerializedBytes, PartialEq)]
pub struct FcmTokenTag {
    pub fcm_project_id: String,
    pub token: String,
}

#[hdk_extern]
pub fn register_fcm_token_for_agent(input: RegisterFcmTokenForAgentInput) -> ExternResult<()> {
    let tag = FcmTokenTag {
        fcm_project_id: input.fcm_project_id,
        token: input.token,
    };

    if let Some(current_token) = get_fcm_token_for_agent(input.agent.clone())? {
        if current_token.eq(&tag) {
            // Token was already in our service: nothing to do
            return Ok(());
        }
    }

    let links = get_links(
        LinkQuery::try_new(input.agent.clone(), LinkTypes::FcmToken)?,
        GetStrategy::Network,
    )?;

    for link in links {
        delete_link(link.create_link_hash, GetOptions::network())?;
    }

    let tag_bytes = SerializedBytes::try_from(tag).map_err(|err| wasm_error!(err))?;

    create_link(
        input.agent.clone(),
        input.agent.clone(),
        LinkTypes::FcmToken,
        tag_bytes.bytes().to_vec(),
    )?;

    info!("Registered new fcm token for agent: {}", input.agent);

    Ok(())
}

pub fn get_fcm_token_for_agent(agent: AgentPubKey) -> ExternResult<Option<FcmTokenTag>> {
    let links = get_links(
        LinkQuery::try_new(agent.clone(), LinkTypes::FcmToken)?,
        GetStrategy::Network,
    )?;

    let Some(link) = links.first().cloned() else {
        return Ok(None);
    };

    let token_tag = FcmTokenTag::try_from(SerializedBytes::from(UnsafeBytes::from(link.tag.0)))
        .map_err(|err| wasm_error!(err))?;

    Ok(Some(token_tag))
}
