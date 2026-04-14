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
use common_roles::simple_role;
use simple_role_macros::enforce_role_auth;
use soroban_sdk::{token::StellarAssetClient, Address, Env};

use crate::{storage, BLOCKLISTER};

pub trait Blocklistable {
    /// Returns the current blocklister address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_blocklister(e: Env) -> Option<Address>;

    /// Updates the blocklister address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `blocklister` - The address that will be authorized to blocklist/unblocklist addresses.
    fn update_blocklister(e: Env, blocklister: Address);

    /// Returns whether an address is blocklisted.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `address` - The address to check.
    fn is_blocklisted(e: Env, address: Address) -> bool;

    /// Blocklists an address, preventing it from interacting with the mint asset.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `address` - The address to blocklist.
    fn blocklist(e: Env, address: Address);

    /// Unblocklists an address, allowing it to interact with the mint asset.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `address` - The address to unblocklist.
    fn unblocklist(e: Env, address: Address);
}

/// Returns whether an address is blocklisted.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `address` - The address to check.
///
/// # Returns
///
/// * `true` – If the address is blocklisted.
/// * `false` – If the address is not blocklisted.
pub fn is_blocklisted(e: Env, address: Address) -> bool {
    let mint_asset = storage::get_mint_asset(&e);
    !StellarAssetClient::new(&e, &mint_asset).authorized(&address)
}

/// Blocklists an address, preventing it from interacting with the mint asset.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `address` - The address to blocklist.
///
/// # Errors
///
/// * [`simple_role::RoleError::RoleNotSet`] – If the blocklister role is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the blocklister fails.
///
/// # Events
///
/// Emitted by the admin contract:
/// * topics - `["blocklisted"]`
/// * data - `[address: Address]`
///
/// Emitted by the mint asset SAC:
/// * topics - `["set_authorized", address: Address, admin: Address]`
/// * data - `authorize: bool`
#[enforce_role_auth(BLOCKLISTER)]
pub fn blocklist(e: Env, address: Address) {
    let mint_asset = storage::get_mint_asset(&e);
    StellarAssetClient::new(&e, &mint_asset).set_authorized(&address, &false);
    crate::emit_blocklisted(&e, &address);
}

/// Unblocklists an address, allowing it to interact with the mint asset.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `address` - The address to unblocklist.
///
/// # Errors
///
/// * [`simple_role::RoleError::RoleNotSet`] – If the blocklister role is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the blocklister fails.
///
/// # Events
///
/// Emitted by the admin contract:
/// * topics - `["unblocklisted"]`
/// * data - `[address: Address]`
///
/// Emitted by the mint asset SAC:
/// * topics - `["set_authorized", address: Address, admin: Address]`
/// * data - `authorize: bool`
#[enforce_role_auth(BLOCKLISTER)]
pub fn unblocklist(e: Env, address: Address) {
    let mint_asset = storage::get_mint_asset(&e);
    StellarAssetClient::new(&e, &mint_asset).set_authorized(&address, &true);
    crate::emit_unblocklisted(&e, &address);
}
