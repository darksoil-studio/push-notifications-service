use hdi::prelude::*;
pub use push_notifications_types::CloneServiceRequest;

pub fn validate_create_clone_service_request(
    _action: EntryCreationAction,
    _clone_service_request: CloneServiceRequest,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_clone_service_request(
    _action: Update,
    _clone_service_request: CloneServiceRequest,
    _original_action: EntryCreationAction,
    _original_clone_service_request: CloneServiceRequest,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(
        "Clone Service Requests cannot be updated".to_string(),
    ))
}

pub fn validate_delete_clone_service_request(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_clone_service_request: CloneServiceRequest,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_create_link_all_clone_service_requests(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash =
        target_address
            .into_action_hash()
            .ok_or(wasm_error!(WasmErrorInner::Guest(
                "No action hash associated with link".to_string()
            )))?;
    let record = must_get_valid_record(action_hash)?;
    let _clone_service_request: crate::CloneServiceRequest = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Linked action must reference an entry".to_string()
        )))?;
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_all_clone_service_requests(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}
