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
#![no_std]

mod blocklistable;
mod contract;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contracterror, contractevent, Address, Env};

/// Role identifier for the blocklister.
pub const BLOCKLISTER: &str = "blocklister";

// ################## ERRORS ##################

/// Errors for the Fiat Token Admin contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FiatTokenAdminError {
    /// The mint asset has not been set.
    MintAssetNotSet = 6800,
    /// The mint asset has already been set and cannot be changed.
    MintAssetAlreadySet = 6801,
    /// The amount must be greater than zero.
    InvalidAmount = 6802,
    /// The minter is not configured with an asset.
    MinterNotConfigured = 6803,
    /// The minter address must not be blocklisted
    InvalidMinterAddress = 6804,
}

// ################## EVENTS ##################

/// Event emitted when the mint asset is set.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintAssetSet {
    pub mint_asset: Address,
}

/// Emits an event when the mint asset is set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `mint_asset` - The mint asset address.
///
/// # Events
///
/// * topics - `["mint_asset_set"]`
/// * data - `[mint_asset: Address]`
pub fn emit_mint_asset_set(e: &Env, mint_asset: &Address) {
    MintAssetSet {
        mint_asset: mint_asset.clone(),
    }
    .publish(e);
}

/// Event emitted when the blocklister address is changed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlocklisterChanged {
    pub new_blocklister: Address,
}

/// Emits an event when the blocklister is changed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_blocklister` - The address of the new blocklister.
pub fn emit_blocklister_changed(e: &Env, new_blocklister: &Address) {
    BlocklisterChanged {
        new_blocklister: new_blocklister.clone(),
    }
    .publish(e);
}

/// Event emitted when an address is blocklisted.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blocklisted {
    pub address: Address,
}

/// Emits an event when an address is blocklisted.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `address` - The address that was blocklisted.
pub fn emit_blocklisted(e: &Env, address: &Address) {
    Blocklisted {
        address: address.clone(),
    }
    .publish(e);
}

/// Event emitted when an address is unblocklisted.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unblocklisted {
    pub address: Address,
}

/// Emits an event when an address is unblocklisted.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `address` - The address that was unblocklisted.
pub fn emit_unblocklisted(e: &Env, address: &Address) {
    Unblocklisted {
        address: address.clone(),
    }
    .publish(e);
}
