// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm_types::loaded_data::runtime_types::{StructIdentifier, StructNameIndex};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::collections::BTreeMap;

#[derive(Clone)]
struct IndexMap<T: Clone + Ord> {
    forward_map: BTreeMap<T, usize>,
    backward_map: Vec<T>,
}

/// A data structure to cache struct identifiers (address, module name, struct name) and
/// use indices instead, to save on the memory consumption and avoid unnecessary cloning.
pub(crate) struct StructNameIndexMap(RwLock<IndexMap<StructIdentifier>>);

impl StructNameIndexMap {
    pub(crate) fn empty() -> Self {
        Self(RwLock::new(IndexMap {
            forward_map: BTreeMap::new(),
            backward_map: vec![],
        }))
    }

    pub(crate) fn struct_name_to_idx(&self, struct_name: StructIdentifier) -> StructNameIndex {
        if let Some(idx) = self.0.read().forward_map.get(&struct_name) {
            return StructNameIndex(*idx);
        }
        let mut index_map = self.0.write();
        let idx = index_map.backward_map.len();
        index_map.forward_map.insert(struct_name.clone(), idx);
        index_map.backward_map.push(struct_name);
        StructNameIndex(idx)
    }

    pub(crate) fn idx_to_struct_name(
        &self,
        idx: StructNameIndex,
    ) -> MappedRwLockReadGuard<StructIdentifier> {
        RwLockReadGuard::map(self.0.read(), |index_map| &index_map.backward_map[idx.0])
    }
}

impl Clone for StructNameIndexMap {
    fn clone(&self) -> Self {
        Self(RwLock::new(self.0.read().clone()))
    }
}
