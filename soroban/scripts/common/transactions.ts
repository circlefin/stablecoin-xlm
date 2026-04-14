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

import { Keypair, rpc, TransactionBuilder, Transaction, xdr } from "@stellar/stellar-sdk";
import { ResultAsync } from "neverthrow";

export function waitForTransaction(
  server: rpc.Server,
  hash: string
): ResultAsync<rpc.Api.GetSuccessfulTransactionResponse, Error> {
  return ResultAsync.fromThrowable(
    async () => {
      let status = await server.getTransaction(hash);
      let attempts = 0;
      while (status.status === "NOT_FOUND" && attempts < 60) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        status = await server.getTransaction(hash);
        attempts++;
      }

      if (status.status !== "SUCCESS") {
        const failedStatus = status as rpc.Api.GetFailedTransactionResponse;
        const resultXdr = failedStatus.resultXdr?.toXDR("base64") ?? "unknown";
        const resultMetaXdr = failedStatus.resultMetaXdr?.toXDR("base64") ?? "unknown";
        throw new Error(
          `Transaction ${hash} failed (${status.status}).\n` +
          `  resultXdr: ${resultXdr}\n` +
          `  resultMetaXdr: ${resultMetaXdr}`
        );
      }

      return status as rpc.Api.GetSuccessfulTransactionResponse;
    },
    (e) => e as Error
  )();
}

export interface TxConfig {
  server: rpc.Server;
  signer: Keypair;
  networkPassphrase: string;
  fee?: string;
  timeout?: number;
}

export interface TxResult {
  txHash: string;
  ledger: number;
}

export const submitTx = (
  config: TxConfig,
  buildOp: () => xdr.Operation
): ResultAsync<TxResult, Error> => {
  const { server, signer, networkPassphrase, fee = "1000000", timeout = 30 } = config;

  return ResultAsync.fromThrowable(
    async () => {
      const account = await server.getAccount(signer.publicKey());
      const tx = new TransactionBuilder(account, { fee, networkPassphrase })
        .addOperation(buildOp())
        .setTimeout(timeout)
        .build();

      tx.sign(signer);
      const result = await server.sendTransaction(tx);
      if (result.status === "ERROR") {
        throw new Error(`Transaction failed: ${result.errorResult?.toXDR("base64")}`);
      }

      const txResult = await waitForTransaction(server, result.hash);
      if (txResult.isErr()) throw txResult.error;
      return { txHash: result.hash, ledger: txResult.value.ledger };
    },
    (e) => e as Error
  )();
};

export function submitPreparedTx(config: TxConfig, buildOp: () => xdr.Operation): ResultAsync<TxResult, Error>;
export function submitPreparedTx<T>(config: TxConfig, buildOp: () => xdr.Operation, extractResult: (rv: xdr.ScVal) => T): ResultAsync<TxResult & { value: T }, Error>;
export function submitPreparedTx<T>(
  config: TxConfig,
  buildOp: () => xdr.Operation,
  extractResult?: (rv: xdr.ScVal) => T
): ResultAsync<TxResult | (TxResult & { value: T }), Error> {
  const { server, signer, networkPassphrase, fee = "10000000", timeout = 300 } = config;

  return ResultAsync.fromThrowable(
    async () => {
      const account = await server.getAccount(signer.publicKey());
      const tx = new TransactionBuilder(account, { fee, networkPassphrase })
        .addOperation(buildOp())
        .setTimeout(timeout)
        .build();

      const prepared = await server.prepareTransaction(tx);
      (prepared as Transaction).sign(signer);
      const result = await server.sendTransaction(prepared);

      if (result.status === "ERROR") {
        throw new Error(`Transaction failed: ${result.errorResult?.toXDR("base64")}`);
      }

      const txResult = await waitForTransaction(server, result.hash);
      if (txResult.isErr()) throw txResult.error;

      const base: TxResult = { txHash: result.hash, ledger: txResult.value.ledger };

      if (extractResult && txResult.value.returnValue) {
        return { ...base, value: extractResult(txResult.value.returnValue) };
      }

      return base;
    },
    (e) => e as Error
  )();
}
