use hdk::prelude::*;
use push_notifications_service_integrity::*;
use push_notifications_types::RegisterFcmTokenInput;

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct FcmTokenTag {
    pub fcm_project_id: String,
    pub token: String,
}

#[hdk_extern]
pub fn register_fcm_token_for_agent(input: RegisterFcmTokenInput) -> ExternResult<()> {
    let links = get_links(
        GetLinksInputBuilder::try_new(input.agent.clone(), LinkTypes::FcmToken)?.build(),
    )?;

    for link in links {
        delete_link(link.create_link_hash)?;
    }

    let tag = FcmTokenTag {
        fcm_project_id: input.fcm_project_id,
        token: input.token,
    };

    let tag_bytes = SerializedBytes::try_from(tag).map_err(|err| wasm_error!(err))?;

    create_link(
        input.agent.clone(),
        input.agent,
        LinkTypes::FcmToken,
        tag_bytes.bytes().to_vec(),
    )?;

    Ok(())
}

pub fn get_fcm_token_for_agent(agent: AgentPubKey) -> ExternResult<Option<FcmTokenTag>> {
    let links =
        get_links(GetLinksInputBuilder::try_new(agent.clone(), LinkTypes::FcmToken)?.build())?;

    let Some(link) = links.first().cloned() else {
        return Ok(None);
    };

    let token_tag = FcmTokenTag::try_from(SerializedBytes::from(UnsafeBytes::from(link.tag.0)))
        .map_err(|err| wasm_error!(err))?;

    Ok(Some(token_tag))
}
