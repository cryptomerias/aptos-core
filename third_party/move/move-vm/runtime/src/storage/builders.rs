// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loader::{Module, Script},
    storage::{
        module_storage::ModuleStorage, struct_name_index_map::StructNameIndexMap,
        verifier::Verifier,
    },
};
use move_binary_format::{
    access::{ModuleAccess, ScriptAccess},
    errors::PartialVMResult,
    file_format::CompiledScript,
    CompiledModule,
};
use std::sync::Arc;

/// Given loader's context, builds a new verified script instance.
pub(crate) fn build_script(
    struct_name_index_map: &StructNameIndexMap,
    verifier: &impl Verifier,
    module_storage: &dyn ModuleStorage,
    compiled_script: Arc<CompiledScript>,
    script_hash: [u8; 32],
) -> PartialVMResult<Script> {
    // Verify local properties of the script.
    verifier.verify_script(compiled_script.as_ref())?;

    // Fetch all dependencies of this script, and verify them as well.
    let imm_dependencies = compiled_script
        .immediate_dependencies_iter()
        .map(|(addr, name)| {
            module_storage.fetch_or_create_verified_module(addr, name, &|cm| {
                build_module(struct_name_index_map, verifier, module_storage, cm)
            })
        })
        .collect::<PartialVMResult<Vec<_>>>()?;

    // Perform checks on script and its dependencies.
    verifier.verify_script_with_dependencies(
        compiled_script.as_ref(),
        imm_dependencies.iter().map(|m| m.as_ref()),
    )?;

    Script::new_v2(
        struct_name_index_map,
        module_storage,
        compiled_script,
        script_hash,
    )
}

/// Given loader's context, builds a new verified module instance.
pub(crate) fn build_module(
    struct_name_index_map: &StructNameIndexMap,
    verifier: &impl Verifier,
    module_storage: &dyn ModuleStorage,
    compiled_module: Arc<CompiledModule>,
) -> PartialVMResult<Module> {
    // Verify local properties of the module.
    verifier.verify_module(compiled_module.as_ref())?;

    // Fetch all dependencies of this module, ensuring they are verified as well.
    let f = |cm| build_module(struct_name_index_map, verifier, module_storage, cm);
    let imm_dependencies = compiled_module
        .immediate_dependencies_iter()
        .map(|(addr, name)| module_storage.fetch_or_create_verified_module(addr, name, &f))
        .collect::<PartialVMResult<Vec<_>>>()?;

    // Perform checks on the module with its immediate dependencies.
    verifier.verify_module_with_dependencies(
        compiled_module.as_ref(),
        imm_dependencies.iter().map(|m| m.as_ref()),
    )?;
    Module::new_v2(module_storage, struct_name_index_map, compiled_module)
}
