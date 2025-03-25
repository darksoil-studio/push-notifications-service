use hdk::prelude::*;
use push_notifications_service_integrity::*;
use push_notifications_types::PublishServiceAccountKeyInput;

fn service_account_key_path(fcm_project_id: &String) -> Path {
    Path::from(format!("service_account_keys.{}", fcm_project_id))
}

#[hdk_extern]
pub fn publish_service_account_key(input: PublishServiceAccountKeyInput) -> ExternResult<()> {
    let links = get_links(
        GetLinksInputBuilder::try_new(
            service_account_key_path(&input.fcm_project_id).path_entry_hash()?,
            LinkTypes::ServiceAccountKeys,
        )?
        .build(),
    )?;

    for link in links {
        delete_link(link.create_link_hash)?;
    }

    let action_hash = create_entry(EntryTypes::ServiceAccountKey(input.service_account_key))?;

    create_link(
        service_account_key_path(&input.fcm_project_id).path_entry_hash()?,
        action_hash,
        LinkTypes::ServiceAccountKeys,
        (),
    )?;

    Ok(())
}

pub fn get_current_service_account_key(
    fcm_project_id: &String,
) -> ExternResult<Option<ServiceAccountKey>> {
    let links = get_links(
        GetLinksInputBuilder::try_new(
            service_account_key_path(fcm_project_id).path_entry_hash()?,
            LinkTypes::ServiceAccountKeys,
        )?
        .build(),
    )?;

    let Some(link) = links.first().cloned() else {
        return Ok(None);
    };

    let Some(record) = get(
        link.target
            .into_any_dht_hash()
            .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                "Malformed link"
            ))))?,
        GetOptions::default(),
    )?
    else {
        return Ok(None);
    };

    let key: ServiceAccountKey = record
        .entry()
        .as_option()
        .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
            "Malformed key"
        ))))?
        .try_into()?;

    Ok(Some(key))
}
