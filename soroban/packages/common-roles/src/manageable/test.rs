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

use super::Manageable;
use crate::test_utils::manageable::{
    mock_accept_admin_auth, mock_migrate_auth, mock_transfer_admin_auth, mock_upgrade_auth,
};
use crate::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, testutils::Address as _,
    testutils::Ledger as _, Address, BytesN, Env, IntoVal, Symbol, Vec,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::Upgradeable;

// WASM for upgrade testing - we use the upgrade_v2 contract as the upgrade target
const UPGRADE_WASM: &[u8] = include_bytes!("testdata/upgrade_v2.wasm");

// WASM for migratable upgrade testing
const UPGRADE_MIGRATABLE_WASM: &[u8] = include_bytes!("testdata/upgrade_v2_migratable.wasm");

fn upload_upgrade_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(UPGRADE_WASM)
}

fn upload_upgrade_migratable_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(UPGRADE_MIGRATABLE_WASM)
}

/// Storage key for test data - matches upgrade_v2 contract for upgrade testing.
pub const DATA: Symbol = symbol_short!("DATA");

/// Migration data struct - must match the one in upgrade_v2_migratable contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V2MigrationData {
    pub num1: u32,
    pub num2: u32,
}

#[derive(Upgradeable)]
#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn __constructor(e: &Env, admin: Option<Address>) {
        if let Some(admin) = admin {
            super::set_admin_unchecked(e, &admin);
        }
    }

    pub fn set_admin_unchecked(env: Env, admin: Address) {
        super::set_admin_unchecked(&env, &admin);
    }

    /// Stores arbitrary data in instance storage for testing persistence.
    pub fn set_data(e: &Env, data: u32) {
        e.storage().instance().set(&DATA, &data);
    }

    /// Retrieves stored data from instance storage.
    pub fn get_data(e: &Env) -> Option<u32> {
        e.storage().instance().get(&DATA)
    }
}

impl UpgradeableInternal for TestContract {
    fn _require_auth(e: &Env, _operator: &Address) {
        super::storage::enforce_admin_auth(e);
    }
}

#[contractimpl]
impl Manageable for TestContract {
    fn get_admin(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::ADMIN)
    }

    fn get_pending_admin(e: &Env) -> Option<Address> {
        super::get_pending_admin(e)
    }

    fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32) {
        super::transfer_admin(e, &new_admin, expires_in_ledgers)
    }

    fn accept_admin(e: &Env) {
        super::accept_admin(e)
    }
}

// =============================================================================
// Constructor & Initial State Tests
// =============================================================================

#[test]
fn test_set_and_get_admin() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    let stored_admin = client.get_admin();
    assert_eq!(
        Some(admin),
        stored_admin,
        "get_admin should return the admin set in the constructor"
    );
}

#[test]
fn test_get_admin_returns_none_when_not_set() {
    let env = Env::default();

    let contract_id = env.register(TestContract, (None::<Address>,));
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_admin(),
        None,
        "get_admin should return None when no admin was set in constructor"
    );
}

#[test]
fn test_get_pending_admin_returns_none_when_not_set() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be None when no transfer has been initiated"
    );
}

#[test]
fn test_constructor_without_admin_emits_no_admin_events() {
    let env = Env::default();

    let contract_id = env.register(TestContract, (None::<Address>,));

    let events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(0);
}

// =============================================================================
// set_admin_unchecked Tests
// =============================================================================

#[test]
fn test_set_admin_unchecked_sets_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (None::<Address>,));

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(0);

    let client = TestContractClient::new(&env, &contract_id);
    assert_eq!(client.get_admin(), None, "Admin should be None before set");

    client.set_admin_unchecked(&admin);

    events.assert_event_count(1);
    events.assert_admin_changed(None, &admin);

    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should be set after set_admin_unchecked"
    );
}

#[test]
#[should_panic(expected = "#7201")]
fn test_set_admin_unchecked_fails_if_already_set() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin1),));

    let client = TestContractClient::new(&env, &contract_id);
    client.set_admin_unchecked(&admin2);
}

// =============================================================================
// Two-Step Admin Transfer Tests
// =============================================================================

#[test]
fn test_transfer_admin_two_step_flow() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Verify initial state
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should be set after construction"
    );
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be None before transfer"
    );

    // Step 1: Current admin initiates transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    // Verify pending admin is set but admin hasn't changed yet
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should remain unchanged during pending transfer"
    );
    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin.clone()),
        "Pending admin should be set after transfer_admin"
    );

    // Step 2: New admin accepts transfer
    mock_accept_admin_auth(&env, &contract_id, &new_admin);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin), &new_admin);

    // Verify admin changed and pending is cleared
    assert_eq!(
        client.get_admin(),
        Some(new_admin.clone()),
        "Admin should be updated after accept_admin"
    );
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be cleared after accept_admin"
    );
}

#[test]
fn test_transfer_admin_can_transfer_admin_to_self() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Verify initial state
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should be set after construction"
    );
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be None before transfer"
    );

    mock_transfer_admin_auth(&env, &contract_id, &admin, &admin, 100);
    client.transfer_admin(&admin, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &admin);

    assert_eq!(
        client.get_pending_admin(),
        Some(admin.clone()),
        "Pending admin should be set to self after self-transfer"
    );
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should remain unchanged during pending self-transfer"
    );

    // Accept the self-transfer
    mock_accept_admin_auth(&env, &contract_id, &admin);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin), &admin);

    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should remain the same after self-transfer acceptance"
    );
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be cleared after self-transfer acceptance"
    );
}

#[test]
fn test_chained_admin_transfers() {
    let env = Env::default();

    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    let admin_c = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin_a),));
    let client = TestContractClient::new(&env, &contract_id);

    // Transfer A -> B
    mock_transfer_admin_auth(&env, &contract_id, &admin_a, &admin_b, 100);
    client.transfer_admin(&admin_b, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin_a), &admin_b);

    mock_accept_admin_auth(&env, &contract_id, &admin_b);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin_a), &admin_b);

    assert_eq!(
        client.get_admin(),
        Some(admin_b.clone()),
        "Admin should be B after first transfer"
    );

    // Transfer B -> C (new admin can initiate transfers)
    mock_transfer_admin_auth(&env, &contract_id, &admin_b, &admin_c, 100);
    client.transfer_admin(&admin_c, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin_b), &admin_c);

    mock_accept_admin_auth(&env, &contract_id, &admin_c);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin_b), &admin_c);

    assert_eq!(
        client.get_admin(),
        Some(admin_c.clone()),
        "Admin should be C after second transfer"
    );
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be None after second transfer completed"
    );
}

#[test]
fn test_transfer_admin_overwrites_pending_transfer() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin1 = Address::generate(&env);
    let new_admin2 = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // First transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin1, 100);
    client.transfer_admin(&new_admin1, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin1);

    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin1.clone()),
        "Pending admin should be new_admin1 after first transfer"
    );

    // Second transfer should overwrite the first pending transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin2, 100);
    client.transfer_admin(&new_admin2, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin2);

    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin2.clone()),
        "Pending admin should be overwritten to new_admin2"
    );

    // new_admin2 should be able to accept, not new_admin1
    mock_accept_admin_auth(&env, &contract_id, &new_admin2);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin), &new_admin2);

    assert_eq!(
        client.get_admin(),
        Some(new_admin2.clone()),
        "Admin should be new_admin2 after acceptance"
    );
}

#[test]
fn test_overwritten_pending_admin_cannot_accept() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let first_pending = Address::generate(&env);
    let second_pending = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // First transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &first_pending, 100);
    client.transfer_admin(&first_pending, &100);

    // Overwrite with second transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &second_pending, 100);
    client.transfer_admin(&second_pending, &100);

    // First pending admin should NOT be able to accept
    mock_accept_admin_auth(&env, &contract_id, &first_pending);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_admin();
    }));
    assert!(
        result.is_err(),
        "Overwritten pending admin should not be able to accept"
    );

    // Admin should remain unchanged
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should remain unchanged when overwritten pending admin tries to accept"
    );

    // Second pending admin should still be able to accept
    mock_accept_admin_auth(&env, &contract_id, &second_pending);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin), &second_pending);

    assert_eq!(
        client.get_admin(),
        Some(second_pending.clone()),
        "Second pending admin should successfully accept"
    );
}

#[test]
fn test_transfer_admin_cancel_with_zero() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Initiate transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin.clone()),
        "Pending admin should be set after transfer_admin"
    );

    // Cancel by passing 0
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 0);
    client.transfer_admin(&new_admin, &0);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    // Verify pending admin is cleared
    assert_eq!(
        client.get_pending_admin(),
        None,
        "Pending admin should be cleared after cancellation with expires_in_ledgers=0"
    );
    assert_eq!(
        client.get_admin(),
        Some(admin.clone()),
        "Admin should remain unchanged after cancellation"
    );

    // Accept should now fail
    mock_accept_admin_auth(&env, &contract_id, &new_admin);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_admin();
    }));
    assert!(result.is_err(), "Accept should fail after cancellation");
}

#[test]
fn test_transfer_admin_expires() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Initiate transfer that expires in 10 ledgers
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 10);
    client.transfer_admin(&new_admin, &10);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin.clone()),
        "Pending admin should be set before expiry"
    );

    // Jump 100 ledgers forward to ensure we exceed any minimum TTL
    env.ledger().with_mut(|l| l.sequence_number = 101);

    // Accept should now fail because the pending admin entry has expired
    mock_accept_admin_auth(&env, &contract_id, &new_admin);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.accept_admin();
    }));
    assert!(result.is_err(), "Accept should fail after expiry");
    assert_eq!(
        client.get_admin(),
        Some(admin),
        "Admin should remain unchanged after expired transfer"
    );
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
fn test_enforce_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Trigger an upgrade which requires admin auth
    let wasm_hash = upload_upgrade_wasm(&env);
    client.upgrade(&wasm_hash, &admin);

    let auths = env.auths();
    assert_eq!(
        auths.len(),
        1,
        "Exactly one auth invocation should be recorded"
    );
    let (auth_address, _) = &auths[0];
    assert_eq!(
        auth_address, &admin,
        "Auth invocation should be from the admin address"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_transfer_admin_requires_admin_auth() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Try to transfer without mocking admin auth - should panic
    client.transfer_admin(&new_admin, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #7200)")]
fn test_transfer_admin_fails_when_admin_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (None::<&Address>,));
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with AdminNotSet error (7200)
    client.transfer_admin(&new_admin, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_transfer_admin_fails_when_non_admin_initiates() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Mock auth for non_admin (not the actual admin) to call transfer_admin
    mock_transfer_admin_auth(&env, &contract_id, &non_admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_accept_admin_requires_pending_admin_auth() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Initiate transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);

    // Try to accept without mocking pending admin auth - should panic
    client.accept_admin();
}

#[test]
#[should_panic(expected = "Error(Contract, #2200)")]
fn test_accept_admin_fails_when_no_pending_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Should fail with RoleTransferError::NoPendingTransfer error (2200)
    client.accept_admin();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_accept_admin_fails_when_wrong_address_accepts() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let wrong_address = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Initiate transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);

    // Try to accept with a different address than pending admin
    mock_accept_admin_auth(&env, &contract_id, &wrong_address);
    client.accept_admin();
}

#[test]
fn test_new_admin_can_operate_after_transfer() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Complete a full transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    mock_accept_admin_auth(&env, &contract_id, &new_admin);
    client.accept_admin();

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_changed(Some(&admin), &new_admin);

    // New admin should be able to perform admin-gated operations (upgrade)
    let wasm_hash = upload_upgrade_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &new_admin, &wasm_hash);
    client.upgrade(&wasm_hash, &new_admin);

    // Verify upgrade succeeded
    let result: u32 = env.invoke_contract(
        &contract_id,
        &Symbol::new(&env, "v2_only_function"),
        Vec::new(&env),
    );
    assert_eq!(
        result, 200,
        "New admin should be able to upgrade the contract"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_admin_cannot_operate_after_transfer() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Complete a full transfer
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 100);
    client.transfer_admin(&new_admin, &100);
    mock_accept_admin_auth(&env, &contract_id, &new_admin);
    client.accept_admin();

    // Old admin should NOT be able to perform admin-gated operations
    let wasm_hash = upload_upgrade_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);
}

// =============================================================================
// Upgrade Tests
// =============================================================================

#[test]
fn test_upgrade_succeeds() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    let wasm_hash = upload_upgrade_wasm(&env);

    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    // Verify upgrade succeeded by calling v2-only function
    let result: u32 = env.invoke_contract(
        &contract_id,
        &Symbol::new(&env, "v2_only_function"),
        Vec::new(&env),
    );
    assert_eq!(
        result, 200,
        "V2-only function should return 200 after upgrade"
    );
}

#[test]
fn test_upgrade_preserves_instance_storage() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Store data before upgrade
    let test_value: u32 = 100;
    client.set_data(&test_value);
    assert_eq!(
        client.get_data(),
        Some(test_value),
        "Data should be stored before upgrade"
    );

    // Perform upgrade
    let wasm_hash = upload_upgrade_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    assert_eq!(
        client.get_admin(),
        Some(admin),
        "Admin should be preserved after upgrade"
    );

    // Data should persist after upgrade
    assert_eq!(
        client.get_data(),
        Some(test_value),
        "Instance storage data should persist after upgrade"
    );
}

#[test]
fn test_upgrade_preserves_admin_and_pending_admin() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Initiate a transfer (so pending admin is set)
    mock_transfer_admin_auth(&env, &contract_id, &admin, &new_admin, 1000);
    client.transfer_admin(&new_admin, &1000);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_admin_change_started(Some(&admin), &new_admin);

    assert_eq!(
        client.get_pending_admin(),
        Some(new_admin.clone()),
        "Pending admin should be set before upgrade"
    );

    // Perform upgrade
    let wasm_hash = upload_upgrade_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    // Verify admin is preserved
    assert_eq!(
        client.get_admin(),
        Some(admin),
        "Admin should be preserved after upgrade"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_upgrade_non_admin() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    let wasm_hash = upload_upgrade_wasm(&env);
    // Don't mock any auths - should panic because no auth is provided for the admin
    client.upgrade(&wasm_hash, &admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #7200)")]
fn test_upgrade_admin_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TestContract, (None::<&Address>,));
    let client = TestContractClient::new(&env, &contract_id);

    let wasm_hash = upload_upgrade_wasm(&env);
    let fake_admin = Address::generate(&env);

    // Should fail with AdminNotSet error (7200)
    client.upgrade(&wasm_hash, &fake_admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_upgrade_fails_when_address_impersonates_another_admin() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let impersonator = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    let wasm_hash = upload_upgrade_wasm(&env);
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &impersonator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "upgrade",
            args: (&wasm_hash, &admin).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&wasm_hash, &impersonator);
}

// =============================================================================
// Migratable Upgrade Tests
// =============================================================================

#[test]
fn test_upgrade_migratable_succeeds() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    let wasm_hash = upload_upgrade_migratable_wasm(&env);

    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    // Verify upgrade succeeded by calling v2-only function
    let result: u32 = env.invoke_contract(
        &contract_id,
        &Symbol::new(&env, "v2_only_function"),
        Vec::new(&env),
    );
    assert_eq!(result, 200);
}

#[test]
fn test_upgrade_migratable_with_migration() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Store data before upgrade to verify persistence
    let test_value: u32 = 100;
    client.set_data(&test_value);

    // Upgrade to migratable contract
    let wasm_hash = upload_upgrade_migratable_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    // Perform migration with data
    let migration_data = V2MigrationData { num1: 42, num2: 99 };
    mock_migrate_auth(&env, &contract_id, &admin, migration_data.clone());
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "migrate"),
        (migration_data.clone(), &admin).into_val(&env),
    );

    // Verify migration data was stored
    let stored_migration_data: V2MigrationData = env.invoke_contract(
        &contract_id,
        &Symbol::new(&env, "get_migration_data"),
        Vec::new(&env),
    );
    assert_eq!(stored_migration_data, migration_data);

    // Verify original data persisted through upgrade
    let stored_data: Option<u32> =
        env.invoke_contract(&contract_id, &Symbol::new(&env, "get_data"), Vec::new(&env));
    assert_eq!(stored_data, Some(test_value));
}

#[test]
#[should_panic(expected = "Error(Contract, #1100)")]
fn test_migrate_twice_fails() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(TestContract, (Some(&admin),));
    let client = TestContractClient::new(&env, &contract_id);

    // Upgrade to migratable contract
    let wasm_hash = upload_upgrade_migratable_wasm(&env);
    mock_upgrade_auth(&env, &contract_id, &admin, &wasm_hash);
    client.upgrade(&wasm_hash, &admin);

    // Perform first migration - should succeed
    let migration_data = V2MigrationData { num1: 42, num2: 99 };
    mock_migrate_auth(&env, &contract_id, &admin, migration_data.clone());
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "migrate"),
        (migration_data.clone(), &admin).into_val(&env),
    );

    // Attempt second migration - should fail with MigrationNotAllowed
    // because migrate can only be called once per upgrade
    let migration_data2 = V2MigrationData {
        num1: 100,
        num2: 200,
    };
    mock_migrate_auth(&env, &contract_id, &admin, migration_data2.clone());
    env.invoke_contract::<()>(
        &contract_id,
        &Symbol::new(&env, "migrate"),
        (migration_data2, &admin).into_val(&env),
    );
}
