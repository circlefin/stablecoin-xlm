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
//! Byte manipulation utilities
//!
//! This module provides common functions for serializing and deserializing
//! values to/from bytes in big-endian format, compatible with EVM encoding.

use soroban_sdk::BytesN;

#[cfg(feature = "hazmat-address-utils")]
use soroban_sdk::{address_payload::AddressPayload, Address};

/// Returns true if the provided `BytesN<N>` is all zeros.
///
/// # Arguments
///
/// * `bytes` - The bytes to check.
///
/// # Returns
///
/// `true` if all bytes are zero, `false` otherwise.
///
/// # Example
///
/// ```ignore
/// let zero = BytesN::<32>::from_array(&env, &[0u8; 32]);
/// assert!(is_zero_bytes(&zero));
///
/// let non_zero = BytesN::<32>::from_array(&env, &[1u8; 32]);
/// assert!(!is_zero_bytes(&non_zero));
/// ```
pub fn is_zero_bytes<const N: usize>(bytes: &BytesN<N>) -> bool {
    let zero = BytesN::<N>::from_array(bytes.env(), &[0u8; N]);
    bytes == &zero
}

/// Converts an Address to a BytesN<32> representation.
///
/// Extracts the underlying 32-byte identifier from a Soroban Address,
/// handling both account (Ed25519 public key) and contract addresses.
///
/// # Arguments
///
/// * `address` - The address to convert.
///
/// # Returns
///
/// `Some(BytesN<32>)` containing the address bytes, or `None` if the
/// address type is not recognized.
///
/// # Example
///
/// ```ignore
/// let bytes = address_to_bytes32(&my_address)
///     .unwrap_or_else(|| panic_with_error!(e, MyError::AddressConversionFailed));
/// ```
#[cfg(feature = "hazmat-address-utils")]
pub fn address_to_bytes32(address: &Address) -> Option<BytesN<32>> {
    let payload = address.to_payload()?;
    Some(match payload {
        AddressPayload::AccountIdPublicKeyEd25519(public_key) => public_key,
        AddressPayload::ContractIdHash(contract_id) => contract_id,
    })
}
