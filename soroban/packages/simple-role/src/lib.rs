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

//! # Simple Role
//!
//! Generic utilities for managing single-address roles in Stellar smart contracts.
//!
//! This crate provides a minimal, explicit API for role management:
//! - One address per role (not multiple accounts)
//! - Simple get/set/enforce_auth operations
//! - Event emission via `set_and_emit` or `set_and_emit_with_previous`
//!
//! ## Usage
//!
//! ```ignore
//! use simple_role::{get_role, set_role, enforce_role_auth, set_role_and_emit};
//!
//! const RESCUER: &str = "rescuer";
//!
//! pub fn get_rescuer(e: &Env) -> Address {
//!     get_role(e, RESCUER)
//! }
//!
//! pub fn set_rescuer(e: &Env, new_rescuer: &Address) {
//!     set_role_and_emit(e, RESCUER, new_rescuer, emit_rescuer_changed);
//! }
//!
//! pub fn do_something_as_rescuer(e: &Env) {
//!     enforce_role_auth(e, RESCUER);
//!     // ... business logic ...
//! }
//! ```

#![no_std]

#[cfg(test)]
mod test;

#[cfg(not(target_family = "wasm"))]
pub mod test_utils;

use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env, Symbol};
use stellar_macros::only_owner;

/// Storage key for a simple role (single address per role).
#[contracttype]
pub struct RoleKey {
    pub role: Symbol,
}

/// Errors for role operations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RoleError {
    /// The role has not been set.
    RoleNotSet = 7000,
}

/// Function signature for emitting events with only the new address.
pub type EmitNewFn = fn(&Env, &Address);

/// Function signature for emitting events with both previous and new addresses.
pub type EmitOldAndNewFn = fn(&Env, Option<Address>, &Address);

/// Gets a role address from instance storage.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
///
/// # Returns
///
/// The address assigned to the role.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the role is not set.
pub fn get_role(e: &Env, role: &str) -> Address {
    try_get_role(e, role).unwrap_or_else(|| panic_with_error!(e, RoleError::RoleNotSet))
}

/// Tries to get a role address from instance storage.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
///
/// # Returns
///
/// * `Some(Address)` if the role is set, `None` otherwise.
pub fn try_get_role(e: &Env, role: &str) -> Option<Address> {
    let key = RoleKey {
        role: Symbol::new(e, role),
    };
    e.storage().instance().get(&key)
}

/// Sets a role address in instance storage.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `address` - The address to set for the role.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the owner fails.
#[only_owner]
pub fn set_role(e: &Env, role: &str, address: &Address) {
    set_role_unchecked(e, role, address);
}

/// Sets a role address in instance storage without authorization checks.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `address` - The address to set for the role.
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. Use only in constructors
///   or test setup where the caller has already verified permissions.
pub fn set_role_unchecked(e: &Env, role: &str, address: &Address) {
    let key = RoleKey {
        role: Symbol::new(e, role),
    };
    e.storage().instance().set(&key, address);
}

/// Sets a role address and emits an event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `new_address` - The address to set for the role.
/// * `emit` - Function to emit the event.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the owner fails.
pub fn set_role_and_emit(e: &Env, role: &str, new_address: &Address, emit: EmitNewFn) {
    set_role(e, role, new_address);
    emit(e, new_address);
}

/// Sets a role address and emits an event with the previous address.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `new_address` - The address to set for the role.
/// * `emit` - Function to emit the event.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the owner fails.
pub fn set_role_and_emit_with_previous(
    e: &Env,
    role: &str,
    new_address: &Address,
    emit: EmitOldAndNewFn,
) {
    let previous = try_get_role(e, role);
    set_role(e, role, new_address);
    emit(e, previous, new_address);
}

/// Sets a role address and emits an event, without authorization checks.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `new_address` - The address to set for the role.
/// * `emit` - Function to emit the event.
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. Use only in constructors
///   or test setup where the caller has already verified permissions.
pub fn set_role_and_emit_unchecked(e: &Env, role: &str, new_address: &Address, emit: EmitNewFn) {
    set_role_unchecked(e, role, new_address);
    emit(e, new_address);
}

/// Sets a role address and emits an event with the previous address, without authorization checks.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
/// * `new_address` - The address to set for the role.
/// * `emit` - Function to emit the event.
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. Use only in constructors
///   or test setup where the caller has already verified permissions.
pub fn set_role_and_emit_with_previous_unchecked(
    e: &Env,
    role: &str,
    new_address: &Address,
    emit: EmitOldAndNewFn,
) {
    let previous = try_get_role(e, role);
    set_role_unchecked(e, role, new_address);
    emit(e, previous, new_address);
}

/// Enforces that a role is set and the role holder has authorized the call.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `role` - The role identifier.
///
/// # Returns
///
/// The role address after successful authorization.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the role is not set.
/// * `HostError: Error(Auth, InvalidAction)` – If authorization fails.
pub fn enforce_role_auth(e: &Env, role: &str) -> Address {
    let addr = get_role(e, role);
    addr.require_auth();
    addr
}
