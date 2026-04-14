# stablecoin-xlm

Official implementation of Circle's stablecoin infrastructure for the Stellar network.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

## Overview

This repository contains the tooling and smart contracts for deploying and managing Circle-issued stablecoins (USDC, EURC) on Stellar.

- **Soroban (SAC Stablecoin)** — Smart contracts for on-chain stablecoin minting and administration via Stellar's Soroban smart contract platform, coordinating with Stellar Asset Contracts (SACs) for atomic swap-based minting with role-based access control.

## Repository Structure

```
stablecoin-xlm/
├── soroban/               # Soroban smart contracts and tooling
│   ├── contracts/         # Smart contracts (fiat-token-admin)
│   ├── packages/          # Shared Rust libraries (roles, interfaces, utils)
│   └── scripts/           # TypeScript CLI and deployment tooling
├── .github/               # CI/CD workflows
├── .githooks/             # Pre-commit hooks (format and lint checks)
├── CHANGELOG.md           # Release changelog
├── CONTRIBUTING.md        # Contribution guidelines
├── COPYRIGHT              # Copyright notice
├── LICENSE                # Apache License 2.0
└── SECURITY.md            # Vulnerability reporting policy
```

## Getting Started

Each subproject has its own README with detailed setup and usage instructions:

- **[soroban/README.md](soroban/README.md)** — Prerequisites, development setup, building, testing, and project structure for the Soroban smart contracts.

## Security

If you discover a security vulnerability, please report it through Circle's [Bug Bounty Program](https://hackerone.com/circle-bbp). Do not file public GitHub issues for security vulnerabilities. See [SECURITY.md](SECURITY.md) for details.

## Resources

- [Stellar Developer Documentation](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Circle Developer Hub](https://developers.circle.com/)

## License

This project is licensed under the Apache License, Version 2.0 — see the [LICENSE](LICENSE) file for details.
