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

import { Keypair, rpc, TransactionBuilder, Operation, xdr, hash } from "@stellar/stellar-sdk";
import { ResultAsync, okAsync, err } from "neverthrow";
import * as fs from "fs/promises";
import { waitForTransaction } from "./transactions";

export const readWasmFile = (wasmPath: string): ResultAsync<Buffer, Error> =>
  ResultAsync.fromPromise(fs.readFile(wasmPath), (e) => e as Error);

const wasmExistsOnChain = (
  server: rpc.Server,
  wasmHash: Buffer,
): ResultAsync<boolean, Error> =>
  ResultAsync.fromThrowable(
    () => server.getLedgerEntries(
      xdr.LedgerKey.contractCode(
        new xdr.LedgerKeyContractCode({ hash: wasmHash })
      )
    ),
    (e) => e as Error,
  )().map((response) => response.entries !== null && response.entries.length > 0);

export const uploadWasm = (
  server: rpc.Server,
  deployer: Keypair,
  wasmPath: string,
  networkPassphrase: string
): ResultAsync<Buffer, Error> =>
  readWasmFile(wasmPath)
    .andThen((wasmBuffer) => {
      const wasmHash = Buffer.from(hash(wasmBuffer));
      return wasmExistsOnChain(server, wasmHash)
        .andThen((exists) => {
          if (exists) {
            return okAsync(wasmHash);
          }
          return ResultAsync.fromThrowable(
            async () => {
              const account = await server.getAccount(deployer.publicKey());
              const tx = new TransactionBuilder(account, { fee: "10000000", networkPassphrase })
                .addOperation(Operation.invokeHostFunction({
                  func: xdr.HostFunction.hostFunctionTypeUploadContractWasm(wasmBuffer),
                  auth: [],
                }))
                .setTimeout(300)
                .build();

              const prepared = await server.prepareTransaction(tx);
              prepared.sign(deployer);
              return await server.sendTransaction(prepared);
            },
            (e) => e as Error,
          )()
            .andThen((result) =>
              result.status === "ERROR"
                ? err(new Error(`Upload failed: ${result.errorResult?.toXDR("base64")}`))
                : waitForTransaction(server, result.hash).map((tx) =>
                    tx.returnValue ? Buffer.from(tx.returnValue.bytes()) : wasmHash
                  )
            );
        });
    })
    .mapErr((e) => new Error(`Failed to upload WASM: ${e.message}`));
