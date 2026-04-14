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
import type { ResultAsync } from "neverthrow";
import { okAsync } from "neverthrow";
import type { FundingConfig } from "../common/accounts.js";
import { setSacAdmin } from "../common/index.js";
import { deployMintAsset, deployAllowAsset } from "./asset.js";
import { deployFiatTokenAdmin } from "./fiat-token-admin.js";

export interface AllowAssetConfig {
  assetIssuer: Keypair;
  assetCode: string;
}

export interface DeployStablecoinBaseConfig {
  server: rpc.Server;
  networkPassphrase: string;
  funding?: FundingConfig;
  mintAssetIssuer: Keypair;
  mintAssetCode: string;
  allowAssets: AllowAssetConfig[];
  fiatTokenAdmin: {
    deployer: Keypair;
    owner: string;
    pauser: string;
    admin: string;
    minterAssetController: string;
    blocklister: string;
  };
  wasmHash?: string;
  wasmPath?: string;
  salt?: Buffer;
}

export interface DeployedAsset {
  contractId: string;
  assetCode: string;
  assetIssuer: string;
}

export interface DeployStablecoinBaseResult {
  mintAsset: DeployedAsset;
  allowAssets: DeployedAsset[];
  fiatTokenAdmin: { contractId: string; wasmHash: string };
}

/**
 * Deploys the base stablecoin infrastructure: mint asset SAC, allow asset SAC(s),
 * and FiatTokenAdmin contract, then sets the FiatTokenAdmin as the mint asset's
 * SAC admin.
 *
 * Assumptions:
 * - Deploys a vanilla FiatTokenAdmin with no minter configured — consumers are
 *   responsible for calling `configure_minter` with their own minter.
 * - The mint asset SAC admin is set to the FiatTokenAdmin contract.
 * - Allow assets get AuthRequired | AuthRevocable | AuthClawbackEnabled flags; mint asset gets AuthRevocable.
 * - No trustlines, no test accounts, no minter configuration.
 */
export const deployStablecoinBase = (
  config: DeployStablecoinBaseConfig,
): ResultAsync<DeployStablecoinBaseResult, Error> => {
  const { server, networkPassphrase } = config;

  return deployMintAsset({
    server,
    networkPassphrase,
    assetIssuer: config.mintAssetIssuer,
    assetCode: config.mintAssetCode,
    funding: config.funding,
  }).andThen((mintAssetResult: DeployedAsset) => {
    // Deploy allow assets sequentially
    let allowChain: ResultAsync<DeployedAsset[], Error> = okAsync([]);
    for (const aa of config.allowAssets) {
      allowChain = allowChain.andThen((results) =>
        deployAllowAsset({
          server,
          networkPassphrase,
          assetIssuer: aa.assetIssuer,
          assetCode: aa.assetCode,
          funding: config.funding,
        }).map((r) => [...results, r]),
      );
    }

    return allowChain.andThen((allowAssetResults) =>
      deployFiatTokenAdmin({
        server,
        networkPassphrase,
        deployer: config.fiatTokenAdmin.deployer,
        owner: config.fiatTokenAdmin.owner,
        pauser: config.fiatTokenAdmin.pauser,
        admin: config.fiatTokenAdmin.admin,
        minterAssetController: config.fiatTokenAdmin.minterAssetController,
        blocklister: config.fiatTokenAdmin.blocklister,
        mintAssetId: mintAssetResult.contractId,
        wasmHash: config.wasmHash,
        wasmPath: config.wasmPath,
        funding: config.funding,
        salt: config.salt,
      }).andThen((ftaResult) => {
        return setSacAdmin(
          server,
          config.mintAssetIssuer,
          mintAssetResult.contractId,
          ftaResult.contractId,
          networkPassphrase,
        ).map(() => ({
          mintAsset: {
            contractId: mintAssetResult.contractId,
            assetCode: mintAssetResult.assetCode,
            assetIssuer: mintAssetResult.assetIssuer,
          },
          allowAssets: allowAssetResults.map((r) => ({
            contractId: r.contractId,
            assetCode: r.assetCode,
            assetIssuer: r.assetIssuer,
          })),
          fiatTokenAdmin: {
            contractId: ftaResult.contractId,
            wasmHash: ftaResult.wasmHash,
          },
        }));
      }),
    );
  });
};
