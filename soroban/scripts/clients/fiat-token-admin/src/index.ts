/**
 * Copyright (c) 2026, Circle Internet Group, Inc. All rights reserved.
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

import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}






/**
 * Errors for the Fiat Token Admin contract.
 */
export const FiatTokenAdminError = {
  /**
   * The mint asset has not been set.
   */
  6800: {message:"MintAssetNotSet"},
  /**
   * The mint asset has already been set and cannot be changed.
   */
  6801: {message:"MintAssetAlreadySet"},
  /**
   * The amount must be greater than zero.
   */
  6802: {message:"InvalidAmount"},
  /**
   * The minter is not configured with an asset.
   */
  6803: {message:"MinterNotConfigured"},
  /**
   * The minter address must not be blocklisted
   */
  6804: {message:"InvalidMinterAddress"}
}

export type FiatTokenAdminStorageKey = {tag: "MintAsset", values: void};


export interface FiatTokenAdminContractInitParams {
  admin: string;
  blocklister: string;
  mint_asset: string;
  minter_asset_controller: string;
  owner: string;
  pauser: string;
}



/**
 * Errors for the minter asset controllable module.
 */
export const MinterAssetControllableError = {
  /**
   * The allow asset is invalid.
   */
  6701: {message:"InvalidAllowAsset"},
  /**
   * The minter is not registered.
   */
  6702: {message:"MinterNotFound"}
}


export type MinterAssetControllableStorageKey = {tag: "Minters", values: readonly [string]};



export const ManageableError = {
  7200: {message:"AdminNotSet"},
  7201: {message:"AdminAlreadySet"}
}

/**
 * Storage keys for `Manageable` utility.
 */
export type ManageableStorageKey = {tag: "PendingAdmin", values: void};

/**
 * Errors related to role management operations
 */
export const RoleError = {
  /**
   * The specified role has not been set
   */
  7000: {message:"RoleNotSet"}
}



export const UpgradeableError = {
  /**
   * When migration is attempted but not allowed due to upgrade state.
   */
  1100: {message:"MigrationNotAllowed"}
}



export const MerkleDistributorError = {
  /**
   * The merkle root is not set.
   */
  1300: {message:"RootNotSet"},
  /**
   * The provided index was already claimed.
   */
  1301: {message:"IndexAlreadyClaimed"},
  /**
   * The proof is invalid.
   */
  1302: {message:"InvalidProof"}
}

/**
 * Storage keys for the data associated with `MerkleDistributor`
 */
export type MerkleDistributorStorageKey = {tag: "Root", values: void} | {tag: "Claimed", values: readonly [u32]};

export type Rounding = {tag: "Floor", values: void} | {tag: "Ceil", values: void};

export const SorobanFixedPointError = {
  /**
   * The operation failed because the denominator is 0.
   */
  1500: {message:"ZeroDenominator"},
  /**
   * The operation failed because a phantom overflow occurred.
   */
  1501: {message:"PhantomOverflow"},
  /**
   * The operation failed because the result does not fit in Self.
   */
  1502: {message:"ResultOverflow"}
}

export const CryptoError = {
  /**
   * The merkle proof length is out of bounds.
   */
  1400: {message:"MerkleProofOutOfBounds"},
  /**
   * The index of the leaf is out of bounds.
   */
  1401: {message:"MerkleIndexOutOfBounds"},
  /**
   * No data in hasher state.
   */
  1402: {message:"HasherEmptyState"}
}



export const PausableError = {
  /**
   * The operation failed because the contract is paused.
   */
  1000: {message:"EnforcedPause"},
  /**
   * The operation failed because the contract is not paused.
   */
  1001: {message:"ExpectedPause"}
}

/**
 * Storage key for the pausable state
 */
export type PausableStorageKey = {tag: "Paused", values: void};


/**
 * Storage key for a simple role (single address per role).
 */
export interface RoleKey {
  role: string;
}

export const RoleTransferError = {
  2200: {message:"NoPendingTransfer"},
  2201: {message:"InvalidLiveUntilLedger"},
  2202: {message:"InvalidPendingAccount"}
}





export const AccessControlError = {
  2000: {message:"Unauthorized"},
  2001: {message:"AdminNotSet"},
  2002: {message:"IndexOutOfBounds"},
  2003: {message:"AdminRoleNotFound"},
  2004: {message:"RoleCountIsNotZero"},
  2005: {message:"RoleNotFound"},
  2006: {message:"AdminAlreadySet"},
  2007: {message:"RoleNotHeld"},
  2008: {message:"RoleIsEmpty"}
}




/**
 * Storage key for enumeration of accounts per role.
 */
export interface RoleAccountKey {
  index: u32;
  role: string;
}

/**
 * Storage keys for the data associated with the access control
 */
export type AccessControlStorageKey = {tag: "RoleAccounts", values: readonly [RoleAccountKey]} | {tag: "HasRole", values: readonly [string, string]} | {tag: "RoleAccountsCount", values: readonly [string]} | {tag: "RoleAdmin", values: readonly [string]} | {tag: "Admin", values: void} | {tag: "PendingAdmin", values: void};

export const OwnableError = {
  2100: {message:"OwnerNotSet"},
  2101: {message:"TransferInProgress"},
  2102: {message:"OwnerAlreadySet"}
}




/**
 * Storage keys for `Ownable` utility.
 */
export type OwnableStorageKey = {tag: "Owner", values: void} | {tag: "PendingOwner", values: void};

export interface Client {
  /**
   * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  paused: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash, operator}: {new_wasm_hash: Buffer, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a blocklist transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  blocklist: ({address}: {address: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a swap_mint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Swaps the minter's registered allow asset 1:1 for `mint_asset` by burning `amount` from `minter` and minting `amount` to `to`.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `to` - The recipient.
   * * `amount` - The amount to mint (must be greater than zero).
   * * `minter` - The minter address (must be registered via `configure_minter`).
   * 
   * # Errors
   * 
   * * [`FiatTokenAdminError::InvalidAmount`] – If amount is less than or equal to zero.
   * * [`MinterAssetControllableError::MinterNotFound`] – If the minter is not registered.
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`
   * from `stellar_contract_utils::pausable`, OZ fork).
   * * `HostError: Error(Auth, InvalidAction)` – `minter` authorization fails.
   * 
   * # Events
   * 
   * Emitted by the allow asset SAC:
   * * topics - `["burn", from: Address, asset: Symbol]`
   * * data - `amount: i128`
   * 
   * Emitted by the mint asset SAC:
   * * topics - `["mint", to: Address, asset: Symbol]`
   * * data - `amount: i128`
   */
  swap_mint: ({to, amount, minter}: {to: string, amount: i128, minter: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_pauser transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pauser: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a unblocklist transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unblocklist: ({address}: {address: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a remove_minter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_minter: ({minter}: {minter: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_pauser transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_pauser: ({new_pauser}: {new_pauser: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_mint_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the mint asset address.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The mint asset address.
   */
  get_mint_asset: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a is_blocklisted transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_blocklisted: ({address}: {address: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a transfer_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_admin: ({new_admin, expires_in_ledgers}: {new_admin: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_blocklister transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current blocklister address if set.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * * `Some(Address)` – The blocklister address if set.
   * * `None` – If the blocklister has not been set.
   */
  get_blocklister: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a accept_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_ownership: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a configure_minter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  configure_minter: ({minter, allow_asset}: {minter: string, allow_asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_pending_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_pending_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_ownership: ({new_owner, expires_in_ledgers}: {new_owner: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_blocklister transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Updates the blocklister address.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `blocklister` - The address that will be authorized to blocklist/unblocklist addresses.
   * 
   * # Errors
   * 
   * * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
   * 
   * # Events
   * 
   * * topics - `["blocklister_changed"]`
   * * data - `[new_blocklister: Address]`
   */
  update_blocklister: ({blocklister}: {blocklister: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_minter_allow_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_minter_allow_asset: ({minter}: {minter: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_minter_asset_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_minter_asset_controller: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a update_minter_asset_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_minter_asset_controller: ({new_controller}: {new_controller: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {params}: {params: FiatTokenAdminContractInitParams},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({params}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABQAAAClFdmVudCBlbWl0dGVkIHdoZW4gdGhlIG1pbnQgYXNzZXQgaXMgc2V0LgAAAAAAAAAAAAAMTWludEFzc2V0U2V0AAAAAQAAAA5taW50X2Fzc2V0X3NldAAAAAAAAQAAAAAAAAAKbWludF9hc3NldAAAAAAAEwAAAAAAAAAC",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGJsb2NrbGlzdGVyIGFkZHJlc3MgaXMgY2hhbmdlZC4AAAAAAAAAAAASQmxvY2tsaXN0ZXJDaGFuZ2VkAAAAAAABAAAAE2Jsb2NrbGlzdGVyX2NoYW5nZWQAAAAAAQAAAAAAAAAPbmV3X2Jsb2NrbGlzdGVyAAAAABMAAAAAAAAAAg==",
        "AAAABAAAAClFcnJvcnMgZm9yIHRoZSBGaWF0IFRva2VuIEFkbWluIGNvbnRyYWN0LgAAAAAAAAAAAAATRmlhdFRva2VuQWRtaW5FcnJvcgAAAAAFAAAAIFRoZSBtaW50IGFzc2V0IGhhcyBub3QgYmVlbiBzZXQuAAAAD01pbnRBc3NldE5vdFNldAAAABqQAAAAOlRoZSBtaW50IGFzc2V0IGhhcyBhbHJlYWR5IGJlZW4gc2V0IGFuZCBjYW5ub3QgYmUgY2hhbmdlZC4AAAAAABNNaW50QXNzZXRBbHJlYWR5U2V0AAAAGpEAAAAlVGhlIGFtb3VudCBtdXN0IGJlIGdyZWF0ZXIgdGhhbiB6ZXJvLgAAAAAAAA1JbnZhbGlkQW1vdW50AAAAAAAakgAAACtUaGUgbWludGVyIGlzIG5vdCBjb25maWd1cmVkIHdpdGggYW4gYXNzZXQuAAAAABNNaW50ZXJOb3RDb25maWd1cmVkAAAAGpMAAAAqVGhlIG1pbnRlciBhZGRyZXNzIG11c3Qgbm90IGJlIGJsb2NrbGlzdGVkAAAAAAAUSW52YWxpZE1pbnRlckFkZHJlc3MAABqU",
        "AAAAAgAAAAAAAAAAAAAAGEZpYXRUb2tlbkFkbWluU3RvcmFnZUtleQAAAAEAAAAAAAAAAAAAAAlNaW50QXNzZXQAAAA=",
        "AAAAAAAAAAAAAAAFcGF1c2UAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAGcGF1c2VkAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAHdW5wYXVzZQAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAACAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAJYmxvY2tsaXN0AAAAAAAAAQAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAJZ2V0X2FkbWluAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAJZ2V0X293bmVyAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAA8FTd2FwcyB0aGUgbWludGVyJ3MgcmVnaXN0ZXJlZCBhbGxvdyBhc3NldCAxOjEgZm9yIGBtaW50X2Fzc2V0YCBieSBidXJuaW5nIGBhbW91bnRgIGZyb20gYG1pbnRlcmAgYW5kIG1pbnRpbmcgYGFtb3VudGAgdG8gYHRvYC4KCiMgQXJndW1lbnRzCgoqIGBlYCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KKiBgdG9gIC0gVGhlIHJlY2lwaWVudC4KKiBgYW1vdW50YCAtIFRoZSBhbW91bnQgdG8gbWludCAobXVzdCBiZSBncmVhdGVyIHRoYW4gemVybykuCiogYG1pbnRlcmAgLSBUaGUgbWludGVyIGFkZHJlc3MgKG11c3QgYmUgcmVnaXN0ZXJlZCB2aWEgYGNvbmZpZ3VyZV9taW50ZXJgKS4KCiMgRXJyb3JzCgoqIFtgRmlhdFRva2VuQWRtaW5FcnJvcjo6SW52YWxpZEFtb3VudGBdIOKAkyBJZiBhbW91bnQgaXMgbGVzcyB0aGFuIG9yIGVxdWFsIHRvIHplcm8uCiogW2BNaW50ZXJBc3NldENvbnRyb2xsYWJsZUVycm9yOjpNaW50ZXJOb3RGb3VuZGBdIOKAkyBJZiB0aGUgbWludGVyIGlzIG5vdCByZWdpc3RlcmVkLgoqIGBIb3N0RXJyb3I6IEVycm9yKENvbnRyYWN0LCAjMTAwMClgIOKAkyBDb250cmFjdCBpcyBwYXVzZWQgKGBFbmZvcmNlZFBhdXNlZGAKZnJvbSBgc3RlbGxhcl9jb250cmFjdF91dGlsczo6cGF1c2FibGVgLCBPWiBmb3JrKS4KKiBgSG9zdEVycm9yOiBFcnJvcihBdXRoLCBJbnZhbGlkQWN0aW9uKWAg4oCTIGBtaW50ZXJgIGF1dGhvcml6YXRpb24gZmFpbHMuCgojIEV2ZW50cwoKRW1pdHRlZCBieSB0aGUgYWxsb3cgYXNzZXQgU0FDOgoqIHRvcGljcyAtIGBbImJ1cm4iLCBmcm9tOiBBZGRyZXNzLCBhc3NldDogU3ltYm9sXWAKKiBkYXRhIC0gYGFtb3VudDogaTEyOGAKCkVtaXR0ZWQgYnkgdGhlIG1pbnQgYXNzZXQgU0FDOgoqIHRvcGljcyAtIGBbIm1pbnQiLCB0bzogQWRkcmVzcywgYXNzZXQ6IFN5bWJvbF1gCiogZGF0YSAtIGBhbW91bnQ6IGkxMjhgAAAAAAAACXN3YXBfbWludAAAAAAAAAMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAABm1pbnRlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAKZ2V0X3BhdXNlcgAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAALdW5ibG9ja2xpc3QAAAAAAQAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAA=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABnBhcmFtcwAAAAAH0AAAACBGaWF0VG9rZW5BZG1pbkNvbnRyYWN0SW5pdFBhcmFtcwAAAAA=",
        "AAAAAAAAAAAAAAANcmVtb3ZlX21pbnRlcgAAAAAAAAEAAAAAAAAABm1pbnRlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANdXBkYXRlX3BhdXNlcgAAAAAAAAEAAAAAAAAACm5ld19wYXVzZXIAAAAAABMAAAAA",
        "AAAAAAAAAHxSZXR1cm5zIHRoZSBtaW50IGFzc2V0IGFkZHJlc3MuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCgojIFJldHVybnMKClRoZSBtaW50IGFzc2V0IGFkZHJlc3MuAAAADmdldF9taW50X2Fzc2V0AAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAOaXNfYmxvY2tsaXN0ZWQAAAAAAAEAAAAAAAAAB2FkZHJlc3MAAAAAEwAAAAEAAAAB",
        "AAAAAAAAAAAAAAAOdHJhbnNmZXJfYWRtaW4AAAAAAAIAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAEmV4cGlyZXNfaW5fbGVkZ2VycwAAAAAABAAAAAA=",
        "AAAAAAAAANxSZXR1cm5zIHRoZSBjdXJyZW50IGJsb2NrbGlzdGVyIGFkZHJlc3MgaWYgc2V0LgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoKIyBSZXR1cm5zCgoqIGBTb21lKEFkZHJlc3MpYCDigJMgVGhlIGJsb2NrbGlzdGVyIGFkZHJlc3MgaWYgc2V0LgoqIGBOb25lYCDigJMgSWYgdGhlIGJsb2NrbGlzdGVyIGhhcyBub3QgYmVlbiBzZXQuAAAAD2dldF9ibG9ja2xpc3RlcgAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAQYWNjZXB0X293bmVyc2hpcAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAQY29uZmlndXJlX21pbnRlcgAAAAIAAAAAAAAABm1pbnRlcgAAAAAAEwAAAAAAAAALYWxsb3dfYXNzZXQAAAAAEwAAAAA=",
        "AAAAAQAAAAAAAAAAAAAAIEZpYXRUb2tlbkFkbWluQ29udHJhY3RJbml0UGFyYW1zAAAABgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAtibG9ja2xpc3RlcgAAAAATAAAAAAAAAAptaW50X2Fzc2V0AAAAAAATAAAAAAAAABdtaW50ZXJfYXNzZXRfY29udHJvbGxlcgAAAAATAAAAAAAAAAVvd25lcgAAAAAAABMAAAAAAAAABnBhdXNlcgAAAAAAEw==",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfYWRtaW4AAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfb3duZXIAAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAASdHJhbnNmZXJfb3duZXJzaGlwAAAAAAACAAAAAAAAAAluZXdfb3duZXIAAAAAAAATAAAAAAAAABJleHBpcmVzX2luX2xlZGdlcnMAAAAAAAQAAAAA",
        "AAAAAAAAAW9VcGRhdGVzIHRoZSBibG9ja2xpc3RlciBhZGRyZXNzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBibG9ja2xpc3RlcmAgLSBUaGUgYWRkcmVzcyB0aGF0IHdpbGwgYmUgYXV0aG9yaXplZCB0byBibG9ja2xpc3QvdW5ibG9ja2xpc3QgYWRkcmVzc2VzLgoKIyBFcnJvcnMKCiogYEhvc3RFcnJvcjogRXJyb3IoQXV0aCwgSW52YWxpZEFjdGlvbilgIOKAkyBBdXRob3JpemF0aW9uIGZyb20gdGhlIGNvbnRyYWN0IG93bmVyIGZhaWxzLgoKIyBFdmVudHMKCiogdG9waWNzIC0gYFsiYmxvY2tsaXN0ZXJfY2hhbmdlZCJdYAoqIGRhdGEgLSBgW25ld19ibG9ja2xpc3RlcjogQWRkcmVzc11gAAAAABJ1cGRhdGVfYmxvY2tsaXN0ZXIAAAAAAAEAAAAAAAAAC2Jsb2NrbGlzdGVyAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAWZ2V0X21pbnRlcl9hbGxvd19hc3NldAAAAAAAAQAAAAAAAAAGbWludGVyAAAAAAATAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAbZ2V0X21pbnRlcl9hc3NldF9jb250cm9sbGVyAAAAAAAAAAABAAAD6AAAABM=",
        "AAAAAAAAAAAAAAAedXBkYXRlX21pbnRlcl9hc3NldF9jb250cm9sbGVyAAAAAAABAAAAAAAAAA5uZXdfY29udHJvbGxlcgAAAAAAEwAAAAA=",
        "AAAABQAAACdFdmVudCBlbWl0dGVkIHdoZW4gYSBtaW50ZXIgaXMgcmVtb3ZlZC4AAAAAAAAAAA1NaW50ZXJSZW1vdmVkAAAAAAAAAQAAAA5taW50ZXJfcmVtb3ZlZAAAAAAAAQAAAAAAAAAGbWludGVyAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAADxFdmVudCBlbWl0dGVkIHdoZW4gYSBtaW50ZXIgaXMgYWRkZWQgd2l0aCB0aGVpciBhbGxvdyBhc3NldC4AAAAAAAAAEE1pbnRlckNvbmZpZ3VyZWQAAAABAAAAEW1pbnRlcl9jb25maWd1cmVkAAAAAAAAAgAAAAAAAAAGbWludGVyAAAAAAATAAAAAAAAAAAAAAALYWxsb3dfYXNzZXQAAAAAEwAAAAAAAAAC",
        "AAAABAAAADBFcnJvcnMgZm9yIHRoZSBtaW50ZXIgYXNzZXQgY29udHJvbGxhYmxlIG1vZHVsZS4AAAAAAAAAHE1pbnRlckFzc2V0Q29udHJvbGxhYmxlRXJyb3IAAAACAAAAG1RoZSBhbGxvdyBhc3NldCBpcyBpbnZhbGlkLgAAAAARSW52YWxpZEFsbG93QXNzZXQAAAAAABotAAAAHVRoZSBtaW50ZXIgaXMgbm90IHJlZ2lzdGVyZWQuAAAAAAAADk1pbnRlck5vdEZvdW5kAAAAABou",
        "AAAABQAAADpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIG1pbnRlciBhc3NldCBjb250cm9sbGVyIGlzIGNoYW5nZWQuAAAAAAAAAAAAHE1pbnRlckFzc2V0Q29udHJvbGxlckNoYW5nZWQAAAABAAAAH21pbnRlcl9hc3NldF9jb250cm9sbGVyX2NoYW5nZWQAAAAAAQAAAAAAAAAObmV3X2NvbnRyb2xsZXIAAAAAABMAAAAAAAAAAg==",
        "AAAAAgAAAAAAAAAAAAAAIU1pbnRlckFzc2V0Q29udHJvbGxhYmxlU3RvcmFnZUtleQAAAAAAAAEAAAABAAAAAAAAAAdNaW50ZXJzAAAAAAEAAAAT",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgY29tcGxldGVkLgAAAAAAAAAAAAxBZG1pbkNoYW5nZWQAAAABAAAADWFkbWluX2NoYW5nZWQAAAAAAAACAAAAAAAAAAlvbGRfYWRtaW4AAAAAAAPoAAAAEwAAAAAAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAAg==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgaW5pdGlhdGVkLgAAAAAAAAAAABJBZG1pbkNoYW5nZVN0YXJ0ZWQAAAAAAAEAAAAUYWRtaW5fY2hhbmdlX3N0YXJ0ZWQAAAACAAAAAAAAAAlvbGRfYWRtaW4AAAAAAAPoAAAAEwAAAAAAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAAg==",
        "AAAABAAAAAAAAAAAAAAAD01hbmFnZWFibGVFcnJvcgAAAAACAAAAAAAAAAtBZG1pbk5vdFNldAAAABwgAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAcIQ==",
        "AAAAAgAAACZTdG9yYWdlIGtleXMgZm9yIGBNYW5hZ2VhYmxlYCB1dGlsaXR5LgAAAAAAAAAAABRNYW5hZ2VhYmxlU3RvcmFnZUtleQAAAAEAAAAAAAAAAAAAAAxQZW5kaW5nQWRtaW4=",
        "AAAABAAAACxFcnJvcnMgcmVsYXRlZCB0byByb2xlIG1hbmFnZW1lbnQgb3BlcmF0aW9ucwAAAAAAAAAJUm9sZUVycm9yAAAAAAAAAQAAACNUaGUgc3BlY2lmaWVkIHJvbGUgaGFzIG5vdCBiZWVuIHNldAAAAAAKUm9sZU5vdFNldAAAAAAbWA==",
        "AAAABQAAAClFdmVudCBlbWl0dGVkIHdoZW4gdGhlIHBhdXNlciBpcyBjaGFuZ2VkLgAAAAAAAAAAAAANUGF1c2VyQ2hhbmdlZAAAAAAAAAEAAAAOcGF1c2VyX2NoYW5nZWQAAAAAAAEAAAAAAAAAC25ld19hZGRyZXNzAAAAABMAAAAAAAAAAg==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gdGhlIHJlc2N1ZXIgYWRkcmVzcyBpcyBjaGFuZ2VkLgAAAAAAAAAAAA5SZXNjdWVyQ2hhbmdlZAAAAAAAAQAAAA9yZXNjdWVyX2NoYW5nZWQAAAAAAQAAAAAAAAALbmV3X3Jlc2N1ZXIAAAAAEwAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVhYmxlRXJyb3IAAAABAAAAQVdoZW4gbWlncmF0aW9uIGlzIGF0dGVtcHRlZCBidXQgbm90IGFsbG93ZWQgZHVlIHRvIHVwZ3JhZGUgc3RhdGUuAAAAAAAAE01pZ3JhdGlvbk5vdEFsbG93ZWQAAAAETA==",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIG1lcmtsZSByb290IGlzIHNldC4AAAAAAAAAAAAHU2V0Um9vdAAAAAABAAAACHNldF9yb290AAAAAQAAAAAAAAAEcm9vdAAAAA4AAAAAAAAAAg==",
        "AAAABQAAACdFdmVudCBlbWl0dGVkIHdoZW4gYW4gaW5kZXggaXMgY2xhaW1lZC4AAAAAAAAAAApTZXRDbGFpbWVkAAAAAAABAAAAC3NldF9jbGFpbWVkAAAAAAEAAAAAAAAABWluZGV4AAAAAAAAAAAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAFk1lcmtsZURpc3RyaWJ1dG9yRXJyb3IAAAAAAAMAAAAbVGhlIG1lcmtsZSByb290IGlzIG5vdCBzZXQuAAAAAApSb290Tm90U2V0AAAAAAUUAAAAJ1RoZSBwcm92aWRlZCBpbmRleCB3YXMgYWxyZWFkeSBjbGFpbWVkLgAAAAATSW5kZXhBbHJlYWR5Q2xhaW1lZAAAAAUVAAAAFVRoZSBwcm9vZiBpcyBpbnZhbGlkLgAAAAAAAAxJbnZhbGlkUHJvb2YAAAUW",
        "AAAAAgAAAD1TdG9yYWdlIGtleXMgZm9yIHRoZSBkYXRhIGFzc29jaWF0ZWQgd2l0aCBgTWVya2xlRGlzdHJpYnV0b3JgAAAAAAAAAAAAABtNZXJrbGVEaXN0cmlidXRvclN0b3JhZ2VLZXkAAAAAAgAAAAAAAAAoVGhlIE1lcmtsZSByb290IG9mIHRoZSBkaXN0cmlidXRpb24gdHJlZQAAAARSb290AAAAAQAAACNNYXBzIGFuIGluZGV4IHRvIGl0cyBjbGFpbWVkIHN0YXR1cwAAAAAHQ2xhaW1lZAAAAAABAAAABA==",
        "AAAAAgAAAAAAAAAAAAAACFJvdW5kaW5nAAAAAgAAAAAAAAAAAAAABUZsb29yAAAAAAAAAAAAAAAAAAAEQ2VpbA==",
        "AAAABAAAAAAAAAAAAAAAFlNvcm9iYW5GaXhlZFBvaW50RXJyb3IAAAAAAAMAAAAyVGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgZGVub21pbmF0b3IgaXMgMC4AAAAAAA9aZXJvRGVub21pbmF0b3IAAAAF3AAAADlUaGUgb3BlcmF0aW9uIGZhaWxlZCBiZWNhdXNlIGEgcGhhbnRvbSBvdmVyZmxvdyBvY2N1cnJlZC4AAAAAAAAPUGhhbnRvbU92ZXJmbG93AAAABd0AAAA9VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgcmVzdWx0IGRvZXMgbm90IGZpdCBpbiBTZWxmLgAAAAAAAA5SZXN1bHRPdmVyZmxvdwAAAAAF3g==",
        "AAAABAAAAAAAAAAAAAAAC0NyeXB0b0Vycm9yAAAAAAMAAAApVGhlIG1lcmtsZSBwcm9vZiBsZW5ndGggaXMgb3V0IG9mIGJvdW5kcy4AAAAAAAAWTWVya2xlUHJvb2ZPdXRPZkJvdW5kcwAAAAAFeAAAACdUaGUgaW5kZXggb2YgdGhlIGxlYWYgaXMgb3V0IG9mIGJvdW5kcy4AAAAAFk1lcmtsZUluZGV4T3V0T2ZCb3VuZHMAAAAABXkAAAAYTm8gZGF0YSBpbiBoYXNoZXIgc3RhdGUuAAAAEEhhc2hlckVtcHR5U3RhdGUAAAV6",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAAAAAAAAAAGUGF1c2VkAAAAAAABAAAABnBhdXNlZAAAAAAAAAAAAAI=",
        "AAAABQAAACxFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHVucGF1c2VkLgAAAAAAAAAIVW5wYXVzZWQAAAABAAAACHVucGF1c2VkAAAAAAAAAAI=",
        "AAAABAAAAAAAAAAAAAAADVBhdXNhYmxlRXJyb3IAAAAAAAACAAAANFRoZSBvcGVyYXRpb24gZmFpbGVkIGJlY2F1c2UgdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAANRW5mb3JjZWRQYXVzZQAAAAAAA+gAAAA4VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgY29udHJhY3QgaXMgbm90IHBhdXNlZC4AAAANRXhwZWN0ZWRQYXVzZQAAAAAAA+k=",
        "AAAAAgAAACJTdG9yYWdlIGtleSBmb3IgdGhlIHBhdXNhYmxlIHN0YXRlAAAAAAAAAAAAElBhdXNhYmxlU3RvcmFnZUtleQAAAAAAAQAAAAAAAAAySW5kaWNhdGVzIHdoZXRoZXIgdGhlIGNvbnRyYWN0IGlzIGluIHBhdXNlZCBzdGF0ZS4AAAAAAAZQYXVzZWQAAA==",
        "AAAAAQAAADhTdG9yYWdlIGtleSBmb3IgYSBzaW1wbGUgcm9sZSAoc2luZ2xlIGFkZHJlc3MgcGVyIHJvbGUpLgAAAAAAAAAHUm9sZUtleQAAAAABAAAAAAAAAARyb2xlAAAAEQ==",
        "AAAABAAAABtFcnJvcnMgZm9yIHJvbGUgb3BlcmF0aW9ucy4AAAAAAAAAAAlSb2xlRXJyb3IAAAAAAAABAAAAGlRoZSByb2xlIGhhcyBub3QgYmVlbiBzZXQuAAAAAAAKUm9sZU5vdFNldAAAAAAbWA==",
        "AAAABAAAAAAAAAAAAAAAEVJvbGVUcmFuc2ZlckVycm9yAAAAAAAAAwAAAAAAAAARTm9QZW5kaW5nVHJhbnNmZXIAAAAAAAiYAAAAAAAAABZJbnZhbGlkTGl2ZVVudGlsTGVkZ2VyAAAAAAiZAAAAAAAAABVJbnZhbGlkUGVuZGluZ0FjY291bnQAAAAAAAia",
        "AAAABQAAACVFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGlzIGdyYW50ZWQuAAAAAAAAAAAAAAtSb2xlR3JhbnRlZAAAAAABAAAADHJvbGVfZ3JhbnRlZAAAAAMAAAAAAAAABHJvbGUAAAARAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAQAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAACVFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGlzIHJldm9rZWQuAAAAAAAAAAAAAAtSb2xlUmV2b2tlZAAAAAABAAAADHJvbGVfcmV2b2tlZAAAAAMAAAAAAAAABHJvbGUAAAARAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAQAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAAC9FdmVudCBlbWl0dGVkIHdoZW4gdGhlIGFkbWluIHJvbGUgaXMgcmVub3VuY2VkLgAAAAAAAAAADkFkbWluUmVub3VuY2VkAAAAAAABAAAAD2FkbWluX3Jlbm91bmNlZAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAAAg==",
        "AAAABQAAACtFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGFkbWluIGlzIGNoYW5nZWQuAAAAAAAAAAAQUm9sZUFkbWluQ2hhbmdlZAAAAAEAAAAScm9sZV9hZG1pbl9jaGFuZ2VkAAAAAAADAAAAAAAAAARyb2xlAAAAEQAAAAEAAAAAAAAAE3ByZXZpb3VzX2FkbWluX3JvbGUAAAAAEQAAAAAAAAAAAAAADm5ld19hZG1pbl9yb2xlAAAAAAARAAAAAAAAAAI=",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAH0AAAAAAAAAALQWRtaW5Ob3RTZXQAAAAH0QAAAAAAAAAQSW5kZXhPdXRPZkJvdW5kcwAAB9IAAAAAAAAAEUFkbWluUm9sZU5vdEZvdW5kAAAAAAAH0wAAAAAAAAASUm9sZUNvdW50SXNOb3RaZXJvAAAAAAfUAAAAAAAAAAxSb2xlTm90Rm91bmQAAAfVAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAH1gAAAAAAAAALUm9sZU5vdEhlbGQAAAAH1wAAAAAAAAALUm9sZUlzRW1wdHkAAAAH2A==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgY29tcGxldGVkLgAAAAAAAAAAABZBZG1pblRyYW5zZmVyQ29tcGxldGVkAAAAAAABAAAAGGFkbWluX3RyYW5zZmVyX2NvbXBsZXRlZAAAAAIAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAABAAAAAAAAAA5wcmV2aW91c19hZG1pbgAAAAAAEwAAAAAAAAAC",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgaW5pdGlhdGVkLgAAAAAAAAAAABZBZG1pblRyYW5zZmVySW5pdGlhdGVkAAAAAAABAAAAGGFkbWluX3RyYW5zZmVyX2luaXRpYXRlZAAAAAMAAAAAAAAADWN1cnJlbnRfYWRtaW4AAAAAAAATAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAAAAAAAAAAAEWxpdmVfdW50aWxfbGVkZ2VyAAAAAAAABAAAAAAAAAAC",
        "AAAAAQAAADFTdG9yYWdlIGtleSBmb3IgZW51bWVyYXRpb24gb2YgYWNjb3VudHMgcGVyIHJvbGUuAAAAAAAAAAAAAA5Sb2xlQWNjb3VudEtleQAAAAAAAgAAAAAAAAAFaW5kZXgAAAAAAAAEAAAAAAAAAARyb2xlAAAAEQ==",
        "AAAAAgAAADxTdG9yYWdlIGtleXMgZm9yIHRoZSBkYXRhIGFzc29jaWF0ZWQgd2l0aCB0aGUgYWNjZXNzIGNvbnRyb2wAAAAAAAAAF0FjY2Vzc0NvbnRyb2xTdG9yYWdlS2V5AAAAAAYAAAABAAAAAAAAAAxSb2xlQWNjb3VudHMAAAABAAAH0AAAAA5Sb2xlQWNjb3VudEtleQAAAAAAAQAAAAAAAAAHSGFzUm9sZQAAAAACAAAAEwAAABEAAAABAAAAAAAAABFSb2xlQWNjb3VudHNDb3VudAAAAAAAAAEAAAARAAAAAQAAAAAAAAAJUm9sZUFkbWluAAAAAAAAAQAAABEAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAADFBlbmRpbmdBZG1pbg==",
        "AAAABAAAAAAAAAAAAAAADE93bmFibGVFcnJvcgAAAAMAAAAAAAAAC093bmVyTm90U2V0AAAACDQAAAAAAAAAElRyYW5zZmVySW5Qcm9ncmVzcwAAAAAINQAAAAAAAAAPT3duZXJBbHJlYWR5U2V0AAAACDY=",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGluaXRpYXRlZC4AAAAAAAAAAAART3duZXJzaGlwVHJhbnNmZXIAAAAAAAABAAAAEm93bmVyc2hpcF90cmFuc2ZlcgAAAAAAAwAAAAAAAAAJb2xkX293bmVyAAAAAAAAEwAAAAAAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAAAAABFsaXZlX3VudGlsX2xlZGdlcgAAAAAAAAQAAAAAAAAAAg==",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gb3duZXJzaGlwIGlzIHJlbm91bmNlZC4AAAAAAAAAAAAST3duZXJzaGlwUmVub3VuY2VkAAAAAAABAAAAE293bmVyc2hpcF9yZW5vdW5jZWQAAAAAAQAAAAAAAAAJb2xkX293bmVyAAAAAAAAEwAAAAAAAAAC",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGNvbXBsZXRlZC4AAAAAAAAAAAAaT3duZXJzaGlwVHJhbnNmZXJDb21wbGV0ZWQAAAAAAAEAAAAcb3duZXJzaGlwX3RyYW5zZmVyX2NvbXBsZXRlZAAAAAEAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAg==",
        "AAAAAgAAACNTdG9yYWdlIGtleXMgZm9yIGBPd25hYmxlYCB1dGlsaXR5LgAAAAAAAAAAEU93bmFibGVTdG9yYWdlS2V5AAAAAAAAAgAAAAAAAAAAAAAABU93bmVyAAAAAAAAAAAAAAAAAAAMUGVuZGluZ093bmVy" ]),
      options
    )
  }
  public readonly fromJSON = {
    pause: this.txFromJSON<null>,
        paused: this.txFromJSON<boolean>,
        unpause: this.txFromJSON<null>,
        upgrade: this.txFromJSON<null>,
        blocklist: this.txFromJSON<null>,
        get_admin: this.txFromJSON<Option<string>>,
        get_owner: this.txFromJSON<Option<string>>,
        swap_mint: this.txFromJSON<null>,
        get_pauser: this.txFromJSON<Option<string>>,
        unblocklist: this.txFromJSON<null>,
        accept_admin: this.txFromJSON<null>,
        remove_minter: this.txFromJSON<null>,
        update_pauser: this.txFromJSON<null>,
        get_mint_asset: this.txFromJSON<string>,
        is_blocklisted: this.txFromJSON<boolean>,
        transfer_admin: this.txFromJSON<null>,
        get_blocklister: this.txFromJSON<Option<string>>,
        accept_ownership: this.txFromJSON<null>,
        configure_minter: this.txFromJSON<null>,
        get_pending_admin: this.txFromJSON<Option<string>>,
        get_pending_owner: this.txFromJSON<Option<string>>,
        transfer_ownership: this.txFromJSON<null>,
        update_blocklister: this.txFromJSON<null>,
        get_minter_allow_asset: this.txFromJSON<Option<string>>,
        get_minter_asset_controller: this.txFromJSON<Option<string>>,
        update_minter_asset_controller: this.txFromJSON<null>
  }
}