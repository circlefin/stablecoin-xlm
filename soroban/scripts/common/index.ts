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

// Account utilities
export { generateAccount, generateAccounts, createFundedAccount, fundAccount, fundAccounts, fundAccountViaTransfer, fundAccountIfConfigured, createTrustline, mintAsset, clawbackAsset, createSigner } from "./accounts";
export type { FundingConfig } from "./accounts";

// Transaction helpers
export { waitForTransaction, submitTx, submitPreparedTx } from "./transactions";
export type { TxConfig, TxResult } from "./transactions";

// WASM upload
export { uploadWasm, readWasmFile } from "./wasm";

// Contract deployment
export { uploadWasmAndDeployContract, deployContractInstance } from "./deploy";
export type { ContractDeployConfig, ContractInstanceConfig } from "./deploy";

// SAC operations
export { setIssuerAuthFlags, setTrustlineAuthorization, deployStellarAssetContract, setSacAdmin } from "./sac";

// Test container
export { startQuickstartContainer } from "./stellar-container";
export type { StartedQuickstartContainer, QuickstartUrls, QuickstartConfig } from "./stellar-container";

// Client factory
export { createClientFactory } from "./client";
export type { ContractClientOptions } from "./client";

// Salt utility
export { asciiToSalt, SALT_BYTES } from "./salt";

// CLI utilities
export { loadNetworkConfig } from "./network-config";
export type { NetworkConfig } from "./network-config.schema";
export {
  requireEnv, requireArg, requireArgOrEnv,
  requireEoaSecret, requireEoaKeypair, requireEoaPublicKey, requireAddressArg, resolveAddressArg,
  resolveAssetCode, requireAssetIssuerSecret, requireAssetIssuerPublic, requireAssetContractId,
  extractEoas, extractAssetContractIds, extractAssetCodes,
  loadEnvFile, upsertEnvFile, upsertEnvVars, resolveOrCreateEoa, initCli, buildFundingConfig,
} from "./cli-utils";
