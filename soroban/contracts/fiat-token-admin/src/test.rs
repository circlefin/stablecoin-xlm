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
use super::contract::{
    FiatTokenAdminContract, FiatTokenAdminContractClient, FiatTokenAdminContractInitParams,
};
use common_roles::test_utils::manageable::{
    mock_accept_admin_auth, mock_transfer_admin_auth, mock_upgrade_auth,
};
use common_roles::test_utils::ownable::{mock_accept_ownership_auth, mock_transfer_ownership_auth};
use common_roles::test_utils::pausable::{
    mock_pause_auth, mock_unpause_auth, mock_update_pauser_auth,
};
use common_roles::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use mock_minter::{MockMinterContract, MockMinterContractClient};
use soroban_sdk::testutils::storage::Instance;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};
use stablecoin_roles::test_utils::minter_asset_controllable::mock_update_minter_asset_controller_auth;
use stablecoin_roles::test_utils::StablecoinEventAssertions;
use stellar_utils::storage::ttl::{DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD};

extern crate alloc;

// Reuse existing test WASM from the roles package
const UPGRADE_V2_WASM: &[u8] =
    include_bytes!("../../../packages/common-roles/src/manageable/testdata/upgrade_v2.wasm");

#[derive(Clone)]
struct TestContext {
    env: Env,
    contract_id: Address,
    owner: Address,
    pauser: Address,
    admin: Address,
    mint_asset: Address,
    minter_asset_controller: Address,
    allowance_asset_issuer: Address,
    allowance_asset: Address,
    mock_minter: Address,
    blocklister: Address,
}

impl TestContext {
    fn client(&self) -> FiatTokenAdminContractClient<'_> {
        FiatTokenAdminContractClient::new(&self.env, &self.contract_id)
    }

    fn mock_minter_client(&self) -> MockMinterContractClient<'_> {
        MockMinterContractClient::new(&self.env, &self.mock_minter)
    }

    fn allowance_asset_client(&self) -> soroban_sdk::token::StellarAssetClient<'_> {
        soroban_sdk::token::StellarAssetClient::new(&self.env, &self.allowance_asset)
    }

    fn mint_asset_client(&self) -> soroban_sdk::token::StellarAssetClient<'_> {
        soroban_sdk::token::StellarAssetClient::new(&self.env, &self.mint_asset)
    }

    /// Helper to register the mock minter with the allowance asset
    fn register_mock_minter_with_allowance_asset(&self) {
        stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
            &self.env,
            &self.contract_id,
            &self.minter_asset_controller,
            &self.mock_minter,
            &self.allowance_asset,
        );
        self.client()
            .configure_minter(&self.mock_minter, &self.allowance_asset);
    }

    /// Helper to mint allowance asset tokens to the mock minter
    fn mint_allowance_asset_to_minter(&self, amount: i128) {
        self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &self.allowance_asset_issuer,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &self.allowance_asset,
                fn_name: "mint",
                args: (&self.mock_minter, amount).into_val(&self.env),
                sub_invokes: &[],
            },
        }]);
        self.allowance_asset_client()
            .mint(&self.mock_minter, &amount);
    }

    /// Helper to mint allowance asset tokens to an arbitrary address
    fn mint_allowance_asset_to(&self, to: &Address, amount: i128) {
        self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &self.allowance_asset_issuer,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &self.allowance_asset,
                fn_name: "mint",
                args: (to, amount).into_val(&self.env),
                sub_invokes: &[],
            },
        }]);
        self.allowance_asset_client().mint(to, &amount);
    }
}

fn setup_contract() -> TestContext {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let admin = Address::generate(&env);
    let minter_asset_controller = Address::generate(&env);
    let blocklister = Address::generate(&env);

    // Register stellar assets for testing FIRST
    let allowance_asset_issuer = Address::generate(&env);
    let mint_asset_issuer = Address::generate(&env);
    let sac_allowance = env.register_stellar_asset_contract_v2(allowance_asset_issuer.clone());
    let sac_mint = env.register_stellar_asset_contract_v2(mint_asset_issuer.clone());

    // Enable AUTH_REVOCABLE flag so set_authorized works on the mint asset
    sac_mint
        .issuer()
        .set_flag(soroban_sdk::xdr::AccountFlags::RevocableFlag);

    // Register the admin contract with the REAL stellar asset as mint_asset
    let contract_id = env.register(
        FiatTokenAdminContract,
        (FiatTokenAdminContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            admin: admin.clone(),
            mint_asset: sac_mint.address(),
            minter_asset_controller: minter_asset_controller.clone(),
            blocklister: blocklister.clone(),
        },),
    );

    // Set the admin contract as the admin of the mint asset so it can mint and blocklist
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &mint_asset_issuer,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &sac_mint.address(),
            fn_name: "set_admin",
            args: (&contract_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    soroban_sdk::token::StellarAssetClient::new(&env, &sac_mint.address()).set_admin(&contract_id);

    // Register the mock minter contract
    let mock_minter = env.register(MockMinterContract, ());

    TestContext {
        env,
        contract_id,
        owner,
        pauser,
        admin,
        mint_asset: sac_mint.address(),
        minter_asset_controller,
        allowance_asset_issuer,
        allowance_asset: sac_allowance.address(),
        mock_minter,
        blocklister,
    }
}

#[test]
fn test_constructor_initializes_state_and_emits_events() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let admin = Address::generate(&env);
    let minter_asset_controller = Address::generate(&env);
    let blocklister = Address::generate(&env);

    // Register mint asset SAC
    let mint_asset_issuer = Address::generate(&env);
    let sac_mint = env.register_stellar_asset_contract_v2(mint_asset_issuer.clone());
    // Register the contract
    let contract_id = env.register(
        FiatTokenAdminContract,
        (FiatTokenAdminContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            admin: admin.clone(),
            mint_asset: sac_mint.address(),
            minter_asset_controller: minter_asset_controller.clone(),
            blocklister: blocklister.clone(),
        },),
    );

    let client = FiatTokenAdminContractClient::new(&env, &contract_id);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(6);
    events.assert_ownership_transfer_completed(&owner);
    events.assert_pauser_changed(&pauser);
    events.assert_admin_changed(None, &admin);
    events.assert_mint_asset_set(&sac_mint.address());
    events.assert_minter_asset_controller_changed(&minter_asset_controller);
    events.assert_blocklister_changed(&blocklister);

    assert_eq!(client.get_owner(), Some(owner.clone()));
    assert_eq!(client.get_pauser(), Some(pauser.clone()));
    assert_eq!(client.get_admin(), Some(admin.clone()));
    assert_eq!(client.get_mint_asset(), sac_mint.address());
    assert_eq!(
        client.get_minter_asset_controller(),
        Some(minter_asset_controller.clone())
    );
    assert_eq!(client.get_blocklister(), Some(blocklister.clone()));
    assert!(!client.paused());
}

#[test]
fn test_constructor_sets_instance_ttl_to_max_ttl() {
    let ctx = setup_contract();
    let ttl = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl, ctx.env.storage().max_ttl());
}

#[test]
fn test_contract_is_ownable() {
    let ctx = setup_contract();
    let client = ctx.client();

    common_roles::assert_contract_is_ownable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_contract_is_pausable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_pausable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.pauser,
        &ctx.owner
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_swap_mint_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    let minter = Address::generate(&ctx.env);
    let to = Address::generate(&ctx.env);

    // #[when_not_paused] check runs before the function body, so no minter auth is required
    // to trigger the pause failure
    client.swap_mint(&to, &0i128, &minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_configure_minter_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    let minter = Address::generate(&ctx.env);
    let asset = Address::generate(&ctx.env);

    client.configure_minter(&minter, &asset);
}

/// While paused, privileged operations (blocklist, ownership/admin transfer and accept,
/// upgrade, role updates, unpause) still succeed; only pause-restricted actions (e.g. swap_mint,
/// configure_minter) are blocked.
#[test]
fn test_while_paused_privileged_operations_succeed() {
    let ctx = setup_contract();
    let client = ctx.client();

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();
    assert!(client.paused());

    // Blocklist while paused should succeed
    let blocklist_target = Address::generate(&ctx.env);
    mock_blocklist_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.blocklister,
        &blocklist_target,
    );
    client.blocklist(&blocklist_target);
    assert!(client.is_blocklisted(&blocklist_target));

    // Unblocklist while paused should succeed
    mock_unblocklist_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.blocklister,
        &blocklist_target,
    );
    client.unblocklist(&blocklist_target);
    assert!(!client.is_blocklisted(&blocklist_target));

    // Transfer Ownership while paused should succeed
    let expires = 10u32;
    let new_owner = Address::generate(&ctx.env);
    mock_transfer_ownership_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_owner, expires);
    client.transfer_ownership(&new_owner, &expires);

    // Accept Ownership while paused should succeed
    mock_accept_ownership_auth(&ctx.env, &ctx.contract_id, &new_owner);
    client.accept_ownership();
    assert_eq!(client.get_owner(), Some(new_owner.clone()));

    // Transfer Admin while paused should succeed
    let new_admin = Address::generate(&ctx.env);
    mock_transfer_admin_auth(&ctx.env, &ctx.contract_id, &ctx.admin, &new_admin, expires);
    client.transfer_admin(&new_admin, &expires);

    // Accept Admin while paused should succeed
    mock_accept_admin_auth(&ctx.env, &ctx.contract_id, &new_admin);
    client.accept_admin();
    assert_eq!(client.get_admin(), Some(new_admin.clone()));

    // Update Blocklister while paused should succeed
    let new_blocklister = Address::generate(&ctx.env);
    mock_update_blocklister_auth(&ctx.env, &ctx.contract_id, &new_owner, &new_blocklister);
    client.update_blocklister(&new_blocklister);
    assert_eq!(client.get_blocklister(), Some(new_blocklister.clone()));

    // Update Minter Asset Controller while paused should succeed
    let new_minter_asset_controller = Address::generate(&ctx.env);
    mock_update_minter_asset_controller_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_owner,
        &new_minter_asset_controller,
    );
    client.update_minter_asset_controller(&new_minter_asset_controller);
    assert_eq!(
        client.get_minter_asset_controller(),
        Some(new_minter_asset_controller.clone())
    );

    let new_pauser = Address::generate(&ctx.env);
    mock_update_pauser_auth(&ctx.env, &ctx.contract_id, &new_owner, &new_pauser);
    client.update_pauser(&new_pauser);
    assert_eq!(client.get_pauser(), Some(new_pauser.clone()));

    // Unpause while paused should succeed
    mock_unpause_auth(&ctx.env, &ctx.contract_id, &new_pauser);
    client.unpause();
    assert!(!client.paused());

    // Pause again (using current pauser = new_pauser) then upgrade
    mock_pause_auth(&ctx.env, &ctx.contract_id, &new_pauser);
    client.pause();
    assert!(client.paused());

    // Upgrade while paused should succeed
    let v2_wasm_hash = ctx.env.deployer().upload_contract_wasm(UPGRADE_V2_WASM);
    mock_upgrade_auth(&ctx.env, &ctx.contract_id, &new_admin, &v2_wasm_hash);
    client.upgrade(&v2_wasm_hash, &new_admin);
    let v2_result: u32 = ctx.env.invoke_contract(
        &ctx.contract_id,
        &soroban_sdk::Symbol::new(&ctx.env, "v2_only_function"),
        soroban_sdk::Vec::new(&ctx.env),
    );
    assert_eq!(v2_result, 200);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6701)")]
fn test_configure_minter_fails_when_allow_asset_is_mint_asset() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter = Address::generate(&ctx.env);

    client.configure_minter(&minter, &ctx.mint_asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_swap_mint_requires_minter_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter = Address::generate(&ctx.env);
    let to = Address::generate(&ctx.env);

    client.swap_mint(&to, &100i128, &minter);
}

#[test]
fn test_contract_is_manageable() {
    let ctx = setup_contract();
    let client = ctx.client();

    common_roles::assert_contract_is_manageable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.owner,
        &ctx.admin,
        UPGRADE_V2_WASM
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6801)")]
fn test_mint_asset_cannot_be_set_twice() {
    let ctx = setup_contract();

    let new_mint_asset = Address::generate(&ctx.env);
    ctx.env.as_contract(&ctx.contract_id, || {
        crate::storage::set_mint_asset_unchecked(&ctx.env, &new_mint_asset);
    });
}

#[test]
fn test_get_minter_allow_asset() {
    let ctx = setup_contract();
    let client = ctx.client();

    ctx.env.mock_all_auths();

    // minter_allow_asset is None
    let minter_allow_asset = client.get_minter_allow_asset(&ctx.mock_minter);
    assert!(minter_allow_asset.is_none());

    // Set minter_allow_asset
    client.configure_minter(&ctx.mock_minter, &ctx.allowance_asset);

    // minter_allow_asset was set
    let minter_allow_asset = client.get_minter_allow_asset(&ctx.mock_minter).unwrap();
    assert_eq!(minter_allow_asset, ctx.allowance_asset);
}

#[test]
fn test_update_minter_asset_controller() {
    let ctx = setup_contract();
    let client = ctx.client();
    let new_controller = Address::generate(&ctx.env);
    let minter_asset_controller = client.get_minter_asset_controller().unwrap();

    assert_eq!(minter_asset_controller, ctx.minter_asset_controller);

    // Set new_controller as minter asset controller
    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &ctx.owner,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &ctx.contract_id,
            fn_name: "update_minter_asset_controller",
            args: (&new_controller,).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);
    client.update_minter_asset_controller(&new_controller);

    // Minter asset controller was set
    let minter_asset_controller = client.get_minter_asset_controller().unwrap();
    assert_eq!(minter_asset_controller, new_controller);
}

// =============================================================================
// swap_mint Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6803)")]
fn test_swap_mint_fails_when_minter_not_registered() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter = Address::generate(&ctx.env);
    let to = Address::generate(&ctx.env);

    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &minter,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &ctx.contract_id,
            fn_name: "swap_mint",
            args: (&to, &100i128, &minter).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);

    // Minter is not registered, should fail with MinterNotConfigured (#6803)
    client.swap_mint(&to, &100i128, &minter);
}

#[test]
fn test_swap_mint_successful_operation() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);

    assert_eq!(ctx.allowance_asset_client().balance(&ctx.mock_minter), 900);
    assert_eq!(ctx.mint_asset_client().balance(&to), 100);
}

#[test]
fn test_swap_mint_emits_burn_and_mint_events() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);

    let mut allowance_events = EventAssertion::new(&ctx.env, ctx.allowance_asset.clone());
    allowance_events.assert_sac_burn(&ctx.mock_minter, 100);

    let mut mint_events = EventAssertion::new(&ctx.env, ctx.mint_asset.clone());
    mint_events.assert_sac_mint(&to, 100);
}

#[test]
fn test_swap_mint_extends_instance_ttl_by_extend_amount_maximally() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    let initial_ttl = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());

    // Jump forward by the initial TTL ledgers
    ctx.env.ledger().with_mut(|li| {
        li.sequence_number += initial_ttl;
    });

    // Verify the TTL is now 0
    let ttl_before = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_before, 0);

    // Execute a swap mint
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);

    // Verify the TTL was extended
    let ttl_after = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_after, DEFAULT_EXTEND_AMOUNT);
}

#[test]
fn test_swap_mint_extends_instance_ttl_when_ttl_is_below_threshold() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    let initial_ttl = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());

    // Jump forward till near the threshold
    ctx.env.ledger().with_mut(|li| {
        li.sequence_number += initial_ttl - DEFAULT_TTL_THRESHOLD - 1;
    });

    // Verify the TTL is now near the threshold
    let ttl_before = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_before, DEFAULT_TTL_THRESHOLD + 1);

    // Execute a swap mint
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);

    // Verify the TTL was NOT extended
    let ttl_after = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_after, DEFAULT_TTL_THRESHOLD + 1);

    // Jump forward by another ledger
    ctx.env.ledger().with_mut(|li| {
        li.sequence_number += 1;
    });

    // Verify the TTL is at the threshold
    let ttl_before = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_before, DEFAULT_TTL_THRESHOLD);

    // Execute a swap mint
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);

    // Verify the TTL was extended to the extend amount
    let ttl_after = ctx
        .env
        .as_contract(&ctx.contract_id, || ctx.env.storage().instance().get_ttl());
    assert_eq!(ttl_after, DEFAULT_EXTEND_AMOUNT);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6802)")]
fn test_swap_mint_fails_with_zero_amount() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    // Zero amount should fail with InvalidAmount (#6802)
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &0);
}

#[test]
fn test_swap_mint_multiple_operations() {
    let ctx = setup_contract();

    let to1 = Address::generate(&ctx.env);
    let to2 = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to1, &300);

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to2, &200);

    assert_eq!(ctx.allowance_asset_client().balance(&ctx.mock_minter), 500);
    assert_eq!(ctx.mint_asset_client().balance(&to1), 300);
    assert_eq!(ctx.mint_asset_client().balance(&to2), 200);
}

#[test]
fn test_swap_mint_to_same_address_as_minter() {
    let ctx = setup_contract();

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client().execute_swap_mint(
        &ctx.contract_id,
        &ctx.allowance_asset,
        &ctx.mock_minter,
        &100,
    );

    assert_eq!(ctx.allowance_asset_client().balance(&ctx.mock_minter), 900);
    assert_eq!(ctx.mint_asset_client().balance(&ctx.mock_minter), 100);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6803)")]
fn test_swap_mint_fails_after_minter_removed() {
    let ctx = setup_contract();
    let client = ctx.client();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    stablecoin_roles::test_utils::minter_asset_controllable::mock_remove_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &ctx.mock_minter,
    );
    client.remove_minter(&ctx.mock_minter);

    // After removing the minter, should fail with MinterNotConfigured (#6803)
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);
}

// Stellar Asset Contracts wrap Stellar Classic assets which use int64 limits.
// However, the Soroban test environment does NOT enforce these limits - it uses i128
// throughout. These limits are only enforced on actual Stellar networks.
#[test]
fn test_swap_mint_with_max_amount() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    let large_amount: i128 = i128::from(i64::MAX);

    ctx.mint_allowance_asset_to_minter(large_amount);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client().execute_swap_mint(
        &ctx.contract_id,
        &ctx.allowance_asset,
        &to,
        &large_amount,
    );

    assert_eq!(ctx.allowance_asset_client().balance(&ctx.mock_minter), 0);
    assert_eq!(ctx.mint_asset_client().balance(&to), large_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6802)")]
fn test_swap_mint_fails_with_negative_amount() {
    let ctx = setup_contract();
    let client = ctx.client();

    let to = Address::generate(&ctx.env);

    ctx.register_mock_minter_with_allowance_asset();

    // Call swap_mint directly to test our InvalidAmount check
    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &ctx.mock_minter,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &ctx.contract_id,
            fn_name: "swap_mint",
            args: (&to, &(-100i128), &ctx.mock_minter).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);

    client.swap_mint(&to, &(-100i128), &ctx.mock_minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6802)")]
fn test_swap_mint_fails_with_i128_min() {
    let ctx = setup_contract();
    let client = ctx.client();

    let to = Address::generate(&ctx.env);

    ctx.register_mock_minter_with_allowance_asset();

    // Call swap_mint directly to test our InvalidAmount check
    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &ctx.mock_minter,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &ctx.contract_id,
            fn_name: "swap_mint",
            args: (&to, &i128::MIN, &ctx.mock_minter).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);

    client.swap_mint(&to, &i128::MIN, &ctx.mock_minter);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_swap_mint_fails_with_insufficient_allowance_asset_balance() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(100);
    ctx.register_mock_minter_with_allowance_asset();

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &200);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_swap_mint_fails_when_minter_is_deauthorized() {
    let ctx = setup_contract();

    let to = Address::generate(&ctx.env);

    // Create a special allowance asset with AUTH_REVOCABLE flag enabled
    let revocable_issuer = Address::generate(&ctx.env);
    let revocable_sac = ctx
        .env
        .register_stellar_asset_contract_v2(revocable_issuer.clone());
    revocable_sac
        .issuer()
        .set_flag(soroban_sdk::xdr::AccountFlags::RevocableFlag);
    let revocable_asset = revocable_sac.address();

    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &revocable_issuer,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &revocable_asset,
            fn_name: "mint",
            args: (&ctx.mock_minter, &1000i128).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);
    soroban_sdk::token::StellarAssetClient::new(&ctx.env, &revocable_asset)
        .mint(&ctx.mock_minter, &1000);

    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &ctx.mock_minter,
        &revocable_asset,
    );
    ctx.client()
        .configure_minter(&ctx.mock_minter, &revocable_asset);

    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &revocable_issuer,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &revocable_asset,
            fn_name: "set_authorized",
            args: (&ctx.mock_minter, &false).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);
    soroban_sdk::token::StellarAssetClient::new(&ctx.env, &revocable_asset)
        .set_authorized(&ctx.mock_minter, &false);

    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &revocable_asset, &to, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6804)")]
fn test_swap_mint_fails_when_minter_is_blocklisted() {
    let ctx = setup_contract();
    let client = ctx.client();
    let minter = ctx.mock_minter.clone();
    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    // Blocklist the recipient
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &minter);
    client.blocklist(&minter);
    assert!(client.is_blocklisted(&minter));

    // Attempt to mint to blocklisted address - should fail
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_swap_mint_fails_when_recipient_is_blocklisted() {
    let ctx = setup_contract();
    let client = ctx.client();

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to_minter(1000);
    ctx.register_mock_minter_with_allowance_asset();

    // Blocklist the recipient
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &to);
    client.blocklist(&to);
    assert!(client.is_blocklisted(&to));

    // Attempt to mint to blocklisted address - should fail
    ctx.mock_minter_client()
        .execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &100);
}

#[test]
fn test_swap_mint_with_different_minters() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter1 = ctx.env.register(MockMinterContract, ());
    let minter1_client = MockMinterContractClient::new(&ctx.env, &minter1);

    let minter2 = ctx.env.register(MockMinterContract, ());
    let minter2_client = MockMinterContractClient::new(&ctx.env, &minter2);

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to(&minter1, 500);
    ctx.mint_allowance_asset_to(&minter2, 300);

    // Register minter1 with the allowance asset
    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter1,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter1, &ctx.allowance_asset);

    // Register minter2 with the allowance asset
    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter2,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter2, &ctx.allowance_asset);

    minter1_client.execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &200);
    minter2_client.execute_swap_mint(&ctx.contract_id, &ctx.allowance_asset, &to, &150);

    assert_eq!(ctx.allowance_asset_client().balance(&minter1), 300);
    assert_eq!(ctx.allowance_asset_client().balance(&minter2), 150);
    assert_eq!(ctx.mint_asset_client().balance(&to), 350);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_swap_mint_fails_when_minter_impersonates_another_minter() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter1 = ctx.env.register(MockMinterContract, ());
    let minter2 = ctx.env.register(MockMinterContract, ());

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to(&minter1, 500);
    ctx.mint_allowance_asset_to(&minter2, 500);

    // Register both minters
    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter1,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter1, &ctx.allowance_asset);

    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter2,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter2, &ctx.allowance_asset);

    // minter1 tries to call swap_mint with minter2 as the minter parameter
    // This should fail because minter2 hasn't authorized the call
    ctx.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &minter1,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &ctx.contract_id,
            fn_name: "swap_mint",
            args: (&to, &100i128, &minter2).into_val(&ctx.env),
            sub_invokes: &[],
        },
    }]);

    client.swap_mint(&to, &100i128, &minter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_swap_mint_fails_when_contract_minter_impersonates_another_minter() {
    let ctx = setup_contract();
    let client = ctx.client();

    let minter1 = ctx.env.register(MockMinterContract, ());
    let minter1_client = MockMinterContractClient::new(&ctx.env, &minter1);

    let minter2 = ctx.env.register(MockMinterContract, ());

    let to = Address::generate(&ctx.env);

    ctx.mint_allowance_asset_to(&minter1, 500);
    ctx.mint_allowance_asset_to(&minter2, 500);

    // Register both minters
    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter1,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter1, &ctx.allowance_asset);

    stablecoin_roles::test_utils::minter_asset_controllable::mock_configure_minter_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.minter_asset_controller,
        &minter2,
        &ctx.allowance_asset,
    );
    client.configure_minter(&minter2, &ctx.allowance_asset);

    // minter1 contract calls swap_mint but passes minter2 as the minter parameter
    // This should fail because minter2 hasn't authorized the call
    minter1_client.execute_swap_mint_as(
        &ctx.contract_id,
        &ctx.allowance_asset,
        &to,
        &100,
        &minter2,
    );
}

mod proptest_tests {
    use super::*;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    fn swap_sequence_strategy() -> impl Strategy<Value = Vec<i128>> {
        prop::collection::vec(1i128..=1_000_000_000i128, 1..=20)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn prop_swap_mint_balance_invariant(amounts in swap_sequence_strategy()) {
            let ctx = setup_contract();

            let total: i128 = amounts.iter().copied().sum();

            ctx.mint_allowance_asset_to_minter(total);
            ctx.register_mock_minter_with_allowance_asset();

            let initial_allowance_balance = ctx.allowance_asset_client().balance(&ctx.mock_minter);

            let mut total_minted = 0i128;
            for amount in amounts {
                let recipient = Address::generate(&ctx.env);

                ctx.mock_minter_client().execute_swap_mint(
                    &ctx.contract_id,
                    &ctx.allowance_asset,
                    &recipient,
                    &amount,
                );

                prop_assert_eq!(ctx.mint_asset_client().balance(&recipient), amount);
                total_minted += amount;
            }

            let final_allowance_balance = ctx.allowance_asset_client().balance(&ctx.mock_minter);
            let allowance_burned = initial_allowance_balance - final_allowance_balance;

            prop_assert_eq!(allowance_burned, total_minted,
                "Balance invariant violated: burned {} allowance but minted {} mint tokens",
                allowance_burned, total_minted);
        }
    }
}

// =============================================================================
// SAC Blocklist Tests
// =============================================================================

/// Helper to mock blocklister auth for blocklist calls
fn mock_blocklist_auth(env: &Env, contract_id: &Address, blocklister: &Address, address: &Address) {
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: blocklister,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "blocklist",
            args: (address,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Helper to mock blocklister auth for unblocklist calls
fn mock_unblocklist_auth(
    env: &Env,
    contract_id: &Address,
    blocklister: &Address,
    address: &Address,
) {
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: blocklister,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "unblocklist",
            args: (address,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Helper to mock owner auth for update_blocklister calls
fn mock_update_blocklister_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    blocklister: &Address,
) {
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: owner,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_blocklister",
            args: (blocklister,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

#[test]
fn test_blocklister_blocklist() {
    let ctx = setup_contract();
    let client = ctx.client();
    let target = Address::generate(&ctx.env);

    common_roles::assert_initialized_role_holder_management!(
        env: &ctx.env,
        client: &client,
        contract_id: &ctx.contract_id,
        owner: &ctx.owner,
        role_holder: &ctx.blocklister,
        get_fn: get_blocklister,
        set_fn: update_blocklister,
        mock_set_auth: mock_update_blocklister_auth,
        action: |env: &Env, client: &FiatTokenAdminContractClient, contract_id: &Address, holder: &Address| {
            mock_blocklist_auth(env, contract_id, holder, &target);
            client.blocklist(&target);
            target.clone()
        },
        verify_action_persists: |client: &FiatTokenAdminContractClient, blocklisted: &Address| {
            assert!(client.is_blocklisted(blocklisted));
        },
        post_transfer_action: |env: &Env, client: &FiatTokenAdminContractClient, contract_id: &Address, holder: &Address| {
            // Unblocklist the same target
            mock_unblocklist_auth(env, contract_id, holder, &target);
            client.unblocklist(&target);
            assert!(!client.is_blocklisted(&target));
        }
    );
}

#[test]
fn test_blocklister_unblocklist() {
    let ctx = setup_contract();
    let client = ctx.client();
    let target = Address::generate(&ctx.env);

    common_roles::assert_initialized_role_holder_management!(
        env: &ctx.env,
        client: &client,
        contract_id: &ctx.contract_id,
        owner: &ctx.owner,
        role_holder: &ctx.blocklister,
        get_fn: get_blocklister,
        set_fn: update_blocklister,
        mock_set_auth: mock_update_blocklister_auth,
        action: |env: &Env, client: &FiatTokenAdminContractClient, contract_id: &Address, holder: &Address| {
            // Blocklist first, then unblocklist
            mock_blocklist_auth(env, contract_id, holder, &target);
            client.blocklist(&target);
            mock_unblocklist_auth(env, contract_id, holder, &target);
            client.unblocklist(&target);
            target.clone()
        },
        verify_action_persists: |client: &FiatTokenAdminContractClient, unblocklisted: &Address| {
            assert!(!client.is_blocklisted(unblocklisted));
        },
        post_transfer_action: |env: &Env, client: &FiatTokenAdminContractClient, contract_id: &Address, holder: &Address| {
            mock_blocklist_auth(env, contract_id, holder, &target);
            client.blocklist(&target);
            assert!(client.is_blocklisted(&target));
        }
    );
}

#[test]
fn test_update_and_get_blocklister() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_blocklister(), Some(ctx.blocklister.clone()));

    let new_blocklister = Address::generate(&ctx.env);
    mock_update_blocklister_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_blocklister);
    client.update_blocklister(&new_blocklister);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_blocklister_changed(&new_blocklister);

    assert_eq!(client.get_blocklister(), Some(new_blocklister));
}

#[test]
fn test_blocklist_methods_delegate_to_sac() {
    let ctx = setup_contract();
    let client = ctx.client();
    let sac_client = soroban_sdk::token::StellarAssetClient::new(&ctx.env, &ctx.mint_asset);

    let address = Address::generate(&ctx.env);

    // Address is not blocklisted by default
    assert!(!client.is_blocklisted(&address));
    assert!(sac_client.authorized(&address));

    // Blocklist the address
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address);
    client.blocklist(&address);

    let mut sac_events = EventAssertion::new(&ctx.env, ctx.mint_asset.clone());
    sac_events.assert_sac_set_authorized(&address, false);

    let mut admin_events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    admin_events.assert_blocklisted(&address);

    assert!(client.is_blocklisted(&address));
    assert!(!sac_client.authorized(&address));

    // Unblocklist the address
    mock_unblocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address);
    client.unblocklist(&address);

    let mut sac_events = EventAssertion::new(&ctx.env, ctx.mint_asset.clone());
    sac_events.assert_sac_set_authorized(&address, true);

    let mut admin_events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    admin_events.assert_unblocklisted(&address);

    assert!(!client.is_blocklisted(&address));
    assert!(sac_client.authorized(&address));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unblocklist_requires_blocklister_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    let address = Address::generate(&ctx.env);

    // Don't mock auth - should fail
    client.unblocklist(&address);
}

#[test]
fn test_blocklist_twice_is_idempotent() {
    let ctx = setup_contract();
    let client = ctx.client();

    let address = Address::generate(&ctx.env);

    // Blocklist the address
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address);
    client.blocklist(&address);
    assert!(client.is_blocklisted(&address));

    // Blocklist the same address again - should succeed
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address);
    client.blocklist(&address);

    // Both the SAC (set_authorized) and admin contract (Blocklisted) emit events
    // even when the address is already blocklisted.
    let mut admin_events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    admin_events.assert_event_count(2);
    admin_events.assert_blocklisted(&address);

    assert!(client.is_blocklisted(&address));
}

#[test]
fn test_multiple_addresses_can_be_blocklisted() {
    let ctx = setup_contract();
    let client = ctx.client();

    let address1 = Address::generate(&ctx.env);
    let address2 = Address::generate(&ctx.env);
    let address3 = Address::generate(&ctx.env);

    // Blocklist multiple addresses
    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address1);
    client.blocklist(&address1);

    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address2);
    client.blocklist(&address2);

    mock_blocklist_auth(&ctx.env, &ctx.contract_id, &ctx.blocklister, &address3);
    client.blocklist(&address3);

    // Verify all are blocklisted
    assert!(client.is_blocklisted(&address1));
    assert!(client.is_blocklisted(&address2));
    assert!(client.is_blocklisted(&address3));
}
