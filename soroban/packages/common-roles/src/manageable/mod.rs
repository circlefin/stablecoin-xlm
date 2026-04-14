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
//! # Manageable Contract Module.
//!
//! Two-step admin management following the Starknet `manageable` pattern.
//! Only the current admin can initiate a transfer and the pending admin must
//! accept to complete it.
//!
//! This module exposes:
//! - Getting the current admin
//! - Getting the pending admin
//! - Initiating an admin transfer
//! - Accepting an admin transfer
//!
//! ## Usage
//!
//! Contracts implementing the `Manageable` trait use `simple_role` directly:
//!
//! ```ignore
//! use common_roles::{manageable, simple_role};
//!
//! impl Manageable for MyContract {
//!     fn get_admin(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, manageable::ADMIN)
//!     }
//!
//!     fn get_pending_admin(e: &Env) -> Option<Address> {
//!         manageable::get_pending_admin(e)
//!     }
//!
//!     fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32) {
//!         manageable::transfer_admin(e, &new_admin, expires_in_ledgers)
//!     }
//!
//!     fn accept_admin(e: &Env) {
//!         manageable::accept_admin(e)
//!     }
//! }
//! ```
//!
//! Forked from https://github.com/OpenZeppelin/stellar-contracts/blob/main/packages/access/src/ownable/mod.rs
//! Modifications:
//! 1. Adapted for admin role instead of owner role
//! 2. Renamed functions and events to use "admin" terminology

mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{contractevent, Address, Env};

pub use storage::{
    accept_admin, enforce_admin_auth, get_pending_admin, set_admin_unchecked, transfer_admin,
    ManageableError, ManageableStorageKey,
};

/// Role identifier for the admin.
pub const ADMIN: &str = "admin";

/// A trait for managing the admin role with a 2-step transfer process.
pub trait Manageable {
    /// Returns the current admin address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_admin(e: &Env) -> Option<Address>;

    /// Returns the pending admin address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_pending_admin(e: &Env) -> Option<Address>;

    /// Initiates a 2-step admin transfer to a new address.
    ///
    /// Requires authorization from the current admin. The pending admin must
    /// later call `accept_admin()` to complete the transfer.
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
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   current admin fails.
    /// * [`ManageableError::AdminNotSet`] – If the admin is not set.
    /// * [`stellar_access::role_transfer::RoleTransferError::NoPendingTransfer`] –
    ///   If trying to cancel a transfer that doesn't exist.
    /// * [`stellar_access::role_transfer::RoleTransferError::InvalidLiveUntilLedger`] –
    ///   If the specified ledger is in the past.
    /// * [`stellar_access::role_transfer::RoleTransferError::InvalidPendingAccount`] –
    ///   If the specified pending account is not the same as the provided
    ///   `new_admin` address.
    fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32);

    /// Accepts a pending admin transfer.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   pending admin fails.
    /// * [`stellar_access::role_transfer::RoleTransferError::NoPendingTransfer`] –
    ///   If there is no pending transfer to accept.
    ///
    /// # Events
    ///
    /// * topics - `["admin_changed"]`
    /// * data - `[old_admin: Option<Address>, new_admin: Address]`
    fn accept_admin(e: &Env);
}

// ================================ EVENTS ====================================

/// Event emitted when an admin transfer is initiated.
#[contractevent]
pub struct AdminChangeStarted {
    pub old_admin: Option<Address>,
    pub new_admin: Address,
}

/// Event emitted when an admin transfer is completed.
#[contractevent]
pub struct AdminChanged {
    pub old_admin: Option<Address>,
    pub new_admin: Address,
}

/// Emits an AdminChangeStarted event.
pub fn emit_admin_change_started(e: &Env, old_admin: Option<Address>, new_admin: &Address) {
    let event = AdminChangeStarted {
        old_admin,
        new_admin: new_admin.clone(),
    };
    event.publish(e);
}

/// Emits an AdminChanged event.
pub fn emit_admin_changed(e: &Env, old_admin: Option<Address>, new_admin: &Address) {
    let event = AdminChanged {
        old_admin,
        new_admin: new_admin.clone(),
    };
    event.publish(e);
}
