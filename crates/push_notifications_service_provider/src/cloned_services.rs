use holochain::prelude::{CreateCloneCellPayload, DnaModifiersOpt, YamlProperties};
use holochain_client::{AppWebsocket, ExternIO, ZomeCallTarget};
use push_notifications_types::CloneServiceRequest;

const SERVICE_PROVIDERS_ROLE_NAME: &'static str = "service_providers";
const PUSH_NOTIFICATIONS_SERVICE_ROLE_NAME: &'static str = "push_notifications_service";

pub async fn reconcile_cloned_services(app_ws: &AppWebsocket) -> anyhow::Result<()> {
    app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service_providers_manager".into()),
            "push_notifications_service_providers_manager".into(),
            "get_all_clone_service_requests".into(),
            ExternIO::encode(())?,
        )
        .await?;

    Ok(())
}

pub async fn clone_service(
    app_ws: AppWebsocket,
    clone_service_request: CloneServiceRequest,
) -> anyhow::Result<()> {
    let properties = YamlProperties::try_from(clone_service_request.dna_modifiers.properties)?;
    app_ws
        .create_clone_cell(CreateCloneCellPayload {
            role_name: SERVICE_PROVIDERS_ROLE_NAME.into(),
            modifiers: DnaModifiersOpt {
                network_seed: Some(clone_service_request.dna_modifiers.network_seed.clone()),
                origin_time: Some(clone_service_request.dna_modifiers.origin_time),
                quantum_time: Some(clone_service_request.dna_modifiers.quantum_time),
                properties: Some(properties.clone()),
            },
            membrane_proof: None,
            name: None,
        })
        .await?;
    app_ws
        .create_clone_cell(CreateCloneCellPayload {
            role_name: PUSH_NOTIFICATIONS_SERVICE_ROLE_NAME.into(),
            modifiers: DnaModifiersOpt {
                network_seed: Some(clone_service_request.dna_modifiers.network_seed),
                origin_time: Some(clone_service_request.dna_modifiers.origin_time),
                quantum_time: Some(clone_service_request.dna_modifiers.quantum_time),
                properties: Some(properties),
            },
            membrane_proof: None,
            name: None,
        })
        .await?;
    Ok(())
}
