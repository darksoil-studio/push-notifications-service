use hdk::prelude::*;
use push_notifications_service_integrity::*;

#[hdk_extern]
pub fn create_service_account_key(service_account_key: ServiceAccountKey) -> ExternResult<Record> {
    let service_account_key_hash =
        create_entry(&EntryTypes::ServiceAccountKey(service_account_key.clone()))?;
    let record =
        get(service_account_key_hash.clone(), GetOptions::default())?.ok_or(wasm_error!(
            WasmErrorInner::Guest("Could not find the newly created ServiceAccountKey".to_string())
        ))?;
    Ok(record)
}

#[hdk_extern]
pub fn get_service_account_key(
    service_account_key_hash: ActionHash,
) -> ExternResult<Option<Record>> {
    let Some(details) = get_details(service_account_key_hash, GetOptions::default())? else {
        return Ok(None);
    };
    match details {
        Details::Record(details) => Ok(Some(details.record)),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Malformed get details response".to_string()
        ))),
    }
}

#[hdk_extern]
pub fn delete_service_account_key(
    original_service_account_key_hash: ActionHash,
) -> ExternResult<ActionHash> {
    delete_entry(original_service_account_key_hash)
}

#[hdk_extern]
pub fn get_all_deletes_for_service_account_key(
    original_service_account_key_hash: ActionHash,
) -> ExternResult<Option<Vec<SignedActionHashed>>> {
    let Some(details) = get_details(original_service_account_key_hash, GetOptions::default())?
    else {
        return Ok(None);
    };
    match details {
        Details::Entry(_) => Err(wasm_error!(WasmErrorInner::Guest(
            "Malformed details".into()
        ))),
        Details::Record(record_details) => Ok(Some(record_details.deletes)),
    }
}

#[hdk_extern]
pub fn get_oldest_delete_for_service_account_key(
    original_service_account_key_hash: ActionHash,
) -> ExternResult<Option<SignedActionHashed>> {
    let Some(mut deletes) =
        get_all_deletes_for_service_account_key(original_service_account_key_hash)?
    else {
        return Ok(None);
    };
    deletes.sort_by(|delete_a, delete_b| {
        delete_a
            .action()
            .timestamp()
            .cmp(&delete_b.action().timestamp())
    });
    Ok(deletes.first().cloned())
}
