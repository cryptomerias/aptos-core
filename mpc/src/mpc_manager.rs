// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;
use futures_channel::oneshot;
use futures_util::{FutureExt, StreamExt};
use anyhow::{anyhow, bail, ensure, Result};
use aptos_channels::aptos_channel;
use aptos_crypto::HashValue;
use aptos_logger::{error, info};
use aptos_types::epoch_state::EpochState;
use aptos_types::mpc::{MPCEvent, MpcState};
use aptos_types::validator_txn::{Topic, ValidatorTransaction};
use aptos_validator_transaction_pool::{TxnGuard, VTxnPoolState};
use move_core_types::account_address::AccountAddress;
use crate::network::IncomingRpcRequest;

pub struct MPCManager {
    my_index: usize,
    my_addr: AccountAddress,
    epoch_state: Arc<EpochState>,
    vtxn_pool: VTxnPoolState,
    stopped: bool,
    vtxn_guard: Option<TxnGuard>,
}

impl MPCManager {
    pub fn new(
        my_index: usize,
        my_addr: AccountAddress,
        epoch_state: Arc<EpochState>,
        vtxn_pool: VTxnPoolState,
    ) -> Self {
        Self {
            my_addr,
            my_index,
            epoch_state,
            vtxn_pool,
            stopped: false,
            vtxn_guard: None,
        }
    }

    pub async fn run(
        mut self,
        mpc_state: MpcState,
        mut mpc_event_rx: aptos_channel::Receiver<(), MPCEvent>,
        mut rpc_msg_rx: aptos_channel::Receiver<
            AccountAddress,
            (AccountAddress, IncomingRpcRequest),
        >,
        close_rx: oneshot::Receiver<oneshot::Sender<()>>
    ) {
        info!(
            epoch = self.epoch_state.epoch,
            my_addr = self.my_addr.to_hex().as_str(),
            "[MPC] MPCManager started."
        );

        let mut close_rx = close_rx.into_stream();
        while !self.stopped {
            let handling_result = tokio::select! {
                mpc_event = mpc_event_rx.select_next_some() => {
                    self.process_mpc_event(mpc_event).await.map_err(|e|anyhow!("[MPC] process_mpc_event failed: {e}"))
                },
                close_req = close_rx.select_next_some() => {
                    self.process_close_cmd(close_req.ok())
                },
            };

            if let Err(e) = handling_result {
                error!(
                    epoch = self.epoch_state.epoch,
                    my_addr = self.my_addr.to_hex().as_str(),
                    "[MPC] MPCManager handling error: {e}"
                );
            }
        }


        info!(
            epoch = self.epoch_state.epoch,
            my_addr = self.my_addr.to_hex().as_str(),
            "[MPC] MPCManager finished."
        );
    }

    async fn process_mpc_event(&mut self, event: MPCEvent) -> Result<()> {
        info!(
            epoch = self.epoch_state.epoch,
            my_addr = self.my_addr,
            "[MPC] Processing MPC event."
        );
        //mpc todo: real processing.
        ensure!(self.vtxn_guard.is_none());
        let txn = ValidatorTransaction::MPCStateUpdate;
        let vtxn_guard = self.vtxn_pool.put(
            Topic::MPC,
            Arc::new(txn),
            None,
        );
        self.vtxn_guard = Some(vtxn_guard);
        Ok(())
    }

    fn process_close_cmd(&mut self, ack_tx: Option<oneshot::Sender<()>>) -> Result<()> {
        self.stopped = true;
        if let Some(tx) = ack_tx {
            let _ = tx.send(());
        }

        Ok(())
    }
}
