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
use soroban_sdk::{token, Address, Env};

use super::RESCUER;

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
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the rescuer fails.
/// * [`simple_role::RoleError::RoleNotSet`] – Rescuer is not set.
#[enforce_role_auth(RESCUER)]
pub fn rescue_sep41(e: &Env, token_contract: &Address, to: &Address, amount: &i128) {
    let token_client = token::TokenClient::new(e, token_contract);
    let contract_address = e.current_contract_address();
    token_client.transfer(&contract_address, to, amount);
}
