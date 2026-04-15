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

use super::bytes::is_zero_bytes;
use soroban_sdk::{BytesN, Env};

#[cfg(feature = "hazmat-address-utils")]
use super::bytes::address_to_bytes32;
#[cfg(feature = "hazmat-address-utils")]
use soroban_sdk::{contract, contractimpl, testutils::Address};

#[test]
fn test_is_zero_bytes_all_zeros() {
    let env = Env::default();
    let zero = BytesN::<32>::from_array(&env, &[0u8; 32]);
    assert!(is_zero_bytes(&zero));
}

#[test]
fn test_is_zero_bytes_single_nonzero_each_position() {
    let env = Env::default();
    for i in 0..32 {
        let mut arr = [0u8; 32];
        arr[i] = 1;
        let bytes = BytesN::<32>::from_array(&env, &arr);
        assert!(!is_zero_bytes(&bytes));
    }
}

#[test]
fn test_is_zero_bytes_various_sizes() {
    let env = Env::default();

    let empty = BytesN::<0>::from_array(&env, &[0u8; 0]);
    assert!(is_zero_bytes(&empty));

    assert!(is_zero_bytes(&BytesN::<1>::from_array(&env, &[0u8; 1])));
    assert!(!is_zero_bytes(&BytesN::<1>::from_array(&env, &[1u8; 1])));

    assert!(is_zero_bytes(&BytesN::<20>::from_array(&env, &[0u8; 20])));
    assert!(!is_zero_bytes(&BytesN::<20>::from_array(&env, &[1u8; 20])));
}

// Minimal contract for testing contract address conversion.
#[cfg(feature = "hazmat-address-utils")]
#[contract]
struct TestContract;

#[cfg(feature = "hazmat-address-utils")]
#[contractimpl]
impl TestContract {}

/// Test convert account Address to BytesN<32>.
///
/// Test ignored if feature "hazmat-address-utils" not compiled.
///
/// # Usage
///
/// * `cargo test --features hazmat-address-utils`
#[cfg(feature = "hazmat-address-utils")]
#[test]
fn test_address_to_bytes32_account() {
    let env = Env::default();

    let address = <soroban_sdk::Address>::generate(&env);
    let bytes = address_to_bytes32(&address);
    assert!(bytes.is_some());
    assert!(!is_zero_bytes(&bytes.unwrap()));
}

/// Test convert contract Address to BytesN<32>.
#[cfg(feature = "hazmat-address-utils")]
#[test]
fn test_address_to_bytes32_contract() {
    let env = Env::default();

    let contract_id = env.register(TestContract, ());
    let bytes = address_to_bytes32(&contract_id);
    assert!(bytes.is_some());
    assert!(!is_zero_bytes(&bytes.unwrap()));
}
