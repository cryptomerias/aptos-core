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
use std::{marker::PhantomData, sync::Arc};

pub struct ScriptBuilder<V: Verifier>(PhantomData<V>);

impl<V: Verifier> ScriptBuilder<V> {
    pub(crate) fn build(
        module_storage: &impl ModuleStorage,
        struct_name_index_map: &StructNameIndexMap,
        compiled_script: Arc<CompiledScript>,
    ) -> PartialVMResult<Script> {
        // Verify local properties of the script.
        V::verify_script(compiled_script.as_ref())?;

        // Fetch all dependencies of this script, and verify them as well.
        let imm_dependencies = compiled_script
            .immediate_dependencies_iter()
            .map(|(addr, name)| {
                module_storage.fetch_or_create_verified_module(addr, name, |cm| {
                    ModuleBuilder::<V>::build(module_storage, struct_name_index_map, cm)
                })
            })
            .collect::<PartialVMResult<Vec<_>>>()?;

        // Perform checks on modules with their dependencies.
        V::verify_script_with_dependencies(
            compiled_script.as_ref(),
            imm_dependencies.iter().map(|m| m.module()),
        )?;
        Script::new_v2(module_storage, struct_name_index_map, compiled_script)
    }
}

pub struct ModuleBuilder<V: Verifier>(PhantomData<V>);

impl<V: Verifier> ModuleBuilder<V> {
    pub(crate) fn build(
        module_storage: &impl ModuleStorage,
        struct_name_index_map: &StructNameIndexMap,
        compiled_module: Arc<CompiledModule>,
    ) -> PartialVMResult<Module> {
        // Verify local properties of the module.
        V::verify_module(compiled_module.as_ref())?;

        // Fetch all dependencies of this module, ensuring they are verified as well.
        let imm_dependencies = compiled_module
            .immediate_dependencies_iter()
            .map(|(addr, name)| {
                module_storage.fetch_or_create_verified_module(addr, name, |cm| {
                    Self::build(module_storage, struct_name_index_map, cm)
                })
            })
            .collect::<PartialVMResult<Vec<_>>>()?;

        // Perform checks on modules with their dependencies.
        V::verify_module_with_dependencies(
            compiled_module.as_ref(),
            imm_dependencies.iter().map(|m| m.module()),
        )?;
        Module::new_v2(module_storage, struct_name_index_map, compiled_module)
    }
}
