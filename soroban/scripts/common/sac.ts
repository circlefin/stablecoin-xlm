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

import { Keypair, rpc, Operation, Address, Asset, nativeToScVal, xdr, AuthFlag } from "@stellar/stellar-sdk";
import { ResultAsync } from "neverthrow";
import { submitTx, submitPreparedTx, TxResult } from "./transactions";

export const setIssuerAuthFlags = (
  server: rpc.Server,
  issuer: Keypair,
  flags: number,
  networkPassphrase: string
): ResultAsync<void, Error> =>
  submitTx(
    { server, signer: issuer, networkPassphrase },
    () => Operation.setOptions({ setFlags: flags as AuthFlag })
  ).map(() => undefined).mapErr((e) => new Error(`Failed to set issuer auth flags: ${e.message}`));

export const setTrustlineAuthorization = (
  server: rpc.Server,
  issuer: Keypair,
  accountId: string,
  assetCode: string,
  authorize: boolean,
  networkPassphrase: string
): ResultAsync<TxResult, Error> =>
  submitTx(
    { server, signer: issuer, networkPassphrase },
    () => Operation.setTrustLineFlags({
      trustor: accountId,
      asset: new Asset(assetCode, issuer.publicKey()),
      flags: { authorized: authorize, authorizedToMaintainLiabilities: false },
    })
  ).mapErr((e) => new Error(`Failed to set trustline authorization: ${e.message}`));

export const deployStellarAssetContract = (
  server: rpc.Server,
  issuer: Keypair,
  assetCode: string,
  networkPassphrase: string
): ResultAsync<string, Error> => {
  const asset = new Asset(assetCode, issuer.publicKey());

  return submitPreparedTx(
    { server, signer: issuer, networkPassphrase },
    () => Operation.invokeHostFunction({
      func: xdr.HostFunction.hostFunctionTypeCreateContract(
        new xdr.CreateContractArgs({
          contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAsset(asset.toXDRObject()),
          executable: xdr.ContractExecutable.contractExecutableStellarAsset(),
        })
      ),
      auth: [],
    }),
    (returnValue) => Address.fromScVal(returnValue).toString()
  ).map((r) => r.value).mapErr((e) => new Error(`Failed to deploy SAC: ${e.message}`));
};

export const setSacAdmin = (
  server: rpc.Server,
  issuer: Keypair,
  assetContractId: string,
  adminAddress: string,
  networkPassphrase: string
): ResultAsync<TxResult, Error> =>
  submitPreparedTx(
    { server, signer: issuer, networkPassphrase },
    () => Operation.invokeContractFunction({
      contract: assetContractId,
      function: "set_admin",
      args: [nativeToScVal(adminAddress, { type: "address" })],
    })
  ).mapErr((e) => new Error(`Failed to set SAC admin: ${e.message}`));
