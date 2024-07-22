// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loader::{Module, Script},
    storage::{
        module_storage::ModuleStorage,
        script_storage::{ScriptHash, ScriptStorage},
        verifier::Verifier,
    },
};
use move_binary_format::{errors::PartialVMResult, file_format::CompiledScript, CompiledModule};
use move_core_types::{account_address::AccountAddress, identifier::IdentStr};
use std::sync::Arc;

#[allow(dead_code)]
pub struct DummyStorage;

impl ModuleStorage for DummyStorage {
    fn check_module_exists(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<bool> {
        todo!()
    }

    fn fetch_module_size_in_bytes(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<usize> {
        todo!()
    }

    fn fetch_deserialized_module(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
    ) -> PartialVMResult<Arc<CompiledModule>> {
        todo!()
    }

    fn fetch_or_create_verified_module<F: Fn(Arc<CompiledModule>) -> PartialVMResult<Module>>(
        &self,
        _address: &AccountAddress,
        _module_name: &IdentStr,
        _f: F,
    ) -> PartialVMResult<Arc<Module>> {
        todo!()
    }
}

impl ScriptStorage for DummyStorage {
    fn fetch_deserialized_script(
        &self,
        _serialized_script: &[u8],
    ) -> PartialVMResult<Arc<CompiledScript>> {
        todo!()
    }

    fn fetch_or_create_verified_script<F: Fn(Arc<CompiledScript>) -> PartialVMResult<Script>>(
        &self,
        _serialized_script: &[u8],
        _f: F,
    ) -> PartialVMResult<Arc<Script>> {
        todo!()
    }

    fn fetch_existing_verified_script(&self, _script_hash: &ScriptHash) -> Arc<Script> {
        todo!()
    }
}

pub struct DummyVerifier;

impl Verifier for DummyVerifier {
    fn verify_script(_script: &CompiledScript) -> PartialVMResult<()> {
        Ok(())
    }

    fn verify_script_with_dependencies<'a>(
        _script: &CompiledScript,
        _dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn verify_module(_module: &CompiledModule) -> PartialVMResult<()> {
        Ok(())
    }

    fn verify_module_with_dependencies<'a>(
        _module: &CompiledModule,
        _dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()> {
        Ok(())
    }
}
