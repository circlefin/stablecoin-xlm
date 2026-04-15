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
extern crate std;

use event_assertion::EventAssertion;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::RoleError;
use crate::test_utils::{
    mock_enforce_role_auth, mock_enforce_role_auth_for, mock_set_role_and_emit_auth,
    mock_set_role_auth, mock_set_role_emit_prev_auth, SimpleRoleEventAssertions, TestContract,
    TestContractClient,
};

// =============================================================================
// try_get_role Tests
// =============================================================================

#[test]
fn test_try_get_role_returns_none_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let result = client.test_try_get_role();
    assert_eq!(result, None);
}

#[test]
fn test_try_get_role_returns_address_when_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    client.set_role_unchecked(&address);

    let result = client.test_try_get_role();
    assert_eq!(result, Some(address));
}

// =============================================================================
// get_role Tests
// =============================================================================

#[test]
fn test_get_role_returns_address_when_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    client.set_role_unchecked(&address);

    let result = client.get_role();
    assert_eq!(result, address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7000)")]
fn test_get_role_panics_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.get_role();
}

// =============================================================================
// set_role Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_role_fails_without_owner_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let not_owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    // Auth with wrong address
    mock_set_role_auth(&env, &contract_id, &not_owner, &address);
    client.set_role(&address);
}

#[test]
fn test_set_role_can_update_existing() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_auth(&env, &contract_id, &owner, &address1);
    client.set_role(&address1);
    assert_eq!(client.get_role(), address1);

    mock_set_role_auth(&env, &contract_id, &owner, &address2);
    client.set_role(&address2);
    assert_eq!(client.get_role(), address2);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_set_role_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    // Owner not set - should fail with OwnerNotSet (#2100)
    client.set_role(&address);
}

// =============================================================================
// set_role_unchecked Tests
// =============================================================================

#[test]
fn test_set_role_unchecked_does_not_require_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    // No owner set, no auth - should still work
    client.set_role_unchecked(&address);

    assert_eq!(client.get_role(), address);
}

// =============================================================================
// set_role_and_emit Tests
// =============================================================================

#[test]
fn test_set_role_and_emit_sets_role() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_and_emit_auth(&env, &contract_id, &owner, &address);
    client.set_role_and_emit(&address);

    assert_eq!(client.get_role(), address);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_role_and_emit_fails_without_owner_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let not_owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_and_emit_auth(&env, &contract_id, &not_owner, &address);
    client.set_role_and_emit(&address);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_set_role_and_emit_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    // Owner not set - should fail with OwnerNotSet (#2100)
    client.set_role_and_emit(&address);
}

// =============================================================================
// set_role_and_emit_unchecked Tests
// =============================================================================

#[test]
fn test_set_role_and_emit_unchecked_does_not_require_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    // No owner set, no auth - should still work
    client.set_role_emit_unchecked(&address);

    // Verify role is set
    assert_eq!(client.get_role(), address);
}

// =============================================================================
// set_role_and_emit_with_previous Tests
// =============================================================================

#[test]
fn test_set_role_and_emit_with_previous_sets_role() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    client.set_owner(&owner);

    // First set
    mock_set_role_emit_prev_auth(&env, &contract_id, &owner, &address1);
    client.set_role_emit_prev(&address1);

    assert_eq!(client.get_role(), address1);

    // Second set - should update
    mock_set_role_emit_prev_auth(&env, &contract_id, &owner, &address2);
    client.set_role_emit_prev(&address2);

    // Verify role is updated
    assert_eq!(client.get_role(), address2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_role_and_emit_with_previous_fails_without_owner_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let not_owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_emit_prev_auth(&env, &contract_id, &not_owner, &address);
    client.set_role_emit_prev(&address);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_set_role_and_emit_with_previous_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    // Owner not set - should fail with OwnerNotSet (#2100)
    client.set_role_emit_prev(&address);
}

// =============================================================================
// set_role_and_emit_with_previous_unchecked Tests
// =============================================================================

#[test]
fn test_set_role_emit_with_prev_unchecked_does_not_require_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    // First set - no auth required
    client.set_role_emit_prev_unchk(&address1);

    // Verify role is set
    assert_eq!(client.get_role(), address1);

    // Second set
    client.set_role_emit_prev_unchk(&address2);

    // Verify role is updated
    assert_eq!(client.get_role(), address2);
}

// =============================================================================
// enforce_role_auth Tests
// =============================================================================

#[test]
fn test_enforce_role_auth_succeeds_with_valid_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let role_holder = Address::generate(&env);

    client.set_role_unchecked(&role_holder);

    mock_enforce_role_auth(&env, &contract_id, &role_holder);
    let result = client.enforce_role_auth();

    assert_eq!(result, role_holder);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7000)")]
fn test_enforce_role_auth_fails_when_role_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let role_holder = Address::generate(&env);

    // Role not set - should fail with RoleNotSet
    mock_enforce_role_auth(&env, &contract_id, &role_holder);
    client.enforce_role_auth();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_enforce_role_auth_fails_with_wrong_address() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let role_holder = Address::generate(&env);
    let wrong_address = Address::generate(&env);

    client.set_role_unchecked(&role_holder);

    // Auth with wrong address
    mock_enforce_role_auth(&env, &contract_id, &wrong_address);
    client.enforce_role_auth();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_enforce_role_auth_fails_with_owner_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let role_holder = Address::generate(&env);

    client.set_owner(&owner);
    client.set_role_unchecked(&role_holder);

    // Owner cannot act as the role holder
    mock_enforce_role_auth(&env, &contract_id, &owner);
    client.enforce_role_auth();
}

// =============================================================================
// Multiple Roles Tests
// =============================================================================

#[test]
fn test_multiple_roles_are_independent() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    let role1 = soroban_sdk::String::from_str(&env, "role_one");
    let role2 = soroban_sdk::String::from_str(&env, "role_two");

    client.set_role_unchecked_for(&role1, &address1);
    client.set_role_unchecked_for(&role2, &address2);

    assert_eq!(client.get_role_for(&role1), address1);
    assert_eq!(client.get_role_for(&role2), address2);

    // Updating one role doesn't affect the other
    let address3 = Address::generate(&env);
    client.set_role_unchecked_for(&role1, &address3);

    assert_eq!(client.get_role_for(&role1), address3);
    assert_eq!(client.get_role_for(&role2), address2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_role1_holder_cannot_perform_role2_action() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let role1_holder = Address::generate(&env);
    let role2_holder = Address::generate(&env);

    let role1 = soroban_sdk::String::from_str(&env, "role_one");
    let role2 = soroban_sdk::String::from_str(&env, "role_two");

    // Assign different addresses to different roles
    client.set_role_unchecked_for(&role1, &role1_holder);
    client.set_role_unchecked_for(&role2, &role2_holder);

    // role1_holder tries to perform an action that requires role2 authorization
    mock_enforce_role_auth_for(&env, &contract_id, &role1_holder, &role2);
    client.enforce_role_auth_for(&role2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_role_holder_cannot_set_own_role() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let role_holder = Address::generate(&env);
    let new_address = Address::generate(&env);

    client.set_owner(&owner);
    client.set_role_unchecked(&role_holder);

    // Role holder (not owner) tries to update their own role
    mock_set_role_auth(&env, &contract_id, &role_holder, &new_address);
    client.set_role(&new_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_role_holder_cannot_set_role_and_emit() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let role_holder = Address::generate(&env);
    let new_address = Address::generate(&env);

    client.set_owner(&owner);
    client.set_role_unchecked(&role_holder);

    // Role holder (not owner) tries to update via set_role_and_emit
    mock_set_role_and_emit_auth(&env, &contract_id, &role_holder, &new_address);
    client.set_role_and_emit(&new_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_role_holder_cannot_set_role_and_emit_with_previous() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let role_holder = Address::generate(&env);
    let new_address = Address::generate(&env);

    client.set_owner(&owner);
    client.set_role_unchecked(&role_holder);

    // Role holder (not owner) tries to update via set_role_and_emit_with_previous
    mock_set_role_emit_prev_auth(&env, &contract_id, &role_holder, &new_address);
    client.set_role_emit_prev(&new_address);
}

// =============================================================================
// Event Emission Tests
// =============================================================================

#[test]
fn test_set_role_and_emit_emits_role_set_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_and_emit_auth(&env, &contract_id, &owner, &address);
    client.set_role_and_emit(&address);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_role_set(&address);
}

#[test]
fn test_set_role_emit_unchecked_emits_role_set_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address = Address::generate(&env);

    client.set_role_emit_unchecked(&address);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_role_set(&address);
}

#[test]
fn test_set_role_emit_prev_emits_role_updated_event_with_none_previous() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address = Address::generate(&env);

    client.set_owner(&owner);

    mock_set_role_emit_prev_auth(&env, &contract_id, &owner, &address);
    client.set_role_emit_prev(&address);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_role_updated(None, &address);
}

#[test]
fn test_set_role_emit_prev_emits_role_updated_event_with_previous() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    client.set_owner(&owner);

    // First set
    mock_set_role_emit_prev_auth(&env, &contract_id, &owner, &address1);
    client.set_role_emit_prev(&address1);

    // Second set - should include previous address in event
    mock_set_role_emit_prev_auth(&env, &contract_id, &owner, &address2);
    client.set_role_emit_prev(&address2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_role_updated(Some(&address1), &address2);
}

#[test]
fn test_set_role_emit_prev_unchk_emits_role_updated_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let address1 = Address::generate(&env);
    let address2 = Address::generate(&env);

    // First set - no previous
    client.set_role_emit_prev_unchk(&address1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_role_updated(None, &address1);

    // Second set - has previous
    client.set_role_emit_prev_unchk(&address2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_role_updated(Some(&address1), &address2);
}

// =============================================================================
// RoleError Tests
// =============================================================================

#[test]
fn test_role_error_code() {
    // Verify the error code is as documented
    assert_eq!(RoleError::RoleNotSet as u32, 7000);
}
