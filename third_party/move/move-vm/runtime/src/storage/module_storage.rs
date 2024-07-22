// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loader::Module;
use move_binary_format::{errors::PartialVMResult, CompiledModule};
use move_core_types::{account_address::AccountAddress, identifier::IdentStr, metadata::Metadata};
use std::sync::Arc;

pub trait ModuleStorage {
    fn check_module_exists(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<bool>;

    fn fetch_module_size_in_bytes(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<usize>;

    fn fetch_module_metadata(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<&[Metadata]>;

    fn fetch_deserialized_module(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<Arc<CompiledModule>>;

    fn fetch_or_create_verified_module(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
        f: &dyn Fn(Arc<CompiledModule>) -> PartialVMResult<Module>,
    ) -> PartialVMResult<Arc<Module>>;
}
