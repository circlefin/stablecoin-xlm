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
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # Simple Role Macros
//!
//! Procedural macros for role-based authorization in Stellar smart contracts.
//!
//! ## `#[enforce_role_auth]`
//!
//! Injects `simple_role::enforce_role_auth(e, "role")` at the start of a function.
//!
//! ```ignore
//! use simple_role_macros::enforce_role_auth;
//!
//! #[enforce_role_auth("rescuer")]
//! pub fn rescue_tokens(e: &Env, ...) {
//!     // Only runs if rescuer has authorized
//! }
//! ```

mod helpers;

use helpers::parse_env_arg;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Injects `simple_role::enforce_role_auth(e, role)` at the start of a function.
///
/// # Requirement
///
/// - The first argument of the decorated function must be of type `Env` or `&Env`
///
/// This is very similar to the generate_role_check in the OpenZeppelin stellar-macros crate:
/// https://github.com/OpenZeppelin/stellar-contracts/blob/ab67189c8d11b351b6c056ad254b9763e00ab785/packages/macros/src/access_control.rs#L14
///
/// # Usage
///
/// ```ignore
/// // With a constant
/// #[enforce_role_auth(RESCUER)]
/// pub fn rescue_sep41(e: &Env, ...) { ... }
///
/// // Or with a string literal
/// #[enforce_role_auth("rescuer")]
/// pub fn rescue_sep41(e: &Env, ...) { ... }
/// ```
///
/// Generates:
///
/// ```ignore
/// pub fn rescue_sep41(e: &Env, ...) {
///     simple_role::enforce_role_auth(e, RESCUER);
///     ...
/// }
/// ```
#[proc_macro_attribute]
pub fn enforce_role_auth(attr: TokenStream, item: TokenStream) -> TokenStream {
    let storage_key: syn::Expr = parse_macro_input!(attr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let env_param = parse_env_arg(&input_fn);

    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // Enforce role authorization
            simple_role::enforce_role_auth(#env_param, #storage_key);
            // Then, call the rest of the function
            #fn_block
        }
    };

    TokenStream::from(expanded)
}
