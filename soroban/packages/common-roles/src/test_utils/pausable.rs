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

//! Test utilities for verifying Pausable contract implementations
//!
//! This module provides assertion macros to verify that a contract correctly
//! implements the Pausable role, including:
//! - Pausing and unpausing the contract
//! - Authorization checks (only pauser can pause/unpause)
//! - Ownership checks (only owner can update pauser)
//!
//! Inspired by: https://github.com/circlefin/hyperevm-circle-contracts/blob/2e2c774904c5dd4c05bec6502679bda4821a18d0/test/TestUtils.sol#L52

/// Asserts that a contract correctly implements the Pausable interface.
///
/// # Arguments
/// * `$env` - The test environment
/// * `$client` - Client for the contract being tested (must have Pausable methods)
/// * `$contract_id` - Address of the contract being tested
/// * `$pauser` - Address of the current pauser
/// * `$owner` - Address of the contract owner
#[macro_export]
macro_rules! assert_contract_is_pausable {
    ($env:expr, $client:expr, $contract_id:expr, $pauser:expr, $owner:expr) => {{
        extern crate std;
        use soroban_sdk::{testutils::Address as _, Address};

        let env = $env;
        let contract_client = $client;
        let contract_id = $contract_id;
        let pauser = $pauser;
        let owner = $owner;

        // Verify initial state: contract should not be paused
        assert!(
            !contract_client.paused(),
            "Contract should not be paused initially"
        );

        // Verify pauser is set correctly
        let stored_pauser = contract_client.get_pauser();
        assert_eq!(
            stored_pauser,
            Some(pauser.clone()),
            "Pauser should be set and retrievable"
        );

        // Pauser can pause the contract
        $crate::test_utils::pausable::mock_pause_auth(env, &contract_id, &pauser);
        contract_client.pause();
        assert!(
            contract_client.paused(),
            "Contract should be paused after pause()"
        );

        // Cannot pause when already paused (should panic)
        $crate::test_utils::pausable::mock_pause_auth(env, &contract_id, &pauser);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.pause();
        }));
        assert!(
            result.is_err(),
            "Pausing an already paused contract should fail"
        );

        // Pauser can unpause the contract
        $crate::test_utils::pausable::mock_unpause_auth(env, &contract_id, &pauser);
        contract_client.unpause();
        assert!(
            !contract_client.paused(),
            "Contract should be unpaused after unpause()"
        );

        // Cannot unpause when already unpaused (should panic)
        $crate::test_utils::pausable::mock_unpause_auth(env, &contract_id, &pauser);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.unpause();
        }));
        assert!(
            result.is_err(),
            "Unpausing an already unpaused contract should fail"
        );

        // Non-pauser cannot pause (explicitly mock wrong pauser)
        let wrong_pauser = Address::generate(env);
        $crate::test_utils::pausable::mock_pause_auth(env, &contract_id, &wrong_pauser);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.pause();
        }));
        assert!(result.is_err(), "Non-pauser should not be able to pause");

        // Pause for unpause auth test
        $crate::test_utils::pausable::mock_pause_auth(env, &contract_id, &pauser);
        contract_client.pause();

        // Non-pauser cannot unpause (explicitly mock wrong pauser)
        let wrong_pauser = Address::generate(env);
        $crate::test_utils::pausable::mock_unpause_auth(env, &contract_id, &wrong_pauser);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.unpause();
        }));
        assert!(result.is_err(), "Non-pauser should not be able to unpause");

        // Reset state for owner tests
        $crate::test_utils::pausable::mock_unpause_auth(env, &contract_id, &pauser);
        contract_client.unpause();

        // Non-owner cannot update pauser (explicitly mock wrong owner)
        let non_owner = Address::generate(env);
        let new_pauser = Address::generate(env);
        $crate::test_utils::pausable::mock_update_pauser_auth(
            env,
            &contract_id,
            &non_owner,
            &new_pauser,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.update_pauser(&new_pauser);
        }));
        assert!(
            result.is_err(),
            "Non-owner should not be able to update pauser"
        );

        // Owner can update pauser
        $crate::test_utils::pausable::mock_update_pauser_auth(
            env,
            &contract_id,
            &owner,
            &new_pauser,
        );
        contract_client.update_pauser(&new_pauser);

        let stored_pauser = contract_client.get_pauser();
        assert_eq!(
            stored_pauser,
            Some(new_pauser.clone()),
            "Pauser should be updated to new address"
        );

        // New pauser can pause
        $crate::test_utils::pausable::mock_pause_auth(env, &contract_id, &new_pauser);
        contract_client.pause();
        assert!(
            contract_client.paused(),
            "New pauser should be able to pause"
        );

        // New pauser can unpause
        $crate::test_utils::pausable::mock_unpause_auth(env, &contract_id, &new_pauser);
        contract_client.unpause();
        assert!(
            !contract_client.paused(),
            "New pauser should be able to unpause"
        );
    }};
}

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

/// Sets mock auth so that the provided pauser can call `pause` on `contract_id`.
pub fn mock_pause_auth(env: &Env, contract_id: &Address, pauser: &Address) {
    env.mock_auths(&[MockAuth {
        address: pauser,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "pause",
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the provided pauser can call `unpause` on `contract_id`.
pub fn mock_unpause_auth(env: &Env, contract_id: &Address, pauser: &Address) {
    env.mock_auths(&[MockAuth {
        address: pauser,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "unpause",
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the owner can call `update_pauser` on `contract_id`.
pub fn mock_update_pauser_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    pauser: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_pauser",
            args: (pauser,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
