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

//! Integration tests for simple-role-macros
//!
//! Tests the #[enforce_role_auth] macro with both `e: Env` and `e: &Env` parameter patterns.

#[cfg(test)]
mod tests {
    use simple_role::set_role_unchecked;
    use simple_role_macros::enforce_role_auth;
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, MockAuth, MockAuthInvoke},
        Address, Env, IntoVal,
    };

    const TEST_ROLE: &str = "test_role";

    // =============================================================================
    // Test Contracts
    // =============================================================================

    /// Contract that uses `enforce_role_auth` on impl methods (e: Env - owned)
    #[contract]
    struct OwnedEnvContract;

    #[contractimpl]
    impl OwnedEnvContract {
        pub fn set_role(e: Env, addr: Address) {
            set_role_unchecked(&e, TEST_ROLE, &addr);
        }

        /// Uses `e: Env` (owned) - macro should generate `&e`
        #[enforce_role_auth(TEST_ROLE)]
        pub fn protected_action(e: Env, value: i128) -> i128 {
            value * 2
        }
    }

    // =============================================================================
    // Standalone functions with &Env (reference pattern)
    // =============================================================================

    /// Standalone function with `e: &Env` - macro should NOT add `&`
    #[enforce_role_auth(TEST_ROLE)]
    fn standalone_protected_action(e: &Env, value: i128) -> i128 {
        value * 3
    }

    /// Contract that wraps standalone functions
    #[contract]
    struct RefEnvContract;

    #[contractimpl]
    impl RefEnvContract {
        pub fn set_role(e: Env, addr: Address) {
            set_role_unchecked(&e, TEST_ROLE, &addr);
        }

        pub fn protected_action(e: Env, value: i128) -> i128 {
            standalone_protected_action(&e, value)
        }
    }

    // =============================================================================
    // Tests for `e: Env` (owned) pattern - contract impl methods
    // =============================================================================

    #[test]
    fn test_owned_env_protected_action_succeeds_with_auth() {
        let env = Env::default();
        let contract_id = env.register(OwnedEnvContract, ());
        let client = OwnedEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        env.mock_auths(&[MockAuth {
            address: &role_holder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "protected_action",
                args: (21_i128,).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.protected_action(&21);
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7000")]
    fn test_owned_env_panics_when_role_not_set() {
        let env = Env::default();
        let contract_id = env.register(OwnedEnvContract, ());
        let client = OwnedEnvContractClient::new(&env, &contract_id);

        // Role not set - should panic with RoleNotSet error
        client.protected_action(&21);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_owned_env_fails_without_auth() {
        let env = Env::default();
        let contract_id = env.register(OwnedEnvContract, ());
        let client = OwnedEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        // Role set but no auth provided - should panic
        client.protected_action(&21);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_owned_env_fails_with_wrong_auth() {
        let env = Env::default();
        let contract_id = env.register(OwnedEnvContract, ());
        let client = OwnedEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);
        let wrong_address = Address::generate(&env);

        client.set_role(&role_holder);

        // Auth with wrong address - should panic
        env.mock_auths(&[MockAuth {
            address: &wrong_address,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "protected_action",
                args: (21_i128,).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        client.protected_action(&21);
    }

    // =============================================================================
    // Tests for `e: &Env` (reference) pattern - standalone functions
    // =============================================================================

    #[test]
    fn test_ref_env_protected_action_succeeds_with_auth() {
        let env = Env::default();
        let contract_id = env.register(RefEnvContract, ());
        let client = RefEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        env.mock_auths(&[MockAuth {
            address: &role_holder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "protected_action",
                args: (10_i128,).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.protected_action(&10);
        assert_eq!(result, 30); // Uses standalone fn which multiplies by 3
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7000")]
    fn test_ref_env_panics_when_role_not_set() {
        let env = Env::default();
        let contract_id = env.register(RefEnvContract, ());
        let client = RefEnvContractClient::new(&env, &contract_id);

        // Role not set - should panic with RoleNotSet error
        client.protected_action(&10);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_ref_env_fails_without_auth() {
        let env = Env::default();
        let contract_id = env.register(RefEnvContract, ());
        let client = RefEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        // Role set but no auth provided - should panic
        client.protected_action(&10);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_ref_env_fails_with_wrong_auth() {
        let env = Env::default();
        let contract_id = env.register(RefEnvContract, ());
        let client = RefEnvContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);
        let wrong_address = Address::generate(&env);

        client.set_role(&role_holder);

        // Auth with wrong address - should panic
        env.mock_auths(&[MockAuth {
            address: &wrong_address,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "protected_action",
                args: (10_i128,).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        client.protected_action(&10);
    }

    // =============================================================================
    // Test string literal vs constant role name
    // =============================================================================

    const LITERAL_ROLE: &str = "literal_role";

    #[contract]
    struct LiteralRoleContract;

    #[contractimpl]
    impl LiteralRoleContract {
        pub fn set_role(e: Env, addr: Address) {
            set_role_unchecked(&e, "literal_role", &addr);
        }

        /// Uses string literal instead of constant
        #[enforce_role_auth("literal_role")]
        pub fn literal_protected_action(e: Env) -> i128 {
            123
        }

        /// Uses constant
        #[enforce_role_auth(LITERAL_ROLE)]
        pub fn const_protected_action(e: Env) -> i128 {
            456
        }
    }

    #[test]
    fn test_string_literal_role_succeeds() {
        let env = Env::default();
        let contract_id = env.register(LiteralRoleContract, ());
        let client = LiteralRoleContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        env.mock_auths(&[MockAuth {
            address: &role_holder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "literal_protected_action",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.literal_protected_action();
        assert_eq!(result, 123);
    }

    #[test]
    fn test_const_role_succeeds() {
        let env = Env::default();
        let contract_id = env.register(LiteralRoleContract, ());
        let client = LiteralRoleContractClient::new(&env, &contract_id);
        let role_holder = Address::generate(&env);

        client.set_role(&role_holder);

        env.mock_auths(&[MockAuth {
            address: &role_holder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "const_protected_action",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.const_protected_action();
        assert_eq!(result, 456);
    }
}
