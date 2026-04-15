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
//! Forked from https://github.com/OpenZeppelin/stellar-contracts/blob/main/packages/access/src/ownable/storage.rs
//! Modifications:
//! 1. Adapted for admin role instead of owner role
//! 2. Renamed functions and events to use "admin" terminology

use simple_role::RoleKey;
use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env, Symbol};
use stellar_access::role_transfer::{accept_transfer, transfer_role};

use crate::manageable::{emit_admin_change_started, emit_admin_changed, ADMIN};

/// Storage keys for `Manageable` utility.
#[contracttype]
pub enum ManageableStorageKey {
    PendingAdmin,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ManageableError {
    AdminNotSet = 7200,
    AdminAlreadySet = 7201,
}

/// Returns the pending admin address if set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage()
        .temporary()
        .get::<_, Address>(&ManageableStorageKey::PendingAdmin)
}

/// Sets the admin without performing authorization checks.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `admin` - The admin address to set.
///
/// # Errors
///
/// * [`ManageableError::AdminAlreadySet`] - If the admin is already set.
///
/// # Events
///
/// * topics - `["admin_changed"]`
/// * data - `[old_admin: None, new_admin: Address]`
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call
///   this function only in the constructor!
pub fn set_admin_unchecked(e: &Env, admin: &Address) {
    // Check if admin is already set
    if simple_role::try_get_role(e, ADMIN).is_some() {
        panic_with_error!(e, ManageableError::AdminAlreadySet);
    }
    simple_role::set_role_unchecked(e, ADMIN, admin);
    emit_admin_changed(e, None, admin);
}

/// Initiates a 2-step admin transfer to a new admin.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_admin` - The proposed new admin.
/// * `expires_in_ledgers` - Number of ledgers from now the pending admin has to
///   accept. A value of `0` cancels any pending transfer.
///
/// # Errors
///
/// * refer to [`transfer_role`] errors.
/// * refer to [`enforce_admin_auth`] errors.
///
/// # Events
///
/// * topics - `["admin_change_started"]`
/// * data - `[old_admin: Option<Address>, new_admin: Address]`
///
/// # Notes
///
/// * Authorization for the current admin is required.
pub fn transfer_admin(e: &Env, new_admin: &Address, expires_in_ledgers: u32) {
    let current_admin = enforce_admin_auth(e);

    let live_until_ledger = if expires_in_ledgers == 0 {
        0
    } else {
        e.ledger()
            .sequence()
            .checked_add(expires_in_ledgers)
            .expect("ledger sequence overflow")
    };

    transfer_role(
        e,
        new_admin,
        &ManageableStorageKey::PendingAdmin,
        live_until_ledger,
    );

    emit_admin_change_started(e, Some(current_admin), new_admin);
}

/// Completes the 2-step admin transfer process.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Errors
///
/// * refer to [`accept_transfer`] errors.
///
/// # Events
///
/// * topics - `["admin_changed"]`
/// * data - `[old_admin: Option<Address>, new_admin: Address]`
///
/// # Notes
///
/// * Authorization for the pending admin is required.
pub fn accept_admin(e: &Env) {
    let old_admin = simple_role::try_get_role(e, ADMIN);

    let admin_key = RoleKey {
        role: Symbol::new(e, ADMIN),
    };

    let new_admin = accept_transfer(e, &admin_key, &ManageableStorageKey::PendingAdmin);

    emit_admin_changed(e, old_admin, &new_admin);
}

// ################## LOW-LEVEL HELPERS ##################

/// Enforces authorization from the current admin.
///
/// This is used internally by role macros to gate access.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Errors
///
/// * [`ManageableError::AdminNotSet`] – If the admin is not set.
/// * `HostError: Error(Auth, InvalidAction)` – If authorization fails.
pub fn enforce_admin_auth(e: &Env) -> Address {
    let Some(admin) = simple_role::try_get_role(e, ADMIN) else {
        panic_with_error!(e, ManageableError::AdminNotSet);
    };
    admin.require_auth();
    admin
}
