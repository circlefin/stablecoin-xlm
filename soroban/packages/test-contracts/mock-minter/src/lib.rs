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

use soroban_sdk::{contract, contractimpl, Address, Env};
use stablecoin_interfaces::SwapMinterClient;

/// MockMinterContract is a minter contract that can swap mint tokens by interacting with
/// an admin contract (like FiatTokenAdminContract).
///
/// This is a realistic flow that Circle minter products will follow:
/// 1. Approve the allowance asset for the admin contract
/// 2. Call swap_mint on the admin contract
#[contract]
pub struct MockMinterContract;

#[contractimpl]
impl MockMinterContract {
    /// Executes a swap mint operation by:
    /// 1. Approving the admin contract to burn the allowance asset from this contract
    /// 2. Calling swap_mint on the admin contract
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `admin_contract` - The address of the admin contract (e.g., FiatTokenAdminContract).
    /// * `allowance_asset` - The address of the asset to be burned (the minter's registered asset).
    /// * `to` - The recipient of the minted tokens.
    /// * `amount` - The amount to swap mint.
    ///
    /// # Notes
    /// - This function is intentionally permissionless to make testing easier.
    pub fn execute_swap_mint(
        env: Env,
        admin_contract: Address,
        allowance_asset: Address,
        to: Address,
        amount: i128,
    ) {
        let this_contract = env.current_contract_address();

        let allowance_client = soroban_sdk::token::StellarAssetClient::new(&env, &allowance_asset);
        allowance_client.approve(
            &this_contract,
            &admin_contract,
            &amount,
            &env.ledger().sequence(),
        );

        let admin_client = SwapMinterClient::new(&env, &admin_contract);
        admin_client.swap_mint(&to, &amount, &this_contract);
    }

    /// Attempts to execute a swap mint operation while impersonating another minter.
    /// This is used for testing that the admin contract properly validates minter auth.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `admin_contract` - The address of the admin contract.
    /// * `allowance_asset` - The address of the asset to be burned.
    /// * `to` - The recipient of the minted tokens.
    /// * `amount` - The amount to swap mint.
    /// * `minter` - The minter address to pass (may differ from this contract).
    pub fn execute_swap_mint_as(
        env: Env,
        admin_contract: Address,
        allowance_asset: Address,
        to: Address,
        amount: i128,
        minter: Address,
    ) {
        let this_contract = env.current_contract_address();

        let allowance_client = soroban_sdk::token::StellarAssetClient::new(&env, &allowance_asset);
        allowance_client.approve(
            &this_contract,
            &admin_contract,
            &amount,
            &env.ledger().sequence(),
        );

        let admin_client = SwapMinterClient::new(&env, &admin_contract);
        admin_client.swap_mint(&to, &amount, &minter);
    }
}
