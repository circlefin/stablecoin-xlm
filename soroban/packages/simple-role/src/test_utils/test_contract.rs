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

//! Test contract and mock auth helpers for simple-role tests.

extern crate std;

use soroban_sdk::{
    contract, contractevent, contractimpl,
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};
use stellar_access::ownable::set_owner;

// =============================================================================
// Constants and Events
// =============================================================================

/// Default role name used by the test contract.
pub const TEST_ROLE: &str = "test_role";

/// Event emitted when a role is set (new address only).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleSet {
    pub new_address: Address,
}

/// Event emitted when a role is updated (old and new address).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleUpdated {
    pub previous_address: Option<Address>,
    pub new_address: Address,
}

/// Emits a RoleSet event.
pub fn emit_role_set(e: &Env, new_address: &Address) {
    RoleSet {
        new_address: new_address.clone(),
    }
    .publish(e);
}

/// Emits a RoleUpdated event.
pub fn emit_role_updated(e: &Env, previous_address: Option<Address>, new_address: &Address) {
    RoleUpdated {
        previous_address,
        new_address: new_address.clone(),
    }
    .publish(e);
}

/// Helper to convert soroban_sdk::String to std::string::String.
pub fn string_to_str(s: &soroban_sdk::String) -> std::string::String {
    let len = s.len() as usize;
    let mut bytes = std::vec![0u8; len];
    s.copy_into_slice(&mut bytes);
    std::string::String::from_utf8(bytes).expect("Invalid UTF-8")
}

// =============================================================================
// Test Contract
// =============================================================================

/// Test contract for simple-role tests.
#[contract]
pub struct TestContract;

#[contractimpl]
impl TestContract {
    /// Sets the owner address (unchecked, for test setup).
    pub fn set_owner(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }

    // =========================================================================
    // Methods using TEST_ROLE constant (used by most tests)
    // =========================================================================

    pub fn test_try_get_role(env: Env) -> Option<Address> {
        crate::try_get_role(&env, TEST_ROLE)
    }

    pub fn get_role(env: Env) -> Address {
        crate::get_role(&env, TEST_ROLE)
    }

    pub fn set_role(env: Env, address: Address) {
        crate::set_role(&env, TEST_ROLE, &address);
    }

    pub fn set_role_unchecked(env: Env, address: Address) {
        crate::set_role_unchecked(&env, TEST_ROLE, &address);
    }

    pub fn set_role_and_emit(env: Env, address: Address) {
        crate::set_role_and_emit(&env, TEST_ROLE, &address, emit_role_set);
    }

    pub fn set_role_emit_unchecked(env: Env, address: Address) {
        crate::set_role_and_emit_unchecked(&env, TEST_ROLE, &address, emit_role_set);
    }

    pub fn set_role_emit_prev(env: Env, address: Address) {
        crate::set_role_and_emit_with_previous(&env, TEST_ROLE, &address, emit_role_updated);
    }

    pub fn set_role_emit_prev_unchk(env: Env, address: Address) {
        crate::set_role_and_emit_with_previous_unchecked(
            &env,
            TEST_ROLE,
            &address,
            emit_role_updated,
        );
    }

    pub fn enforce_role_auth(env: Env) -> Address {
        crate::enforce_role_auth(&env, TEST_ROLE)
    }

    // =========================================================================
    // Methods with custom role parameter (for multi-role tests)
    // =========================================================================

    pub fn get_role_for(env: Env, role: soroban_sdk::String) -> Address {
        crate::get_role(&env, string_to_str(&role).as_str())
    }

    pub fn set_role_unchecked_for(env: Env, role: soroban_sdk::String, address: Address) {
        crate::set_role_unchecked(&env, string_to_str(&role).as_str(), &address);
    }

    pub fn enforce_role_auth_for(env: Env, role: soroban_sdk::String) -> Address {
        crate::enforce_role_auth(&env, string_to_str(&role).as_str())
    }
}

// =============================================================================
// Mock Auth Helpers
// =============================================================================

/// Mocks authorization for the `set_role` function.
///
/// # Arguments
///
/// * `env` - The Soroban test environment
/// * `contract_id` - The contract address
/// * `owner` - The owner address that should authorize the call
/// * `address` - The address being set as the role holder
pub fn mock_set_role_auth(env: &Env, contract_id: &Address, owner: &Address, address: &Address) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_role",
            args: (address.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for the `set_role_and_emit` function.
///
/// # Arguments
///
/// * `env` - The Soroban test environment
/// * `contract_id` - The contract address
/// * `owner` - The owner address that should authorize the call
/// * `address` - The address being set as the role holder
pub fn mock_set_role_and_emit_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    address: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_role_and_emit",
            args: (address.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for the `set_role_emit_prev` function.
///
/// # Arguments
///
/// * `env` - The Soroban test environment
/// * `contract_id` - The contract address
/// * `owner` - The owner address that should authorize the call
/// * `address` - The address being set as the role holder
pub fn mock_set_role_emit_prev_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    address: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_role_emit_prev",
            args: (address.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for the `enforce_role_auth` function.
///
/// # Arguments
///
/// * `env` - The Soroban test environment
/// * `contract_id` - The contract address
/// * `role_holder` - The role holder address that should authorize the call
pub fn mock_enforce_role_auth(env: &Env, contract_id: &Address, role_holder: &Address) {
    env.mock_auths(&[MockAuth {
        address: role_holder,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "enforce_role_auth",
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for the `enforce_role_auth_for` function with a custom role.
///
/// # Arguments
///
/// * `env` - The Soroban test environment
/// * `contract_id` - The contract address
/// * `role_holder` - The role holder address that should authorize the call
/// * `role` - The role name as a Soroban String
pub fn mock_enforce_role_auth_for(
    env: &Env,
    contract_id: &Address,
    role_holder: &Address,
    role: &soroban_sdk::String,
) {
    env.mock_auths(&[MockAuth {
        address: role_holder,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "enforce_role_auth_for",
            args: (role.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
