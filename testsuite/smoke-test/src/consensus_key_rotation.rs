// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    genesis::enable_sync_only_mode,
    smoke_test_environment::SwarmBuilder,
};
use aptos::common::types::GasOptions;
use aptos_config::config::{IdentityBlob, InitialSafetyRulesConfig, OverrideNodeConfig, PersistableConfig};
use aptos_forge::{NodeExt, Swarm, SwarmExt};
use aptos_logger::{debug, info};
use aptos_types::{on_chain_config::OnChainRandomnessConfig, randomness::PerBlockRandomness};
use std::{
    ops::Add,
    sync::Arc,
    time::{Duration, Instant},
};
use rand::thread_rng;
use aptos_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use aptos_crypto::{bls12381, Uniform};

#[tokio::test]
async fn consensus_key_rotation() {
    let epoch_duration_secs = 20;

    let (mut swarm, mut cli, _faucet) = SwarmBuilder::new_local(4)
        .with_num_fullnodes(1)
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


    // let root_addr = swarm.chain_info().root_account().address();
    // let root_idx = cli.add_account_with_address_to_cli(swarm.root_key(), root_addr);
    let rest_client = swarm.validators().next().unwrap().rest_client();

    info!("Wait for epoch 2.");
    swarm
        .wait_for_all_nodes_to_catchup_to_epoch(2, Duration::from_secs(epoch_duration_secs * 2))
        .await
        .expect("Epoch 2 taking too long to arrive!");

    info!("Stop node 3; change its consensus key; restart");
    if let Some(validator) = swarm.validators_mut().nth(3) {
        let operator_sk = validator.account_private_key().as_ref().unwrap().private_key();
        let operator_idx = cli.add_account_to_cli(operator_sk);
        validator.stop();
        tokio::time::sleep(Duration::from_secs(5)).await;

        let new_sk = bls12381::PrivateKey::generate(&mut thread_rng());
        let pop = bls12381::ProofOfPossession::create(&new_sk);
        let new_pk = bls12381::PublicKey::from(&new_sk);
        let InitialSafetyRulesConfig::FromFile { identity_blob_path, .. } = &validator.config().consensus.safety_rules.initial_safety_rules_config else { unreachable!() };
        let mut validator_identity_blob = validator.config()
            .consensus.safety_rules
            .initial_safety_rules_config
            .identity_blob().unwrap();
        validator_identity_blob.consensus_private_key = Some(new_sk);
        validator_identity_blob.to_file(identity_blob_path).unwrap();
        info!("Restarting node.");
        validator.start().unwrap();
        info!("Let node bake for 5 secs.");
        tokio::time::sleep(Duration::from_secs(5)).await;

        info!("Update on-chain.");
        let mut attempts = 10;
        while attempts > 0 {
            attempts -= 1;
            let update_result = cli.update_consensus_key(operator_idx, None, new_pk.clone(), pop.clone()).await;
            println!("update_result={:?}", update_result);
            if let Ok(txn_smry) = update_result {
                if txn_smry.success == Some(true) {
                    break;
                }
            }
        }

        assert!(attempts >= 1);
    } else {
        assert!(false);
    }

    // info!("Wait for long enough to see an epoch switch.");
    // tokio::time::sleep(Duration::from_secs(30)).await;

    info!("All nodes should be alive.");
    let liveness_check_result = swarm
        .liveness_check(Instant::now().add(Duration::from_secs(30)))
        .await;

    assert!(liveness_check_result.is_ok());
}
