// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::TypeTag;
use move_core_types::move_resource::MoveStructType;
use crate::on_chain_config::OnChainConfig;
use crate::move_any::Any as MoveAny;

#[derive(Deserialize, Serialize)]
pub struct TaskSpec {
    pub variant: MoveAny,
}

#[derive(Deserialize, Serialize)]
pub struct TaskState {
    pub task: TaskSpec,
    pub result: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize)]
pub struct SharedSecretState {
    pub transcript_for_cur_epoch: Option<Vec<u8>>,
    pub transcript_for_next_epoch: Option<Vec<u8>>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MpcState {
    pub shared_secrets: Vec<SharedSecretState>,
    pub tasks: Vec<TaskState>,
}

impl OnChainConfig for MpcState {
    const MODULE_IDENTIFIER: &'static str = "mpc";
    const TYPE_IDENTIFIER: &'static str = "State";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MPCEvent {
    field_1: u64,
}

impl MoveStructType for MPCEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("mpc");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MPCEvent");
}

pub static MPC_EVENT_MOVE_TYPE_TAG: Lazy<TypeTag> =
    Lazy::new(|| TypeTag::Struct(Box::new(MPCEvent::struct_tag())));
