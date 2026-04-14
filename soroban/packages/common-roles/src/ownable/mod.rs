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
//! # Ownable Contract Module.
//!
//! This module introduces a simple access control mechanism where a contract
//! has an account (owner) that can be granted exclusive access to specific
//! functions.
//!
//! The `Ownable` trait exposes methods for:
//! - Getting the current owner
//! - Transferring ownership
//!
//! The helper `enforce_owner_auth()` is available to restrict access to only
//! the owner. You can also use the `#[only_owner]` macro (provided elsewhere)
//! to simplify this.
//!
//! ```ignore
//! #[only_owner]
//! fn set_config(e: &Env, new_config: u32) { ... }
//! ```
//!
//! See `examples/ownable/src/contract.rs` for a working example.
//!
//! ## Note
//!
//! The ownership transfer is processed in 2 steps:
//!
//! 1. Initiating the ownership transfer by the current owner
//! 2. Accepting the ownership by the designated owner
//!
//! Not providing a direct ownership transfer is a deliberate design decision to
//! help avoid mistakes by transferring to a wrong address.
//!
//! Forked from https://github.com/OpenZeppelin/stellar-contracts/blob/969a22096aea7326adf076f0efcde290cc476cba/packages/access/src/ownable/mod.rs
//! Modifications:
//! 1. Remove renounce_ownership function (12/1/2025)
//! 2. Adjust transfer_ownership to use expires_in_ledgers instead of live_until_ledger (12/22/2025)

use soroban_sdk::{Address, Env};

pub use stellar_access::ownable::{accept_ownership, get_owner};

mod storage;
#[cfg(test)]
mod test;
pub use storage::{get_pending_owner, set_owner_unchecked, transfer_ownership};

/// A trait for managing contract ownership using a 2-step transfer pattern.
///
/// Provides functions to query ownership and initiate a 2-step ownership
/// transfer.
pub trait Ownable {
    /// Returns the current owner address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_owner(e: &Env) -> Option<Address>;

    /// Returns the pending owner address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_pending_owner(e: &Env) -> Option<Address>;

    /// Initiates a 2-step ownership transfer to a new address.
    ///
    /// Requires authorization from the current owner. The new owner must later
    /// call `accept_ownership()` to complete the transfer.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_owner` - The proposed new owner.
    /// * `expires_in_ledgers` - Number of ledgers **relative to the current
    ///   ledger sequence** until which the new owner can accept. This is
    ///   converted to an absolute ledger sequence number before being passed to
    ///   the underlying OpenZeppelin `transfer_ownership`, which expects an
    ///   absolute `live_until_ledger` value. A value of `0` cancels any pending
    ///   transfer.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   current owner fails.
    /// * [`OwnableError::OwnerNotSet`] – If the owner is not set.
    /// * [`stellar_access::role_transfer::RoleTransferError::NoPendingTransfer`] –
    ///   If trying to cancel a transfer that doesn't exist.
    /// * [`stellar_access::role_transfer::RoleTransferError::InvalidLiveUntilLedger`] –
    ///   If the specified ledger is in the past.
    /// * [`stellar_access::role_transfer::RoleTransferError::InvalidPendingAccount`] –
    ///   If the specified pending account is not the same as the provided `new`
    ///   address.
    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32);

    /// Accepts a pending ownership transfer.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   pending owner fails.
    /// * [`stellar_access::role_transfer::RoleTransferError::NoPendingTransfer`] –
    ///   If there is no pending transfer to accept.
    ///
    /// # Events
    ///
    /// * topics - `["ownership_transfer_completed"]`
    /// * data - `[new_owner: Address]`
    fn accept_ownership(e: &Env);
}
