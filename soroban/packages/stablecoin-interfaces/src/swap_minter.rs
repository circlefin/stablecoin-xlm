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
//! SwapMinter module defining an interface for swapping registered allowance assets for a mintable asset.

use soroban_sdk::{contractclient, Address, Env};

/// Performs swaps of registered allowance assets for a mintable asset.
#[contractclient(name = "SwapMinterClient")]
pub trait SwapMinter {
    /// Swaps the minter's registered allowance asset for the mint asset.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `to` - The recipient.
    /// * `amount` - The amount to mint (must be greater than zero).
    /// * `minter` - The minter address (must be registered via `configure_minter`).
    fn swap_mint(e: Env, to: Address, amount: i128, minter: Address);
}
