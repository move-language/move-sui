// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

#![no_main]
use libfuzzer_sys::fuzz_target;
use move_binary_format::file_format::CompiledModule;

#[no_mangle]
pub extern "C" fn main() -> i32 {
    libfuzzer_sys::initialize(std::ptr::null(), std::ptr::null());
    return 0;
}

fuzz_target!(|module: CompiledModule| {
    let _ = move_bytecode_verifier::verify_module_unmetered(&module);
});
