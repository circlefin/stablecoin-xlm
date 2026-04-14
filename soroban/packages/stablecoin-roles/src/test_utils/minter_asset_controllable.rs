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

//! Test utilities for the MinterAssetControllable role.

use soroban_sdk::{testutils::MockAuth, Address, Env, IntoVal};

/// Mocks authorization for updating the minter asset controller.
///
/// This function should be called before invoking `update_minter_asset_controller`
/// to authorize the owner to change the controller.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The contract address
/// * `owner` - The owner address (must be authorized)
/// * `new_controller` - The new minter asset controller address
pub fn mock_update_minter_asset_controller_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    new_controller: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_minter_asset_controller",
            args: (new_controller.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for adding a minter.
///
/// This function should be called before invoking `configure_minter`
/// to authorize the minter asset controller to add a minter.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The contract address
/// * `controller` - The minter asset controller address (must be authorized)
/// * `minter` - The minter address to add
/// * `asset` - The asset address to assign to the minter
pub fn mock_configure_minter_auth(
    env: &Env,
    contract_id: &Address,
    controller: &Address,
    minter: &Address,
    asset: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: controller,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "configure_minter",
            args: (minter.clone(), asset.clone()).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for removing a minter.
///
/// This function should be called before invoking `remove_minter`
/// to authorize the minter asset controller to remove a minter.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The contract address
/// * `controller` - The minter asset controller address (must be authorized)
/// * `minter` - The minter address to remove
pub fn mock_remove_minter_auth(
    env: &Env,
    contract_id: &Address,
    controller: &Address,
    minter: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: controller,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "remove_minter",
            args: (minter.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
