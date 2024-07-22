// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::{errors::PartialVMResult, file_format::CompiledScript, CompiledModule};

pub trait Verifier {
    fn verify_script(script: &CompiledScript) -> PartialVMResult<()>;

    fn verify_script_with_dependencies<'a>(
        script: &CompiledScript,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()>;

    fn verify_module(module: &CompiledModule) -> PartialVMResult<()>;

    fn verify_module_with_dependencies<'a>(
        module: &CompiledModule,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()>;
}
