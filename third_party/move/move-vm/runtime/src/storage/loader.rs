// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::VMConfig,
    loader::{Function, Module, Script, TypeCache},
    module_traversal::TraversalContext,
    storage::{
        module_storage::ModuleStorage,
        script_storage::{script_hash, ScriptStorage},
        struct_name_index_map::StructNameIndexMap,
        verifier::Verifier,
    },
    LoadedFunction,
};
use move_binary_format::{
    access::{ModuleAccess, ScriptAccess},
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::CompiledScript,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::NumBytes,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    vm_status::StatusCode,
};
use move_vm_types::{
    gas::GasMeter,
    loaded_data::runtime_types::{StructType, Type, TypeBuilder},
};
use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::Arc};
use typed_arena::Arena;

pub(crate) struct LoaderV2<V: Clone + Verifier> {
    // Map to from struct names to indices, to save on unnecessary cloning and
    // reduce memory consumption.
    pub(crate) struct_name_index_map: StructNameIndexMap,
    // Configuration of the VM, which own this loader. Contains information about
    // enabled checks, etc.
    vm_config: VMConfig,
    verifier: V,

    // Local caches:
    //   These caches are owned by this loader and are not affected by module
    //   upgrades. When a new cache is added, the safety guarantees (i.e., why
    //   it is safe for the loader to own this cache) MUST be documented.
    pub(crate) ty_cache: RwLock<TypeCache>,
}

impl<V: Clone + Verifier> LoaderV2<V> {
    pub(crate) fn check_script_dependencies_and_check_gas(
        &self,
        module_storage: &impl ModuleStorage,
        script_storage: &impl ScriptStorage,
        gas_meter: &mut impl GasMeter,
        traversal_context: &mut TraversalContext,
        serialized_script: &[u8],
    ) -> VMResult<()> {
        let compiled_script = script_storage
            .fetch_deserialized_script(serialized_script)
            .map_err(|e| e.finish(Location::Undefined))?;
        let compiled_script = traversal_context.referenced_scripts.alloc(compiled_script);

        for (addr, name) in compiled_script.immediate_dependencies_iter() {
            if !module_storage
                .check_module_exists(addr, name)
                .map_err(|e| e.finish(Location::Undefined))?
            {
                return Err(PartialVMError::new(StatusCode::LINKER_ERROR)
                    .with_message(format!(
                        "Script dependency {}::{} does not exist",
                        addr, name,
                    ))
                    .finish(Location::Undefined));
            }
        }

        self.check_dependencies_and_charge_gas(
            module_storage,
            gas_meter,
            &mut traversal_context.visited,
            traversal_context.referenced_modules,
            compiled_script.immediate_dependencies_iter(),
        )?;

        Ok(())
    }

    pub(crate) fn check_dependencies_and_charge_gas<'a, I>(
        &self,
        module_storage: &impl ModuleStorage,
        gas_meter: &mut impl GasMeter,
        visited: &mut BTreeMap<(&'a AccountAddress, &'a IdentStr), ()>,
        referenced_modules: &'a Arena<Arc<CompiledModule>>,
        // Note: the associated modules must exist and the caller has to check that!
        ids: I,
    ) -> VMResult<()>
    where
        I: IntoIterator<Item = (&'a AccountAddress, &'a IdentStr)>,
        I::IntoIter: DoubleEndedIterator,
    {
        let mut stack = Vec::with_capacity(512);
        for (addr, name) in ids.into_iter().rev() {
            if !addr.is_special() && visited.insert((addr, name), ()).is_none() {
                stack.push((addr, name));
            }
        }

        while let Some((addr, name)) = stack.pop() {
            let size = module_storage
                .fetch_module_size_in_bytes(addr, name)
                .map_err(|e| e.finish(Location::Undefined))?;
            gas_meter
                .charge_dependency(false, addr, name, NumBytes::new(size as u64))
                .map_err(|err| {
                    err.finish(Location::Module(ModuleId::new(*addr, name.to_owned())))
                })?;

            let compiled_module = module_storage
                .fetch_deserialized_module(addr, name)
                .map_err(|e| e.finish(Location::Undefined))?;
            let compiled_module = referenced_modules.alloc(compiled_module);
            for (addr, name) in compiled_module
                .immediate_dependencies_iter()
                .chain(compiled_module.immediate_friends_iter())
                .rev()
            {
                if !addr.is_special() && visited.insert((addr, name), ()).is_none() {
                    stack.push((addr, name));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn load_script(
        &self,
        module_storage: &impl ModuleStorage,
        script_storage: &impl ScriptStorage,
        serialized_script: &[u8],
        ty_args: &[TypeTag],
    ) -> VMResult<LoadedFunction> {
        let script_hash = script_hash(serialized_script);
        let main = script_storage
            .fetch_or_create_verified_script(serialized_script, &|cs| {
                self.build_script(
                    module_storage,
                    cs,
                    // TODO(George): We re-calculate the script hash because function
                    //   is not aware of the context in which it executes. Revisit.
                    script_hash,
                )
            })
            .map_err(|e| e.finish(Location::Script))?
            .entry_point();

        let ty_args = ty_args
            .iter()
            .map(|ty| self.load_ty(module_storage, ty))
            .collect::<VMResult<Vec<_>>>()?;

        Type::verify_ty_arg_abilities(main.ty_param_abilities(), &ty_args).map_err(|e| {
            e.with_message(format!(
                "Failed to verify type arguments for script {}",
                &main.name
            ))
            .finish(Location::Script)
        })?;

        Ok(LoadedFunction {
            ty_args,
            function: main,
        })
    }

    pub(crate) fn load_module(
        &self,
        module_storage: &dyn ModuleStorage,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> VMResult<Arc<Module>> {
        module_storage
            .fetch_or_create_verified_module(address, module_name, &|cm| {
                self.build_module(module_storage, cm)
            })
            .map_err(|e| {
                e.finish(Location::Module(ModuleId::new(
                    *address,
                    module_name.to_owned(),
                )))
            })
    }

    pub(crate) fn load_function_without_ty_args(
        &self,
        module_storage: &dyn ModuleStorage,
        address: &AccountAddress,
        module_name: &IdentStr,
        function_name: &IdentStr,
    ) -> VMResult<Arc<Function>> {
        let module = self.load_module(module_storage, address, module_name)?;
        Ok(module
            .function_map
            .get(function_name)
            .and_then(|idx| module.function_defs.get(*idx))
            .ok_or_else(|| {
                PartialVMError::new(StatusCode::FUNCTION_RESOLUTION_FAILURE)
                    .with_message(format!(
                        "Function {}::{}::{} does not exist",
                        address, module_name, function_name
                    ))
                    .finish(Location::Undefined)
            })?
            .clone())
    }

    pub(crate) fn load_struct_ty(
        &self,
        module_storage: &impl ModuleStorage,
        address: &AccountAddress,
        module_name: &IdentStr,
        struct_name: &IdentStr,
    ) -> VMResult<Arc<StructType>> {
        let module = self.load_module(module_storage, address, module_name)?;
        Ok(module
            .struct_map
            .get(struct_name)
            .and_then(|idx| module.structs.get(*idx))
            .ok_or_else(|| {
                PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
                    .with_message(format!(
                        "Struct {}::{}::{} does not exist",
                        address, module_name, struct_name
                    ))
                    .finish(Location::Undefined)
            })?
            .definition_struct_type
            .clone())
    }

    pub(crate) fn load_ty(
        &self,
        module_storage: &impl ModuleStorage,
        ty_tag: &TypeTag,
    ) -> VMResult<Type> {
        self.ty_builder().create_ty(ty_tag, |st| {
            self.load_struct_ty(
                module_storage,
                &st.address,
                st.module.as_ident_str(),
                st.name.as_ident_str(),
            )
        })
    }

    pub(crate) fn verify_modules_for_publication(
        &self,
        _module_storage: &impl ModuleStorage,
        _modules: &[CompiledModule],
    ) -> VMResult<()> {
        unimplemented!()
    }

    pub(crate) fn vm_config(&self) -> &VMConfig {
        &self.vm_config
    }

    pub(crate) fn ty_builder(&self) -> &TypeBuilder {
        &self.vm_config.ty_builder
    }

    /// Given loader's context, builds a new verified script instance.
    fn build_script(
        &self,
        module_storage: &dyn ModuleStorage,
        compiled_script: Arc<CompiledScript>,
        script_hash: [u8; 32],
    ) -> PartialVMResult<Script> {
        // Verify local properties of the script.
        self.verifier.verify_script(compiled_script.as_ref())?;

        // Fetch all dependencies of this script, and verify them as well.
        let imm_dependencies = compiled_script
            .immediate_dependencies_iter()
            .map(|(addr, name)| {
                module_storage.fetch_or_create_verified_module(addr, name, &|cm| {
                    self.build_module(module_storage, cm)
                })
            })
            .collect::<PartialVMResult<Vec<_>>>()?;

        // Perform checks on script and its dependencies.
        self.verifier.verify_script_with_dependencies(
            compiled_script.as_ref(),
            imm_dependencies.iter().map(|m| m.as_ref()),
        )?;

        Script::new_v2(module_storage, compiled_script, script_hash)
    }

    /// Given loader's context, builds a new verified module instance.
    fn build_module(
        &self,
        module_storage: &dyn ModuleStorage,
        compiled_module: Arc<CompiledModule>,
    ) -> PartialVMResult<Module> {
        // Verify local properties of the module.
        self.verifier.verify_module(compiled_module.as_ref())?;

        // Fetch all dependencies of this module, ensuring they are verified as well.
        let f = |cm| self.build_module(module_storage, cm);
        let imm_dependencies = compiled_module
            .immediate_dependencies_iter()
            .map(|(addr, name)| module_storage.fetch_or_create_verified_module(addr, name, &f))
            .collect::<PartialVMResult<Vec<_>>>()?;

        // Perform checks on the module with its immediate dependencies.
        self.verifier.verify_module_with_dependencies(
            compiled_module.as_ref(),
            imm_dependencies.iter().map(|m| m.as_ref()),
        )?;
        Module::new_v2(module_storage, compiled_module)
    }
}

impl<V: Clone + Verifier> Clone for LoaderV2<V> {
    fn clone(&self) -> Self {
        Self {
            struct_name_index_map: self.struct_name_index_map.clone(),
            vm_config: self.vm_config.clone(),
            verifier: self.verifier.clone(),
            ty_cache: RwLock::new(self.ty_cache.read().clone()),
        }
    }
}
