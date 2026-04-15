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
use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use crate::FiatTokenAdminError;

#[contracttype]
pub enum FiatTokenAdminStorageKey {
    MintAsset,
}

/// Sets the mint asset address and emits an event.
///
/// Can only be called once. Panics if the mint asset has already been set,
/// ensuring true immutability even within the contract itself.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `mint_asset` - The mint asset address.
///
/// # Errors
///
/// * [`FiatTokenAdminError::MintAssetAlreadySet`] – If the mint asset has already been set.
///
/// # Events
///
/// * topics - `["mint_asset_set"]`
/// * data - `[mint_asset: Address]`
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call this function only in the constructor!
pub fn set_mint_asset_unchecked(e: &Env, mint_asset: &Address) {
    if e.storage()
        .instance()
        .has(&FiatTokenAdminStorageKey::MintAsset)
    {
        panic_with_error!(e, FiatTokenAdminError::MintAssetAlreadySet);
    }

    e.storage()
        .instance()
        .set(&FiatTokenAdminStorageKey::MintAsset, mint_asset);
    crate::emit_mint_asset_set(e, mint_asset);
}

/// Returns the mint asset address.
///
/// This value is set once during contract initialization and cannot be changed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The mint asset address.
///
/// # Errors
///
/// * [`FiatTokenAdminError::MintAssetNotSet`] – If the mint asset has not been set.
pub fn get_mint_asset(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&FiatTokenAdminStorageKey::MintAsset)
        .unwrap_or_else(|| panic_with_error!(e, FiatTokenAdminError::MintAssetNotSet))
}
