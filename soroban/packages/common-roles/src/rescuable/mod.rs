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

//! # Rescuable Contract Module
//!
//! This module provides functionality for managing a "rescuer" role in Stellar smart contracts.
//! The rescuer is an address that is authorized to rescue tokens accidentally sent to a contract.
//!
//! ## Usage
//!
//! Contracts implementing the `Rescuable` trait use `simple_role` directly:
//!
//! ```ignore
//! use common_roles::{rescuable, simple_role};
//!
//! impl Rescuable for MyContract {
//!     fn get_rescuer(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, rescuable::RESCUER)
//!     }
//!
//!     fn update_rescuer(e: &Env, new_rescuer: Address) {
//!         simple_role::set_role_and_emit(
//!             e,
//!             rescuable::RESCUER,
//!             &new_rescuer,
//!             rescuable::emit_rescuer_changed,
//!         );
//!     }
//!
//!     fn rescue_sep41(e: &Env, token_contract: Address, to: Address, amount: i128) {
//!         rescuable::rescue_sep41(e, &token_contract, &to, &amount)
//!     }
//! }
//! ```

mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{contractevent, Address, Env};

pub use storage::rescue_sep41;

/// Role identifier for the rescuer.
pub const RESCUER: &str = "rescuer";

/// A trait for contracts that implement the Rescuable role.
///
/// This trait provides a standardized interface for rescuing tokens
/// that have been accidentally sent to a contract.
pub trait Rescuable {
    /// Returns the current rescuer address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if the rescuer is set, `None` otherwise.
    fn get_rescuer(e: &Env) -> Option<Address>;

    /// Updates the rescuer address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_rescuer` - The address that will be authorized to rescue tokens.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["RescuerChanged"]`
    /// * data - `[new_rescuer: Address]`
    fn update_rescuer(e: &Env, new_rescuer: Address);

    /// Rescues SEP-41 tokens that were sent to the contract.
    ///
    /// This function transfers tokens from the contract to a specified address.
    /// Only the designated rescuer can call this function.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `token_contract` - The address of the token contract to rescue from.
    /// * `to` - The address to send the rescued tokens to.
    /// * `amount` - The amount of tokens to rescue.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   rescuer fails.
    /// * [`simple_role::RoleError::RoleNotSet`] – Rescuer is not set.
    fn rescue_sep41(e: &Env, token_contract: Address, to: Address, amount: i128);
}

// ################## EVENTS ##################

/// Event emitted when the rescuer address is changed.
#[contractevent]
pub struct RescuerChanged {
    pub new_rescuer: Address,
}

/// Emits an event when the rescuer address is changed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_rescuer` - The new rescuer address.
pub fn emit_rescuer_changed(e: &Env, new_rescuer: &Address) {
    RescuerChanged {
        new_rescuer: new_rescuer.clone(),
    }
    .publish(e);
}
