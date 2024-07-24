// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loader::{Module, Script},
    storage::{module_storage::ModuleStorage, script_storage::ScriptStorage, verifier::Verifier},
};
use move_binary_format::{
    errors::{PartialVMError, PartialVMResult},
    file_format::CompiledScript,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress, identifier::IdentStr, metadata::Metadata,
    vm_status::StatusCode,
};
use std::sync::Arc;

macro_rules! not_implemented {
    () => {
        Err(PartialVMError::new(StatusCode::FEATURE_UNDER_GATING)
            .with_message("New loader and code cache are not yet implemented".to_string()))
    };
}

/// Dummy implementation of code storage, to be removed in the future. Used as a placeholder
/// so that existing APIs can work
pub struct DummyStorage;

impl ModuleStorage for DummyStorage {
    fn check_module_exists(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<bool> {
        not_implemented!()
    }

    fn fetch_module_size_in_bytes(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<usize> {
        not_implemented!()
    }

    fn fetch_module_metadata(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<&[Metadata]> {
        not_implemented!()
    }

    fn fetch_deserialized_module(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<Arc<CompiledModule>> {
        not_implemented!()
    }

    fn fetch_or_create_verified_module(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
        _f: &dyn Fn(Arc<CompiledModule>) -> PartialVMResult<Module>,
    ) -> PartialVMResult<Arc<Module>> {
        not_implemented!()
    }
}

impl ScriptStorage for DummyStorage {
    fn fetch_deserialized_script(
        &self,
        _serialized_script: &[u8],
    ) -> PartialVMResult<Arc<CompiledScript>> {
        not_implemented!()
    }

    fn fetch_or_create_verified_script(
        &self,
        _serialized_script: &[u8],
        _f: &dyn Fn(Arc<CompiledScript>) -> PartialVMResult<Script>,
    ) -> PartialVMResult<Arc<Script>> {
        not_implemented!()
    }

    fn fetch_existing_verified_script(&self, _script_hash: &[u8; 32]) -> Arc<Script> {
        unimplemented!()
    }
}

/// Placeholder to use for now before an actual verifier is implemented.
#[derive(Clone)]
pub struct DummyVerifier;

impl Verifier for DummyVerifier {
    fn verify_script(&self, _script: &CompiledScript) -> PartialVMResult<()> {
        not_implemented!()
    }

    fn verify_script_with_dependencies<'a>(
        &self,
        _script: &CompiledScript,
        _dependencies: impl IntoIterator<Item = &'a Module>,
    ) -> PartialVMResult<()> {
        not_implemented!()
    }

    fn verify_module(&self, _module: &CompiledModule) -> PartialVMResult<()> {
        not_implemented!()
    }

    fn verify_module_with_dependencies<'a>(
        &self,
        _module: &CompiledModule,
        _dependencies: impl IntoIterator<Item = &'a Module>,
    ) -> PartialVMResult<()> {
        not_implemented!()
    }
}
