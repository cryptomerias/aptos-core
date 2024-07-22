// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

mod builders;
mod loader;
pub(crate) mod struct_name_index_map;

// Note: these traits should be defined elsewhere, along with Script and Module types.
mod dummy;
pub mod module_storage;
pub mod script_storage;
pub mod verifier;
