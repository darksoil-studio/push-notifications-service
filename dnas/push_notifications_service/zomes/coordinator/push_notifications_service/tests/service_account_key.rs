#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use hdk::prelude::*;
use holochain::{conductor::config::ConductorConfig, sweettest::*};
use std::time::Duration;

use push_notifications_service_integrity::*;

mod common;
use common::{
    create_service_account_key, sample_service_account_key_1, sample_service_account_key_2,
};

#[tokio::test(flavor = "multi_thread")]
async fn create_service_account_key_test() {
    // Use prebuilt dna file
    let dna_path = std::env::current_dir().unwrap().join(
        std::env::var("DNA_PATH").expect("DNA_PATH not set, must be run using nix flake check"),
    );
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors
        .setup_app("push_notifications_service", &[dna])
        .await
        .unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (_bobbo,)) = apps.into_tuples();

    let alice_zome = alice.zome("push_notifications_service");

    let sample = sample_service_account_key_1(&conductors[0], &alice_zome).await;

    // Alice creates a ServiceAccountKey
    let record: Record =
        create_service_account_key(&conductors[0], &alice_zome, sample.clone()).await;
    let entry: ServiceAccountKey = record.entry().to_app_option().unwrap().unwrap();
    assert!(entry.eq(&sample));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_and_read_service_account_key() {
    // Use prebuilt dna file
    let dna_path = std::env::current_dir().unwrap().join(
        std::env::var("DNA_PATH").expect("DNA_PATH not set, must be run using nix flake check"),
    );
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors
        .setup_app("push_notifications_service", &[dna])
        .await
        .unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (bobbo,)) = apps.into_tuples();

    let alice_zome = alice.zome("push_notifications_service");
    let bob_zome = bobbo.zome("push_notifications_service");

    let sample = sample_service_account_key_1(&conductors[0], &alice_zome).await;

    // Alice creates a ServiceAccountKey
    let record: Record =
        create_service_account_key(&conductors[0], &alice_zome, sample.clone()).await;

    await_consistency(Duration::from_secs(60), [&alice, &bobbo])
        .await
        .expect("Timed out waiting for consistency");

    let get_record: Option<Record> = conductors[1]
        .call(
            &bob_zome,
            "get_service_account_key",
            record.signed_action.action_address().clone(),
        )
        .await;

    assert_eq!(record, get_record.unwrap());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_and_delete_service_account_key() {
    // Use prebuilt dna file
    let dna_path = std::env::current_dir().unwrap().join(
        std::env::var("DNA_PATH").expect("DNA_PATH not set, must be run using nix flake check"),
    );
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors
        .setup_app("push_notifications_service", &[dna])
        .await
        .unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (bobbo,)) = apps.into_tuples();

    let alice_zome = alice.zome("push_notifications_service");
    let bob_zome = bobbo.zome("push_notifications_service");

    let sample_1 = sample_service_account_key_1(&conductors[0], &alice_zome).await;

    // Alice creates a ServiceAccountKey
    let record: Record =
        create_service_account_key(&conductors[0], &alice_zome, sample_1.clone()).await;
    let original_action_hash = record.signed_action.hashed.hash;

    // Alice deletes the ServiceAccountKey
    let delete_action_hash: ActionHash = conductors[0]
        .call(
            &alice_zome,
            "delete_service_account_key",
            original_action_hash.clone(),
        )
        .await;

    await_consistency(Duration::from_secs(60), [&alice, &bobbo])
        .await
        .expect("Timed out waiting for consistency");

    let deletes: Vec<SignedActionHashed> = conductors[1]
        .call(
            &bob_zome,
            "get_all_deletes_for_service_account_key",
            original_action_hash.clone(),
        )
        .await;

    assert_eq!(deletes.len(), 1);
    assert_eq!(deletes[0].hashed.hash, delete_action_hash);
}
