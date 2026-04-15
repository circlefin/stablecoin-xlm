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

//! Test utilities for verifying manageable contract implementations
//!
//! This module provides assertion macros to verify that a contract correctly
//! implements the manageable role with two-step admin transfer, including:
//! - Getting and transferring admin via two-step process
//! - Authorization checks (only admin can upgrade, only admin can initiate transfer)
//! - Performing actual contract upgrades with real WASM
//! - Verifying state persistence after upgrade

/// Asserts that a contract correctly implements the manageable interface with two-step admin transfer.
///
/// Tests authorization checks, role management, actual contract upgrade, and state persistence.
///
/// # Arguments
/// * `$env` - The test environment
/// * `$client` - Client for the contract being tested (must have manageable methods)
/// * `$contract_id` - Address of the contract being tested
/// * `$owner` - Address of the contract owner
/// * `$admin` - Address of the admin
/// * `$v2_wasm` - WASM bytes for testing actual upgrade (e.g., include_bytes!("path/to/v2.wasm"))
#[macro_export]
macro_rules! assert_contract_is_manageable {
    ($env:expr, $client:expr, $contract_id:expr, $owner:expr, $admin:expr, $v2_wasm:expr) => {{
        extern crate std;
        use soroban_sdk::{testutils::Address as _, Address, BytesN, Symbol, Vec as SdkVec};

        let env = $env;
        let contract_client = $client;
        let contract_id = $contract_id;
        let _owner = $owner;
        let admin = $admin;

        // Verify initial state before running tests
        let initial_owner = contract_client.get_owner();
        let initial_admin = contract_client.get_admin();

        // Verify admin can be retrieved
        assert_eq!(
            initial_admin.as_ref(),
            Some(admin),
            "admin should be set and retrievable"
        );

        // Pending admin is None
        assert!(
            contract_client.get_pending_admin().is_none(),
            "Unset pending admin should be None"
        );

        let expires_in_ledgers = 10;
        let next_expires_in_ledgers = 15;

        let non_admin = Address::generate(env);
        let pending_admin = Address::generate(env);
        let second_pending_admin = Address::generate(env);

        // Non-admin cannot initiate transfer_admin.
        $crate::test_utils::manageable::mock_transfer_admin_auth(
            env,
            contract_id,
            &non_admin,
            &pending_admin,
            expires_in_ledgers,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.transfer_admin(&non_admin, &expires_in_ledgers);
        }));
        assert!(result.is_err(), "transfer_admin by non-admin should fail");
        assert_eq!(
            contract_client.get_admin(),
            Some(admin.clone()),
            "Admin should remain unchanged after failed transfer"
        );

        // Admin initiates transfer to pending_admin.
        $crate::test_utils::manageable::mock_transfer_admin_auth(
            env,
            contract_id,
            &admin,
            &pending_admin,
            expires_in_ledgers,
        );
        contract_client.transfer_admin(&pending_admin, &expires_in_ledgers);

        // Pending admin was set
        assert!(
            contract_client.get_pending_admin().is_some(),
            "Pending admin should be Some after transfer_admin"
        );

        // Only pending admin can accept; others should fail.
        $crate::test_utils::manageable::mock_accept_admin_auth(env, contract_id, &non_admin);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.accept_admin();
        }));
        assert!(
            result.is_err(),
            "accept_admin by non-pending admin should fail"
        );
        assert_eq!(
            contract_client.get_admin(),
            Some(admin.clone()),
            "Admin should remain unchanged when acceptance fails"
        );

        // Pending admin accepts.
        $crate::test_utils::manageable::mock_accept_admin_auth(env, contract_id, &pending_admin);
        contract_client.accept_admin();
        let mut events = event_assertion::EventAssertion::new(env, contract_id.clone());
        events.assert_admin_changed(Some(&admin), &pending_admin);
        assert_eq!(
            contract_client.get_admin(),
            Some(pending_admin.clone()),
            "Admin should update after pending admin accepts"
        );
        assert!(
            contract_client.get_pending_admin().is_none(),
            "Pending admin should be removed after admin transfer completed"
        );

        // New owner starts another transfer, then a different pending owner finally accepts.
        $crate::test_utils::manageable::mock_transfer_admin_auth(
            env,
            contract_id,
            &pending_admin,
            &second_pending_admin,
            next_expires_in_ledgers,
        );
        contract_client.transfer_admin(&second_pending_admin, &next_expires_in_ledgers);

        // Previous (now non-pending) owner cannot accept (explicitly mock wrong signer).
        $crate::test_utils::manageable::mock_accept_admin_auth(env, contract_id, &pending_admin);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.accept_admin();
        }));
        assert!(
            result.is_err(),
            "Superseded pending admin should not be able to accept"
        );

        // Latest pending owner accepts.
        $crate::test_utils::manageable::mock_accept_admin_auth(
            env,
            contract_id,
            &second_pending_admin,
        );
        contract_client.accept_admin();
        events.assert_admin_changed(Some(&pending_admin), &second_pending_admin);
        assert_eq!(
            contract_client.get_admin(),
            Some(second_pending_admin.clone()),
            "Admin should update to latest pending admin after acceptance"
        );

        // Non-admin cannot perform upgrade (should panic - explicitly mock wrong signer)
        let wasm_hash = BytesN::from_array(env, &[1u8; 32]);
        let wrong_admin = Address::generate(env);
        $crate::test_utils::manageable::mock_upgrade_auth(
            env,
            &contract_id,
            &wrong_admin,
            &wasm_hash,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.upgrade(&wasm_hash, &wrong_admin);
        }));

        assert!(
            result.is_err(),
            "Non-admin should not be able to upgrade contract"
        );

        // Perform actual upgrade with provided WASM admin
        let current_admin = &second_pending_admin;
        let v2_wasm_hash = env.deployer().upload_contract_wasm($v2_wasm);
        $crate::test_utils::manageable::mock_upgrade_auth(
            env,
            &contract_id,
            current_admin,
            &v2_wasm_hash,
        );
        contract_client.upgrade(&v2_wasm_hash, current_admin);

        // Verify the upgrade happened by calling V2-specific function
        let result: u32 = env.invoke_contract(
            &contract_id,
            &Symbol::new(env, "v2_only_function"),
            SdkVec::new(env),
        );
        assert_eq!(result, 200, "V2 function should return 200");

        // Verify state persistence after upgrade
        assert_eq!(
            contract_client.get_owner(),
            initial_owner,
            "Owner should be preserved after upgrade"
        );
        assert_eq!(
            contract_client.get_admin(),
            Some(current_admin.clone()),
            "admin should be preserved after upgrade"
        );
    }};
}

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

/// Sets mock auth so that the admin can call `transfer_admin` on `contract`.
pub fn mock_transfer_admin_auth(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
    new_admin: &Address,
    expires_in_ledgers: u32,
) {
    env.mock_auths(&[MockAuth {
        address: admin,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "transfer_admin",
            args: (new_admin, expires_in_ledgers).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the pending admin can call `accept_admin` on `contract`.
pub fn mock_accept_admin_auth(env: &Env, contract_id: &Address, pending_admin: &Address) {
    env.mock_auths(&[MockAuth {
        address: pending_admin,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "accept_admin",
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the admin can call `upgrade` on `contract`.
pub fn mock_upgrade_auth(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
    new_wasm_hash: &BytesN<32>,
) {
    env.mock_auths(&[MockAuth {
        address: admin,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "upgrade",
            args: (new_wasm_hash, admin).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the admin can call `migrate` on `contract`.
pub fn mock_migrate_auth<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
    migration_data: T,
) {
    env.mock_auths(&[MockAuth {
        address: admin,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "migrate",
            args: (migration_data, admin).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
