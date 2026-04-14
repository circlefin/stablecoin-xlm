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

import { Keypair, rpc } from "@stellar/stellar-sdk";
import { ResultAsync, okAsync } from "neverthrow";
import { Client as FiatTokenAdminClient } from "../clients/fiat-token-admin/src/index.js";
import {
  uploadWasm,
  deployContractInstance,
} from "../common/index.js";
import type { FundingConfig } from "../common/accounts.js";
import { fundAccountIfConfigured } from "../common/accounts.js";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_WASM_PATH = path.resolve(__dirname, "../../target/wasm32v1-none/release/fiat_token_admin.wasm");

export interface DeployFiatTokenAdminConfig {
  server: rpc.Server;
  networkPassphrase: string;
  deployer: Keypair;
  owner: string;
  pauser: string;
  admin: string;
  minterAssetController: string;
  blocklister: string;
  mintAssetId: string;
  wasmHash?: string;
  wasmPath?: string;
  funding?: FundingConfig;
  salt?: Buffer;
}

export interface DeployFiatTokenAdminResult {
  contractId: string;
  wasmHash: string;
  mintAssetId: string;
}

export const deployFiatTokenAdmin = (config: DeployFiatTokenAdminConfig): ResultAsync<DeployFiatTokenAdminResult, Error> => {
  const {
    server,
    networkPassphrase,
    deployer,
    owner,
    pauser,
    admin,
    minterAssetController,
    blocklister,
    mintAssetId,
    wasmPath = DEFAULT_WASM_PATH,
  } = config;

  return fundAccountIfConfigured(config.funding, deployer.publicKey())
    .andThen(() => resolveWasmHash(config, server, deployer, wasmPath, networkPassphrase))
    .andThen((wasmHash) => {
      const params = {
        owner,
        pauser,
        admin,
        minter_asset_controller: minterAssetController,
        blocklister,
        mint_asset: mintAssetId,
      };

      return deployContractInstance(
        { wasmHash, server, networkPassphrase, deployer, salt: config.salt },
        (opts) => FiatTokenAdminClient.deploy({ params }, opts),
      ).map((contractId) => ({
        contractId,
        wasmHash: wasmHash.toString("hex"),
        mintAssetId,
      }));
    });
};

const resolveWasmHash = (
  config: DeployFiatTokenAdminConfig,
  server: rpc.Server,
  deployer: Keypair,
  wasmPath: string,
  networkPassphrase: string,
): ResultAsync<Buffer, Error> => {
  if (config.wasmHash) {
    return okAsync(Buffer.from(config.wasmHash, "hex"));
  }
  return uploadWasm(server, deployer, wasmPath, networkPassphrase);
};
