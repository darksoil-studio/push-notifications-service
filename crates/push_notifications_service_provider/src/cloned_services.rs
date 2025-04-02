use anyhow::anyhow;
use clone_manager_types::CloneRequest;
use holochain::prelude::{
    CloneCellId, CreateCloneCellPayload, DnaModifiers, DnaModifiersOpt, EnableCloneCellPayload,
    YamlProperties,
};
use holochain_client::{AppWebsocket, CellInfo, ExternIO, ZomeCallTarget};

const SERVICE_PROVIDERS_ROLE_NAME: &'static str = "service_providers";

pub async fn reconcile_cloned_services(app_ws: &AppWebsocket) -> anyhow::Result<()> {
    let clone_requests: Vec<CloneRequest> = app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            "clone_manager".into(),
            "get_all_clone_requests".into(),
            ExternIO::encode(())?,
        )
        .await?
        .decode()?;

    let Some(app_info) = app_ws.app_info().await? else {
        return Err(anyhow!("App is not installed."));
    };

    let service_providers_cells = app_info
        .cell_info
        .get("service_providers")
        .cloned()
        .unwrap_or(vec![]);

    for clone_request in clone_requests {
        let existing_clone = service_providers_cells
            .iter()
            .find(|cell| dna_modifiers(cell).eq(&clone_request.dna_modifiers));

        if let None = existing_clone {
            clone_service(&app_ws, clone_request).await?;
        }
    }

    Ok(())
}

pub fn dna_modifiers(cell: &CellInfo) -> DnaModifiers {
    match cell {
        CellInfo::Provisioned(provisioned) => provisioned.dna_modifiers.clone(),
        CellInfo::Cloned(cloned) => cloned.dna_modifiers.clone(),
        CellInfo::Stem(stem) => stem.dna_modifiers.clone(),
    }
}

pub async fn clone_service(
    app_ws: &AppWebsocket,
    clone_request: CloneRequest,
) -> anyhow::Result<()> {
    let properties = YamlProperties::try_from(clone_request.dna_modifiers.properties)?;

    log::info!(
        "New CloneRequest received. Cloning the {} role.",
        SERVICE_PROVIDERS_ROLE_NAME
    );

    let cell = app_ws
        .create_clone_cell(CreateCloneCellPayload {
            role_name: SERVICE_PROVIDERS_ROLE_NAME.into(),
            modifiers: DnaModifiersOpt {
                network_seed: Some(clone_request.dna_modifiers.network_seed.clone()),
                origin_time: Some(clone_request.dna_modifiers.origin_time),
                quantum_time: Some(clone_request.dna_modifiers.quantum_time),
                properties: Some(properties.clone()),
            },
            membrane_proof: None,
            name: None,
        })
        .await?;
    app_ws
        .enable_clone_cell(EnableCloneCellPayload {
            clone_cell_id: CloneCellId::CloneId(cell.clone_id.clone()),
        })
        .await?;

    app_ws
        .call_zome(
            ZomeCallTarget::CellId(cell.cell_id.clone()),
            "gateway".into(),
            "init".into(),
            ExternIO::encode(())?,
        )
        .await?;
    log::info!("New cloned cell: {cell:?}.");

    Ok(())
}
