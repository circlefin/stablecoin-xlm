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
//! Minter Asset Controllable Module.
//!
//! This module provides functionality for managing minter-to-asset mappings.
//! Each minter has at most one asset controlling its mint allowance.
//! The minter asset controller can add and remove minters,
//! while the contract owner can set who the controller is.
//!
//! ## Usage
//!
//! Contracts implementing the `MinterAssetControllable` trait use `simple_role` directly:
//!
//! ```ignore
//! use stablecoin_roles::{minter_asset_controllable, MinterAssetControllable};
//! use simple_role;
//!
//! impl MinterAssetControllable for MyContract {
//!     fn get_minter_asset_controller(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, minter_asset_controllable::MINTER_ASSET_CONTROLLER)
//!     }
//!
//!     fn update_minter_asset_controller(e: &Env, new_controller: Address) {
//!         simple_role::set_role_and_emit(
//!             e,
//!             minter_asset_controllable::MINTER_ASSET_CONTROLLER,
//!             &new_controller,
//!             minter_asset_controllable::emit_minter_asset_controller_changed,
//!         );
//!     }
//!
//!     fn get_minter_allow_asset(e: &Env, minter: Address) -> Option<Address> {
//!         minter_asset_controllable::get_minter_allow_asset(e, &minter)
//!     }
//!
//!     fn configure_minter(e: &Env, minter: Address, asset: Address) {
//!         minter_asset_controllable::configure_minter(e, &minter, &asset)
//!     }
//!
//!     fn remove_minter(e: &Env, minter: Address) {
//!         minter_asset_controllable::remove_minter(e, &minter)
//!     }
//! }
//! ```

use soroban_sdk::{contracterror, contractevent, Address, Env};

mod storage;
#[cfg(test)]
mod test;

pub use storage::{configure_minter, get_minter_allow_asset, remove_minter};

/// Role identifier for the minter asset controller.
pub const MINTER_ASSET_CONTROLLER: &str = "minter_asset_controller";

/// A trait for managing minter-to-allow-asset mappings.
///
/// Provides functions to query and manage minters with their assigned assets,
/// as well as manage the controller who has permission to modify the mappings.
pub trait MinterAssetControllable {
    /// Returns the minter asset controller address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` – The minter asset controller address if set.
    /// * `None` – If the minter asset controller has not been set.
    fn get_minter_asset_controller(e: &Env) -> Option<Address>;

    /// Updates the minter asset controller address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_controller` - The address that will be authorized to add/remove minters.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["minter_asset_controller_changed"]`
    /// * data - `[new_controller: Address]`
    fn update_minter_asset_controller(e: &Env, new_controller: Address);

    /// Returns the allow asset assigned to a minter.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `minter` - The minter address to query.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` – The asset address assigned to the minter.
    /// * `None` – If the minter is not registered.
    fn get_minter_allow_asset(e: &Env, minter: Address) -> Option<Address>;

    /// Adds a minter with their assigned asset.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `minter` - The minter address to add.
    /// * `asset` - The asset address to assign to the minter.
    ///
    /// # Errors
    ///
    /// * [`simple_role::RoleError::RoleNotSet`] – If the controller is not set.
    /// * [`MinterAssetControllableError::InvalidAllowAsset`] – If the allow asset is the mint asset.
    /// * `HostError: Error(Auth, InvalidAction)` – If authorization from the controller fails.
    ///
    /// # Events
    ///
    /// * topics - `["minter_configured"]`
    /// * data - `[minter: Address, asset: Address]`
    fn configure_minter(e: &Env, minter: Address, allow_asset: Address);

    /// Removes a minter.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `minter` - The minter address to remove.
    ///
    /// # Errors
    ///
    /// * [`simple_role::RoleError::RoleNotSet`] – If the controller is not set.
    /// * [`MinterAssetControllableError::MinterNotFound`] – If the minter is not registered.
    /// * `HostError: Error(Auth, InvalidAction)` – If authorization from the controller fails.
    ///
    /// # Events
    ///
    /// * topics - `["minter_removed"]`
    /// * data - `[minter: Address]`
    fn remove_minter(e: &Env, minter: Address);
}

// ################## ERRORS ##################

/// Errors for the minter asset controllable module.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MinterAssetControllableError {
    /// The allow asset is invalid.
    InvalidAllowAsset = 6701,
    /// The minter is not registered.
    MinterNotFound = 6702,
}

// ################## EVENTS ##################

/// Event emitted when the minter asset controller is changed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterAssetControllerChanged {
    pub new_controller: Address,
}

/// Emits an event when the minter asset controller is changed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_controller` - The new minter asset controller address.
///
/// # Events
///
/// * topics - `["minter_asset_controller_changed"]`
/// * data - `[new_controller: Address]`
pub fn emit_minter_asset_controller_changed(e: &Env, new_controller: &Address) {
    MinterAssetControllerChanged {
        new_controller: new_controller.clone(),
    }
    .publish(e);
}

/// Event emitted when a minter is added with their allow asset.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterConfigured {
    pub minter: Address,
    pub allow_asset: Address,
}

/// Emits an event when a minter is configured.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `minter` - The minter address.
/// * `allow_asset` - The allow asset address assigned to the minter.
///
/// # Events
///
/// * topics - `["minter_configured"]`
/// * data - `[minter: Address, allow_asset: Address]`
pub fn emit_minter_configured(e: &Env, minter: &Address, allow_asset: &Address) {
    MinterConfigured {
        minter: minter.clone(),
        allow_asset: allow_asset.clone(),
    }
    .publish(e);
}

/// Event emitted when a minter is removed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterRemoved {
    pub minter: Address,
}

/// Emits an event when a minter is removed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `minter` - The minter address that was removed.
///
/// # Events
///
/// * topics - `["minter_removed"]`
/// * data - `[minter: Address]`
pub fn emit_minter_removed(e: &Env, minter: &Address) {
    MinterRemoved {
        minter: minter.clone(),
    }
    .publish(e);
}
