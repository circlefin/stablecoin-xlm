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

//! Test utilities for verifying simple-role implementations.

#[cfg(any(test, feature = "testutils"))]
mod event_assertions;

#[cfg(any(test, feature = "testutils"))]
mod test_contract;

#[cfg(any(test, feature = "testutils"))]
pub use event_assertions::SimpleRoleEventAssertions;

#[cfg(any(test, feature = "testutils"))]
pub use test_contract::{
    emit_role_set, emit_role_updated, mock_enforce_role_auth, mock_enforce_role_auth_for,
    mock_set_role_and_emit_auth, mock_set_role_auth, mock_set_role_emit_prev_auth, string_to_str,
    RoleSet, RoleUpdated, TestContract, TestContractClient, TEST_ROLE,
};
