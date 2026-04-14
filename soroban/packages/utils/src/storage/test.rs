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
use mock_storage::{
    MockStorageContract, MockStorageContractClient, MockStorageKey, EMPTY_STORAGE_VALUE,
};
use soroban_sdk::testutils::storage::{Instance, Persistent};
use soroban_sdk::{testutils::Ledger as _, Env};

use super::ttl::DEFAULT_EXTEND_AMOUNT;

pub const TEST_MIN_PERSISTENT_TTL: u32 = 100;

#[test]
fn test_get_and_extend_persistent_ttl() {
    let env = Env::default();

    // Setup TTL limits for testing
    env.ledger().with_mut(|ledger| {
        ledger.min_persistent_entry_ttl = TEST_MIN_PERSISTENT_TTL;
        ledger.max_entry_ttl = DEFAULT_EXTEND_AMOUNT;
    });

    // Register contract
    let contract_id = env.register(MockStorageContract, ());
    let client = MockStorageContractClient::new(&env, &contract_id);

    // Mock storage settings
    let storage_index: i128 = 0;
    let storage_value: i128 = 42;
    let test_storage_key = MockStorageKey::Persistent(storage_index);
    let extend_ttl = true;

    // TTL expiration was correctly setup
    let initial_ttl = env.as_contract(&contract_id, || {
        env.storage().persistent().get_ttl(&test_storage_key)
    });
    assert_eq!(initial_ttl, TEST_MIN_PERSISTENT_TTL - 1);

    // Initial value is 0
    assert_eq!(
        client.get_persistent(&storage_index, &extend_ttl),
        EMPTY_STORAGE_VALUE
    );

    // Storage value must be updatable
    client.set_persistent(&storage_index, &storage_value);
    assert_eq!(
        client.get_persistent(&storage_index, &extend_ttl),
        storage_value
    );

    // TTL was correctly extended
    let extended_ttl = env.as_contract(&contract_id, || {
        env.storage().persistent().get_ttl(&test_storage_key)
    });
    assert_eq!(extended_ttl, DEFAULT_EXTEND_AMOUNT - 1);
}

#[test]
fn test_get_and_extend_instance_ttl() {
    let env = Env::default();

    // Setup TTL limits for testing
    env.ledger().with_mut(|ledger| {
        ledger.max_entry_ttl = DEFAULT_EXTEND_AMOUNT;
    });

    // Register contract
    let contract_id = env.register(MockStorageContract, ());
    let client = MockStorageContractClient::new(&env, &contract_id);

    // Mock storage settings
    let storage_index: i128 = 0;
    let storage_value: i128 = 42;
    let extend_ttl = true;

    // Initial value is 0
    assert_eq!(
        client.get_instance(&storage_index, &extend_ttl),
        EMPTY_STORAGE_VALUE
    );

    // Storage value must be updatable
    client.set_instance(&storage_index, &storage_value);
    assert_eq!(
        client.get_instance(&storage_index, &extend_ttl),
        storage_value
    );

    // TTL was correctly extended
    let extended_ttl = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
    assert_eq!(extended_ttl, DEFAULT_EXTEND_AMOUNT - 1);
}
