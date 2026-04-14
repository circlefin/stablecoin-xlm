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

import { randomBytes } from "node:crypto";
import { Keypair, rpc } from "@stellar/stellar-sdk";
import { ResultAsync } from "neverthrow";
import { uploadWasm } from "./wasm";
import { createSigner } from "./accounts";

export interface ContractInstanceConfig {
  wasmHash: Buffer;
  server: rpc.Server;
  networkPassphrase: string;
  deployer: Keypair;
  salt?: Buffer;
  maxRetries?: number;
  retryDelayMs?: number;
}

export interface ContractDeployConfig extends Omit<ContractInstanceConfig, "wasmHash"> {
  wasmPath: string;
}

type DeployResult = Promise<{ signAndSend: () => Promise<{ result: { options: { contractId: string } } }> }>;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type DeployFn = (options: any) => DeployResult;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const isRetryableError = (error: Error): boolean =>
  error.message.includes("TRY_AGAIN_LATER") || error.message.includes("try again");

/**
 * Deploys a contract instance from an already-uploaded WASM hash.
 * Includes retry logic for transient network errors.
 */
export const deployContractInstance = (
  config: ContractInstanceConfig,
  deploy: DeployFn
): ResultAsync<string, Error> => {
  const {
    wasmHash,
    server,
    networkPassphrase,
    deployer,
    salt = randomBytes(32),
    maxRetries = 3,
    retryDelayMs = 2000,
  } = config;

  return ResultAsync.fromThrowable(
    async () => {
      // The generated SDK client deploy() expects rpcUrl/allowHttp in its options
      const rpcUrl = server.serverURL.toString();
      const options = {
        wasmHash,
        rpcUrl,
        networkPassphrase,
        allowHttp: rpcUrl.startsWith("http://"),
        salt,
        ...createSigner(deployer, networkPassphrase),
      };

      let lastError: Error | undefined;
      for (let attempt = 0; attempt <= maxRetries; attempt++) {
        try {
          const deployTx = await deploy(options);
          const { result: client } = await deployTx.signAndSend();
          return client.options.contractId;
        } catch (e) {
          lastError = e as Error;
          if (attempt < maxRetries && isRetryableError(lastError)) {
            await sleep(retryDelayMs * (attempt + 1));
            continue;
          }
          throw lastError;
        }
      }
      throw lastError;
    },
    (e) => new Error(`Failed to deploy contract instance: ${(e as Error).message}`)
  )();
};

/**
 * Uploads WASM and deploys a contract in one operation.
 * Use deployContractInstance if you need to deploy multiple instances from the same WASM.
 */
export const uploadWasmAndDeployContract = (
  config: ContractDeployConfig,
  deploy: DeployFn
): ResultAsync<string, Error> => {
  const { wasmPath, server, deployer, networkPassphrase, ...instanceConfig } = config;

  return uploadWasm(server, deployer, wasmPath, networkPassphrase).andThen((wasmHash) =>
    deployContractInstance({ wasmHash, server, deployer, networkPassphrase, ...instanceConfig }, deploy)
  );
};
