/**
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { NetworkConfigSchema, type NetworkConfig } from "./network-config.schema.js";

export const loadNetworkConfig = (): NetworkConfig => {
  return NetworkConfigSchema.parse({
    rpcUrl: process.env.STELLAR_RPC_URL,
    networkPassphrase: process.env.NETWORK_PASSPHRASE,
    friendbotUrl: process.env.FRIENDBOT_URL || undefined,
    allowHttp: process.env.ALLOW_HTTP === "true",
    funderSecret: process.env.EOA_FUNDER_SECRET || undefined,
    funderBalanceThreshold: process.env.FUNDER_BALANCE_THRESHOLD || undefined,
  });
};
