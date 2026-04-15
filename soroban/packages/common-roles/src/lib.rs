#![no_std]
/**
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the \"License\");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an \"AS IS\" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
pub mod manageable;
pub mod ownable;
pub mod pausable;
pub mod rescuable;

// Re-export simple-role crate
pub use simple_role;
pub use simple_role_macros::enforce_role_auth;

pub use manageable::*;
pub use ownable::*;
pub use pausable::*;
pub use rescuable::*;

use soroban_sdk::contracterror;

/// Errors related to role management operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RoleError {
    /// The specified role has not been set
    RoleNotSet = 7000,
}

#[cfg(any(test, feature = "testutils"))]
pub mod test_utils;
