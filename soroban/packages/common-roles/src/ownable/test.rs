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

use crate::test_utils::ownable::{mock_accept_ownership_auth, mock_transfer_ownership_auth};
use crate::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::Ledger as _, Address, Env,
};

use super::Ownable;

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn set_owner_unchecked(env: Env, owner: Address) {
        super::set_owner_unchecked(&env, &owner);
    }
}

#[contractimpl]
impl super::Ownable for TestContract {
    fn get_owner(e: &Env) -> Option<Address> {
        super::get_owner(e)
    }

    fn get_pending_owner(e: &Env) -> Option<Address> {
        super::get_pending_owner(e)
    }

    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32) {
        super::transfer_ownership(e, &new_owner, expires_in_ledgers);
    }

    fn accept_ownership(e: &Env) {
        super::accept_ownership(e);
    }
}

// =============================================================================
// Initial State Tests
// =============================================================================

#[test]
fn test_get_owner_returns_none_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_owner(),
        None,
        "get_owner should return None when no owner is set"
    );
}

#[test]
fn test_get_pending_owner_returns_none_when_not_set() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    assert_eq!(
        client.get_pending_owner(),
        None,
        "get_pending_owner should return None when no transfer is pending"
    );
}

#[test]
#[should_panic(expected = "#2102")]
fn test_set_owner_unchecked_fails_if_already_set() {
    let env = Env::default();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());

    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner1);
    client.set_owner_unchecked(&owner2);
}

// =============================================================================
// Two-Step Ownership Transfer Tests
// =============================================================================

#[test]
fn test_transfer_ownership_two_step_flow() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pending_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());

    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Verify initial state
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should be set after set_owner_unchecked"
    );
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be None before transfer"
    );

    // Step 1: Current owner initiates transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &pending_owner, 10);
    client.transfer_ownership(&pending_owner, &10);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &pending_owner, 10);

    // Verify pending owner is set but owner hasn't changed yet
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should remain unchanged during pending transfer"
    );
    assert_eq!(
        client.get_pending_owner(),
        Some(pending_owner.clone()),
        "Pending owner should be set after transfer_ownership"
    );

    // Step 2: Pending owner accepts transfer
    mock_accept_ownership_auth(&env, &contract_id, &pending_owner);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&pending_owner);

    // Verify owner changed and pending is cleared
    assert_eq!(
        client.get_owner(),
        Some(pending_owner.clone()),
        "Owner should be updated after accept_ownership"
    );
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be cleared after accept_ownership"
    );
}

#[test]
fn test_transfer_ownership_can_transfer_to_self() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Verify initial state
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should be set after set_owner_unchecked"
    );
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be None before transfer"
    );

    mock_transfer_ownership_auth(&env, &contract_id, &owner, &owner, 100);
    client.transfer_ownership(&owner, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &owner, 100);

    assert_eq!(
        client.get_pending_owner(),
        Some(owner.clone()),
        "Pending owner should be set to self after self-transfer"
    );
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should remain unchanged during pending self-transfer"
    );

    // Accept the self-transfer
    mock_accept_ownership_auth(&env, &contract_id, &owner);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&owner);

    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should remain the same after self-transfer acceptance"
    );
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be cleared after self-transfer acceptance"
    );
}

#[test]
fn test_chained_ownership_transfers() {
    let env = Env::default();

    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let owner_c = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner_a);

    // Transfer A -> B
    mock_transfer_ownership_auth(&env, &contract_id, &owner_a, &owner_b, 100);
    client.transfer_ownership(&owner_b, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner_a, &owner_b, 100);

    mock_accept_ownership_auth(&env, &contract_id, &owner_b);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&owner_b);

    assert_eq!(
        client.get_owner(),
        Some(owner_b.clone()),
        "Owner should be B after first transfer"
    );

    // Transfer B -> C (new owner can initiate transfers)
    mock_transfer_ownership_auth(&env, &contract_id, &owner_b, &owner_c, 100);
    client.transfer_ownership(&owner_c, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner_b, &owner_c, 100);

    mock_accept_ownership_auth(&env, &contract_id, &owner_c);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&owner_c);

    assert_eq!(
        client.get_owner(),
        Some(owner_c.clone()),
        "Owner should be C after second transfer"
    );
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be None after second transfer completed"
    );
}

#[test]
fn test_transfer_ownership_overwrites_pending_transfer() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let new_owner1 = Address::generate(&env);
    let new_owner2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // First transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &new_owner1, 100);
    client.transfer_ownership(&new_owner1, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &new_owner1, 100);

    assert_eq!(
        client.get_pending_owner(),
        Some(new_owner1.clone()),
        "Pending owner should be new_owner1 after first transfer"
    );

    // Second transfer should overwrite the first pending transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &new_owner2, 100);
    client.transfer_ownership(&new_owner2, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &new_owner2, 100);

    assert_eq!(
        client.get_pending_owner(),
        Some(new_owner2.clone()),
        "Pending owner should be overwritten to new_owner2"
    );

    // new_owner2 should be able to accept, not new_owner1
    mock_accept_ownership_auth(&env, &contract_id, &new_owner2);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&new_owner2);

    assert_eq!(
        client.get_owner(),
        Some(new_owner2),
        "Owner should be new_owner2 after acceptance"
    );
}

#[test]
fn test_overwritten_pending_owner_cannot_accept() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let first_pending = Address::generate(&env);
    let second_pending = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // First transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &first_pending, 100);
    client.transfer_ownership(&first_pending, &100);

    // Overwrite with second transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &second_pending, 100);
    client.transfer_ownership(&second_pending, &100);

    // First pending owner should NOT be able to accept
    mock_accept_ownership_auth(&env, &contract_id, &first_pending);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_ownership();
    }));
    assert!(
        result.is_err(),
        "Overwritten pending owner should not be able to accept"
    );

    // Owner should remain unchanged
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should remain unchanged when overwritten pending owner tries to accept"
    );

    // Second pending owner should still be able to accept
    mock_accept_ownership_auth(&env, &contract_id, &second_pending);
    client.accept_ownership();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer_completed(&second_pending);

    assert_eq!(
        client.get_owner(),
        Some(second_pending),
        "Second pending owner should successfully accept"
    );
}

#[test]
fn test_transfer_ownership_cancel_with_zero() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pending_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Initiate transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &pending_owner, 10);
    client.transfer_ownership(&pending_owner, &10);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &pending_owner, 10);

    assert_eq!(
        client.get_pending_owner(),
        Some(pending_owner.clone()),
        "Pending owner should be set after transfer_ownership"
    );

    // Cancel by passing 0
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &pending_owner, 0);
    client.transfer_ownership(&pending_owner, &0);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &pending_owner, 0);

    // Verify pending owner is cleared
    assert_eq!(
        client.get_pending_owner(),
        None,
        "Pending owner should be cleared after cancellation with expires_in_ledgers=0"
    );
    assert_eq!(
        client.get_owner(),
        Some(owner.clone()),
        "Owner should remain unchanged after cancellation"
    );

    // Accept should now fail
    mock_accept_ownership_auth(&env, &contract_id, &pending_owner);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_ownership();
    }));
    assert!(result.is_err(), "Accept should fail after cancellation");
    assert_eq!(client.get_owner(), Some(owner));
}

#[test]
fn test_transfer_ownership_expires() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pending_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Pending owner is None
    assert!(client.get_pending_owner().is_none());

    // Initiate transfer that expires in 10 ledgers
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &pending_owner, 10);
    client.transfer_ownership(&pending_owner, &10);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_ownership_transfer(&owner, &pending_owner, 10);

    // Pending owner was set
    assert_eq!(
        &client.get_pending_owner().unwrap(),
        &pending_owner,
        "Pending owner should be set before expiry"
    );

    // Jump 100 ledgers forward to ensure we exceed any minimum TTL
    env.ledger().with_mut(|l| l.sequence_number = 101);

    // Pending owner is None because it's expired
    assert!(
        client.get_pending_owner().is_none(),
        "Pending owner should be None after expiry"
    );

    // Accept should now fail because it's expired
    mock_accept_ownership_auth(&env, &contract_id, &pending_owner);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_ownership();
    }));
    assert!(result.is_err(), "Accept should fail after expiry");
    assert_eq!(client.get_owner(), Some(owner));
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_transfer_ownership_requires_owner_auth() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Try to transfer without mocking owner auth - should panic
    client.transfer_ownership(&new_owner, &100);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_transfer_ownership_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let new_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with OwnerNotSet error (2100)
    client.transfer_ownership(&new_owner, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_transfer_ownership_fails_when_non_owner_initiates() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Mock auth for non_owner (not the actual owner) to call transfer_ownership
    mock_transfer_ownership_auth(&env, &contract_id, &non_owner, &new_owner, 100);
    client.transfer_ownership(&new_owner, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_accept_ownership_requires_pending_owner_auth() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Initiate transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &new_owner, 100);
    client.transfer_ownership(&new_owner, &100);

    // Try to accept without mocking pending owner auth - should panic
    client.accept_ownership();
}

#[test]
#[should_panic(expected = "#2200")]
fn test_accept_ownership_fails_when_no_pending_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Try to accept without any pending transfer
    // Should fail with RoleTransferError::NoPendingTransfer error (2200)
    client.accept_ownership();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_accept_ownership_fails_when_wrong_address_accepts() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let wrong_address = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    client.set_owner_unchecked(&owner);

    // Initiate transfer
    mock_transfer_ownership_auth(&env, &contract_id, &owner, &new_owner, 100);
    client.transfer_ownership(&new_owner, &100);

    // Try to accept with a different address than pending owner
    mock_accept_ownership_auth(&env, &contract_id, &wrong_address);
    client.accept_ownership();
}
