use hdk::prelude::*;
use push_notifications_service_providers_manager_integrity::*;

#[hdk_extern]
pub fn get_all_clone_service_requests() -> ExternResult<Vec<Link>> {
    let path = Path::from("all_clone_service_requests");
    get_links(
        GetLinksInputBuilder::try_new(path.path_entry_hash()?, LinkTypes::AllCloneServiceRequests)?
            .build(),
    )
}
