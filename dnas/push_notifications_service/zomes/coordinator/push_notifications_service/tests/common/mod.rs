pub async fn sample_service_account_key_1(
    conductor: &SweetConductor,
    zome: &SweetZome,
) -> ServiceAccountKey {
    ServiceAccountKey {
        a: "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string(),
    }
}

pub async fn sample_service_account_key_2(
    conductor: &SweetConductor,
    zome: &SweetZome,
) -> ServiceAccountKey {
    ServiceAccountKey {
        a: "Lorem ipsum 2".to_string(),
    }
}

pub async fn create_service_account_key(
    conductor: &SweetConductor,
    zome: &SweetZome,
    service_account_key: ServiceAccountKey,
) -> Record {
    let record: Record = conductor
        .call(zome, "create_service_account_key", service_account_key)
        .await;
    record
}
