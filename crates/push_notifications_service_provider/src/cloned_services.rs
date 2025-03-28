use anyhow::anyhow;
use holochain::prelude::{
    CloneCellId, CreateCloneCellPayload, DnaModifiers, DnaModifiersOpt, YamlProperties,
};
use holochain_client::{AppWebsocket, CellInfo, ExternIO, InstalledAppId, ZomeCallTarget};
use holochain_runtime::HolochainRuntime;
use push_notifications_types::CloneServiceRequest;

const SERVICE_PROVIDERS_ROLE_NAME: &'static str = "service_providers";

pub async fn reconcile_cloned_services(
    runtime: &HolochainRuntime,
    app_id: &InstalledAppId,
) -> anyhow::Result<()> {
    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;

    let clone_service_requests: Vec<CloneServiceRequest> = app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            "push_notifications_service_providers_manager".into(),
            "get_all_clone_service_requests".into(),
            ExternIO::encode(())?,
        )
        .await?
        .decode()?;

    let Some(app_info) = app_ws.app_info().await? else {
        return Err(anyhow!("App is not installed {app_id}"));
    };

    let service_providers_cells = app_info
        .cell_info
        .get("service_providers")
        .cloned()
        .unwrap_or(vec![]);

    for clone_service_request in clone_service_requests {
        let existing_clone = service_providers_cells.iter().find(|cell| {
            dna_modifiers(cell.clone().clone()).eq(&clone_service_request.dna_modifiers)
        });

        if let None = existing_clone {
            clone_service(&app_ws, clone_service_request).await?;
        }
    }

    Ok(())
}

fn dna_modifiers(cell: CellInfo) -> DnaModifiers {
    match cell {
        CellInfo::Provisioned(provisioned) => provisioned.dna_modifiers,
        CellInfo::Cloned(cloned) => cloned.dna_modifiers,
        CellInfo::Stem(stem) => stem.dna_modifiers,
    }
}

pub async fn clone_service(
    app_ws: &AppWebsocket,
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
    Ok(())
}
