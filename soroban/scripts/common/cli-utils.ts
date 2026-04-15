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

import * as fs from "node:fs";
import { parseArgs, type ParseArgsConfig } from "node:util";
import { Keypair, rpc } from "@stellar/stellar-sdk";
import { loadNetworkConfig } from "./network-config.js";
import type { NetworkConfig } from "./network-config.schema.js";
import { fundAccountIfConfigured } from "./accounts.js";
export type { FundingConfig } from "./accounts.js";

type ParsedValues<T extends ParseArgsConfig["options"]> = {
  [K in keyof T]: T[K] extends { type: "boolean" } ? boolean | undefined : string | undefined
} & { "env-file": string | undefined };

/**
 * Parses CLI args (with `--env-file` included by default), loads the env file,
 * and returns the parsed values along with the network config and env file path.
 */
export const initCli = <const T extends ParseArgsConfig["options"]>(options?: T) => {
  const allOptions = { ...options, "env-file": { type: "string" as const } } as const;
  const { values } = parseArgs({ options: allOptions, strict: true });
  const vals = values as ParsedValues<T>;
  const envFile = requireArg(vals, "env-file");
  loadEnvFile(envFile);
  const networkConfig = loadNetworkConfig();
  const funding = buildFundingConfig(networkConfig);
  return { values: vals, envFile, networkConfig, funding };
};

export const requireEnv = (name: string): string => {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Required environment variable ${name} is not set`);
  }
  return value;
};

// parseArgs returns string | boolean | undefined — coerce to string so the
// return type is consistent with env var values, which are always strings.
export const requireArg = (
  values: Record<string, string | boolean | undefined>,
  name: string,
): string => {
  const value = values[name];
  if (value === undefined) {
    throw new Error(`Required argument --${name} is not provided`);
  }
  return String(value);
};

/**
 * Returns a CLI arg value if provided, otherwise falls back to an env var.
 * Throws if neither is available. Coerces non-string arg values to strings.
 */
export const requireArgOrEnv = (
  values: Record<string, string | boolean | undefined>,
  argName: string,
  envName: string,
): string => {
  const argValue = values[argName];
  if (argValue !== undefined) {
    return String(argValue);
  }
  return requireEnv(envName);
};

// --- EOA alias helpers ---

export const requireEoaSecret = (alias: string): string => {
  const envName = `EOA_${alias.toUpperCase()}_SECRET`;
  return requireEnv(envName);
};

export const requireEoaKeypair = (alias: string): Keypair => {
  return Keypair.fromSecret(requireEoaSecret(alias));
};

/**
 * Returns an EOA's public key by alias. Tries `EOA_{ALIAS}_SECRET` first
 * (deriving the public key from it), then falls back to `EOA_{ALIAS}_PUBLIC`.
 * Throws if neither is set.
 */
export const requireEoaPublicKey = (alias: string): string => {
  const upper = alias.toUpperCase();
  const secret = process.env[`EOA_${upper}_SECRET`];
  if (secret) {
    return Keypair.fromSecret(secret).publicKey();
  }
  const pub = process.env[`EOA_${upper}_PUBLIC`];
  if (pub) {
    return pub;
  }
  throw new Error(
    `Neither EOA_${upper}_SECRET nor EOA_${upper}_PUBLIC is set`,
  );
};

/**
 * Resolves an address from --{name} or --{name}-eoa. Returns undefined when
 * neither is provided.
 */
export const resolveAddressArg = (
  values: Record<string, string | boolean | undefined>,
  name: string,
): string | undefined => {
  const raw = values[name];
  if (raw && typeof raw === "string") return raw;
  const eoaArg = `${name}-eoa`;
  const alias = values[eoaArg];
  if (alias && typeof alias === "string") return requireEoaPublicKey(alias);
  return undefined;
};

export const requireAddressArg = (
  values: Record<string, string | boolean | undefined>,
  name: string,
): string => {
  const resolved = resolveAddressArg(values, name);
  if (resolved) return resolved;
  throw new Error(`Either --${name} or --${name}-eoa is required`);
};

// --- Asset code lookup helpers ---

/**
 * Returns the on-chain asset code from the env.
 * Users always pass uppercase (e.g., USDCALLOW), this returns the actual code (e.g., USDCAllow).
 * Falls back to the input if no ASSET_{CODE}_CODE env var is set.
 */
export const resolveAssetCode = (assetCode: string): string => {
  const envName = `ASSET_${assetCode.toUpperCase()}_CODE`;
  return process.env[envName] ?? assetCode;
};

export const requireAssetIssuerSecret = (assetCode: string): string => {
  const envName = `EOA_${assetCode.toUpperCase()}_ISSUER_SECRET`;
  return requireEnv(envName);
};

export const requireAssetIssuerPublic = (assetCode: string): string => {
  const envName = `EOA_${assetCode.toUpperCase()}_ISSUER_PUBLIC`;
  return requireEnv(envName);
};

export const requireAssetContractId = (assetCode: string): string => {
  const envName = `ASSET_${assetCode.toUpperCase()}_CONTRACT_ID`;
  return requireEnv(envName);
};

// --- Amount formatting helpers ---

const STELLAR_DECIMALS = 7n;
const STELLAR_SCALE = 10n ** STELLAR_DECIMALS;

/**
 * Converts a raw amount in stroops (base units) to a human-readable major units string.
 * e.g., 10000000n → "1", 15000000n → "1.5"
 */
export const stroopsToMajor = (stroops: bigint): string => {
  const whole = stroops / STELLAR_SCALE;
  const fractional = stroops % STELLAR_SCALE;
  return fractional > 0n
    ? `${whole}.${fractional.toString().padStart(Number(STELLAR_DECIMALS), "0").replace(/0+$/, "")}`
    : `${whole}`;
};

// --- Funding helpers ---

/**
 * Builds a FundingConfig from a NetworkConfig.
 * Priority: funder EOA > friendbot > no funding.
 */
export const buildFundingConfig = (networkConfig: NetworkConfig): import("./accounts.js").FundingConfig => ({
  friendbotUrl: networkConfig.friendbotUrl,
  funder: networkConfig.funderSecret
    ? {
        keypair: Keypair.fromSecret(networkConfig.funderSecret),
        server: new rpc.Server(networkConfig.rpcUrl, { allowHttp: networkConfig.allowHttp }),
        networkPassphrase: networkConfig.networkPassphrase,
        balanceThreshold: networkConfig.funderBalanceThreshold ?? "1",
      }
    : undefined,
});

// --- EOA resolve-or-create helpers ---

/**
 * Returns an existing EOA keypair from the env, or generates a new one and saves it to the env file.
 * When funding is configured, funds the account (existing or newly created) via the funder EOA
 * or friendbot. This ensures the account has XLM for transaction fees regardless of whether
 * it was just generated or already existed in the env.
 */
export const resolveOrCreateEoa = async (
  alias: string,
  envFile: string,
  funding?: import("./accounts.js").FundingConfig,
): Promise<Keypair> => {
  const secretKey = `EOA_${alias.toUpperCase()}_SECRET`;
  const publicKey = `EOA_${alias.toUpperCase()}_PUBLIC`;

  const existingSecret = process.env[secretKey];
  if (existingSecret) {
    const keypair = Keypair.fromSecret(existingSecret);
    console.log(`Using existing EOA "${alias}": ${keypair.publicKey()}`);
    await fundAccountIfConfigured(funding, keypair.publicKey()).match(
      () => { if (funding?.friendbotUrl || funding?.funder) console.log(`  EOA "${alias}" funded.`); },
      (err) => console.error(`  Funding failed: ${err.message}`),
    );
    return keypair;
  }

  const existingPublic = process.env[publicKey];
  if (existingPublic) {
    console.log(`Using existing EOA "${alias}" (public key only): ${existingPublic}`);
    await fundAccountIfConfigured(funding, existingPublic).match(
      () => { if (funding?.friendbotUrl || funding?.funder) console.log(`  EOA "${alias}" funded.`); },
      (err) => console.error(`  Funding failed: ${err.message}`),
    );
    return Keypair.fromPublicKey(existingPublic);
  }

  const keypair = Keypair.random();
  console.log(`Generated new EOA "${alias}": ${keypair.publicKey()}`);

  await fundAccountIfConfigured(funding, keypair.publicKey()).match(
    () => { if (funding?.friendbotUrl || funding?.funder) console.log(`  EOA "${alias}" funded.`); },
    (err) => console.error(`  Funding failed: ${err.message}`),
  );

  upsertEnvVars(envFile, { [secretKey]: keypair.secret(), [publicKey]: keypair.publicKey() });
  return keypair;
};

// --- Env scanning helpers ---

/**
 * Scans process.env for all EOA_*_SECRET and EOA_*_PUBLIC entries and groups
 * them by alias. For example, EOA_FUNDER_SECRET and EOA_FUNDER_PUBLIC produce
 * { FUNDER: { secret: "S...", public: "G..." } }.
 */
export const extractEoas = (): Record<string, { secret?: string; public?: string }> => {
  const eoas: Record<string, { secret?: string; public?: string }> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (!value) continue;
    const secretMatch = key.match(/^EOA_(.+)_SECRET$/);
    if (secretMatch) {
      const alias = secretMatch[1];
      eoas[alias] ??= {};
      eoas[alias].secret = value;
    }
    const publicMatch = key.match(/^EOA_(.+)_PUBLIC$/);
    if (publicMatch) {
      const alias = publicMatch[1];
      eoas[alias] ??= {};
      eoas[alias].public = value;
    }
  }
  return eoas;
};

/**
 * Scans process.env for ASSET_*_CONTRACT_ID entries and returns a map of
 * uppercase asset code to contract ID.
 */
export const extractAssetContractIds = (): Record<string, string> => {
  const ids: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (!value) continue;
    const match = key.match(/^ASSET_(.+)_CONTRACT_ID$/);
    if (match) ids[match[1]] = value;
  }
  return ids;
};

/**
 * Scans process.env for ASSET_*_CODE entries and returns a map of
 * uppercase asset code to the resolved on-chain code.
 */
export const extractAssetCodes = (): Record<string, string> => {
  const codes: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (!value) continue;
    const match = key.match(/^ASSET_(.+)_CODE$/);
    if (match) codes[match[1]] = value;
  }
  return codes;
};

// --- .env file helpers ---

/**
 * Reads an env file and populates process.env with its values.
 * Does not override existing process.env entries.
 */
export const loadEnvFile = (envPath: string): void => {
  if (!fs.existsSync(envPath)) {
    throw new Error(`Env file not found: ${envPath}`);
  }

  const content = fs.readFileSync(envPath, "utf-8");
  for (const line of content.split("\n")) {
    const match = line.match(/^([A-Za-z_][A-Za-z0-9_]*)=(.*)$/);
    if (!match) continue;
    const [, key, rawValue] = match;
    if (process.env[key] === undefined) {
      process.env[key] = rawValue.replace(/^"(.*)"$/, "$1");
    }
  }
};

export const upsertEnvFile = (envPath: string, key: string, value: string): void => {
  upsertEnvVars(envPath, { [key]: value });
};

export const upsertEnvVars = (envPath: string, vars: Record<string, string>): void => {
  const entries = Object.entries(vars);
  if (entries.length === 0) return;

  if (!fs.existsSync(envPath)) {
    fs.writeFileSync(envPath, entries.map(([k, v]) => `${k}=${v}`).join("\n") + "\n", "utf-8");
    for (const [k, v] of entries) process.env[k] = v;
    return;
  }

  let content = fs.readFileSync(envPath, "utf-8");
  const toAppend: string[] = [];

  for (const [key, value] of entries) {
    const line = `${key}=${value}`;
    const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`^${escaped}=.*$`, "m");

    if (regex.test(content)) {
      content = content.replace(regex, line);
    } else {
      toAppend.push(line);
    }
    process.env[key] = value;
  }

  if (toAppend.length > 0) {
    const endsWithNewline = content.endsWith("\n");
    content += (endsWithNewline ? "" : "\n") + toAppend.join("\n") + "\n";
  }

  fs.writeFileSync(envPath, content, "utf-8");
};
