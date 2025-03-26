use hdk::prelude::*;
use push_notifications_service_providers_manager_integrity::*;

#[hdk_extern]
pub fn create_clone_service_request(
    clone_service_request: CloneServiceRequest,
) -> ExternResult<Record> {
    let clone_service_request_hash = create_entry(&EntryTypes::CloneServiceRequest(
        clone_service_request.clone(),
    ))?;
    let record = get(clone_service_request_hash.clone(), GetOptions::default())?.ok_or(
        wasm_error!(WasmErrorInner::Guest(
            "Could not find the newly created CloneServiceRequest".to_string()
        )),
    )?;
    let path = Path::from("all_clone_service_requests");
    create_link(
        path.path_entry_hash()?,
        clone_service_request_hash.clone(),
        LinkTypes::AllCloneServiceRequests,
        (),
    )?;
    Ok(record)
}

#[hdk_extern]
pub fn get_clone_service_request(
    clone_service_request_hash: ActionHash,
) -> ExternResult<Option<Record>> {
    let Some(details) = get_details(clone_service_request_hash, GetOptions::default())? else {
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
pub fn delete_clone_service_request(
    original_clone_service_request_hash: ActionHash,
) -> ExternResult<ActionHash> {
    let path = Path::from("all_clone_service_requests");
    let links = get_links(
        GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::AllCloneServiceRequests)?
            .build(),
    )?;
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if hash == original_clone_service_request_hash {
                delete_link(link.create_link_hash)?;
            }
        }
    }
    delete_entry(original_clone_service_request_hash)
}

#[hdk_extern]
pub fn get_all_deletes_for_clone_service_request(
    original_clone_service_request_hash: ActionHash,
) -> ExternResult<Option<Vec<SignedActionHashed>>> {
    let Some(details) = get_details(original_clone_service_request_hash, GetOptions::default())?
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
pub fn get_oldest_delete_for_clone_service_request(
    original_clone_service_request_hash: ActionHash,
) -> ExternResult<Option<SignedActionHashed>> {
    let Some(mut deletes) =
        get_all_deletes_for_clone_service_request(original_clone_service_request_hash)?
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
