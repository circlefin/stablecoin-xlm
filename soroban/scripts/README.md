# Stellar Stablecoin Scripts

TypeScript tooling for spinning up a local FiatTokenAdmin development environment.

## Prerequisites

- Node.js >= 20
- Yarn (managed via [Corepack](https://yarnpkg.com/corepack) — run `corepack enable` once)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (for regenerating contract bindings)
- Rust + `wasm32v1-none` target (for rebuilding contracts)
- Docker (required by `setup:local-env`)

## Setup

```bash
corepack enable   # one-time setup
yarn install
```

### Build the contract and regenerate bindings

Only needed when the contract source changes:

```bash
yarn build:fiat-token-admin
yarn generate:bindings:fiat-token-admin
```

## Local Environment

`setup:local-env` starts a local Stellar Quickstart node, deploys all contracts, configures a minter, and writes everything to `.env.local`. No pre-existing env file is required.

```bash
yarn setup:local-env
yarn setup:local-env --from-env .env.local      # read existing keys, write outputs back
yarn setup:local-env --mint-asset-code USDC --allow-asset-code USDCALLOW
```

| Parameter | Required | Description |
|---|---|---|
| `--from-env` | No | Path to an env file to read from and write to. Created if missing. Default: `.env.local` |
| `--mint-asset-code` | No | Asset code for the mint asset. Default: `USDC` |
| `--allow-asset-code` | No | Asset code for the allow asset. Default: `USDCALLOW` |
| `--debug` | No | Stream container logs to stdout. Default: `false` |

**What it does:**

1. Starts a Stellar Quickstart container (standalone network with Friendbot)
2. Creates role accounts: `fiat_token_admin_deployer`, `fiat_token_admin_owner`, `fiat_token_admin_pauser`, `fiat_token_admin_admin`, `fiat_token_admin_minter_asset_controller`, `fiat_token_admin_blocklister`, `minter`, `recipient`
3. Deploys mint asset SAC, allow asset SAC, and FiatTokenAdmin contract
4. Sets FiatTokenAdmin as the mint asset SAC admin
5. Configures the minter on FiatTokenAdmin
6. Creates trustlines and funds the minter with allow asset tokens
7. Writes all keys, contract IDs, and network config to the env file

**Account funding** — if `EOA_FUNDER_SECRET` is set in the env file, the funder is bootstrapped via Friendbot and all subsequent accounts are funded through it (simulating a prod-like environment). Otherwise, Friendbot is used directly.

The node keeps running until `Ctrl-C`.

## Project Structure

```
scripts/
├── cli/setup/
│   └── local-env.ts               # Local environment setup entry point
├── common/                        # Shared utilities
│   ├── accounts.ts                # Account funding, trustlines, minting
│   ├── cli-utils.ts               # Env/arg helpers, EOA resolvers
│   ├── client.ts                  # Contract client factory
│   ├── deploy.ts                  # Contract deploy helpers
│   ├── network-config.ts          # Env-based network config loader
│   ├── network-config.schema.ts   # Network config Zod schema
│   ├── sac.ts                     # SAC operations (auth flags, deploy, admin)
│   ├── salt.ts                    # Deterministic salt generation
│   ├── stellar-container.ts       # Quickstart container management
│   ├── transactions.ts            # Transaction submission helpers
│   └── wasm.ts                    # WASM upload utilities
├── deploy/                        # Deployment logic
│   ├── asset.ts                   # Mint + allow asset SAC deploy
│   ├── fiat-token-admin.ts        # FiatTokenAdmin deploy
│   └── stablecoin-base.ts         # Orchestrates full stablecoin deploy
├── clients/fiat-token-admin/      # Generated contract bindings (do not edit)
├── .env.example                   # Environment variable template
└── package.json
```
