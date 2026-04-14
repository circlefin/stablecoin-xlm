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

import { Keypair } from "@stellar/stellar-sdk";
import { createSigner } from "./accounts";

export type ContractClientOptions = {
  contractId: string;
  rpcUrl: string;
  networkPassphrase: string;
  allowHttp: boolean;
  publicKey: string;
  signTransaction: (
    tx: string,
    opts?: { networkPassphrase?: string }
  ) => Promise<{ signedTxXdr: string; signerAddress?: string }>;
};

/**
 * Creates a factory function for instantiating contract clients.
 *
 * @param ClientClass - The contract client class constructor
 * @param contractId - The deployed contract ID
 * @param rpcUrl - The Stellar RPC URL
 * @param networkPassphrase - The network passphrase
 * @returns A function that takes a Keypair and returns a configured client instance
 *
 * @example
 * const createFiatTokenClient = createClientFactory(FiatTokenAdminClient, contractId, rpcUrl, networkPassphrase);
 * const ownerClient = createFiatTokenClient(ownerKeypair);
 */
export const createClientFactory = <T>(
  ClientClass: new (options: ContractClientOptions) => T,
  contractId: string,
  rpcUrl: string,
  networkPassphrase: string
) =>
  (keypair: Keypair): T =>
    new ClientClass({
      contractId,
      rpcUrl,
      networkPassphrase,
      allowHttp: true,
      ...createSigner(keypair, networkPassphrase),
    });
