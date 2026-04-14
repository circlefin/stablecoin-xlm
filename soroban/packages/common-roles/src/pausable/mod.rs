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
//! Pausable Contract Module.
//!
//! This contract module allows implementing a configurable stop mechanism for
//! your contract.
//!
//! By implementing the trait [`Pausable`] for your contract, you can safely
//! integrate the Pausable functionality. The trait [`Pausable`] has the
//! following methods:
//! - [`paused()`]
//! - [`pause()`]
//! - [`unpause()`]
//!
//! The trait ensures all the required methods are implemented for your
//! contract, and nothing is forgotten. Additionally, if you are to implement
//! multiple extensions or utilities for your contract, the code will be better
//! organized.
//!
//! We also provide two macros `when_paused` and `when_not_paused`. These macros
//! act as guards for your functions. For example:
//!
//! ```ignore
//! #[when_not_paused]
//! fn transfer(e: &env, from: Address, to: Address) {
//!     /* this body will execute ONLY when NOT_PAUSED */
//! }
//! ```
//!
//! ## Usage
//!
//! Contracts implementing the `Pausable` trait use `simple_role` directly:
//!
//! ```ignore
//! use common_roles::{pausable, simple_role};
//!
//! impl Pausable for MyContract {
//!     fn get_pauser(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, pausable::PAUSER)
//!     }
//!
//!     fn update_pauser(e: &Env, new_pauser: Address) {
//!         simple_role::set_role_and_emit(
//!             e,
//!             pausable::PAUSER,
//!             &new_pauser,
//!             pausable::emit_pauser_changed,
//!         );
//!     }
//!
//!     fn paused(e: &Env) -> bool {
//!         pausable::paused(e)
//!     }
//!
//!     fn pause(e: &Env) {
//!         pausable::pause(e)
//!     }
//!
//!     fn unpause(e: &Env) {
//!         pausable::unpause(e)
//!     }
//! }
//! ```
//!
//! For a safe pause/unpause implementation, we expose the underlying functions
//! required for the pausing. These functions work with the Soroban environment
//! required for the Smart Contracts `e: &Env`, and take advantage of the
//! storage by storing a flag for the pause mechanism.
//!
//! Forked from https://github.com/OpenZeppelin/stellar-contracts/blob/969a22096aea7326adf076f0efcde290cc476cba/packages/contract-utils/src/pausable/mod.rs
//! Modifications:
//! 1. Add get_pauser, update_pauser functions (12/1/2025)
//! 2. Enforce pauser auth on pause and unpause implementations (12/1/2025)

use soroban_sdk::{contractevent, Address, Env};
pub use stellar_contract_utils::pausable::paused;

mod storage;
#[cfg(test)]
mod test;
pub use crate::pausable::storage::{pause, unpause};

/// Role identifier for the pauser.
pub const PAUSER: &str = "pauser";

pub trait Pausable {
    /// Returns the current pauser address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_pauser(e: &Env) -> Option<Address>;

    /// Updates the pauser address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_pauser` - The new pauser address.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["pauser_changed"]`
    /// * data - `[new_address: Address]`
    fn update_pauser(e: &Env, new_pauser: Address);

    /// Returns true if the contract is paused, and false otherwise.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn paused(e: &Env) -> bool;

    /// Triggers `Paused` state.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to Soroban environment.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is already paused.
    /// * [`simple_role::RoleError::RoleNotSet`] – If the pauser is not set.
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   pauser fails.
    ///
    /// # Events
    ///
    /// * topics - `["paused"]`
    /// * data - `[]`
    fn pause(e: &Env);

    /// Triggers `Unpaused` state.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to Soroban environment.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1001)` – Contract is already unpaused.
    /// * [`simple_role::RoleError::RoleNotSet`] – If the pauser is not set.
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   pauser fails.
    ///
    /// # Events
    ///
    /// * topics - `["unpaused"]`
    /// * data - `[]`
    fn unpause(e: &Env);
}

// ################## EVENTS ##################

/// Event emitted when the pauser is changed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauserChanged {
    pub new_address: Address,
}

/// Emits an event when the pauser is changed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_address` - The address of the new pauser.
pub fn emit_pauser_changed(e: &Env, new_address: &Address) {
    PauserChanged {
        new_address: new_address.clone(),
    }
    .publish(e);
}
