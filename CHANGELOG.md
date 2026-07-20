# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Community documentation (CONTRIBUTING.md, CODE_OF_CONDUCT.md, MAINTAINERS.md, CHANGELOG.md)

## [0.1.0] - 2024-07-20

### Added
- Initial release of MilestoneX crowdfunding protocol on Stellar Soroban
- Canonical campaign contract (`campaign/`) with milestone-based fund release
- Multi-asset donation support (XLM, USDC, NGNT, custom tokens)
- Freeze/upgrade controls for contract lifecycle management
- Reentrancy protection mechanisms
- Comprehensive analytics endpoints:
  - `get_campaign_report`
  - `get_platform_summary`
  - `get_dashboard_metrics`
  - `get_donation_count`
  - `get_donor_count`
  - `get_release_count`
  - `get_total_tx_count`
- Legacy core contract (`crates/contracts/core/`) for compatibility
- CLI tools (`crates/tools/`) for:
  - Asset management (config, generate, trustline, issue)
  - Key management (encrypt, decrypt, vault operations)
  - Keypair lifecycle (generate, fund, validate)
  - Wallet signing workflows (build-donation, build-campaign, validate)
  - Response processing (process, validate, save, load)
  - Network configuration (config, network, toggle)
- Fee estimation service with real-time fee calculation
- Surge pricing detection (4-level system)
- Multi-currency fee display (10+ currencies)
- Security scanning integration (cargo-audit, cargo-deny)
- Comprehensive Makefile for build, test, format, lint, clean operations
- Soroban network configuration system
- Docker-based local sandbox support
- Deployment scripts for testnet and sandbox
- Environment configuration via .env files

### Security
- Implemented reentrancy lockdown across all contract entry points
- Typed error system for better error handling and security
- Encrypted vault for secure key storage
- Security scanning automation in CI/CD pipeline

### Documentation
- Comprehensive README with quick start guide
- Development setup instructions
- CLI command documentation
- Deployment guides for testnet and sandbox
- Architecture overview
- Security scan documentation

### Infrastructure
- GitHub Actions CI/CD pipeline
- Automated testing on all PRs
- Automated security scanning
- Rust toolchain configuration via rust-toolchain.toml
- WASM target configuration for Soroban contracts

[Unreleased]: https://github.com/MillestoneX/MilestoneX-Contracts/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/MillestoneX/MilestoneX-Contracts/releases/tag/v0.1.0
