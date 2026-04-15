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

use super::MinterAssetControllable;
use crate::test_utils::minter_asset_controllable::{
    mock_configure_minter_auth, mock_remove_minter_auth, mock_update_minter_asset_controller_auth,
};
use crate::test_utils::StablecoinEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};
use stellar_access::ownable::set_owner;

#[contract]
struct TestContract;

#[contractimpl]
impl MinterAssetControllable for TestContract {
    fn get_minter_asset_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::MINTER_ASSET_CONTROLLER)
    }

    fn update_minter_asset_controller(e: &Env, new_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            super::MINTER_ASSET_CONTROLLER,
            &new_controller,
            super::emit_minter_asset_controller_changed,
        )
    }

    fn get_minter_allow_asset(e: &Env, minter: Address) -> Option<Address> {
        super::get_minter_allow_asset(e, &minter)
    }

    fn configure_minter(e: &Env, minter: Address, asset: Address) {
        super::configure_minter(e, &minter, &asset)
    }

    fn remove_minter(e: &Env, minter: Address) {
        super::remove_minter(e, &minter)
    }
}

#[contractimpl]
impl TestContract {
    pub fn update_controller_unchecked(env: Env, controller: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::MINTER_ASSET_CONTROLLER,
            &controller,
            super::emit_minter_asset_controller_changed,
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
fn test_get_minter_asset_controller_returns_none_when_not_set() {
    let env = Env::default();

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_minter_asset_controller(),
        None,
        "get_minter_asset_controller should return None when not set"
    );
}

#[test]
fn test_get_minter_allow_asset_returns_none_when_not_configured() {
    let env = Env::default();

    let minter = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(
        client.get_minter_allow_asset(&minter),
        None,
        "get_minter_allow_asset should return None for unconfigured minter"
    );
}

// =============================================================================
// update_minter_asset_controller Tests
// =============================================================================

#[test]
fn test_update_controller_unchecked_sets_controller() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_minter_asset_controller(), None);

    client.update_controller_unchecked(&controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller),
        "Controller should be set after update_controller_unchecked"
    );
}

#[test]
fn test_update_controller_unchecked_overwrites_existing() {
    let env = Env::default();

    let controller1 = Address::generate(&env);
    let controller2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller1);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller1.clone())
    );

    client.update_controller_unchecked(&controller2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller2);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller2),
        "Controller should be overwritten to controller2"
    );
}

#[test]
fn test_update_controller_unchecked_to_same_address() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller.clone())
    );

    client.update_controller_unchecked(&controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller),
        "Controller should remain the same after updating to same address"
    );
}

#[test]
fn test_update_minter_asset_controller_sets_controller() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    assert_eq!(client.get_minter_asset_controller(), None);

    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &controller);
    client.update_minter_asset_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller),
        "Controller should be set after update_minter_asset_controller"
    );
}

#[test]
fn test_update_minter_asset_controller_changes_existing() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let controller1 = Address::generate(&env);
    let controller2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&controller1);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller1.clone())
    );

    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &controller2);
    client.update_minter_asset_controller(&controller2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller2);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller2),
        "Controller should be changed to controller2"
    );
}

#[test]
fn test_update_minter_asset_controller_to_same_address() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller.clone())
    );

    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &controller);
    client.update_minter_asset_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&controller);

    assert_eq!(
        client.get_minter_asset_controller(),
        Some(controller),
        "Controller should remain the same after updating to same address"
    );
}

// =============================================================================
// configure_minter Tests
// =============================================================================

#[test]
fn test_configure_minter_success() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset);

    assert_eq!(client.get_minter_allow_asset(&minter), Some(asset.clone()));
}

#[test]
fn test_configure_multiple_minters() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let minter2 = Address::generate(&env);
    let asset1 = Address::generate(&env);
    let asset2 = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter1, &asset1);
    client.configure_minter(&minter1, &asset1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter1, &asset1);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter2, &asset2);
    client.configure_minter(&minter2, &asset2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter2, &asset2);

    assert_eq!(
        client.get_minter_allow_asset(&minter1),
        Some(asset1.clone()),
        "Minter1 should retain its asset"
    );
    assert_eq!(
        client.get_minter_allow_asset(&minter2),
        Some(asset2.clone()),
        "Minter2 should have its asset"
    );
}

#[test]
fn test_configure_minter_updates_existing() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset1 = Address::generate(&env);
    let asset2 = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset1);
    client.configure_minter(&minter, &asset1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset1);

    assert_eq!(client.get_minter_allow_asset(&minter), Some(asset1.clone()));

    // Hot replacement: reconfigure same minter with different asset
    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset2);
    client.configure_minter(&minter, &asset2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset2);

    assert_eq!(
        client.get_minter_allow_asset(&minter),
        Some(asset2.clone()),
        "Minter's allow asset should be updated to asset2"
    );
}

// =============================================================================
// remove_minter Tests
// =============================================================================

#[test]
fn test_remove_minter_success() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset);

    assert!(client.get_minter_allow_asset(&minter).is_some());

    mock_remove_minter_auth(&env, &contract_id, &controller, &minter);
    client.remove_minter(&minter);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_removed(&minter);

    assert_eq!(client.get_minter_allow_asset(&minter), None);
}

#[test]
fn test_add_remove_add_same_minter() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset1 = Address::generate(&env);
    let asset2 = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    // Add
    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset1);
    client.configure_minter(&minter, &asset1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset1);

    assert_eq!(client.get_minter_allow_asset(&minter), Some(asset1.clone()));

    // Remove
    mock_remove_minter_auth(&env, &contract_id, &controller, &minter);
    client.remove_minter(&minter);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_removed(&minter);

    assert_eq!(client.get_minter_allow_asset(&minter), None);

    // Add again with different asset
    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset2);
    client.configure_minter(&minter, &asset2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset2);

    assert_eq!(client.get_minter_allow_asset(&minter), Some(asset2.clone()));
}

// =============================================================================
// Role Transfer Tests
// =============================================================================

#[test]
fn test_new_controller_can_configure_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_controller = Address::generate(&env);
    let new_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&old_controller);

    // Update controller
    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.update_minter_asset_controller(&new_controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&new_controller);

    // New controller can configure minters
    mock_configure_minter_auth(&env, &contract_id, &new_controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset);

    assert_eq!(
        client.get_minter_allow_asset(&minter),
        Some(asset.clone()),
        "New controller should be able to configure minters"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_configure_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_controller = Address::generate(&env);
    let new_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&old_controller);

    // Update controller
    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.update_minter_asset_controller(&new_controller);

    // Old controller should NOT be able to configure minters
    mock_configure_minter_auth(&env, &contract_id, &old_controller, &minter, &asset);
    client.configure_minter(&minter, &asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_remove_after_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_controller = Address::generate(&env);
    let new_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&old_controller);

    // Configure a minter as old controller
    mock_configure_minter_auth(&env, &contract_id, &old_controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    // Update controller
    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.update_minter_asset_controller(&new_controller);

    // Old controller should NOT be able to remove minters
    mock_remove_minter_auth(&env, &contract_id, &old_controller, &minter);
    client.remove_minter(&minter);
}

#[test]
fn test_minter_state_persists_after_controller_update() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let old_controller = Address::generate(&env);
    let new_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.update_controller_unchecked(&old_controller);

    // Configure a minter
    mock_configure_minter_auth(&env, &contract_id, &old_controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_configured(&minter, &asset);

    assert_eq!(client.get_minter_allow_asset(&minter), Some(asset.clone()));

    // Update controller
    mock_update_minter_asset_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.update_minter_asset_controller(&new_controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_minter_asset_controller_changed(&new_controller);

    // Minter state should persist
    assert_eq!(
        client.get_minter_allow_asset(&minter),
        Some(asset.clone()),
        "Minter state should persist after controller update"
    );
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_minter_asset_controller_requires_owner_auth() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // No mock auth - should fail
    client.update_minter_asset_controller(&controller);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_update_minter_asset_controller_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Owner not set - should fail with OwnerNotSet (#2100)
    client.update_minter_asset_controller(&controller);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_minter_asset_controller_fails_when_non_owner() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Mock auth for non-owner
    mock_update_minter_asset_controller_auth(&env, &contract_id, &non_owner, &controller);
    client.update_minter_asset_controller(&controller);
}

#[test]
#[should_panic(expected = "#7000")]
fn test_configure_minter_fails_when_controller_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Controller not set - should fail with RoleNotSet (#7000)
    client.configure_minter(&minter, &asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_configure_minter_requires_controller_auth() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    // No mock auth - should fail
    client.configure_minter(&minter, &asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_configure_minter_fails_when_non_controller() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let non_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    // Mock auth for wrong address
    mock_configure_minter_auth(&env, &contract_id, &non_controller, &minter, &asset);
    client.configure_minter(&minter, &asset);
}

#[test]
#[should_panic(expected = "#7000")]
fn test_remove_minter_fails_when_controller_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let minter = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Controller not set - should fail with RoleNotSet (#7000)
    client.remove_minter(&minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_remove_minter_requires_controller_auth() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    // No mock auth for remove - should fail
    client.remove_minter(&minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_remove_minter_fails_when_non_controller() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let non_controller = Address::generate(&env);
    let minter = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_configure_minter_auth(&env, &contract_id, &controller, &minter, &asset);
    client.configure_minter(&minter, &asset);

    // Mock auth for wrong address
    mock_remove_minter_auth(&env, &contract_id, &non_controller, &minter);
    client.remove_minter(&minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6702)")]
fn test_remove_minter_fails_when_not_registered() {
    let env = Env::default();

    let controller = Address::generate(&env);
    let minter = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.update_controller_unchecked(&controller);

    mock_remove_minter_auth(&env, &contract_id, &controller, &minter);
    client.remove_minter(&minter); // Should panic with MinterNotFound
}
