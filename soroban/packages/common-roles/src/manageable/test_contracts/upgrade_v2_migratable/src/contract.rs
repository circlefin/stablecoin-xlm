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
//! UpgradeV2Migratable - A test contract that TestContract can be upgraded TO.
//!
//! This contract uses `#[derive(UpgradeableMigratable)]` and implements
//! `UpgradeableMigratableInternal` to enable upgrades with a migration step, as
//! well as the two-step admin management pattern (Manageable). It is used to
//! test the upgrade-with-migration flow.
//!
//! Based on: https://github.com/OpenZeppelin/stellar-contracts/blob/e23aafa13cbf6062f0bafa9391b75df3fb74d847/examples/upgradeable/v2/src/contract.rs

use common_roles::{manageable, ownable, simple_role, Manageable, Ownable};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};
use stellar_contract_utils::upgradeable::UpgradeableMigratableInternal;
use stellar_macros::UpgradeableMigratable;

pub const MIGRATION_DATA_KEY: Symbol = symbol_short!("MIG_DATA");
pub const DATA: Symbol = symbol_short!("DATA");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MigrationError {
    MigrationNotAllowed = 1100,
}

#[contracttype]
#[derive(Clone)]
pub struct V2MigrationData {
    pub num1: u32,
    pub num2: u32,
}

#[derive(UpgradeableMigratable)]
#[contract]
pub struct UpgradeV2Migratable;

#[contractimpl]
impl UpgradeV2Migratable {
    /// Stores arbitrary data in instance storage for testing persistence.
    pub fn set_data(e: &Env, data: u32) {
        e.storage().instance().set(&DATA, &data);
    }

    /// Retrieves stored data from instance storage (persisted from v1).
    pub fn get_data(e: &Env) -> Option<u32> {
        e.storage().instance().get(&DATA)
    }

    /// V2-only function to demonstrate new functionality after upgrade.
    pub fn v2_only_function() -> u32 {
        200
    }

    /// Retrieves migration data set during migrate().
    pub fn get_migration_data(e: &Env) -> V2MigrationData {
        e.storage().instance().get(&MIGRATION_DATA_KEY).unwrap()
    }
}

impl UpgradeableMigratableInternal for UpgradeV2Migratable {
    type MigrationData = V2MigrationData;

    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }

    fn _migrate(e: &Env, data: &Self::MigrationData) {
        e.storage().instance().set(&MIGRATION_DATA_KEY, data);
    }
}

#[contractimpl]
impl Ownable for UpgradeV2Migratable {
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
impl Manageable for UpgradeV2Migratable {
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
