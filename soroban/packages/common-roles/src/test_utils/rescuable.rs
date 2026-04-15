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

//! Test utilities for verifying Rescuable contract implementations
//!
//! This module provides assertion macros to verify that a contract correctly
//! implements the Rescuable role, including:
//! - Setting and updating the rescuer
//! - Rescuing SEP-41 tokens sent to the contract
//! - Authorization checks (only rescuer can rescue)
//! - Ownership checks (only owner can update rescuer)
//!
//! Inspired by: https://github.com/circlefin/hyperevm-circle-contracts/blob/2e2c774904c5dd4c05bec6502679bda4821a18d0/test/TestUtils.sol#L109

/// Asserts that a contract correctly implements the Rescuable interface.
///
/// # Arguments
/// * `$env` - The test environment
/// * `$client` - Client for the contract being tested (must have Rescuable methods)
/// * `$contract_id` - Address of the contract being tested
/// * `$owner` - Address of the contract owner
#[macro_export]
macro_rules! assert_contract_is_rescuable {
    ($env:expr, $client:expr, $contract_id:expr, $owner:expr) => {{
        extern crate std;
        use soroban_sdk::{
            testutils::{Address as _, Events},
            token, Address,
        };

        let env = $env;
        let contract_client = $client;
        let contract_id = $contract_id;
        let owner = $owner;

        let rescuer = Address::generate(env);
        let recipient = Address::generate(env);
        let rescue_amount = 1000i128;

        // Create and mint test token
        let token_admin = Address::generate(env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        let token_client = token::StellarAssetClient::new(env, &token_address);
        let token_reader = token::TokenClient::new(env, &token_address);

        // Set rescuer and verify it can be retrieved
        $crate::test_utils::rescuable::mock_update_rescuer_auth(
            env,
            &contract_id,
            &owner,
            &rescuer,
        );
        contract_client.update_rescuer(&rescuer);
        let stored_rescuer = contract_client.get_rescuer();
        assert_eq!(
            Some(rescuer.clone()),
            stored_rescuer,
            "Rescuer should be set and retrievable"
        );

        // Successfully rescues tokens
        token_client
            .mock_all_auths()
            .mint(contract_id, &rescue_amount);

        assert_eq!(
            token_reader.balance(contract_id),
            rescue_amount,
            "Contract should have tokens to rescue"
        );
        assert_eq!(
            token_reader.balance(&recipient),
            0,
            "Recipient should start with zero balance"
        );

        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &rescuer,
            &token_address,
            &recipient,
            rescue_amount,
        );
        contract_client.rescue_sep41(&token_address, &recipient, &rescue_amount);

        assert_eq!(
            token_reader.balance(&recipient),
            rescue_amount,
            "Recipient should receive the rescued tokens"
        );
        assert_eq!(
            token_reader.balance(contract_id),
            0,
            "Contract should have zero balance after rescue"
        );

        // Partial rescue (rescue less than full balance)
        let partial_amount = 500i128;
        token_client
            .mock_all_auths()
            .mint(contract_id, &rescue_amount);

        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &rescuer,
            &token_address,
            &recipient,
            partial_amount,
        );
        contract_client.rescue_sep41(&token_address, &recipient, &partial_amount);

        assert_eq!(
            token_reader.balance(&recipient),
            rescue_amount + partial_amount,
            "Recipient should receive partial rescue"
        );
        assert_eq!(
            token_reader.balance(contract_id),
            rescue_amount - partial_amount,
            "Contract should retain remaining tokens after partial rescue"
        );

        // Cannot rescue more than available (should panic)
        let current_balance = token_reader.balance(contract_id);
        let excessive_amount = current_balance + 1;

        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &rescuer,
            &token_address,
            &recipient,
            excessive_amount,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.rescue_sep41(&token_address, &recipient, &excessive_amount);
        }));

        assert!(
            result.is_err(),
            "Rescuing more than available balance should fail"
        );

        // Cannot rescue negative amounts (should panic)
        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &rescuer,
            &token_address,
            &recipient,
            -100,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.rescue_sep41(&token_address, &recipient, &-100);
        }));

        assert!(result.is_err(), "Rescuing negative amounts should fail");

        // Non-rescuer cannot perform rescue (should panic - explicitly mock wrong signer)
        token_client.mock_all_auths().mint(contract_id, &1000);

        let wrong_rescuer = Address::generate(env);
        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &wrong_rescuer,
            &token_address,
            &recipient,
            500,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.rescue_sep41(&token_address, &recipient, &500);
        }));

        assert!(
            result.is_err(),
            "Non-rescuer should not be able to rescue tokens"
        );

        // Non-owner cannot update rescuer (should panic - explicitly mock wrong signer)
        let non_owner = Address::generate(env);
        let new_rescuer = Address::generate(env);
        $crate::test_utils::rescuable::mock_update_rescuer_auth(
            env,
            &contract_id,
            &non_owner,
            &new_rescuer,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.update_rescuer(&new_rescuer);
        }));
        assert!(
            result.is_err(),
            "Non-owner should not be able to update rescuer"
        );

        // Owner can update rescuer
        let new_rescuer = Address::generate(env);
        $crate::test_utils::rescuable::mock_update_rescuer_auth(
            env,
            &contract_id,
            &owner,
            &new_rescuer,
        );
        contract_client.update_rescuer(&new_rescuer);

        let stored_rescuer = contract_client.get_rescuer();
        assert_eq!(
            Some(new_rescuer.clone()),
            stored_rescuer,
            "Rescuer should be updated to new address"
        );

        // Verify new rescuer can perform rescue
        token_client.mock_all_auths().mint(contract_id, &500);
        $crate::test_utils::rescuable::mock_rescue_sep41_auth(
            env,
            &contract_id,
            &new_rescuer,
            &token_address,
            &recipient,
            500,
        );
        contract_client.rescue_sep41(&token_address, &recipient, &500);
    }};
}

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

/// Sets mock auth so that the owner can call `update_rescuer` on `contract`.
pub fn mock_update_rescuer_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    rescuer: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_rescuer",
            args: (rescuer,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the rescuer can call `rescue_sep41` on `contract`.
pub fn mock_rescue_sep41_auth(
    env: &Env,
    contract_id: &Address,
    rescuer: &Address,
    token: &Address,
    to: &Address,
    amount: i128,
) {
    env.mock_auths(&[MockAuth {
        address: rescuer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "rescue_sep41",
            args: (token, to, amount).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
