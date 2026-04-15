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

use super::Pausable;
use crate::test_utils::pausable::{mock_pause_auth, mock_unpause_auth, mock_update_pauser_auth};
use crate::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};
use stellar_access::ownable::set_owner;

#[contract]
struct TestContract;

#[contractimpl]
impl Pausable for TestContract {
    fn get_pauser(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::PAUSER)
    }

    fn update_pauser(e: &Env, new_pauser: Address) {
        simple_role::set_role_and_emit(e, super::PAUSER, &new_pauser, super::emit_pauser_changed)
    }

    fn paused(e: &Env) -> bool {
        super::paused(e)
    }

    fn pause(e: &Env) {
        super::pause(e)
    }

    fn unpause(e: &Env) {
        super::unpause(e)
    }
}

#[contractimpl]
impl TestContract {
    pub fn update_pauser_unchecked(env: Env, pauser: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::PAUSER,
            &pauser,
            super::emit_pauser_changed,
        );
    }

    pub fn set_owner_unchecked(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }
}

// =============================================================================
// Initial State Tests
// =============================================================================

#[test]
fn test_initial_state_not_paused() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert!(!client.paused(), "Contract should not be paused initially");
}

#[test]
fn test_get_pauser_returns_none_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_pauser(),
        None,
        "get_pauser should return None when no pauser is set"
    );
}

// =============================================================================
// Pause / Unpause Tests
// =============================================================================

#[test]
fn test_pause() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    assert!(!client.paused(), "Contract should not be paused initially");

    mock_pause_auth(&env, &contract_id, &pauser);
    client.pause();

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_paused();

    assert!(client.paused(), "Contract should be paused after pause()");
}

#[test]
fn test_unpause() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    mock_pause_auth(&env, &contract_id, &pauser);
    client.pause();
    assert!(client.paused());

    mock_unpause_auth(&env, &contract_id, &pauser);
    client.unpause();

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_unpaused();

    assert!(
        !client.paused(),
        "Contract should be unpaused after unpause()"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_pause_when_already_paused_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);
    client.pause();
    client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1001)")]
fn test_unpause_when_already_unpaused_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);
    client.unpause();
}

// =============================================================================
// update_pauser Tests
// =============================================================================

#[test]
fn test_update_pauser_unchecked_sets_pauser() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_pauser(),
        None,
        "Pauser should be None before set"
    );

    client.update_pauser_unchecked(&pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&pauser);

    assert_eq!(
        client.get_pauser(),
        Some(pauser),
        "Pauser should be set after update_pauser_unchecked"
    );
}

#[test]
fn test_update_pauser_unchecked_overwrites_existing() {
    let env = Env::default();

    let pauser1 = Address::generate(&env);
    let pauser2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser1);

    assert_eq!(
        client.get_pauser(),
        Some(pauser1.clone()),
        "Pauser should be pauser1 initially"
    );

    client.update_pauser_unchecked(&pauser2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&pauser2);

    assert_eq!(
        client.get_pauser(),
        Some(pauser2),
        "Pauser should be overwritten to pauser2"
    );
}

#[test]
fn test_update_pauser_unchecked_to_same_address() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    assert_eq!(client.get_pauser(), Some(pauser.clone()));

    // Update to the same pauser
    client.update_pauser_unchecked(&pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&pauser);

    assert_eq!(
        client.get_pauser(),
        Some(pauser),
        "Pauser should remain the same after updating to same address"
    );
}

#[test]
fn test_update_pauser_to_same_address() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&pauser);

    assert_eq!(client.get_pauser(), Some(pauser.clone()));

    // Update to the same pauser via checked path
    mock_update_pauser_auth(&env, &contract_id, &owner, &pauser);
    client.update_pauser(&pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&pauser);

    assert_eq!(
        client.get_pauser(),
        Some(pauser),
        "Pauser should remain the same after updating to same address"
    );
}

#[test]
fn test_new_pauser_can_operate_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_pauser = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&old_pauser);

    // Change pauser
    mock_update_pauser_auth(&env, &contract_id, &owner, &new_pauser);
    client.update_pauser(&new_pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&new_pauser);

    // New pauser can pause
    mock_pause_auth(&env, &contract_id, &new_pauser);
    client.pause();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_paused();

    assert!(client.paused(), "New pauser should be able to pause");

    // New pauser can unpause
    mock_unpause_auth(&env, &contract_id, &new_pauser);
    client.unpause();

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_unpaused();

    assert!(!client.paused(), "New pauser should be able to unpause");
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_pauser_cannot_pause_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_pauser = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&old_pauser);

    // Change pauser
    mock_update_pauser_auth(&env, &contract_id, &owner, &new_pauser);
    client.update_pauser(&new_pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&new_pauser);

    // Old pauser tries to pause - should fail
    mock_pause_auth(&env, &contract_id, &old_pauser);
    client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_pauser_cannot_unpause_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_pauser = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&old_pauser);

    // Pause with old pauser
    mock_pause_auth(&env, &contract_id, &old_pauser);
    client.pause();

    // Change pauser
    mock_update_pauser_auth(&env, &contract_id, &owner, &new_pauser);
    client.update_pauser(&new_pauser);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_pauser_changed(&new_pauser);

    // Old pauser tries to unpause - should fail
    mock_unpause_auth(&env, &contract_id, &old_pauser);
    client.unpause();
}

#[test]
fn test_paused_state_persists_after_pauser_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_pauser = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&old_pauser);

    // Pause the contract
    mock_pause_auth(&env, &contract_id, &old_pauser);
    client.pause();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_paused();

    assert!(client.paused());

    // Change pauser while paused
    mock_update_pauser_auth(&env, &contract_id, &owner, &new_pauser);
    client.update_pauser(&new_pauser);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_pauser_changed(&new_pauser);

    // Paused state should persist after pauser change
    assert!(
        client.paused(),
        "Paused state should persist after pauser update"
    );
}

#[test]
fn test_unpaused_state_persists_after_pauser_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_pauser = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_pauser_unchecked(&old_pauser);

    assert!(!client.paused());

    // Change pauser while unpaused
    mock_update_pauser_auth(&env, &contract_id, &owner, &new_pauser);
    client.update_pauser(&new_pauser);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_pauser_changed(&new_pauser);

    // Unpaused state should persist after pauser change
    assert!(
        !client.paused(),
        "Unpaused state should persist after pauser update"
    );
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_pause_requires_pauser_auth() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    // Try to pause without mocking pauser auth - should panic
    client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unpause_requires_pauser_auth() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    // Pause the contract first (with proper auth)
    mock_pause_auth(&env, &contract_id, &pauser);
    client.pause();

    // Try to unpause without mocking pauser auth - should panic
    client.unpause();
}

#[test]
#[should_panic(expected = "#7000")]
fn test_pause_fails_when_pauser_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with RoleNotSet error (7000)
    client.pause();
}

#[test]
#[should_panic(expected = "#7000")]
fn test_unpause_fails_when_pauser_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with RoleNotSet error (7000)
    client.unpause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_pause_fails_when_non_pauser() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let non_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    // Mock auth for wrong address
    mock_pause_auth(&env, &contract_id, &non_pauser);
    client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unpause_fails_when_non_pauser() {
    let env = Env::default();

    let pauser = Address::generate(&env);
    let non_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_pauser_unchecked(&pauser);

    // Pause with correct pauser
    mock_pause_auth(&env, &contract_id, &pauser);
    client.pause();

    // Mock auth for wrong address to unpause
    mock_unpause_auth(&env, &contract_id, &non_pauser);
    client.unpause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_pauser_requires_owner_auth() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Try to update pauser without mocking owner auth - should panic
    client.update_pauser(&new_pauser);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_update_pauser_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with OwnerNotSet error (2100)
    client.update_pauser(&new_pauser);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_pauser_fails_when_non_owner() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let new_pauser = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Mock auth for non-owner
    mock_update_pauser_auth(&env, &contract_id, &non_owner, &new_pauser);
    client.update_pauser(&new_pauser);
}
