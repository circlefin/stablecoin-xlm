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
use simple_role_macros::enforce_role_auth;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use super::{
    emit_minter_configured, emit_minter_removed, MinterAssetControllableError,
    MINTER_ASSET_CONTROLLER,
};

#[contracttype]
pub enum MinterAssetControllableStorageKey {
    Minters(Address),
}

/// Returns the allow asset assigned to a minter.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `minter` - The minter address to query.
///
/// # Returns
///
/// * `Some(Address)` – The allow asset address assigned to the minter.
/// * `None` – If the minter is not registered.
pub fn get_minter_allow_asset(e: &Env, minter: &Address) -> Option<Address> {
    e.storage()
        .instance()
        .get(&MinterAssetControllableStorageKey::Minters(minter.clone()))
}

/// Configure the minter with their assigned allow asset.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `minter` - The minter address to add.
/// * `allow_asset` - The allow asset address to assign to the minter.
///
/// # Errors
///
/// * [`simple_role::RoleError::RoleNotSet`] – If the controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – If authorization from the controller fails.
///
/// # Events
///
/// * topics - `["minter_configured"]`
/// * data - `[minter: Address, allow_asset: Address]`
#[enforce_role_auth(MINTER_ASSET_CONTROLLER)]
pub fn configure_minter(e: &Env, minter: &Address, allow_asset: &Address) {
    e.storage().instance().set(
        &MinterAssetControllableStorageKey::Minters(minter.clone()),
        allow_asset,
    );
    emit_minter_configured(e, minter, allow_asset);
}

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
#[enforce_role_auth(MINTER_ASSET_CONTROLLER)]
pub fn remove_minter(e: &Env, minter: &Address) {
    if get_minter_allow_asset(e, minter).is_none() {
        panic_with_error!(e, MinterAssetControllableError::MinterNotFound);
    }

    e.storage()
        .instance()
        .remove(&MinterAssetControllableStorageKey::Minters(minter.clone()));
    emit_minter_removed(e, minter);
}
