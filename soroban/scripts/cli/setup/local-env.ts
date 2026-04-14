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

import { parseArgs } from "node:util";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { Keypair, Networks, rpc } from "@stellar/stellar-sdk";
import { Client as FiatTokenAdminClient } from "../../clients/fiat-token-admin/src/index.js";
import {
  fundAccount,
  createTrustline,
  mintAsset,
  setTrustlineAuthorization,
  startQuickstartContainer,
  createClientFactory,
} from "../../common/index.js";
import type { FundingConfig } from "../../common/accounts.js";
import {
  loadEnvFile,
  upsertEnvVars,
  resolveOrCreateEoa,
} from "../../common/cli-utils.js";
import { deployStablecoinBase } from "../../deploy/stablecoin-base.js";
import type { DeployStablecoinBaseResult } from "../../deploy/stablecoin-base.js";
import { asciiToSalt } from "../../common/salt.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const { values } = parseArgs({
  options: {
    "mint-asset-code": { type: "string", default: "USDC" },
    "allow-asset-code": { type: "string", default: "USDCALLOW" },
    "from-env": { type: "string" },
    debug: { type: "boolean", default: false },
  },
  strict: true,
});

const MINT_ASSET_CODE = values["mint-asset-code"]!;
const ALLOW_ASSET_CODE = values["allow-asset-code"]!;
const ALLOW_ASSET_FUND_AMOUNT = "1000";
const ENV_FILE = values["from-env"]
  ? path.resolve(values["from-env"])
  : path.resolve(__dirname, "../../.env.local");

const EOA_ALIASES = [
  "fiat_token_admin_deployer",
  "fiat_token_admin_owner",
  "fiat_token_admin_pauser",
  "fiat_token_admin_admin",
  "fiat_token_admin_minter_asset_controller",
  "fiat_token_admin_blocklister",
  "minter",
  "recipient",
] as const;

interface LocalEnvContext {
  server: rpc.Server;
  rpcUrl: string;
  networkPassphrase: string;
  funding: FundingConfig;
  fail: (msg: string, err: Error) => Promise<never>;
}

async function startNode(): Promise<
  LocalEnvContext & { stop: () => Promise<void> }
> {
  console.log("Starting local Stellar node...");
  const containerResult = await startQuickstartContainer({
    debug: values.debug!,
  });
  if (containerResult.isErr()) {
    console.error("Failed to start:", containerResult.error.message);
    process.exit(1);
  }

  const { urls, stop } = containerResult.value;
  const rpcUrl = urls.rpcUrl.href;
  const friendbotUrl = urls.friendbotUrl.href;
  const networkPassphrase = Networks.STANDALONE;
  const server = new rpc.Server(rpcUrl, { allowHttp: true });

  const fail = async (msg: string, err: Error): Promise<never> => {
    console.error(`${msg}:`, err.message);
    await stop();
    process.exit(1);
  };

  // Write network config to env file (upsert handles both fresh and rerun)
  upsertEnvVars(ENV_FILE, {
    STELLAR_RPC_URL: rpcUrl,
    NETWORK_PASSPHRASE: networkPassphrase,
    FRIENDBOT_URL: friendbotUrl,
    HORIZON_URL: urls.horizonUrl.href,
    STELLAR_PORT: String(urls.port),
    ALLOW_HTTP: "true",
    FIAT_TOKEN_ADMIN_MINT_ASSET_CODE: MINT_ASSET_CODE,
    FIAT_TOKEN_ADMIN_ALLOW_ASSET_CODE: ALLOW_ASSET_CODE,
  });
  loadEnvFile(ENV_FILE);

  // Bootstrap funder EOA if configured — fund via friendbot so subsequent
  // operations can route through the funder, simulating a prod-like environment.
  let funding: FundingConfig = { friendbotUrl };
  const funderSecret = process.env.EOA_FUNDER_SECRET;
  if (funderSecret) {
    const funderKeypair = Keypair.fromSecret(funderSecret);
    console.log(
      `Bootstrapping funder EOA via friendbot: ${funderKeypair.publicKey()}`,
    );
    await fundAccount(friendbotUrl, funderKeypair.publicKey()).match(
      () => console.log("  Funder EOA funded."),
      (err) => fail("Failed to fund funder EOA", err),
    );
    funding = {
      friendbotUrl,
      funder: {
        keypair: funderKeypair,
        server,
        networkPassphrase,
        balanceThreshold: process.env.FUNDER_BALANCE_THRESHOLD || "1",
      },
    };
  }

  return { server, rpcUrl, networkPassphrase, funding, fail, stop };
}

async function createAccounts(ctx: LocalEnvContext) {
  console.log("Setting up role accounts...");
  const eoas: Record<string, Keypair> = {};
  for (const alias of EOA_ALIASES) {
    eoas[alias] = await resolveOrCreateEoa(alias, ENV_FILE, ctx.funding);
  }

  const mintIssuer = await resolveOrCreateEoa(
    `${MINT_ASSET_CODE}_ISSUER`,
    ENV_FILE,
    ctx.funding,
  );
  const allowIssuer = await resolveOrCreateEoa(
    `${ALLOW_ASSET_CODE}_ISSUER`,
    ENV_FILE,
    ctx.funding,
  );

  return { eoas, mintIssuer, allowIssuer };
}

async function deployContracts(
  ctx: LocalEnvContext,
  eoas: Record<string, Keypair>,
  mintIssuer: Keypair,
  allowIssuer: Keypair,
): Promise<DeployStablecoinBaseResult> {
  console.log("Deploying stablecoin base...");
  const saltHex = process.env.FIAT_TOKEN_ADMIN_SALT;
  const ftaSalt = saltHex
    ? Buffer.from(saltHex, "hex")
    : asciiToSalt("fiat-token-admin");

  const result = await deployStablecoinBase({
    server: ctx.server,
    networkPassphrase: ctx.networkPassphrase,
    funding: ctx.funding,
    mintAssetIssuer: mintIssuer,
    mintAssetCode: MINT_ASSET_CODE,
    allowAssets: [{ assetIssuer: allowIssuer, assetCode: ALLOW_ASSET_CODE }],
    fiatTokenAdmin: {
      deployer: eoas.fiat_token_admin_deployer,
      owner: eoas.fiat_token_admin_owner.publicKey(),
      pauser: eoas.fiat_token_admin_pauser.publicKey(),
      admin: eoas.fiat_token_admin_admin.publicKey(),
      minterAssetController:
        eoas.fiat_token_admin_minter_asset_controller.publicKey(),
      blocklister: eoas.fiat_token_admin_blocklister.publicKey(),
    },
    salt: ftaSalt,
  }).match(
    (r) => r,
    (err) => ctx.fail("Failed to deploy stablecoin base", err),
  );

  upsertEnvVars(ENV_FILE, {
    [`ASSET_${MINT_ASSET_CODE}_CONTRACT_ID`]: result.mintAsset.contractId,
    [`ASSET_${MINT_ASSET_CODE}_CODE`]: MINT_ASSET_CODE,
    [`ASSET_${ALLOW_ASSET_CODE}_CONTRACT_ID`]: result.allowAssets[0].contractId,
    [`ASSET_${ALLOW_ASSET_CODE}_CODE`]: ALLOW_ASSET_CODE,
    FIAT_TOKEN_ADMIN_CONTRACT_ID: result.fiatTokenAdmin.contractId,
    FIAT_TOKEN_ADMIN_WASM_HASH: result.fiatTokenAdmin.wasmHash,
    FIAT_TOKEN_ADMIN_SALT: ftaSalt.toString("hex"),
  });
  console.log(`  Mint asset:       ${result.mintAsset.contractId}`);
  console.log(`  Allow asset:      ${result.allowAssets[0].contractId}`);
  console.log(`  FiatTokenAdmin:   ${result.fiatTokenAdmin.contractId}`);

  return result;
}

async function configureSetup(
  ctx: LocalEnvContext,
  eoas: Record<string, Keypair>,
  baseResult: DeployStablecoinBaseResult,
  mintIssuer: Keypair,
  allowIssuer: Keypair,
) {
  console.log("Configuring minter...");
  const minter = eoas.minter;
  const controllerClient = createClientFactory(
    FiatTokenAdminClient,
    baseResult.fiatTokenAdmin.contractId,
    ctx.rpcUrl,
    ctx.networkPassphrase,
  )(eoas.fiat_token_admin_minter_asset_controller);

  await (
    await controllerClient.configure_minter({
      minter: minter.publicKey(),
      allow_asset: baseResult.allowAssets[0].contractId,
    })
  ).signAndSend();
  console.log("  Minter configured.");

  // Fund minter with allow asset
  await createTrustline(
    ctx.server,
    minter,
    ALLOW_ASSET_CODE,
    allowIssuer.publicKey(),
    ctx.networkPassphrase,
  )
    .andThen(() =>
      setTrustlineAuthorization(
        ctx.server,
        allowIssuer,
        minter.publicKey(),
        ALLOW_ASSET_CODE,
        true,
        ctx.networkPassphrase,
      ),
    )
    .andThen(() =>
      mintAsset(
        ctx.server,
        allowIssuer,
        minter.publicKey(),
        ALLOW_ASSET_CODE,
        ALLOW_ASSET_FUND_AMOUNT,
        ctx.networkPassphrase,
      ),
    )
    .match(
      () =>
        console.log(
          `  Minter funded with ${ALLOW_ASSET_FUND_AMOUNT} ${ALLOW_ASSET_CODE}.`,
        ),
      (err) => ctx.fail("Failed to setup minter allow asset", err),
    );

  // Minter needs mint asset trustline to receive minted tokens
  await createTrustline(
    ctx.server,
    minter,
    MINT_ASSET_CODE,
    mintIssuer.publicKey(),
    ctx.networkPassphrase,
  ).match(
    () => console.log(`  Minter has ${MINT_ASSET_CODE} trustline.`),
    (err) => ctx.fail("Failed to create minter mint asset trustline", err),
  );

  // Recipient needs mint asset trustline
  await createTrustline(
    ctx.server,
    eoas.recipient,
    MINT_ASSET_CODE,
    mintIssuer.publicKey(),
    ctx.networkPassphrase,
  ).match(
    () => console.log(`  Recipient has ${MINT_ASSET_CODE} trustline.`),
    (err) => ctx.fail("Failed to create recipient mint asset trustline", err),
  );
}

async function main() {
  const { stop, ...ctx } = await startNode();
  const { eoas, mintIssuer, allowIssuer } = await createAccounts(ctx);
  const baseResult = await deployContracts(ctx, eoas, mintIssuer, allowIssuer);
  await configureSetup(ctx, eoas, baseResult, mintIssuer, allowIssuer);

  console.log("\n========== Local Environment Ready ==========\n");
  console.log(`  .env.local:  ${ENV_FILE}`);
  console.log(`  RPC URL:     ${ctx.rpcUrl}`);
  console.log(`  Contract:    ${baseResult.fiatTokenAdmin.contractId}`);
  console.log(
    `  Mint asset:  ${MINT_ASSET_CODE} (${baseResult.mintAsset.contractId})`,
  );
  console.log(
    `  Allow asset: ${ALLOW_ASSET_CODE} (${baseResult.allowAssets[0].contractId})`,
  );
  console.log(`\nPress Ctrl-C to stop the node.\n`);

  await new Promise<void>((resolve) => {
    const keepAlive = setInterval(() => {}, 1 << 30);
    let shuttingDown = false;
    const shutdown = async () => {
      if (shuttingDown) {
        resolve();
        return;
      }
      shuttingDown = true;
      clearInterval(keepAlive);
      console.log("\nShutting down...");
      await stop();
      resolve();
    };
    process.on("SIGINT", shutdown);
    process.on("SIGTERM", shutdown);
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
