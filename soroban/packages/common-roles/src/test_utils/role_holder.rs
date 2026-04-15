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

//! Generic test utilities for role holder management patterns.
//!
//! Many roles in the system follow a common pattern:
//! - `get_<role>()` returns `Option<Address>`
//! - `set_<role>()` requires owner auth
//! - `set_<role>_unchecked()` sets without auth (for initialization)
//!
//! This module provides a macro to test these common patterns, reducing
//! duplication across role test files.
//!
//! # Applicable Roles
//!
//! This macro can be used for:
//! - `token_controller`
//! - `pauser`
//! - `rescuer`
//! - `admin`
//! - `allow_asset_controller`
//! - `attester_manager`

/// Role holder management test macro for already-initialized contracts.
///
/// Use this when the contract is already set up with owner and role holder.
/// This macro tests the following behaviors:
///
/// 1. **Non-owner cannot set role holder** - Non-owners are rejected (auth failure)
/// 2. **Non-role holder cannot perform action** - Random addresses cannot perform role actions (auth failure)
/// 3. **Current holder can perform action** - The current role holder can perform actions
/// 4. **Role change works** - Owner can transfer the role to a new holder
/// 5. **State persists after role change** - Pre-transfer state is preserved
/// 6. **Old holder cannot perform action after transfer** - Revoked holder is rejected (auth failure)
/// 7. **Old holder cannot perform post-transfer action after transfer** - Fully revoked (auth failure)
/// 8. **New holder can perform post-transfer action** - New holder has full permissions
///
/// # Arguments
///
/// * `env` - Reference to the Soroban test environment
/// * `client` - Reference to the contract client
/// * `contract_id` - Reference to the contract address
/// * `owner` - Reference to the owner address
/// * `role_holder` - Reference to the current role holder address
/// * `get_fn` - Identifier for the getter method (e.g., `get_blocklister`)
/// * `set_fn` - Identifier for the setter method (e.g., `set_blocklister`)
/// * `mock_set_auth` - Path to the mock auth helper function for setting the role
/// * `action` - Closure to perform pre-transfer action, returns data for verification
/// * `verify_action_persists` - Closure to verify pre-transfer state persists after role change
/// * `post_transfer_action` - Closure to perform post-transfer action (different operation)
///
/// # Example
///
/// ```ignore
/// common_roles::assert_initialized_role_holder_management!(
///     env: &ctx.env,
///     client: &client,
///     contract_id: &ctx.contract_id,
///     owner: &ctx.owner,
///     role_holder: &ctx.blocklister,
///     get_fn: get_blocklister,
///     set_fn: set_blocklister,
///     mock_set_auth: mock_set_blocklister_auth,
///     action: |env, client, contract_id, holder| {
///         let target = Address::generate(env);
///         mock_blocklist_auth(env, contract_id, holder, &target);
///         client.blocklist(&target);
///         target
///     },
///     verify_action_persists: |client, target| {
///         assert!(client.is_blocklisted(target));
///     },
///     post_transfer_action: |env, client, contract_id, holder| {
///         let target = Address::generate(env);
///         mock_blocklist_auth(env, contract_id, holder, &target);
///         client.blocklist(&target);
///         mock_unblocklist_auth(env, contract_id, holder, &target);
///         client.unblocklist(&target);
///     }
/// );
/// ```
#[macro_export]
macro_rules! assert_initialized_role_holder_management {
    // Variant without verify_action_persists (defaults to no-op)
    (
        env: $env:expr,
        client: $client:expr,
        contract_id: $contract_id:expr,
        owner: $owner:expr,
        role_holder: $role_holder:expr,
        get_fn: $get_fn:ident,
        set_fn: $set_fn:ident,
        mock_set_auth: $mock_set_auth:path,
        action: $action:expr,
        post_transfer_action: $post_transfer_action:expr
    ) => {{
        $crate::assert_initialized_role_holder_management!(
            env: $env,
            client: $client,
            contract_id: $contract_id,
            owner: $owner,
            role_holder: $role_holder,
            get_fn: $get_fn,
            set_fn: $set_fn,
            mock_set_auth: $mock_set_auth,
            action: $action,
            verify_action_persists: |_client, _data| {},
            post_transfer_action: $post_transfer_action
        )
    }};

    // Full variant with verify_action_persists
    (
        env: $env:expr,
        client: $client:expr,
        contract_id: $contract_id:expr,
        owner: $owner:expr,
        role_holder: $role_holder:expr,
        get_fn: $get_fn:ident,
        set_fn: $set_fn:ident,
        mock_set_auth: $mock_set_auth:path,
        action: $action:expr,
        verify_action_persists: $verify_action_persists:expr,
        post_transfer_action: $post_transfer_action:expr
    ) => {{
        extern crate std;
        use soroban_sdk::testutils::Address as _;

        let env = $env;
        let client = $client;
        let contract_id = $contract_id;
        let owner = $owner;
        let old_holder = $role_holder;

        const EXPECTED_AUTH_PANIC: &str = "Error(Auth, InvalidAction)";

        let panic_msg =
            |p: &std::boxed::Box<dyn std::any::Any + Send>| -> std::string::String {
                if let Some(s) = p.downcast_ref::<&str>() {
                    std::string::String::from(*s)
                } else if let Some(s) = p.downcast_ref::<std::string::String>() {
                    s.clone()
                } else {
                    std::string::String::from("<non-string panic payload>")
                }
            };

        // Verify role holder is set as expected
        assert_eq!(
            client.$get_fn(),
            Some(old_holder.clone()),
            "Role holder should be set initially"
        );

        // =================================================================
        // Test 1: Non-owner cannot set role holder
        // =================================================================
        let another_holder = soroban_sdk::Address::generate(env);
        let non_owner = soroban_sdk::Address::generate(env);
        $mock_set_auth(env, contract_id, &non_owner, &another_holder);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.$set_fn(&another_holder);
        }));
        assert!(
            result.is_err(),
            "Non-owner should not be able to set role holder"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_AUTH_PANIC),
                "Non-owner set panicked for unexpected reason. expected_substring={EXPECTED_AUTH_PANIC:?}, got={msg:?}"
            );
        }

        // Verify role holder unchanged
        assert_eq!(
            client.$get_fn(),
            Some(old_holder.clone()),
            "Role holder should remain unchanged after failed non-owner set"
        );

        // =================================================================
        // Test 2: Non-role holder cannot perform action
        // =================================================================
        let non_holder = soroban_sdk::Address::generate(env);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $action(env, client, contract_id, &non_holder);
        }));
        assert!(
            result.is_err(),
            "Non-role holder should not be able to perform action"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_AUTH_PANIC),
                "Non-holder action panicked for unexpected reason. expected_substring={EXPECTED_AUTH_PANIC:?}, got={msg:?}"
            );
        }

        // =================================================================
        // Test 3: Current holder can perform pre-transfer action
        // =================================================================
        let action_data = $action(env, client, contract_id, old_holder);

        // =================================================================
        // Test 4: Change role holder
        // =================================================================
        let new_holder = soroban_sdk::Address::generate(env);
        $mock_set_auth(env, contract_id, owner, &new_holder);
        client.$set_fn(&new_holder);
        assert_eq!(client.$get_fn(), Some(new_holder.clone()));

        // =================================================================
        // Test 5: Verify pre-transfer state persists after role change
        // =================================================================
        $verify_action_persists(client, &action_data);

        // =================================================================
        // Test 6: Old holder cannot perform action after role change
        // =================================================================
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $action(env, client, contract_id, old_holder);
        }));
        assert!(
            result.is_err(),
            "Old role holder should not be able to perform action after being replaced"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_AUTH_PANIC),
                "Old-holder action panicked for unexpected reason. expected_substring={EXPECTED_AUTH_PANIC:?}, got={msg:?}"
            );
        }

        // =================================================================
        // Test 7: Old holder cannot perform post-transfer action
        // =================================================================
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $post_transfer_action(env, client, contract_id, old_holder);
        }));
        assert!(
            result.is_err(),
            "Old role holder should not be able to perform post-transfer action"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_AUTH_PANIC),
                "Old-holder post-transfer action panicked for unexpected reason. expected_substring={EXPECTED_AUTH_PANIC:?}, got={msg:?}"
            );
        }

        // =================================================================
        // Test 8: New holder can perform post-transfer action
        // =================================================================
        $post_transfer_action(env, client, contract_id, &new_holder);
    }};
}

/// Comprehensive role holder management test macro for uninitialized contracts.
///
/// This macro tests the full lifecycle including uninitialized state tests,
/// then delegates to [`assert_initialized_role_holder_management!`] for the
/// initialized state tests.
///
/// This macro tests the following behaviors:
///
/// 1. **Get returns None initially** - Before any role holder is set, the getter returns `None`
/// 2. **RoleNotSet when unset** - Role actions panic with `RoleNotSet` (`#7000`) when the role is not set
/// 3. **OwnerNotSet when unset** - Setting the role holder panics with `OwnerNotSet` (`#2100`) when the owner is not set
/// 4. **Owner can set role holder** - The owner can set the role holder with proper auth
/// 5. **Remaining tests** - See [`assert_initialized_role_holder_management!`] for tests 5-12
///
/// # Arguments
///
/// * `env` - Reference to the Soroban test environment
/// * `client` - Reference to the contract client
/// * `contract_id` - Reference to the contract address
/// * `owner` - Reference to the owner address
/// * `set_owner_unchecked_fn` - Identifier for the unchecked owner setter method on the client
/// * `get_fn` - Identifier for the getter method (e.g., `get_token_controller`)
/// * `set_fn` - Identifier for the setter method (e.g., `set_token_controller`)
/// * `mock_set_auth` - Path to the mock auth helper function
/// * `assert_event` - Identifier for the event assertion method on EventAssertion
/// * `action` - Closure to perform pre-transfer action, returns data for verification
/// * `verify_action_persists` - Closure to verify pre-transfer state persists after role change
/// * `post_transfer_action` - Closure to perform post-transfer action (different operation)
#[macro_export]
macro_rules! assert_role_holder_management {
    // Variant without verify_action_persists (defaults to no-op)
    (
        env: $env:expr,
        client: $client:expr,
        contract_id: $contract_id:expr,
        owner: $owner:expr,
        set_owner_unchecked_fn: $set_owner_unchecked_fn:ident,
        get_fn: $get_fn:ident,
        set_fn: $set_fn:ident,
        mock_set_auth: $mock_set_auth:path,
        assert_event: $assert_event:ident,
        action: $action:expr,
        post_transfer_action: $post_transfer_action:expr
    ) => {{
        $crate::assert_role_holder_management!(
            env: $env,
            client: $client,
            contract_id: $contract_id,
            owner: $owner,
            set_owner_unchecked_fn: $set_owner_unchecked_fn,
            get_fn: $get_fn,
            set_fn: $set_fn,
            mock_set_auth: $mock_set_auth,
            assert_event: $assert_event,
            action: $action,
            verify_action_persists: |_client, _data| {},
            post_transfer_action: $post_transfer_action
        )
    }};

    // Full variant with verify_action_persists
    (
        env: $env:expr,
        client: $client:expr,
        contract_id: $contract_id:expr,
        owner: $owner:expr,
        set_owner_unchecked_fn: $set_owner_unchecked_fn:ident,
        get_fn: $get_fn:ident,
        set_fn: $set_fn:ident,
        mock_set_auth: $mock_set_auth:path,
        assert_event: $assert_event:ident,
        action: $action:expr,
        verify_action_persists: $verify_action_persists:expr,
        post_transfer_action: $post_transfer_action:expr
    ) => {{
        extern crate std;
        use soroban_sdk::testutils::Address as _;

        let env = $env;
        let client = $client;
        let contract_id = $contract_id;
        let owner = $owner;

        const EXPECTED_ROLE_NOT_SET_PANIC: &str = "#7000";
        const EXPECTED_OWNER_NOT_SET_PANIC: &str = "#2100";

        let panic_msg =
            |p: &std::boxed::Box<dyn std::any::Any + Send>| -> std::string::String {
                if let Some(s) = p.downcast_ref::<&str>() {
                    std::string::String::from(*s)
                } else if let Some(s) = p.downcast_ref::<std::string::String>() {
                    s.clone()
                } else {
                    std::string::String::from("<non-string panic payload>")
                }
            };

        // =================================================================
        // Test 1: Get returns None initially
        // =================================================================
        assert_eq!(
            client.$get_fn(),
            None,
            "Role holder should not be set initially"
        );

        // =================================================================
        // Test 2: Action fails with RoleNotSet when role holder is not set
        // =================================================================
        let unset_holder = soroban_sdk::Address::generate(env);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $action(env, client, contract_id, &unset_holder);
        }));
        assert!(
            result.is_err(),
            "Role action should fail when role holder is not set"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_ROLE_NOT_SET_PANIC),
                "Unset role action panicked for unexpected reason. expected_substring={EXPECTED_ROLE_NOT_SET_PANIC:?}, got={msg:?}"
            );
        }

        // =================================================================
        // Test 3: Setting role holder fails with OwnerNotSet when owner is not set
        // =================================================================
        let holder_when_owner_unset = soroban_sdk::Address::generate(env);
        let caller_when_owner_unset = soroban_sdk::Address::generate(env);
        $mock_set_auth(env, contract_id, &caller_when_owner_unset, &holder_when_owner_unset);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.$set_fn(&holder_when_owner_unset);
        }));
        assert!(
            result.is_err(),
            "Setting role holder should fail when owner is not set"
        );
        if let Err(p) = &result {
            let msg = panic_msg(p);
            assert!(
                msg.contains(EXPECTED_OWNER_NOT_SET_PANIC),
                "Owner-not-set set panicked for unexpected reason. expected_substring={EXPECTED_OWNER_NOT_SET_PANIC:?}, got={msg:?}"
            );
        }

        // =================================================================
        // Test 4: Set up owner and initial role holder
        // =================================================================
        client.$set_owner_unchecked_fn(owner);

        let initial_holder = soroban_sdk::Address::generate(env);
        $mock_set_auth(env, contract_id, owner, &initial_holder);
        client.$set_fn(&initial_holder);
        assert_eq!(client.$get_fn(), Some(initial_holder.clone()));

        // =================================================================
        // Delegate to initialized role holder management tests
        // =================================================================
        $crate::assert_initialized_role_holder_management!(
            env: env,
            client: client,
            contract_id: contract_id,
            owner: owner,
            role_holder: &initial_holder,
            get_fn: $get_fn,
            set_fn: $set_fn,
            mock_set_auth: $mock_set_auth,
            action: $action,
            verify_action_persists: $verify_action_persists,
            post_transfer_action: $post_transfer_action
        );
    }};
}
