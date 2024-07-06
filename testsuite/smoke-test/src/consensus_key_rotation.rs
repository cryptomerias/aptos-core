// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SwarmBuilder;
use aptos::common::types::GasOptions;
use aptos_config::config::{OverrideNodeConfig, PersistableConfig};
use aptos_crypto::{bls12381, Uniform};
use aptos_forge::{NodeExt, Swarm, SwarmExt};
use aptos_logger::info;
use aptos_types::on_chain_config::{ConfigurationResource, OnChainRandomnessConfig, ValidatorSet};
use rand::{Rng, thread_rng};
use std::{
    fs::File,
    io::Write,
    ops::Add,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use std::path::PathBuf;
use tempfile::tempdir;
use aptos_types::validator_verifier::ValidatorVerifier;
use crate::utils::get_on_chain_resource;

#[tokio::test]
async fn consensus_key_rotation() {
    let epoch_duration_secs = 100;
    let n = 4;
    let (mut swarm, mut cli, _faucet) = SwarmBuilder::new_local(n)
        .with_aptos()
        .with_init_config(Arc::new(|_, conf, _| {
            conf.api.failpoints_enabled = true;
        }))
        .with_init_genesis_config(Arc::new(move |conf| {
            conf.epoch_duration_secs = epoch_duration_secs;

            // Ensure randomness is enabled.
            conf.consensus_config.enable_validator_txns();
            conf.randomness_config_override = Some(OnChainRandomnessConfig::default_enabled());
        }))
        .build_with_cli(0)
        .await;

    info!("Wait for epoch 2.");
    swarm
        .wait_for_all_nodes_to_catchup_to_epoch(2, Duration::from_secs(epoch_duration_secs * 2))
        .await
        .expect("Epoch 2 taking too long to arrive!");

    let rest_client = swarm.validators().next().unwrap().rest_client();
    // let validator_set = get_on_chain_resource::<ValidatorSet>(&rest_client).await;
    // println!("validator_set={}", validator_set);

    let (operator_addr, new_pk, pop, operator_idx) =
        if let Some(validator) = swarm.validators_mut().nth(n - 1) {
            let operator_sk = validator
                .account_private_key()
                .as_ref()
                .unwrap()
                .private_key();
            let operator_sk_hex = operator_sk.to_bytes();
            let operator_idx = cli.add_account_to_cli(operator_sk);
            info!("Stopping the last node.");

            validator.stop();
            tokio::time::sleep(Duration::from_secs(5)).await;

            let new_identity_path = PathBuf::from(format!("/tmp/{}-new-validator-identity.yaml", thread_rng().gen::<u64>()).as_str());
            info!(
                "Generating and writing new validator identity to {:?}.",
                new_identity_path
            );
            let new_sk = bls12381::PrivateKey::generate(&mut thread_rng());
            let pop = bls12381::ProofOfPossession::create(&new_sk);
            let new_pk = bls12381::PublicKey::from(&new_sk);
            let mut validator_identity_blob = validator
                .config()
                .consensus
                .safety_rules
                .initial_safety_rules_config
                .identity_blob()
                .unwrap();
            validator_identity_blob.consensus_private_key = Some(new_sk);
            let operator_addr = validator_identity_blob.account_address.unwrap();
            // let operator_sk_hex_2 = validator_identity_blob
            //     .account_private_key
            //     .as_ref()
            //     .unwrap()
            //     .to_bytes();
            // assert_eq!(operator_sk_hex, operator_sk_hex_2);

            Write::write_all(
                &mut File::create(&new_identity_path).unwrap(),
                serde_yaml::to_string(&validator_identity_blob)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();

            info!("Updating node config accordingly.");
            let config_path = validator.config_path();
            let mut validator_override_config =
                OverrideNodeConfig::load_config(config_path.clone()).unwrap();
            *validator_override_config
                .override_config_mut()
                .consensus
                .safety_rules
                .initial_safety_rules_config
                .identity_blob_path_mut() = new_identity_path;
            validator_override_config.save_config(config_path).unwrap();

            info!("Restarting node.");
            validator.start().unwrap();
            info!("Let node bake for 5 secs.");
            tokio::time::sleep(Duration::from_secs(5)).await;
            (operator_addr, new_pk, pop, operator_idx)
        } else {
            unreachable!()
        };

    info!("Update on-chain. Retry is needed in case randomness is enabled.");
    swarm
        .chain_info()
        .into_aptos_public_info()
        .mint(operator_addr, 99999999999)
        .await
        .unwrap();
    let mut attempts = 10;
    while attempts > 0 {
        attempts -= 1;
        let gas_options = GasOptions {
            gas_unit_price: Some(100),
            max_gas: Some(200000),
            expiration_secs: 60,
        };
        let update_result = cli
            .update_consensus_key(
                operator_idx,
                None,
                new_pk.clone(),
                pop.clone(),
                Some(gas_options),
            )
            .await;
        println!("update_result={:?}", update_result);
        if let Ok(txn_smry) = update_result {
            if txn_smry.success == Some(true) {
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    assert!(attempts >= 1);

    info!("Wait for epoch 3.");
    let mut attempts = 100;
    while attempts > 0 {
        attemps -= 1;
        let c = get_on_chain_resource::<ConfigurationResource>(&rest_client).await;
        if c.epoch() == 3 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("All nodes should be alive.");
    let liveness_check_result = swarm
        .liveness_check(Instant::now().add(Duration::from_secs(30)))
        .await;

    // let validator_set = get_on_chain_resource::<ValidatorSet>(&rest_client).await;
    // println!("validator_set={}", validator_set);

    assert!(liveness_check_result.is_ok());
    assert!(false);
}
