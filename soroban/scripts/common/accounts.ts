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

import { Keypair, rpc, TransactionBuilder, Asset, Operation } from "@stellar/stellar-sdk";
import type { xdr } from "@stellar/stellar-sdk";
import { ResultAsync, okAsync } from "neverthrow";
import { waitForTransaction, TxResult } from "./transactions";

/**
 * Builds, signs, and submits a single-operation transaction.
 */
const signAndSubmitOperation = (
  server: rpc.Server,
  signer: Keypair,
  networkPassphrase: string,
  operation: xdr.Operation,
): ResultAsync<TxResult, Error> =>
  ResultAsync.fromThrowable(
    async () => {
      const account = await server.getAccount(signer.publicKey());
      const tx = new TransactionBuilder(account, { fee: "1000000", networkPassphrase })
        .addOperation(operation)
        .setTimeout(30)
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
    (e) => e as Error,
  )();

/**
 * Creates a signer configuration for use with generated contract clients.
 * Returns an object with `publicKey` and `signTransaction` that matches
 * the ContractClientOptions interface.
 */
export const createSigner = (keypair: Keypair, networkPassphrase: string) => {
  return {
    publicKey: keypair.publicKey(),
    signTransaction: async (
      tx: string,
      opts?: { networkPassphrase?: string }
    ): Promise<{ signedTxXdr: string; signerAddress?: string }> => {
      const txn = TransactionBuilder.fromXDR(tx, opts?.networkPassphrase ?? networkPassphrase);
      txn.sign(keypair);
      return {
        signedTxXdr: txn.toXDR(),
        signerAddress: keypair.publicKey(),
      };
    },
  };
};

export const generateAccount = (): Keypair => Keypair.random();

export const generateAccounts = <T extends string>(names: T[]): Record<T, Keypair> =>
  Object.fromEntries(names.map((name) => [name, Keypair.random()])) as Record<T, Keypair>;

export const createFundedAccount = (friendbotUrl: string): ResultAsync<Keypair, Error> => {
  const account = Keypair.random();
  return fundAccount(friendbotUrl, account.publicKey()).map(() => account);
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const fundAccount = (
  friendbotUrl: string,
  publicKey: string,
  maxRetries = 5,
  retryDelayMs = 2000
): ResultAsync<void, Error> =>
  ResultAsync.fromThrowable(
    async () => {
      let lastError: Error | undefined;
      for (let attempt = 0; attempt <= maxRetries; attempt++) {
        const response = await fetch(`${friendbotUrl}?addr=${publicKey}`);
        if (response.ok) {
          return;
        }

        const body = await response.text().catch(() => "no body");
        
        // "already funded" is not an error — account exists and has funds
        if (response.status === 400 && body.includes("already funded")) {
          return;
        }

        // Retry on 500 errors (Friendbot not ready)
        if (response.status >= 500 && attempt < maxRetries) {
          console.log(`Friendbot not ready (attempt ${attempt + 1}/${maxRetries + 1}), retrying in ${retryDelayMs}ms...`);
          await sleep(retryDelayMs);
          continue;
        }

        console.error(`Friendbot error for ${publicKey.substring(0, 10)}...:`);
        console.error(`  Status: ${response.status} ${response.statusText}`);
        console.error(`  URL: ${friendbotUrl}?addr=${publicKey}`);
        console.error(`  Body: ${body.substring(0, 500)}`);
        lastError = new Error(`Friendbot failed: ${response.status} ${response.statusText} - ${body.substring(0, 200)}`);
      }
      throw lastError ?? new Error("Friendbot failed after retries");
    },
    (e) => new Error(`Failed to fund account ${publicKey.substring(0, 10)}...: ${(e as Error).message}`)
  )();

export const fundAccounts = (
  friendbotUrl: string,
  accounts: Keypair[]
): ResultAsync<void[], Error> =>
  ResultAsync.combine(accounts.map((a) => fundAccount(friendbotUrl, a.publicKey())));

/**
 * Funds an account via a direct XLM transfer from a funder keypair.
 * - If the account does not exist, uses `createAccount` with `balanceThreshold` as the starting balance.
 * - If the account already exists, skips (idempotent on reruns).
 */
export const fundAccountViaTransfer = (
  server: rpc.Server,
  funder: Keypair,
  destinationPublicKey: string,
  networkPassphrase: string,
  balanceThreshold: string = "1",
): ResultAsync<void, Error> =>
  ResultAsync.fromThrowable(
    async () => {
      // Check if account already exists — skip if so
      try {
        await server.getAccount(destinationPublicKey);
        return;
      } catch (e) {
        if (!(e instanceof Error) || !e.message.includes("Account not found")) throw e;
      }

      const sourceAccount = await server.getAccount(funder.publicKey());
      const tx = new TransactionBuilder(sourceAccount, {
        fee: "1000000",
        networkPassphrase,
      })
        .addOperation(Operation.createAccount({
          destination: destinationPublicKey,
          startingBalance: balanceThreshold,
        }))
        .setTimeout(30)
        .build();

      tx.sign(funder);

      const result = await server.sendTransaction(tx);
      if (result.status === "ERROR") {
        throw new Error(`Fund via transfer failed: ${result.errorResult?.toXDR("base64")}`);
      }

      const txResult = await waitForTransaction(server, result.hash);
      if (txResult.isErr()) throw txResult.error;
    },
    (e) => new Error(`Failed to fund account ${destinationPublicKey.substring(0, 10)}...: ${(e as Error).message}`)
  )();

// --- Unified funding ---

export interface FundingConfig {
  friendbotUrl?: string;
  funder?: {
    keypair: Keypair;
    server: rpc.Server;
    networkPassphrase: string;
    balanceThreshold: string;
  };
}

/**
 * Funds an account using the best available method from the config.
 * Priority: funder EOA transfer > friendbot > no-op.
 * Funder takes precedence so environments that set both (e.g., local dev
 * simulating prod) route through the funder path.
 */
export const fundAccountIfConfigured = (
  funding: FundingConfig | undefined,
  publicKey: string,
): ResultAsync<void, Error> => {
  if (funding?.funder) {
    const { keypair, server, networkPassphrase, balanceThreshold } = funding.funder;
    return fundAccountViaTransfer(server, keypair, publicKey, networkPassphrase, balanceThreshold);
  }
  if (funding?.friendbotUrl) {
    return fundAccount(funding.friendbotUrl, publicKey);
  }
  return okAsync(undefined);
};

/**
 * Creates a trustline for an account to hold a specific asset.
 * Required for SAC authorization checks on classic assets.
 */
export const createTrustline = (
  server: rpc.Server,
  account: Keypair,
  assetCode: string,
  assetIssuer: string,
  networkPassphrase: string,
): ResultAsync<TxResult, Error> =>
  signAndSubmitOperation(server, account, networkPassphrase,
    Operation.changeTrust({ asset: new Asset(assetCode, assetIssuer) }));

/**
 * Mints tokens to an account via classic Stellar payment from the issuer.
 * On Stellar, the issuer can send any amount of their asset - this is effectively minting.
 */
export const mintAsset = (
  server: rpc.Server,
  issuer: Keypair,
  destination: string,
  assetCode: string,
  amount: string,
  networkPassphrase: string,
): ResultAsync<TxResult, Error> =>
  signAndSubmitOperation(server, issuer, networkPassphrase,
    Operation.payment({ destination, asset: new Asset(assetCode, issuer.publicKey()), amount }));

/**
 * Claws back tokens from an account via classic Stellar clawback operation.
 * The issuer must have AuthClawbackEnabledFlag set.
 */
export const clawbackAsset = (
  server: rpc.Server,
  issuer: Keypair,
  from: string,
  assetCode: string,
  amount: string,
  networkPassphrase: string,
): ResultAsync<TxResult, Error> =>
  signAndSubmitOperation(server, issuer, networkPassphrase,
    Operation.clawback({ asset: new Asset(assetCode, issuer.publicKey()), from, amount }));
