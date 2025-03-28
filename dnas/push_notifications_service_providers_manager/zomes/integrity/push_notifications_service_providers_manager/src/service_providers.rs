use hdi::prelude::*;
pub use push_notifications_types::CloneServiceRequest;

pub fn validate_create_link_service_providers(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let Some(_) = base_address.into_entry_hash() else {
        return Ok(ValidateCallbackResult::Invalid(String::from(
            "Base address for a ServiceProviders link must be an entry hash",
        )));
    };
    let Some(_) = target_address.into_agent_pub_key() else {
        return Ok(ValidateCallbackResult::Invalid(String::from(
            "Base address for a ServiceProviders link must be an AgentPubKey",
        )));
    };
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_service_providers(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}
