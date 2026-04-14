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
use soroban_sdk::Env;
use stellar_contract_utils::pausable;

use super::PAUSER;

/// Triggers `Paused` state.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Errors
///
/// * `HostError: Error(Contract, #1000)` – Contract is already paused.
/// * [`simple_role::RoleError::RoleNotSet`] – If the pauser is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the pauser
///   fails.
///
/// # Events
///
/// * topics - `["paused"]`
/// * data - `[]`
#[enforce_role_auth(PAUSER)]
pub fn pause(e: &Env) {
    pausable::pause(e);
}

/// Triggers `Unpaused` state.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Errors
///
/// * `HostError: Error(Contract, #1001)` – Contract is already unpaused.
/// * [`simple_role::RoleError::RoleNotSet`] – If the pauser is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the pauser
///   fails.
///
/// # Events
///
/// * topics - `["unpaused"]`
/// * data - `[]`
#[enforce_role_auth(PAUSER)]
pub fn unpause(e: &Env) {
    pausable::unpause(e);
}
