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
use common_roles::{manageable, ownable, pausable, simple_role, Manageable, Ownable, Pausable};
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token::StellarAssetClient, Address, Env,
};
use stablecoin_interfaces::SwapMinter;
use stablecoin_roles::{
    minter_asset_controllable, MinterAssetControllable, MinterAssetControllableError,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{when_not_paused, Upgradeable};
use stellar_utils::storage::ttl::{
    extend_instance_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use crate::{
    blocklistable, blocklistable::Blocklistable, emit_blocklister_changed, storage,
    FiatTokenAdminError, BLOCKLISTER,
};

#[derive(Upgradeable)]
#[contract]
pub struct FiatTokenAdminContract;

#[derive(Clone)]
#[contracttype]
pub struct FiatTokenAdminContractInitParams {
    pub owner: Address,
    pub pauser: Address,
    pub admin: Address,
    pub mint_asset: Address,
    pub minter_asset_controller: Address,
    pub blocklister: Address,
}

#[contractimpl]
impl FiatTokenAdminContract {
    pub fn __constructor(e: Env, params: FiatTokenAdminContractInitParams) {
        ownable::set_owner_unchecked(&e, &params.owner);
        simple_role::set_role_and_emit_unchecked(
            &e,
            pausable::PAUSER,
            &params.pauser,
            pausable::emit_pauser_changed,
        );
        manageable::set_admin_unchecked(&e, &params.admin);
        storage::set_mint_asset_unchecked(&e, &params.mint_asset);
        simple_role::set_role_and_emit_unchecked(
            &e,
            minter_asset_controllable::MINTER_ASSET_CONTROLLER,
            &params.minter_asset_controller,
            minter_asset_controllable::emit_minter_asset_controller_changed,
        );
        simple_role::set_role_and_emit_unchecked(
            &e,
            BLOCKLISTER,
            &params.blocklister,
            emit_blocklister_changed,
        );
        extend_instance_ttl(&e, e.storage().max_ttl(), e.storage().max_ttl());
    }

    /// Returns the mint asset address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The mint asset address.
    pub fn get_mint_asset(e: Env) -> Address {
        storage::get_mint_asset(&e)
    }
}

#[contractimpl]
impl Blocklistable for FiatTokenAdminContract {
    /// Returns the current blocklister address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` – The blocklister address if set.
    /// * `None` – If the blocklister has not been set.
    fn get_blocklister(e: Env) -> Option<Address> {
        simple_role::try_get_role(&e, BLOCKLISTER)
    }
    /// Updates the blocklister address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `blocklister` - The address that will be authorized to blocklist/unblocklist addresses.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["blocklister_changed"]`
    /// * data - `[new_blocklister: Address]`
    fn update_blocklister(e: Env, blocklister: Address) {
        simple_role::set_role_and_emit(&e, BLOCKLISTER, &blocklister, emit_blocklister_changed);
    }
    fn is_blocklisted(e: Env, address: Address) -> bool {
        blocklistable::is_blocklisted(e, address)
    }
    fn blocklist(e: Env, address: Address) {
        blocklistable::blocklist(e, address);
    }
    fn unblocklist(e: Env, address: Address) {
        blocklistable::unblocklist(e, address);
    }
}

#[contractimpl]
impl Pausable for FiatTokenAdminContract {
    fn get_pauser(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, pausable::PAUSER)
    }
    fn update_pauser(e: &Env, new_pauser: Address) {
        simple_role::set_role_and_emit(
            e,
            pausable::PAUSER,
            &new_pauser,
            pausable::emit_pauser_changed,
        );
    }
    fn pause(e: &Env) {
        pausable::pause(e);
    }
    fn unpause(e: &Env) {
        pausable::unpause(e);
    }
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }
}

#[contractimpl]
impl Ownable for FiatTokenAdminContract {
    fn get_owner(e: &Env) -> Option<Address> {
        ownable::get_owner(e)
    }
    fn get_pending_owner(e: &Env) -> Option<Address> {
        ownable::get_pending_owner(e)
    }
    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32) {
        ownable::transfer_ownership(e, &new_owner, expires_in_ledgers);
    }
    fn accept_ownership(e: &Env) {
        ownable::accept_ownership(e);
    }
}

#[contractimpl]
impl Manageable for FiatTokenAdminContract {
    fn get_admin(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, manageable::ADMIN)
    }

    fn get_pending_admin(e: &Env) -> Option<Address> {
        manageable::get_pending_admin(e)
    }

    fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32) {
        manageable::transfer_admin(e, &new_admin, expires_in_ledgers)
    }

    fn accept_admin(e: &Env) {
        manageable::accept_admin(e)
    }
}

#[contractimpl]
impl MinterAssetControllable for FiatTokenAdminContract {
    fn get_minter_asset_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, minter_asset_controllable::MINTER_ASSET_CONTROLLER)
    }
    fn update_minter_asset_controller(e: &Env, new_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            minter_asset_controllable::MINTER_ASSET_CONTROLLER,
            &new_controller,
            minter_asset_controllable::emit_minter_asset_controller_changed,
        )
    }
    fn get_minter_allow_asset(e: &Env, minter: Address) -> Option<Address> {
        minter_asset_controllable::get_minter_allow_asset(e, &minter)
    }
    #[when_not_paused]
    fn configure_minter(e: &Env, minter: Address, allow_asset: Address) {
        if allow_asset == storage::get_mint_asset(e) {
            panic_with_error!(e, MinterAssetControllableError::InvalidAllowAsset);
        }
        minter_asset_controllable::configure_minter(e, &minter, &allow_asset)
    }
    fn remove_minter(e: &Env, minter: Address) {
        minter_asset_controllable::remove_minter(e, &minter)
    }
}

#[contractimpl]
impl SwapMinter for FiatTokenAdminContract {
    /// Swaps the minter's registered allow asset 1:1 for `mint_asset` by burning `amount` from `minter` and minting `amount` to `to`.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `to` - The recipient.
    /// * `amount` - The amount to mint (must be greater than zero).
    /// * `minter` - The minter address (must be registered via `configure_minter`).
    ///
    /// # Errors
    ///
    /// * [`FiatTokenAdminError::InvalidAmount`] – If amount is less than or equal to zero.
    /// * [`MinterAssetControllableError::MinterNotFound`] – If the minter is not registered.
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`
    ///   from `stellar_contract_utils::pausable`, OZ fork).
    /// * `HostError: Error(Auth, InvalidAction)` – `minter` authorization fails.
    ///
    /// # Events
    ///
    /// Emitted by the allow asset SAC:
    /// * topics - `["burn", from: Address, asset: Symbol]`
    /// * data - `amount: i128`
    ///
    /// Emitted by the mint asset SAC:
    /// * topics - `["mint", to: Address, asset: Symbol]`
    /// * data - `amount: i128`
    #[when_not_paused]
    fn swap_mint(e: Env, to: Address, amount: i128, minter: Address) {
        minter.require_auth();
        extend_instance_ttl(&e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        // Validate minter is not blocklisted
        let mint_asset = storage::get_mint_asset(&e);
        if !StellarAssetClient::new(&e, &mint_asset).authorized(&minter) {
            panic_with_error!(e, FiatTokenAdminError::InvalidMinterAddress);
        }

        // Swap amount must be greater than 0
        if amount <= 0 {
            panic_with_error!(e, FiatTokenAdminError::InvalidAmount);
        }

        let allow_asset = minter_asset_controllable::get_minter_allow_asset(&e, &minter)
            .unwrap_or_else(|| panic_with_error!(e, FiatTokenAdminError::MinterNotConfigured));

        StellarAssetClient::new(&e, &allow_asset).burn_from(
            &e.current_contract_address(),
            &minter,
            &amount,
        );

        let asset_client = StellarAssetClient::new(&e, &mint_asset);
        asset_client.mint(&to, &amount);
    }
}

impl UpgradeableInternal for FiatTokenAdminContract {
    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }
}
