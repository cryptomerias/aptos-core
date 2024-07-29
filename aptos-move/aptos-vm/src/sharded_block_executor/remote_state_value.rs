// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_types::state_store::state_value::StateValue;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use crate::sharded_block_executor::counters::SHARDED_EXECUTOR_SERVICE_SECONDS;

#[derive(Clone)]
// This struct is used to store the status of a remote state value. It provides semantics for
// blocking on a remote state value to be available locally while it is being asynchronously
// fetched from a remote server.
pub struct RemoteStateValue {
    value_condition: Arc<(Mutex<RemoteValueStatus>, Condvar, Instant)>,
}

impl RemoteStateValue {
    pub fn waiting() -> Self {
        Self {
            value_condition: Arc::new((Mutex::new(RemoteValueStatus::Waiting), Condvar::new(), Instant::now())),
        }
    }

    pub fn set_value(&self, value: Option<StateValue>) {
        let _timer = SHARDED_EXECUTOR_SERVICE_SECONDS.with_label_values(&["0", "kv_recv_wait_time_shard_diff"]).start_timer();
        let (lock, cvar, start_time) = &*self.value_condition;
        let mut status = lock.lock().unwrap();
        *status = RemoteValueStatus::Ready(value);
        cvar.notify_all();
        SHARDED_EXECUTOR_SERVICE_SECONDS
            .with_label_values(&["0", "kv_recv_wait_time_shard"]).observe(start_time.elapsed().as_secs_f64());
    }

    pub fn get_value(&self) -> Option<StateValue> {
        let _timer = SHARDED_EXECUTOR_SERVICE_SECONDS.with_label_values(&["0", "kv_read_wait_time_shard_diff"]).start_timer();
        let (lock, cvar, start_time) = &*self.value_condition;
        let mut status = lock.lock().unwrap();
        while let RemoteValueStatus::Waiting = *status {
            status = cvar.wait(status).unwrap();
        }
        SHARDED_EXECUTOR_SERVICE_SECONDS
            .with_label_values(&["0", "kv_read_wait_time_shard"]).observe(start_time.elapsed().as_secs_f64());
        match &*status {
            RemoteValueStatus::Ready(value) => value.clone(),
            RemoteValueStatus::Waiting => unreachable!(),
        }
    }

    pub fn is_ready(&self) -> bool {
        let (lock, _cvar, _) = &*self.value_condition;
        let status = lock.lock().unwrap();
        matches!(&*status, RemoteValueStatus::Ready(_))
    }
}

#[derive(Clone)]
pub enum RemoteValueStatus {
    /// The state value is available as a result of cross shard execution
    Ready(Option<StateValue>),
    /// We are still waiting for remote shard to push the state value
    Waiting,
}
