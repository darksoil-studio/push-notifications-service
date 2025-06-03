use anyhow::Result;
use holochain_types::prelude::*;
use std::{fs, path::PathBuf};

mod dynamic_provider;
pub use dynamic_provider::*;
pub mod fcm_client;
mod setup;
mod static_provider;
pub use static_provider::*;

pub const SERVICE_PROVIDERS_ROLE_NAME: &'static str = "service_providers";

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
