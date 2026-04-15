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

use super::Rescuable;
use crate::test_utils::rescuable::{mock_rescue_sep41_auth, mock_update_rescuer_auth};
use crate::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, token, Address, Env};
use stellar_access::ownable::set_owner;

use ::proptest::prelude::*;

#[contract]
struct TestContract;

#[contractimpl]
impl Rescuable for TestContract {
    fn get_rescuer(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::RESCUER)
    }

    fn update_rescuer(e: &Env, new_rescuer: Address) {
        simple_role::set_role_and_emit(
            e,
            super::RESCUER,
            &new_rescuer,
            super::emit_rescuer_changed,
        );
    }

    fn rescue_sep41(e: &Env, token_contract: Address, to: Address, amount: i128) {
        super::rescue_sep41(e, &token_contract, &to, &amount)
    }
}

#[contractimpl]
impl TestContract {
    pub fn update_rescuer_unchecked(env: Env, rescuer: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::RESCUER,
            &rescuer,
            super::emit_rescuer_changed,
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
fn test_get_rescuer_returns_none_when_not_set() {
    let env = Env::default();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_rescuer(),
        None,
        "get_rescuer should return None when no rescuer was set"
    );
}

// =============================================================================
// update_rescuer Tests
// =============================================================================

#[test]
fn test_update_rescuer_unchecked_sets_rescuer() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_rescuer(),
        None,
        "Rescuer should be None before set"
    );

    client.update_rescuer_unchecked(&rescuer);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer),
        "Rescuer should be set after update_rescuer_unchecked"
    );
}

#[test]
fn test_update_rescuer_unchecked_overwrites_existing() {
    let env = Env::default();

    let rescuer1 = Address::generate(&env);
    let rescuer2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_rescuer_unchecked(&rescuer1);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer1.clone()),
        "Rescuer should be rescuer1 initially"
    );

    client.update_rescuer_unchecked(&rescuer2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer2);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer2),
        "Rescuer should be overwritten to rescuer2"
    );
}

#[test]
fn test_update_rescuer_unchecked_to_same_address() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_rescuer_unchecked(&rescuer);

    assert_eq!(client.get_rescuer(), Some(rescuer.clone()));

    // Update to the same rescuer
    client.update_rescuer_unchecked(&rescuer);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer),
        "Rescuer should remain the same after updating to same address"
    );
}

#[test]
fn test_update_rescuer_sets_rescuer() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    assert_eq!(
        client.get_rescuer(),
        None,
        "Rescuer should be None before set"
    );

    mock_update_rescuer_auth(&env, &contract_id, &owner, &rescuer);
    client.update_rescuer(&rescuer);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer),
        "Rescuer should be set after update_rescuer"
    );
}

#[test]
fn test_update_rescuer_to_same_address() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_rescuer_unchecked(&rescuer);

    assert_eq!(client.get_rescuer(), Some(rescuer.clone()));

    // Update to the same rescuer via checked path
    mock_update_rescuer_auth(&env, &contract_id, &owner, &rescuer);
    client.update_rescuer(&rescuer);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer),
        "Rescuer should remain the same after updating to same address"
    );
}

#[test]
fn test_update_rescuer_changes_existing_rescuer() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let rescuer1 = Address::generate(&env);
    let rescuer2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_rescuer_unchecked(&rescuer1);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer1.clone()),
        "Rescuer should be rescuer1 initially"
    );

    mock_update_rescuer_auth(&env, &contract_id, &owner, &rescuer2);
    client.update_rescuer(&rescuer2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&rescuer2);

    assert_eq!(
        client.get_rescuer(),
        Some(rescuer2),
        "Rescuer should be changed to rescuer2 after update"
    );
}

// =============================================================================
// rescue_sep41 Tests
// =============================================================================

#[test]
fn test_rescue_sep41_partial() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    token_client.mock_all_auths().mint(&contract_id, &1000);

    client.update_rescuer_unchecked(&rescuer);

    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &rescuer,
        &token_address,
        &recipient,
        500,
    );
    client.rescue_sep41(&token_address, &recipient, &500);

    let token_reader = token::TokenClient::new(&env, &token_address);
    assert_eq!(token_reader.balance(&recipient), 500);
    assert_eq!(token_reader.balance(&contract_id), 500);
}

#[test]
fn test_rescue_sep41_all() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    token_client.mock_all_auths().mint(&contract_id, &1000);

    client.update_rescuer_unchecked(&rescuer);

    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &rescuer,
        &token_address,
        &recipient,
        1000,
    );
    client.rescue_sep41(&token_address, &recipient, &1000);

    let token_reader = token::TokenClient::new(&env, &token_address);
    assert_eq!(token_reader.balance(&recipient), 1000);
    assert_eq!(token_reader.balance(&contract_id), 0);
}

#[test]
fn test_rescue_sep41_max_i64() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    let max_amount = i64::MAX as i128;

    token_client
        .mock_all_auths()
        .mint(&contract_id, &max_amount);

    client.update_rescuer_unchecked(&rescuer);

    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &rescuer,
        &token_address,
        &recipient,
        max_amount,
    );
    client.rescue_sep41(&token_address, &recipient, &max_amount);

    let token_reader = token::TokenClient::new(&env, &token_address);
    assert_eq!(token_reader.balance(&recipient), max_amount);
    assert_eq!(token_reader.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_rescue_sep41_exceeds_i64_max() {
    // Test that attempting to rescue an amount > i64::MAX fails
    // This is critical because Stellar assets use 64-bit balances,
    // and values like 2^64 could potentially wrap to negative values (min(i64))
    let env = Env::default();
    env.mock_all_auths();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.update_rescuer_unchecked(&rescuer);

    // Try to rescue 2^64, which exceeds i64::MAX and could wrap to negative
    // This should panic/fail before attempting the transfer
    let invalid_amount = 2i128.pow(64); // 2^64 = 18446744073709551616
    client.rescue_sep41(&token_address, &recipient, &invalid_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_rescue_sep41_more_than_available() {
    let env = Env::default();
    env.mock_all_auths();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    // Mint 1000 tokens to the test contract
    token_client.mock_all_auths().mint(&contract_id, &1000);

    client.update_rescuer_unchecked(&rescuer);

    // Try to rescue more than available - should panic with insufficient balance error
    client.rescue_sep41(&token_address, &recipient, &1500);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_rescue_sep41_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    // Mint some tokens to the test contract
    token_client.mock_all_auths().mint(&contract_id, &1000);

    client.update_rescuer_unchecked(&rescuer);

    // Try to rescue negative amount - should panic with negative amount error
    client.rescue_sep41(&token_address, &recipient, &-100);
}

// =============================================================================
// Role Transfer Tests
// =============================================================================

#[test]
fn test_new_rescuer_can_rescue_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_rescuer = Address::generate(&env);
    let new_rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    client.set_owner_unchecked(&owner);
    client.update_rescuer_unchecked(&old_rescuer);

    // Update rescuer
    mock_update_rescuer_auth(&env, &contract_id, &owner, &new_rescuer);
    client.update_rescuer(&new_rescuer);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_rescuer_changed(&new_rescuer);

    // New rescuer can rescue
    token_client.mock_all_auths().mint(&contract_id, &1000);
    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &new_rescuer,
        &token_address,
        &recipient,
        500,
    );
    client.rescue_sep41(&token_address, &recipient, &500);

    let token_reader = token::TokenClient::new(&env, &token_address);
    assert_eq!(
        token_reader.balance(&recipient),
        500,
        "New rescuer should be able to rescue tokens"
    );
    assert_eq!(token_reader.balance(&contract_id), 500);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_rescuer_cannot_rescue_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_rescuer = Address::generate(&env);
    let new_rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    client.set_owner_unchecked(&owner);
    client.update_rescuer_unchecked(&old_rescuer);

    // Update to new rescuer
    mock_update_rescuer_auth(&env, &contract_id, &owner, &new_rescuer);
    client.update_rescuer(&new_rescuer);

    // Old rescuer should NOT be able to rescue
    token_client.mock_all_auths().mint(&contract_id, &1000);
    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &old_rescuer,
        &token_address,
        &recipient,
        500,
    );
    client.rescue_sep41(&token_address, &recipient, &500);
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_rescuer_requires_owner_auth() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // No mock auth - should fail
    client.update_rescuer(&rescuer);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_update_rescuer_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Owner not set - should fail with OwnerNotSet (#2100)
    client.update_rescuer(&rescuer);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_rescuer_fails_when_non_owner() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Mock auth for non-owner
    mock_update_rescuer_auth(&env, &contract_id, &non_owner, &rescuer);
    client.update_rescuer(&rescuer);
}

#[test]
#[should_panic(expected = "#7000")]
fn test_rescue_sep41_fails_when_rescuer_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let recipient = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    token_client.mock_all_auths().mint(&contract_id, &1000);

    // Should fail with RoleNotSet (#7000)
    client.rescue_sep41(&token_address, &recipient, &500);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_rescue_sep41_requires_rescuer_auth() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    client.update_rescuer_unchecked(&rescuer);
    token_client.mock_all_auths().mint(&contract_id, &1000);

    // No mock auth for rescuer - should fail
    client.rescue_sep41(&token_address, &recipient, &500);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_rescue_sep41_fails_when_non_rescuer() {
    let env = Env::default();

    let rescuer = Address::generate(&env);
    let non_rescuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    client.update_rescuer_unchecked(&rescuer);
    token_client.mock_all_auths().mint(&contract_id, &1000);

    // Mock auth for wrong address
    mock_rescue_sep41_auth(
        &env,
        &contract_id,
        &non_rescuer,
        &token_address,
        &recipient,
        500,
    );
    client.rescue_sep41(&token_address, &recipient, &500);
}

// =============================================================================
// Prop Tests
// =============================================================================

#[derive(Debug, Clone)]
struct RescueInput {
    initial_balance: i128,
    rescue_amount: i128,
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property test: rescue operations should maintain token balance invariants
    /// Tests that for any valid rescue operation:
    /// - Total tokens are conserved (recipient + contract = initial)
    /// - All balances remain non-negative
    /// - Rescue amount is correctly transferred
    #[test]
    fn prop_rescue_sep41_maintains_invariants(
        initial_balance in 1i128..=10_000_000i128,
        rescue_amount in 1i128..=10_000_000i128,
    ) {
        // Only test when rescue_amount <= initial_balance
        prop_assume!(rescue_amount <= initial_balance);

        let input = RescueInput {
            initial_balance,
            rescue_amount,
        };

        let env = Env::default();
        env.mock_all_auths();

        let rescuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        let contract_id = env.register(TestContract, ());
        let client = TestContractClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        let token_client = token::StellarAssetClient::new(&env, &token_address);

        // Mint initial balance to the test contract
        token_client.mock_all_auths().mint(&contract_id, &input.initial_balance);

        client.update_rescuer_unchecked(&rescuer);

        // Perform the rescue
        client.rescue_sep41(&token_address, &recipient, &input.rescue_amount);

        // Assert invariants
        assert_rescue_invariants(
            &env,
            &token_address,
            &contract_id,
            &recipient,
            &input,
        );
    }

    /// Property test: multiple rescue operations should maintain invariants
    #[test]
    fn prop_rescue_sep41_multiple_rescues(
        initial_balance in 100i128..=1_000_000_000_000i128,
        first_amount in 1i128..=50_000_000_000i128,
        second_amount in 1i128..=50_000_000_000i128,
    ) {
        // Only test when both rescues are valid
        prop_assume!(first_amount <= initial_balance);
        prop_assume!(second_amount <= initial_balance - first_amount);

        let env = Env::default();
        env.mock_all_auths();

        let rescuer = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);

        let contract_id = env.register(TestContract, ());
        let client = TestContractClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        let token_client = token::StellarAssetClient::new(&env, &token_address);

        token_client.mock_all_auths().mint(&contract_id, &initial_balance);

        client.update_rescuer_unchecked(&rescuer);

        // First rescue
        client.rescue_sep41(&token_address, &recipient1, &first_amount);

        // Second rescue
        client.rescue_sep41(&token_address, &recipient2, &second_amount);

        // Verify total conservation
        let token_reader = token::TokenClient::new(&env, &token_address);
        let recipient1_balance = token_reader.balance(&recipient1);
        let recipient2_balance = token_reader.balance(&recipient2);
        let contract_balance = token_reader.balance(&contract_id);

        assert_eq!(recipient1_balance, first_amount);
        assert_eq!(recipient2_balance, second_amount);
        assert_eq!(contract_balance, initial_balance - first_amount - second_amount);

        // Total tokens should equal initial balance
        let total = recipient1_balance + recipient2_balance + contract_balance;
        assert_eq!(total, initial_balance);
    }
}

/// Helper function to assert invariants after a rescue operation
fn assert_rescue_invariants(
    env: &Env,
    token_address: &Address,
    contract_id: &Address,
    recipient: &Address,
    input: &RescueInput,
) {
    let token_reader = token::TokenClient::new(env, token_address);

    let recipient_balance = token_reader.balance(recipient);
    let contract_balance = token_reader.balance(contract_id);

    // Invariant 1: All balances must be non-negative
    assert!(recipient_balance >= 0);
    assert!(contract_balance >= 0);

    // Invariant 2: Recipient received the exact rescue amount
    assert_eq!(recipient_balance, input.rescue_amount);

    // Invariant 3: Contract balance is reduced by rescue amount
    assert_eq!(
        contract_balance,
        input.initial_balance - input.rescue_amount
    );

    // Invariant 4: Total tokens are conserved
    let total_balance = recipient_balance + contract_balance;
    assert_eq!(total_balance, input.initial_balance);
}
