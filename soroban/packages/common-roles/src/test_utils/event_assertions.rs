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

//! Event assertion extensions for common-roles.
//!
//! This module provides the `CommonEventAssertions` trait which adds
//! event assertion methods for pausable, ownable, rescuable, and manageable
//! events to `EventAssertion`.
//!
//! # Example
//!
//! ```ignore
//! use event_assertion::EventAssertion;
//! use common_roles::test_utils::CommonEventAssertions;
//!
//! let mut events = EventAssertion::new(&env, contract.address.clone());
//! events.assert_paused();
//! events.assert_pauser_changed(&new_pauser);
//! ```

use event_assertion::EventAssertion;
use soroban_sdk::{Address, IntoVal, Symbol, Val};

/// Extension trait for common-roles event assertions.
///
/// Import this trait to get access to pausable, ownable, rescuable, and
/// manageable event assertion methods on `EventAssertion`.
pub trait CommonEventAssertions {
    // Pausable events
    fn assert_pauser_changed(&mut self, expected_pauser: &Address);
    fn assert_paused(&mut self);
    fn assert_unpaused(&mut self);

    // Rescuable events
    fn assert_rescuer_changed(&mut self, expected_rescuer: &Address);

    // Ownable events
    fn assert_ownership_transfer(
        &mut self,
        expected_old_owner: &Address,
        expected_new_owner: &Address,
        expected_live_until_ledger: u32,
    );
    fn assert_ownership_transfer_completed(&mut self, expected_owner: &Address);

    // Manageable events
    fn assert_admin_change_started(
        &mut self,
        expected_old_admin: Option<&Address>,
        expected_new_admin: &Address,
    );
    fn assert_admin_changed(
        &mut self,
        expected_old_admin: Option<&Address>,
        expected_new_admin: &Address,
    );
}

impl<'a> CommonEventAssertions for EventAssertion<'a> {
    /// Asserts that a PauserChanged event was emitted with the expected address.
    fn assert_pauser_changed(&mut self, expected_pauser: &Address) {
        let event = self.find_event_by_symbol("pauser_changed");

        assert!(
            event.is_some(),
            "pauser_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "pauser_changed event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "pauser_changed event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "pauser_changed"),
            "Event topic should be pauser_changed"
        );

        let event_map: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        assert_eq!(
            event_map.len(),
            1,
            "pauser_changed event should have 1 field"
        );

        let event_pauser = event_map
            .get(Symbol::new(self.env(), "new_address"))
            .expect("pauser_changed event missing new_address field");
        assert_eq!(
            &event_pauser, expected_pauser,
            "pauser_changed event has wrong address"
        );
    }

    /// Asserts that a paused event was emitted.
    fn assert_paused(&mut self) {
        let event = self.find_event_by_symbol("paused");

        assert!(event.is_some(), "paused event not found in event log");

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "paused event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "paused event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "paused"),
            "Event topic should be 'paused'"
        );
    }

    /// Asserts that an unpaused event was emitted.
    fn assert_unpaused(&mut self) {
        let event = self.find_event_by_symbol("unpaused");

        assert!(event.is_some(), "unpaused event not found in event log");

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "unpaused event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "unpaused event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "unpaused"),
            "Event topic should be 'unpaused'"
        );
    }

    /// Asserts that a RescuerChanged event was emitted with the expected address.
    fn assert_rescuer_changed(&mut self, expected_rescuer: &Address) {
        let event = self.find_event_by_symbol("rescuer_changed");

        assert!(
            event.is_some(),
            "rescuer_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "rescuer_changed event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "rescuer_changed event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "rescuer_changed"),
            "Event topic should be rescuer_changed"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        assert_eq!(
            event_data.len(),
            1,
            "rescuer_changed event should have 1 field"
        );

        let event_rescuer = event_data
            .get(Symbol::new(self.env(), "new_rescuer"))
            .expect("rescuer_changed event missing new_rescuer field");
        assert_eq!(
            &event_rescuer, expected_rescuer,
            "rescuer_changed event has wrong address"
        );
    }

    /// Asserts that an ownership_transfer event was emitted with the expected addresses.
    fn assert_ownership_transfer(
        &mut self,
        expected_old_owner: &Address,
        expected_new_owner: &Address,
        expected_live_until_ledger: u32,
    ) {
        let event = self.find_event_by_symbol("ownership_transfer");

        assert!(
            event.is_some(),
            "ownership_transfer event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "ownership_transfer event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "ownership_transfer event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "ownership_transfer"),
            "Event topic should be 'ownership_transfer'"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        assert_eq!(
            event_data.len(),
            3,
            "ownership_transfer event should have 3 fields"
        );

        let event_old_owner: Address = event_data
            .get(Symbol::new(self.env(), "old_owner"))
            .expect("ownership_transfer event missing old_owner field")
            .into_val(self.env());
        assert_eq!(
            &event_old_owner, expected_old_owner,
            "ownership_transfer event has wrong old_owner"
        );

        let event_new_owner: Address = event_data
            .get(Symbol::new(self.env(), "new_owner"))
            .expect("ownership_transfer event missing new_owner field")
            .into_val(self.env());
        assert_eq!(
            &event_new_owner, expected_new_owner,
            "ownership_transfer event has wrong address"
        );

        let event_live_until_ledger: u32 = event_data
            .get(Symbol::new(self.env(), "live_until_ledger"))
            .expect("ownership_transfer event missing live_until_ledger field")
            .into_val(self.env());
        assert_eq!(
            event_live_until_ledger, expected_live_until_ledger,
            "ownership_transfer event has wrong live_until_ledger"
        );
    }

    /// Asserts that an ownership_transfer_completed event was emitted with the expected address.
    fn assert_ownership_transfer_completed(&mut self, expected_owner: &Address) {
        let event = self.find_event_by_symbol("ownership_transfer_completed");

        assert!(
            event.is_some(),
            "ownership_transfer_completed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "ownership_transfer_completed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "ownership_transfer_completed event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "ownership_transfer_completed"),
            "Event topic should be 'ownership_transfer_completed'"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        assert_eq!(
            event_data.len(),
            1,
            "ownership_transfer_completed event should have 1 field"
        );

        let event_owner = event_data
            .get(Symbol::new(self.env(), "new_owner"))
            .expect("ownership_transfer_completed event missing new_owner field");
        assert_eq!(
            &event_owner, expected_owner,
            "ownership_transfer_completed event has wrong address"
        );
    }

    #[allow(clippy::manual_map)]
    /// Asserts that an admin_change_started event was emitted with the expected addresses.
    fn assert_admin_change_started(
        &mut self,
        expected_old_admin: Option<&Address>,
        expected_new_admin: &Address,
    ) {
        let event = self.find_event_by_symbol("admin_change_started");

        assert!(
            event.is_some(),
            "admin_change_started event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "admin_change_started event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "admin_change_started event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "admin_change_started"),
            "Event topic should be admin_change_started"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        assert_eq!(
            event_data.len(),
            2,
            "admin_change_started event should have 2 fields"
        );

        let event_old_admin: Option<Address> = event_data
            .get(Symbol::new(self.env(), "old_admin"))
            .expect("admin_change_started event missing old_admin field")
            .into_val(self.env());
        let event_old_admin = if let Some(admin_address) = event_old_admin {
            Some(&admin_address.clone())
        } else {
            None
        };
        assert_eq!(
            event_old_admin, expected_old_admin,
            "admin_change_started event has wrong address"
        );

        let event_new_admin: Address = event_data
            .get(Symbol::new(self.env(), "new_admin"))
            .expect("admin_change_started event missing new_admin field")
            .into_val(self.env());
        assert_eq!(
            &event_new_admin, expected_new_admin,
            "admin_change_started event has wrong new_admin"
        );
    }

    #[allow(clippy::manual_map)]
    /// Asserts that an admin_changed event was emitted with the expected address.
    fn assert_admin_changed(
        &mut self,
        expected_old_admin: Option<&Address>,
        expected_new_admin: &Address,
    ) {
        let event = self.find_event_by_symbol("admin_changed");

        assert!(
            event.is_some(),
            "admin_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "admin_changed event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "admin_changed event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "admin_changed"),
            "Event topic should be admin_changed"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        assert_eq!(
            event_data.len(),
            2,
            "admin_changed event should have 2 fields"
        );

        let event_old_admin: Option<Address> = event_data
            .get(Symbol::new(self.env(), "old_admin"))
            .expect("admin_changed event missing old_admin field")
            .into_val(self.env());
        let event_old_admin = if let Some(admin_address) = event_old_admin {
            Some(&admin_address.clone())
        } else {
            None
        };
        assert_eq!(
            event_old_admin, expected_old_admin,
            "admin_changed event has wrong old_admin"
        );

        let event_new_admin: Address = event_data
            .get(Symbol::new(self.env(), "new_admin"))
            .expect("admin_changed event missing new_admin field")
            .into_val(self.env());
        assert_eq!(
            &event_new_admin, expected_new_admin,
            "admin_changed event has wrong new_admin"
        );
    }
}
