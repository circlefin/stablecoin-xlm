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
//! UpgradeV2 - A test contract that TestContract can be upgraded TO.
//!
//! This contract uses the `#[derive(Upgradeable)]` macro and implements
//! `UpgradeableInternal` to enable further upgrades, as well as the two-step
//! admin management pattern (Manageable). It is used to test the simple upgrade
//! flow (without migration).
//!
//! Based on: https://github.com/OpenZeppelin/stellar-contracts/blob/e23aafa13cbf6062f0bafa9391b75df3fb74d847/examples/upgradeable/v1/src/contract.rs

use common_roles::{manageable, ownable, simple_role, Manageable, Ownable};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::Upgradeable;

pub const DATA: Symbol = symbol_short!("DATA");

#[derive(Upgradeable)]
#[contract]
pub struct UpgradeV2;

#[contractimpl]
impl UpgradeV2 {
    pub fn __constructor(e: &Env, owner: Address, admin: Address) {
        ownable::set_owner_unchecked(e, &owner);
        manageable::set_admin_unchecked(e, &admin);
    }

    /// Stores arbitrary data in instance storage for testing persistence.
    pub fn set_data(e: &Env, data: u32) {
        e.storage().instance().set(&DATA, &data);
    }

    /// Retrieves stored data from instance storage.
    pub fn get_data(e: &Env) -> Option<u32> {
        e.storage().instance().get(&DATA)
    }

    /// V2-only function to demonstrate new functionality after upgrade.
    pub fn v2_only_function() -> u32 {
        200
    }
}

impl UpgradeableInternal for UpgradeV2 {
    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }
}

#[contractimpl]
impl Ownable for UpgradeV2 {
    fn get_owner(e: &Env) -> Option<Address> {
        ownable::get_owner(e)
    }

    fn get_pending_owner(e: &Env) -> Option<Address> {
        ownable::get_pending_owner(e)
    }

    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32) {
        ownable::transfer_ownership(e, &new_owner, expires_in_ledgers)
    }

    fn accept_ownership(e: &Env) {
        ownable::accept_ownership(e)
    }
}

#[contractimpl]
impl Manageable for UpgradeV2 {
    fn get_admin(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, manageable::ADMIN)
    }

    fn get_pending_admin(e: &Env) -> Option<Address> {
        manageable::get_pending_admin(e)
    }

    fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32) {
        manageable::transfer_admin(e, &new_admin, expires_in_ledgers)
    }

    fn accept_admin(e: &Env) {
        manageable::accept_admin(e)
    }
}
