// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loader::ModuleStorageAdapter,
    storage::{loader::LoaderV2, verifier::Verifier},
    ModuleStorage,
};
use move_binary_format::{errors::PartialVMResult, file_format::StructHandle};
use move_core_types::{identifier::IdentStr, language_storage::ModuleId};

/// We use this trait so that ability checks can be implemented differently for
/// V1 and V2 loaders.
pub(crate) trait StructTypeAbilityChecker {
    /// Checks if the struct type abilities defined by the struct handle are compatible
    /// with abilities at the struct definition site.
    fn paranoid_check(
        &self,
        module_id: &ModuleId,
        struct_name: &IdentStr,
        struct_handle: &StructHandle,
    ) -> PartialVMResult<()>;
}

pub(crate) struct LoaderV2StructTypeAbilityChecker<'a, V: Clone + Verifier> {
    pub(crate) loader: &'a LoaderV2<V>,
    pub(crate) module_storage: &'a dyn ModuleStorage,
}

impl<'a, V: Clone + Verifier> StructTypeAbilityChecker for LoaderV2StructTypeAbilityChecker<'a, V> {
    fn paranoid_check(
        &self,
        module_id: &ModuleId,
        struct_name: &IdentStr,
        struct_handle: &StructHandle,
    ) -> PartialVMResult<()> {
        self.loader
            .load_struct_ty(
                self.module_storage,
                module_id.address(),
                module_id.name(),
                struct_name,
            )?
            .check_compatibility(struct_handle)
    }
}

pub(crate) struct LoaderV1StructTypeAbilityChecker<'a> {
    pub(crate) module_store: &'a ModuleStorageAdapter,
}

impl<'a> StructTypeAbilityChecker for LoaderV1StructTypeAbilityChecker<'a> {
    fn paranoid_check(
        &self,
        module_id: &ModuleId,
        struct_name: &IdentStr,
        struct_handle: &StructHandle,
    ) -> PartialVMResult<()> {
        self.module_store
            .get_struct_type_by_identifier(struct_name, module_id)?
            .check_compatibility(struct_handle)
    }
}
