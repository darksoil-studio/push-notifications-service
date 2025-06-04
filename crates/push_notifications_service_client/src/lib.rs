use anyhow::{anyhow, Result};
use clone_manager_types::CloneRequest;
use fcm_v1::auth::ServiceAccountKey;
use holochain_client::ZomeCallTarget;
use holochain_runtime::*;
use holochain_types::prelude::*;
use push_notifications_types::PublishServiceAccountKeyInput;
use roles_types::Properties;
use setup::setup;
use std::{collections::BTreeMap, fs, path::PathBuf};

mod setup;

pub const SERVICE_PROVIDERS_ROLE_NAME: &'static str = "service_providers";

pub async fn publish_service_account_key(
    data_dir: PathBuf,
    network_config: NetworkConfig,
    app_id: String,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
    fcm_project_id: String,
    service_account_key: ServiceAccountKey,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), network_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![]), config).await?;
    setup(
        &runtime,
        &app_id,
        &push_notifications_service_provider_happ_path,
        progenitors,
    )
    .await?;

    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;

    app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            ZomeName::from("push_notifications_service"),
            "publish_service_account_key".into(),
            ExternIO::encode(PublishServiceAccountKeyInput {
                fcm_project_id: fcm_project_id.clone(),
                service_account_key: from(service_account_key.clone()),
            })?,
        )
        .await?;

    let result = app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            ZomeName::from("push_notifications_service"),
            "get_current_service_account_key".into(),
            ExternIO::encode(fcm_project_id)?,
        )
        .await?;

    let maybe_key: Option<push_notifications_types::ServiceAccountKey> = result.decode()?;

    let Some(key) = maybe_key else {
        return Err(anyhow!("Failed to publish service account key"));
    };

    if key.ne(&from(service_account_key)) {
        return Err(anyhow!("Failed to publish service account key"));
    }

    println!("Successfully uploaded service account key");

    Ok(())
}

pub async fn create_clone_request(
    data_dir: PathBuf,
    network_config: NetworkConfig,
    app_id: String,
    push_notifications_service_provider_happ_path: PathBuf,
    progenitors: Vec<AgentPubKey>,
    network_seed: String,
) -> anyhow::Result<()> {
    let config = HolochainRuntimeConfig::new(data_dir.clone(), network_config);

    let runtime = HolochainRuntime::launch(vec_to_locked(vec![]), config).await?;
    setup(
        &runtime,
        &app_id,
        &push_notifications_service_provider_happ_path,
        progenitors.clone(),
    )
    .await?;

    let app_ws = runtime
        .app_websocket(app_id.clone(), holochain_client::AllowedOrigins::Any)
        .await?;
    let roles_properties = Properties {
        progenitors: progenitors.into_iter().map(|p| p.into()).collect(),
    };
    let properties = SerializedBytes::try_from(roles_properties)?;

    let clone_request = CloneRequest {
        dna_modifiers: DnaModifiers {
            network_seed,
            properties,
        },
    };

    app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            ZomeName::from("clone_manager"),
            "create_clone_request".into(),
            ExternIO::encode(clone_request.clone())?,
        )
        .await?;

    let result = app_ws
        .call_zome(
            ZomeCallTarget::RoleName("push_notifications_service".into()),
            ZomeName::from("create_clone_request"),
            "get_all_clone_requests".into(),
            ExternIO::encode(())?,
        )
        .await?;

    let all_clone_requests: BTreeMap<EntryHashB64, CloneRequest> = result.decode()?;

    if !all_clone_requests
        .into_values()
        .any(|created_clone_request| created_clone_request.eq(&clone_request))
    {
        return Err(anyhow!("Failed to create clone request."));
    }

    println!("Successfully created clone request");

    Ok(())
}

pub async fn read_from_file(happ_bundle_path: &PathBuf) -> Result<AppBundle> {
    let bytes = fs::read(happ_bundle_path)?;
    Ok(AppBundle::decode(bytes.as_slice())?)
}

pub fn into(key: push_notifications_types::ServiceAccountKey) -> fcm_v1::auth::ServiceAccountKey {
    fcm_v1::auth::ServiceAccountKey {
        key_type: key.key_type,
        project_id: key.project_id,
        private_key_id: key.private_key_id,
        private_key: key.private_key,
        client_email: key.client_email,
        client_id: key.client_id,
        auth_uri: key.auth_uri,
        token_uri: key.token_uri,
        auth_provider_x509_cert_url: key.auth_provider_x509_cert_url,
        client_x509_cert_url: key.client_x509_cert_url,
    }
}

pub fn from(key: fcm_v1::auth::ServiceAccountKey) -> push_notifications_types::ServiceAccountKey {
    push_notifications_types::ServiceAccountKey {
        key_type: key.key_type,
        project_id: key.project_id,
        private_key_id: key.private_key_id,
        private_key: key.private_key,
        client_email: key.client_email,
        client_id: key.client_id,
        auth_uri: key.auth_uri,
        token_uri: key.token_uri,
        auth_provider_x509_cert_url: key.auth_provider_x509_cert_url,
        client_x509_cert_url: key.client_x509_cert_url,
    }
}
