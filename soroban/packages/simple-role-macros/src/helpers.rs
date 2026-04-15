/*
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 */

//! This module is a collection of utility functions for stellar related macros.
//! It is not intended to be used directly, but rather imported into other
//! macros.
//!
//! Forked from https://github.com/OpenZeppelin/stellar-contracts/blob/969a22096aea7326adf076f0efcde290cc476cba/packages/macros/src/helpers.rs
//!
//! Modifications:
//! 1. Removed the generate_auth_check function (unused). (1/26/2026)

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Type, TypePath};

/// Parses the environment argument from the function signature
pub fn parse_env_arg(input_fn: &ItemFn) -> TokenStream {
    let (env_ident, is_ref) = check_env_arg(input_fn);

    if is_ref {
        quote! { #env_ident }
    } else {
        quote! { &#env_ident }
    }
}

fn check_env_arg(input_fn: &ItemFn) -> (Ident, bool) {
    // Get the first argument
    let first_arg = input_fn.sig.inputs.first().unwrap_or_else(|| {
        panic!(
            "function '{}' must have at least one argument",
            input_fn.sig.ident
        )
    });

    // Extract the pattern and type from the argument
    let FnArg::Typed(PatType { pat, ty, .. }) = first_arg else {
        panic!(
            "first argument of function '{}' must be a typed parameter",
            input_fn.sig.ident
        );
    };

    // Get the identifier from the pattern
    let Pat::Ident(pat_ident) = &**pat else {
        panic!(
            "first argument of function '{}' must be an identifier",
            input_fn.sig.ident
        );
    };
    let ident = pat_ident.ident.clone();

    // Check if the type is Env or &Env
    let is_ref = match &**ty {
        Type::Reference(type_ref) => {
            let Type::Path(path) = &*type_ref.elem else {
                panic!(
                    "first argument of function '{}' must be Env or &Env",
                    input_fn.sig.ident
                );
            };
            check_is_env(path, &input_fn.sig.ident);
            true
        }
        Type::Path(path) => {
            check_is_env(path, &input_fn.sig.ident);
            false
        }
        _ => panic!(
            "first argument of function '{}' must be Env or &Env",
            input_fn.sig.ident
        ),
    };

    (ident, is_ref)
}

fn check_is_env(path: &TypePath, fn_name: &Ident) {
    let is_env = path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "Env")
        .unwrap_or(false);

    if !is_env {
        panic!("first argument of function '{fn_name}' must be Env or &Env",);
    }
}
