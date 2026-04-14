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

//! Event assertion extensions for simple-role.
//!
//! This module provides the `SimpleRoleEventAssertions` trait which adds
//! event assertion methods for simple-role events to `EventAssertion`.
//!
//! # Example
//!
//! ```ignore
//! use event_assertion::EventAssertion;
//! use simple_role::test_utils::SimpleRoleEventAssertions;
//!
//! let mut events = EventAssertion::new(&env, contract.address.clone());
//! events.assert_role_set(&new_address);
//! events.assert_role_updated(Some(&previous_address), &new_address);
//! ```

use event_assertion::EventAssertion;
use soroban_sdk::{Address, IntoVal, Symbol, TryIntoVal, Val};

/// Extension trait for simple-role event assertions.
///
/// Import this trait to get access to role_set and role_updated event
/// assertion methods on `EventAssertion`.
pub trait SimpleRoleEventAssertions {
    /// Asserts that a RoleSet event was emitted with the expected new address.
    fn assert_role_set(&mut self, expected_new_address: &Address);

    /// Asserts that a RoleUpdated event was emitted with the expected addresses.
    fn assert_role_updated(
        &mut self,
        expected_previous_address: Option<&Address>,
        expected_new_address: &Address,
    );
}

impl<'a> SimpleRoleEventAssertions for EventAssertion<'a> {
    /// Asserts that a RoleSet event was emitted with the expected new address.
    ///
    /// # Arguments
    ///
    /// * `expected_new_address` - The expected new address in the event
    fn assert_role_set(&mut self, expected_new_address: &Address) {
        let event = self.find_event_by_symbol("role_set");

        assert!(event.is_some(), "role_set event not found in event log");

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "role_set event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "role_set event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "role_set"),
            "Event topic should be role_set"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_new_address = event_data
            .get(Symbol::new(self.env(), "new_address"))
            .expect("role_set event missing new_address field");
        assert_eq!(
            &event_new_address, expected_new_address,
            "role_set event has wrong new_address"
        );
    }

    /// Asserts that a RoleUpdated event was emitted with the expected addresses.
    ///
    /// # Arguments
    ///
    /// * `expected_previous_address` - The expected previous address (None if role was not set)
    /// * `expected_new_address` - The expected new address
    fn assert_role_updated(
        &mut self,
        expected_previous_address: Option<&Address>,
        expected_new_address: &Address,
    ) {
        let event = self.find_event_by_symbol("role_updated");

        assert!(event.is_some(), "role_updated event not found in event log");

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "role_updated event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "role_updated event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "role_updated"),
            "Event topic should be role_updated"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_previous_address: Option<Address> = event_data
            .get(Symbol::new(self.env(), "previous_address"))
            .expect("role_updated event missing previous_address field")
            .try_into_val(self.env())
            .ok();

        match (expected_previous_address, &event_previous_address) {
            (Some(expected), Some(actual)) => {
                assert_eq!(
                    expected, actual,
                    "role_updated event has wrong previous_address"
                );
            }
            (None, None) => (),
            _ => panic!("role_updated event previous_address mismatch"),
        }

        let event_new_address: Address = event_data
            .get(Symbol::new(self.env(), "new_address"))
            .expect("role_updated event missing new_address field")
            .into_val(self.env());
        assert_eq!(
            &event_new_address, expected_new_address,
            "role_updated event has wrong new_address"
        );
    }
}
