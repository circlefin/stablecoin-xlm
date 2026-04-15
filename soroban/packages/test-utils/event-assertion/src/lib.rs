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

#![cfg_attr(target_family = "wasm", no_std)]

//! Core event assertion infrastructure for Soroban contract testing.
//!
//! This crate provides the base `EventAssertion` struct and core utilities for
//! asserting on contract events. Domain-specific assertions are provided by
//! extension traits in their respective crates:
//!
//! - `common-roles`: Pausable, Ownable, Rescuable event assertions
//! - `stablecoin-roles`: Stablecoin-specific event assertions
//!
//! # Example
//!
//! ```ignore
//! use event_assertion::EventAssertion;
//! use common_roles::test_utils::CommonEventAssertions;
//!
//! let mut events = EventAssertion::new(&env, contract.address.clone());
//! events.assert_paused();  // From CommonEventAssertions trait
//! events.assert_event_count(1);
//! ```

// Adapted from OpenZeppelin's event-assertion helper:
// https://github.com/OpenZeppelin/stellar-contracts/blob/b4597b16962c7bddecb99ea16b2341371806d681/packages/test-utils/event-assertion/src/lib.rs

#[cfg(not(target_family = "wasm"))]
mod implementation {
    use soroban_sdk::{symbol_short, testutils::Events, Address, Env, IntoVal, Symbol, Val, Vec};

    /// Helper for asserting on contract events.
    ///
    /// This struct provides core infrastructure for event assertions.
    /// Domain-specific assertions are added via extension traits.
    pub struct EventAssertion<'a> {
        env: &'a Env,
        contract: Address,
    }

    impl<'a> EventAssertion<'a> {
        /// Creates a new EventAssertion for the given contract.
        pub fn new(env: &'a Env, contract: Address) -> Self {
            Self { env, contract }
        }

        /// Returns a reference to the environment.
        pub fn env(&self) -> &Env {
            self.env
        }

        /// Returns a reference to the contract address.
        pub fn contract(&self) -> &Address {
            &self.contract
        }

        /// Asserts the total number of events emitted.
        pub fn assert_event_count(&self, expected: usize) {
            let events = self.env.events().all();
            assert_eq!(
                events.len() as usize,
                expected,
                "Expected {} events, found {}",
                expected,
                events.len()
            );
        }

        /// Returns all events matching the given topic symbol name.
        pub fn find_all_events_by_symbol(
            &self,
            symbol_name: &str,
        ) -> std::vec::Vec<(Address, Vec<Val>, Val)> {
            let target_symbol = match symbol_name {
                "transfer" => symbol_short!("transfer"),
                "mint" => symbol_short!("mint"),
                "burn" => symbol_short!("burn"),
                "approve" => symbol_short!("approve"),
                _ => Symbol::new(self.env, symbol_name),
            };

            let events = self.env.events().all();
            let mut matching = std::vec::Vec::new();

            for event in events.iter() {
                let (contract, topics, data) = event;
                let topics_clone = topics.clone();

                if let Some(first_topic) = topics_clone.first() {
                    let topic_symbol: Symbol = first_topic.into_val(self.env);

                    if topic_symbol == target_symbol {
                        matching.push((contract.clone(), topics_clone, data));
                    }
                }
            }
            matching
        }

        /// Finds an event by its topic symbol name (returns first match).
        pub fn find_event_by_symbol(&self, symbol_name: &str) -> Option<(Address, Vec<Val>, Val)> {
            self.find_all_events_by_symbol(symbol_name)
                .into_iter()
                .next()
        }

        /// Asserts that a Stellar Asset Contract burn event was emitted.
        ///
        /// # Arguments
        ///
        /// * `from` - The address burning the tokens
        /// * `amount` - The amount burned
        ///
        /// # Notes
        ///
        /// SAC burn events have the structure:
        /// * topics - `["burn", from: Address, asset_id: String]`
        /// * data - `i128` (the amount)
        pub fn assert_sac_burn(&mut self, from: &Address, amount: i128) {
            let event = self.find_event_by_symbol("burn");

            assert!(event.is_some(), "Burn event not found in event log");

            let (contract, topics, data) = event.unwrap();
            assert_eq!(contract, self.contract, "Burn event from wrong contract");

            assert_eq!(topics.len(), 3, "Burn event should have 3 topics");

            let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env);
            assert_eq!(
                topic_symbol,
                symbol_short!("burn"),
                "Event topic should be burn"
            );

            let event_from: Address = topics.get_unchecked(1).into_val(self.env);

            assert_eq!(&event_from, from, "Burn event has wrong from address");

            let event_amount: i128 = data.into_val(self.env);
            assert_eq!(event_amount, amount, "Burn event has wrong amount");
        }

        /// Asserts that a Stellar Asset Contract mint event was emitted.
        ///
        /// # Arguments
        ///
        /// * `to` - The address receiving the minted tokens
        /// * `amount` - The amount minted
        ///
        /// # Notes
        ///
        /// SAC mint events have the structure:
        /// * topics - `["mint", to: Address, asset_id: String]`
        /// * data - `i128` (the amount)
        pub fn assert_sac_mint(&mut self, to: &Address, amount: i128) {
            let event = self.find_event_by_symbol("mint");

            assert!(event.is_some(), "Mint event not found in event log");

            let (contract, topics, data) = event.unwrap();
            assert_eq!(contract, self.contract, "Mint event from wrong contract");

            assert_eq!(topics.len(), 3, "Mint event should have 3 topics");

            let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env);
            assert_eq!(
                topic_symbol,
                symbol_short!("mint"),
                "Event topic should be mint"
            );

            let event_to: Address = topics.get_unchecked(1).into_val(self.env);

            assert_eq!(&event_to, to, "Mint event has wrong to address");

            let event_amount: i128 = data.into_val(self.env);
            assert_eq!(event_amount, amount, "Mint event has wrong amount");
        }

        /// Asserts that a Stellar Asset Contract set_authorized event was emitted.
        ///
        /// # Arguments
        ///
        /// * `id` - The address being (de-)authorized
        /// * `authorize` - Whether the address was authorized or deauthorized
        ///
        /// # Notes
        ///
        /// SAC set_authorized events have the structure:
        /// * topics - `["set_authorized", id: Address, admin: Address]`
        /// * data - `bool` (the authorize flag)
        pub fn assert_sac_set_authorized(&mut self, id: &Address, authorize: bool) {
            let event = self.find_event_by_symbol("set_authorized");

            assert!(
                event.is_some(),
                "set_authorized event not found in event log"
            );

            let (contract, topics, data) = event.unwrap();
            assert_eq!(
                contract, self.contract,
                "set_authorized event from wrong contract"
            );

            assert_eq!(topics.len(), 3, "set_authorized event should have 3 topics");

            let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env);
            assert_eq!(
                topic_symbol,
                Symbol::new(self.env, "set_authorized"),
                "Event topic should be set_authorized"
            );

            let event_id: Address = topics.get_unchecked(1).into_val(self.env);
            assert_eq!(&event_id, id, "set_authorized event has wrong id address");

            let event_authorize: bool = data.into_val(self.env);
            assert_eq!(
                event_authorize, authorize,
                "set_authorized event has wrong authorize value"
            );
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub use implementation::*;
