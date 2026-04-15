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

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env};
use stellar_utils::storage::ttl::{
    extend_instance_ttl, get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT,
    DEFAULT_TTL_THRESHOLD,
};

pub const EMPTY_STORAGE_VALUE: i128 = 0;

/// Storage keys for MockStorageContract.
#[derive(Clone)]
#[contracttype]
pub enum MockStorageKey {
    Instance(i128),
    Persistent(i128),
}

/// MockStorageContract is a test contract for testing persistent storage helpers.
#[contract]
pub struct MockStorageContract;

#[contractimpl]
impl MockStorageContract {
    pub fn __constructor(env: Env) {
        let instance_key = MockStorageKey::Instance(0);
        env.storage()
            .instance()
            .set(&instance_key, &EMPTY_STORAGE_VALUE);

        let persistent_key = MockStorageKey::Persistent(0);
        env.storage()
            .persistent()
            .set(&persistent_key, &EMPTY_STORAGE_VALUE);
    }

    pub fn get_instance(env: &Env, index: i128, extend: &bool) -> i128 {
        let key = MockStorageKey::Instance(index);
        if !extend {
            env.storage()
                .instance()
                .get(&key)
                .unwrap_or(EMPTY_STORAGE_VALUE)
        } else {
            extend_instance_ttl(env, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);
            env.storage()
                .instance()
                .get(&key)
                .unwrap_or(EMPTY_STORAGE_VALUE)
        }
    }

    pub fn get_persistent(env: &Env, index: i128, extend: &bool) -> i128 {
        let key = MockStorageKey::Persistent(index);
        if !extend {
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(EMPTY_STORAGE_VALUE)
        } else {
            get_and_extend_persistent_ttl(env, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
                .unwrap_or(EMPTY_STORAGE_VALUE)
        }
    }

    pub fn set_instance(env: &Env, index: i128, value: i128) {
        let key = MockStorageKey::Instance(index);
        env.storage().instance().set(&key, &value);
    }

    pub fn set_persistent(env: &Env, index: i128, value: i128) {
        let key = MockStorageKey::Persistent(index);
        env.storage().persistent().set(&key, &value);
    }
}
