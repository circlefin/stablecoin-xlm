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
use soroban_sdk::{Address, Env};
use stellar_access::ownable::{emit_ownership_transfer_completed, OwnableStorageKey};

/// Returns the pending owner address if set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
pub fn get_pending_owner(e: &Env) -> Option<Address> {
    e.storage()
        .temporary()
        .get(&OwnableStorageKey::PendingOwner)
}

/// Sets the owner and emits an ownership_transfer_completed event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_owner` - The address of the new owner.
///
/// # Events
///
/// * topics - `["ownership_transfer_completed"]`
/// * data - `[new_owner: Address]`
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call this function only in the constructor!
pub fn set_owner_unchecked(e: &Env, new_owner: &Address) {
    stellar_access::ownable::set_owner(e, new_owner);
    emit_ownership_transfer_completed(e, new_owner);
}

/// Initiates a 2-step ownership transfer to a new address.
///
/// Requires authorization from the current owner. The new owner must later
/// call `accept_ownership()` to complete the transfer.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_owner` - The proposed new owner.
/// * `expires_in_ledgers` - Number of ledgers **relative to the current ledger
///   sequence** until which the new owner can accept. This is converted to an
///   absolute ledger sequence number (`current_ledger + expires_in_ledgers`)
///   before being passed to the underlying OpenZeppelin `transfer_ownership`,
///   which expects an absolute `live_until_ledger` value. A value of `0`
///   cancels any pending transfer.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the current
///   owner fails.
pub fn transfer_ownership(e: &Env, new_owner: &Address, expires_in_ledgers: u32) {
    let live_until_ledger = if expires_in_ledgers == 0 {
        0
    } else {
        e.ledger()
            .sequence()
            .checked_add(expires_in_ledgers)
            .expect("ledger sequence overflow")
    };
    stellar_access::ownable::transfer_ownership(e, new_owner, live_until_ledger);
}
