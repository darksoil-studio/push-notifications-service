use hdi::prelude::*;
pub use push_notifications_types::ServiceAccountKey;

pub fn validate_create_service_account_key(
    _action: EntryCreationAction,
    _service_account_key: ServiceAccountKey,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_service_account_key(
    _action: Update,
    _service_account_key: ServiceAccountKey,
    _original_action: EntryCreationAction,
    _original_service_account_key: ServiceAccountKey,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(
        "Service Account Keys cannot be updated".to_string(),
    ))
}

pub fn validate_delete_service_account_key(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_service_account_key: ServiceAccountKey,
) -> ExternResult<ValidateCallbackResult> {
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}
