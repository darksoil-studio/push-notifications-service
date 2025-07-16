use hdk::prelude::*;
use push_notifications_service_integrity::*;

fn fcm_project_path(fcm_project_id: &String) -> ExternResult<TypedPath> {
    Path::from(format!("fcm_projects.{}", fcm_project_id)).typed(LinkTypes::FcmProjectPath)
}

fn fcm_projects_path() -> ExternResult<TypedPath> {
    Path::from("fcm_projects").typed(LinkTypes::FcmProjectPath)
}

#[hdk_extern]
pub fn publish_service_account_key(service_account_key: ServiceAccountKey) -> ExternResult<()> {
    let Some(project_id) = service_account_key.project_id.clone() else {
        return Err(wasm_error!(
            "Invalid ServiceAccountKey: project_id is null."
        ));
    };
    delete_all_service_account_keys(&project_id)?;
    let path = fcm_project_path(&project_id)?;
    path.ensure()?;

    let links = get_links(
        GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::ServiceAccountKeys)?
            .build(),
    )?;

    for link in links {
        get(link.create_link_hash.clone(), Default::default())?;
        delete_link(link.create_link_hash)?;
    }

    let action_hash = create_entry(EntryTypes::ServiceAccountKey(service_account_key))?;

    create_link(
        path.path_entry_hash()?,
        action_hash,
        LinkTypes::ServiceAccountKeys,
        (),
    )?;

    info!("Created new service account key for project {project_id}");

    Ok(())
}

fn delete_all_service_account_keys(fcm_project_id: &String) -> ExternResult<()> {
    let path = fcm_project_path(fcm_project_id)?;

    let links = get_links(
        GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::ServiceAccountKeys)?
            .build(),
    )?;

    for link in links {
        delete_link(link.create_link_hash)?;
    }
    Ok(())
}

#[hdk_extern]
pub fn delete_fcm_project(fcm_project_id: String) -> ExternResult<()> {
    delete_all_service_account_keys(&fcm_project_id)?;

    let path = fcm_projects_path()?;
    let fcm_project_path_entry_hash = fcm_project_path(&fcm_project_id)?.path_entry_hash()?;

    let paths = path.children()?;
    let links_to_delete: Vec<Link> = paths
        .into_iter()
        .filter(|link| {
            let Some(entry_hash) = link.target.clone().into_entry_hash() else {
                return false;
            };
            entry_hash.eq(&fcm_project_path_entry_hash)
        })
        .collect();

    for link in links_to_delete {
        delete_link(link.create_link_hash)?;
    }

    Ok(())
}

#[hdk_extern]
pub fn get_all_fcm_projects() -> ExternResult<Vec<String>> {
    let path = fcm_projects_path()?;

    let paths = path.children_paths()?;
    let fcm_projects: Vec<String> = paths
        .into_iter()
        .filter_map(|path| path.leaf().cloned())
        .filter_map(|component| String::try_from(&component).ok())
        .collect();

    Ok(fcm_projects)
}

#[hdk_extern]
pub fn get_current_service_account_key(
    fcm_project_id: String,
) -> ExternResult<Option<ServiceAccountKey>> {
    let links = get_links(
        GetLinksInputBuilder::try_new(
            fcm_project_path(&fcm_project_id)?.path_entry_hash()?,
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
