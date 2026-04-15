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

//! Event assertion extensions for stablecoin-roles.
//!
//! This module provides the `StablecoinEventAssertions` trait which adds
//! stablecoin-specific event assertion methods to `EventAssertion`.
//!
//! # Example
//!
//! ```ignore
//! use event_assertion::EventAssertion;
//! use stablecoin_roles::test_utils::StablecoinEventAssertions;
//!
//! let mut events = EventAssertion::new(&env, contract.address.clone());
//! events.assert_minter_configured(&minter, &asset);
//! ```

use event_assertion::EventAssertion;
use soroban_sdk::{Address, IntoVal, Symbol};

/// Extension trait for stablecoin-specific event assertions.
///
/// Import this trait to get access to minter, blocklister, and other
/// stablecoin event assertion methods on `EventAssertion`.
pub trait StablecoinEventAssertions {
    // Minter asset controllable events
    fn assert_minter_asset_controller_changed(&mut self, expected_controller: &Address);
    fn assert_minter_configured(&mut self, expected_minter: &Address, expected_asset: &Address);
    fn assert_minter_removed(&mut self, expected_minter: &Address);

    // Mint asset events
    fn assert_mint_asset_set(&mut self, expected_mint_asset: &Address);

    // Allow asset controller events
    fn assert_allow_asset_controller_changed(&mut self, expected_controller: &Address);

    // Blocklister events
    fn assert_blocklister_changed(&mut self, expected_blocklister: &Address);

    // Blocklist action events
    fn assert_blocklisted(&mut self, expected_address: &Address);
    fn assert_unblocklisted(&mut self, expected_address: &Address);
}

impl<'a> StablecoinEventAssertions for EventAssertion<'a> {
    /// Asserts that a MinterAssetControllerChanged event was emitted with the expected address.
    fn assert_minter_asset_controller_changed(&mut self, expected_controller: &Address) {
        let event = self.find_event_by_symbol("minter_asset_controller_changed");

        assert!(
            event.is_some(),
            "minter_asset_controller_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "minter_asset_controller_changed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "minter_asset_controller_changed event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "minter_asset_controller_changed"),
            "Event topic should be minter_asset_controller_changed"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_controller = event_data
            .get(Symbol::new(self.env(), "new_controller"))
            .expect("minter_asset_controller_changed event missing new_controller field");
        assert_eq!(
            &event_controller, expected_controller,
            "minter_asset_controller_changed event has wrong address"
        );
    }

    /// Asserts that a MinterConfigured event was emitted with the expected minter and asset.
    fn assert_minter_configured(&mut self, expected_minter: &Address, expected_asset: &Address) {
        let event = self.find_event_by_symbol("minter_configured");

        assert!(
            event.is_some(),
            "minter_configured event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "minter_configured event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "minter_configured event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "minter_configured"),
            "Event topic should be minter_configured"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_minter = event_data
            .get(Symbol::new(self.env(), "minter"))
            .expect("minter_configured event missing minter field");
        assert_eq!(
            &event_minter, expected_minter,
            "minter_configured event has wrong minter address"
        );
        let event_asset = event_data
            .get(Symbol::new(self.env(), "allow_asset"))
            .expect("minter_configured event missing allow_asset field");
        assert_eq!(
            &event_asset, expected_asset,
            "minter_configured event has wrong allow_asset address"
        );
    }

    /// Asserts that a MinterRemoved event was emitted with the expected minter.
    fn assert_minter_removed(&mut self, expected_minter: &Address) {
        let event = self.find_event_by_symbol("minter_removed");

        assert!(
            event.is_some(),
            "minter_removed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "minter_removed event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "minter_removed event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "minter_removed"),
            "Event topic should be minter_removed"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_minter = event_data
            .get(Symbol::new(self.env(), "minter"))
            .expect("minter_removed event missing minter field");
        assert_eq!(
            &event_minter, expected_minter,
            "minter_removed event has wrong minter address"
        );
    }

    /// Asserts that a MintAssetSet event was emitted with the expected address.
    fn assert_mint_asset_set(&mut self, expected_mint_asset: &Address) {
        let event = self.find_event_by_symbol("mint_asset_set");

        assert!(
            event.is_some(),
            "mint_asset_set event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "mint_asset_set event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "mint_asset_set event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "mint_asset_set"),
            "Event topic should be mint_asset_set"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_mint_asset = event_data
            .get(Symbol::new(self.env(), "mint_asset"))
            .expect("mint_asset_set event missing mint_asset field");
        assert_eq!(
            &event_mint_asset, expected_mint_asset,
            "mint_asset_set event has wrong address"
        );
    }

    /// Asserts that an AllowAssetControllerChanged event was emitted with the expected address.
    fn assert_allow_asset_controller_changed(&mut self, expected_controller: &Address) {
        let event = self.find_event_by_symbol("allow_asset_controller_changed");

        assert!(
            event.is_some(),
            "allow_asset_controller_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "allow_asset_controller_changed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "allow_asset_controller_changed event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "allow_asset_controller_changed"),
            "Event topic should be allow_asset_controller_changed"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_controller = event_data
            .get(Symbol::new(self.env(), "new_controller"))
            .expect("allow_asset_controller_changed event missing new_controller field");
        assert_eq!(
            &event_controller, expected_controller,
            "allow_asset_controller_changed event has wrong address"
        );
    }

    /// Asserts that a BlocklisterChanged event was emitted with the expected address.
    fn assert_blocklister_changed(&mut self, expected_blocklister: &Address) {
        let event = self.find_event_by_symbol("blocklister_changed");

        assert!(
            event.is_some(),
            "blocklister_changed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "blocklister_changed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "blocklister_changed event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "blocklister_changed"),
            "Event topic should be blocklister_changed"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_blocklister = event_data
            .get(Symbol::new(self.env(), "new_blocklister"))
            .expect("blocklister_changed event missing new_blocklister field");
        assert_eq!(
            &event_blocklister, expected_blocklister,
            "blocklister_changed event has wrong address"
        );
    }

    /// Asserts that a Blocklisted event was emitted with the expected address.
    fn assert_blocklisted(&mut self, expected_address: &Address) {
        let event = self.find_event_by_symbol("blocklisted");

        assert!(event.is_some(), "blocklisted event not found in event log");

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "blocklisted event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "blocklisted event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "blocklisted"),
            "Event topic should be blocklisted"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_address = event_data
            .get(Symbol::new(self.env(), "address"))
            .expect("blocklisted event missing address field");
        assert_eq!(
            &event_address, expected_address,
            "blocklisted event has wrong address"
        );
    }

    /// Asserts that an Unblocklisted event was emitted with the expected address.
    fn assert_unblocklisted(&mut self, expected_address: &Address) {
        let event = self.find_event_by_symbol("unblocklisted");

        assert!(
            event.is_some(),
            "unblocklisted event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "unblocklisted event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "unblocklisted event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "unblocklisted"),
            "Event topic should be unblocklisted"
        );

        let event_data: soroban_sdk::Map<Symbol, Address> = data.into_val(self.env());
        let event_address = event_data
            .get(Symbol::new(self.env(), "address"))
            .expect("unblocklisted event missing address field");
        assert_eq!(
            &event_address, expected_address,
            "unblocklisted event has wrong address"
        );
    }
}
