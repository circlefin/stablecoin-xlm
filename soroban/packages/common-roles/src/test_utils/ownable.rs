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

//! Test utilities for verifying Ownable contract implementations.
//!
//! Inspired by the Ownable2Step coverage in the EVM test suite:
//! https://github.com/circlefin/hyperevm-circle-contracts/blob/2e2c774904c5dd4c05bec6502679bda4821a18d0/test/TestUtils.sol

#[macro_export]
macro_rules! assert_contract_is_ownable {
    ($env:expr, $client:expr, $contract_id:expr, $owner:expr) => {{
        extern crate std;
        use soroban_sdk::{testutils::Address as _, Address};

        let env = $env;
        let contract_client = $client;
        let contract_id = $contract_id;
        let owner = $owner;

        // Initial owner should be set and match expectation.
        let initial_owner = contract_client
            .get_owner()
            .expect("Contract should have an owner set");
        assert_eq!(
            initial_owner,
            owner.clone(),
            "Initial owner should match expected owner"
        );

        let expires_in_ledgers = 10;
        let next_expires_in_ledgers = 15;

        let non_owner = Address::generate(env);
        let pending_owner = Address::generate(env);
        let second_pending_owner = Address::generate(env);

        // Pending owner is None
        assert!(
            contract_client.get_pending_owner().is_none(),
            "Unset pending owner should be None"
        );

        // Non-owner cannot initiate transfer_ownership.
        $crate::test_utils::ownable::mock_transfer_ownership_auth(
            env,
            contract_id,
            &non_owner,
            &pending_owner,
            expires_in_ledgers,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.transfer_ownership(&non_owner, &expires_in_ledgers);
        }));
        assert!(
            result.is_err(),
            "transfer_ownership by non-owner should fail"
        );
        assert_eq!(
            contract_client.get_owner(),
            Some(owner.clone()),
            "Owner should remain unchanged after failed transfer"
        );

        // Owner initiates transfer to pending_owner.
        $crate::test_utils::ownable::mock_transfer_ownership_auth(
            env,
            contract_id,
            &owner,
            &pending_owner,
            expires_in_ledgers,
        );
        contract_client.transfer_ownership(&pending_owner, &expires_in_ledgers);

        // Pending owner was set
        assert!(
            contract_client.get_pending_owner().is_some(),
            "Pending owner should be Some after transfer_ownership"
        );

        // Only pending owner can accept; others should fail.
        $crate::test_utils::ownable::mock_accept_ownership_auth(env, contract_id, &non_owner);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.accept_ownership();
        }));
        assert!(
            result.is_err(),
            "accept_ownership by non-pending owner should fail"
        );
        assert_eq!(
            contract_client.get_owner(),
            Some(owner.clone()),
            "Owner should remain unchanged when acceptance fails"
        );

        // Pending owner accepts.
        $crate::test_utils::ownable::mock_accept_ownership_auth(env, contract_id, &pending_owner);
        contract_client.accept_ownership();
        let mut events = event_assertion::EventAssertion::new(env, contract_id.clone());
        events.assert_ownership_transfer_completed(&pending_owner);
        assert_eq!(
            contract_client.get_owner(),
            Some(pending_owner.clone()),
            "Owner should update after pending owner accepts"
        );
        assert!(
            contract_client.get_pending_owner().is_none(),
            "Pending owner should be removed after ownership transfer completed"
        );

        // New owner starts another transfer, then a different pending owner finally accepts.
        $crate::test_utils::ownable::mock_transfer_ownership_auth(
            env,
            contract_id,
            &pending_owner,
            &second_pending_owner,
            next_expires_in_ledgers,
        );
        contract_client.transfer_ownership(&second_pending_owner, &next_expires_in_ledgers);

        // Previous (now non-pending) owner cannot accept (explicitly mock wrong signer).
        $crate::test_utils::ownable::mock_accept_ownership_auth(env, contract_id, &pending_owner);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.accept_ownership();
        }));
        assert!(
            result.is_err(),
            "Superseded pending owner should not be able to accept"
        );

        // Latest pending owner accepts.
        $crate::test_utils::ownable::mock_accept_ownership_auth(
            env,
            contract_id,
            &second_pending_owner,
        );
        contract_client.accept_ownership();
        events.assert_ownership_transfer_completed(&second_pending_owner);
        assert_eq!(
            contract_client.get_owner(),
            Some(second_pending_owner),
            "Owner should update to latest pending owner after acceptance"
        );
    }};
}

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

/// Sets mock auth so that the owner can call `transfer_ownership` on `contract`.
pub fn mock_transfer_ownership_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    new_owner: &Address,
    expires_in_ledgers: u32,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "transfer_ownership",
            args: (new_owner, expires_in_ledgers).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the pending owner can call `accept_ownership` on `contract`.
pub fn mock_accept_ownership_auth(env: &Env, contract_id: &Address, pending_owner: &Address) {
    env.mock_auths(&[MockAuth {
        address: pending_owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "accept_ownership",
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}
