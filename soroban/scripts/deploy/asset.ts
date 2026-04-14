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

import { Keypair, rpc, AuthRequiredFlag, AuthRevocableFlag, AuthClawbackEnabledFlag } from "@stellar/stellar-sdk";
import { ResultAsync } from "neverthrow";
import {
  deployStellarAssetContract,
  setIssuerAuthFlags,
} from "../common/index.js";
import type { FundingConfig } from "../common/accounts.js";
import { fundAccountIfConfigured } from "../common/accounts.js";
import type { DeployedAsset } from "./stablecoin-base.js";

export interface DeployAssetConfig {
  server: rpc.Server;
  networkPassphrase: string;
  assetIssuer: Keypair;
  assetCode: string;
  funding?: FundingConfig;
}

export const deployAsset = (config: DeployAssetConfig, authFlags: number): ResultAsync<DeployedAsset, Error> => {
  const { server, networkPassphrase, assetIssuer, assetCode } = config;

  return fundAccountIfConfigured(config.funding, assetIssuer.publicKey())
    .andThen(() => setIssuerAuthFlags(server, assetIssuer, authFlags, networkPassphrase))
    .andThen(() => deployStellarAssetContract(server, assetIssuer, assetCode, networkPassphrase))
    .map((contractId) => ({
      contractId,
      assetCode,
      assetIssuer: assetIssuer.publicKey(),
    }));
};

/** Deploys a mint asset SAC with AuthRevocable flag. */
export const deployMintAsset = (config: DeployAssetConfig): ResultAsync<DeployedAsset, Error> =>
  deployAsset(config, AuthRevocableFlag);

/**
 * Deploys an allow asset SAC with AuthRequired | AuthRevocable | AuthClawbackEnabled flags.
 * AuthRequired ensures the issuer must explicitly authorize each trustline.
 */
export const deployAllowAsset = (config: DeployAssetConfig): ResultAsync<DeployedAsset, Error> =>
  deployAsset(config, AuthRequiredFlag | AuthRevocableFlag | AuthClawbackEnabledFlag);
