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
use soroban_sdk::{Env, IntoVal, TryFromVal, Val};

/// Averaging 5 seconds per ledger, a day is ~17280 ledgers (24 * 3600) / 5 = 17280)
const DAY_IN_LEDGERS: u32 = 17280;

/// Default persistent storage extensions of ~30 days
pub const DEFAULT_EXTEND_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub const DEFAULT_TTL_THRESHOLD: u32 = DEFAULT_EXTEND_AMOUNT - DAY_IN_LEDGERS;

/// Or, persistent storage extensions of ~90 days
pub const EXTENDED_EXTEND_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;
pub const EXTENDED_TTL_THRESHOLD: u32 = EXTENDED_EXTEND_AMOUNT - DAY_IN_LEDGERS;

/// Gets a value from persistent storage and extends its TTL if it exists.
///
/// Returns `Some(value)` if the key exists, `None` otherwise.
pub fn get_and_extend_persistent_ttl<K, V>(
    env: &Env,
    key: &K,
    threshold: u32,
    amount: u32,
) -> Option<V>
where
    K: IntoVal<Env, Val>,
    V: TryFromVal<Env, Val>,
{
    env.storage().persistent().get(key).inspect(|_| {
        env.storage()
            .persistent()
            .extend_ttl(key, threshold, amount)
    })
}

/// Extends the TTL of the contract instance.
///
/// # Arguments
///
/// * `env` - Access to the Soroban environment.
/// * `threshold` - The threshold below which the TTL will be extended.
/// * `amount` - The amount to extend the TTL by.
pub fn extend_instance_ttl(env: &Env, threshold: u32, amount: u32) {
    env.storage().instance().extend_ttl(threshold, amount)
}
