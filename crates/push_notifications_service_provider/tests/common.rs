use std::path::PathBuf;

pub fn happ_developer_happ_path() -> PathBuf {
    std::option_env!("HAPP_DEVELOPER_HAPP")
        .expect("Failed to find HAPP_DEVELOPER_HAPP")
        .into()
}

pub fn service_provider_happ_path() -> PathBuf {
    std::option_env!("SERVICE_PROVIDER_HAPP")
        .expect("Failed to find SERVICE_PROVIDER_HAPP")
        .into()
}

pub fn infra_provider_happ_path() -> PathBuf {
    std::option_env!("INFRA_PROVIDER_HAPP")
        .expect("Failed to find INFRA_PROVIDER_HAPP")
        .into()
}

pub fn end_user_happ_path() -> PathBuf {
    std::option_env!("END_USER_HAPP")
        .expect("Failed to find END_USER_HAPP")
        .into()
}
