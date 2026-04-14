# Stellar Fiat Token Admin

Soroban smart contracts for the Stellar Fiat Token Admin, enabling stablecoin management on the Stellar network.

## Overview

This directory contains the Soroban smart contracts that enable the minting and administration of Circle-issued stablecoins (USDC, EURC) on the Stellar network. The contracts provide secure, role-based access control for stablecoin operations including minting, pausing, and blocklisting.

### Key Features

- **Swap Minting**: Burn allowance assets and mint stablecoins in a single atomic operation
- **Role-Based Access Control**: Granular permissions with Owner, Admin, Pauser, Blocklister, and Minter Asset Controller roles
- **Two-Step Role Transfers**: Secure ownership and admin transfers requiring explicit acceptance
- **Pausable Operations**: Emergency pause functionality for minting operations
- **Address Blocklisting**: Compliance controls for blocking specific addresses
- **Upgradeable Design**: Contracts can be upgraded by authorized administrators

## Architecture

### Fiat Token Admin Contract

The `fiat-token-admin` contract is the core smart contract that manages stablecoin minting on Stellar. It coordinates with Stellar Asset Contracts (SACs) to burn allowance tokens and mint the corresponding stablecoin.

**Key Roles:**
- **Owner**: Can update all other roles and transfer ownership
- **Admin**: Can upgrade the contract
- **Pauser**: Can pause/unpause minting operations
- **Blocklister**: Can block/unblock addresses from minting
- **Minter Asset Controller**: Can configure and remove minters

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (rustup) - pinned to `1.93.0` via `rust-toolchain.toml`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (optional, for deployment)
- [Node.js](https://nodejs.org/) 20+ with [Corepack](https://nodejs.org/api/corepack.html) enabled (for scripts and E2E tests)

## Development Setup

1. Navigate to the soroban directory:
```bash
cd soroban
```

2. Install the pinned Rust toolchain and `wasm32v1-none` target:
```bash
scripts/setup-rust.sh
```

3. Set up pre-commit hooks (recommended):
```bash
git config core.hooksPath .githooks
```

This enables automatic format and lint checks before each commit.

## Building

Build all contracts:
```bash
cargo build --target wasm32v1-none --release
```

Build a specific contract:
```bash
cargo build --target wasm32v1-none --release -p fiat-token-admin
```

The compiled WASM artifacts will be located in `target/wasm32v1-none/release/`.

A `release-with-logs` profile is also available for debugging builds with log output enabled:
```bash
cargo build --target wasm32v1-none --profile release-with-logs
```

## Testing

Run all tests:
```bash
cargo test
```

Run tests for a specific contract:
```bash
cargo test -p fiat-token-admin
```

Run tests with verbose output:
```bash
cargo test -- --nocapture
```

The `scripts/` directory contains TypeScript tooling for local environment setup and deployment. See [scripts/README.md](scripts/README.md) for full details.

## Code Quality

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy --all-targets --all-features -- --deny warnings
```

Check code without building:
```bash
cargo check
```

## Project Structure

```
soroban/
├── contracts/                 # Smart contracts
│   └── fiat-token-admin/      # Fiat token admin contract
├── packages/                  # Shared libraries
│   ├── common-roles/          # Common role implementations (Ownable, Pausable, etc.)
│   ├── stablecoin-roles/      # Stablecoin-specific roles
│   ├── stablecoin-interfaces/ # Stablecoin interfaces
│   ├── simple-role/           # Base role infrastructure
│   ├── simple-role-macros/    # Procedural macros for roles
│   ├── utils/                 # Utility functions
│   ├── test-utils/            # Testing utilities
│   └── test-contracts/        # Mock contracts for testing
├── scripts/                   # Development, deployment, and testing tooling
│   ├── cli/                   # CLI scripts (e.g., local environment setup)
│   ├── clients/               # Generated TypeScript contract bindings
│   ├── common/                # Shared TypeScript utilities
│   ├── deploy/                # Deployment scripts
│   ├── setup-rust.sh          # Installs rustup and pins toolchain
│   └── README.md              # Detailed scripts documentation
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Locked dependency versions
└── rust-toolchain.toml        # Rust version specification (1.93.0)
```

### Packages

| Package | Description |
|---------|-------------|
| `fiat-token-admin` | Main contract for stablecoin minting and administration |
| `common-roles` | Reusable role implementations (Ownable, Pausable, Manageable, Rescuable) |
| `stablecoin-roles` | Stablecoin-specific roles (MinterAssetControllable) |
| `stablecoin-interfaces` | Contract interfaces (SwapMinter trait) |
| `simple-role` | Generic single-address role management utilities |
| `simple-role-macros` | Procedural macros for role enforcement |
| `utils` | Shared utilities for storage TTL extension and byte operations |
| `test-utils` | Testing utilities and helpers |
| `test-contracts` | Mock contracts for testing |

## Coverage

This project uses `cargo-llvm-cov` for measuring test coverage and enforces a minimum 90% coverage threshold in CI. You can generate a coverage report locally with:
```bash
cargo llvm-cov --all-features --workspace
```
