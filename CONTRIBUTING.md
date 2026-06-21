# Contributing to OrbitChain Contracts

Thanks for helping improve OrbitChain. This repository contains Soroban smart
contracts, CLI tooling, deployment scripts, and project documentation. Keep
changes focused and easy to review.

## Development Setup

Install the Rust toolchain and Soroban/Stellar CLI prerequisites:

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli --features opt
```

Install the optional security scan tools before running the audit targets:

```bash
cargo install cargo-audit --locked
cargo install cargo-deny --locked
```

The Makefile checks for these binaries and prints the same install commands if
they are missing.

## Common Commands

```bash
make build
make test
make fmt
make lint
make audit
make deny
```

Run the smallest command set that matches your change. For documentation-only
pull requests, a Markdown/link review is usually enough; for contract or CLI
changes, include the relevant build and test commands in the pull request.

## Pull Request Flow

1. Fork the repository.
2. Create a focused branch, for example `docs/security-scan-tools`.
3. Keep unrelated formatting, dependency updates, and generated files out of the
   pull request.
4. Use a conventional commit title such as `docs: update security scan setup` or
   `fix: guard missing audit tools`.
5. Open a pull request with a short summary, validation notes, and linked issue.

## Security Scan Expectations

`make audit` runs `cargo audit` and reports known RustSec vulnerabilities.
`make deny` runs `cargo deny check` and reports license, advisory, source, and
crate policy violations.

If either command fails because the tool is missing, install it with the command
shown by the Makefile and retry. If the tool is installed and reports a real
finding, document the finding in the pull request before changing dependencies
or policy files.

Never include private keys, seed phrases, wallet recovery material, API tokens,
or production signing credentials in issues, commits, tests, or pull requests.
