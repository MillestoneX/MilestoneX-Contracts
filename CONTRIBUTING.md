# Contributing to MilestoneX

Thank you for your interest in contributing to MilestoneX! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all interactions with the project.

## Getting Started

### Prerequisites

- **Rust stable toolchain** (automatically managed by `rust-toolchain.toml`)
- **wasm32v1-none target** (auto-installed by toolchain)
- **Soroban CLI** for contract deployment and testing
- Git for version control

### Clone the Repository

```bash
git clone https://github.com/MillestoneX/MilestoneX-Contracts.git
cd MilestoneX-Contracts
```

### Install Dependencies

The project uses `rust-toolchain.toml` to ensure consistent development environments:

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# The project will automatically use the correct Rust version and targets
rustup show
```

Install Soroban CLI:

```bash
# Method 1: Install via cargo (recommended for development)
cargo install soroban-cli

# Method 2: Install via npm (alternative)
npm install -g soroban-cli

# Verify installation
soroban --version
```

### Build the Project

```bash
# Using Make (recommended)
make build

# Or using cargo directly
cargo build -p milestonex-campaign --target wasm32v1-none
```

## Development Workflow

### 1. Create a Branch

Create a new branch for your contribution:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

Use conventional commit prefixes:
- `feat/` for new features
- `fix/` for bug fixes
- `docs/` for documentation changes
- `refactor/` for code refactoring
- `test/` for test additions or changes

### 2. Make Your Changes

- Write clean, readable code following the project's coding standards
- Add tests for new functionality
- Update documentation as needed
- Ensure all existing tests pass

### 3. Test Your Changes

```bash
# Run all tests
make test

# Or using cargo directly
cargo test --workspace

# Format code
make fmt

# Run linter
make lint

# Run security scans
make audit
make deny
```

### 4. Commit Your Changes

Use conventional commit messages:

```bash
feat: add wallet connection modal
fix: resolve donation API error
docs: update project README
refactor: clean up project creation form
test: add unit tests for campaign contract
```

Commit message format:
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect code meaning (formatting, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files

## Pull Request Process

### 1. Push Your Changes

```bash
git push origin feature/your-feature-name
```

### 2. Create a Pull Request

- Go to the [MilestoneX-Contracts repository](https://github.com/MillestoneX/MilestoneX-Contracts)
- Click "New Pull Request"
- Select your branch
- Fill in the PR template with:
  - A clear description of the changes
  - Related issue numbers (if any)
  - Screenshots for UI changes (if applicable)
  - Testing instructions

### 3. PR Review Process

- Your PR will be reviewed by maintainers
- Address any feedback or requested changes
- Ensure CI checks pass
- Once approved, your PR will be merged

### 4. After Merge

- Delete your branch if desired
- Celebrate your contribution! 🎉

## Coding Standards

### Rust Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Write clear, self-documenting code
- Add comments for complex logic
- Use meaningful variable and function names

### Soroban Contract Guidelines

- Keep contracts focused and modular
- Use typed errors for better error handling
- Implement reentrancy protection where applicable
- Add comprehensive unit tests
- Document contract methods and parameters

### Documentation

- Update README.md for user-facing changes
- Add inline documentation for complex functions
- Keep examples up-to-date
- Use clear, concise language

## Testing

### Unit Tests

```bash
# Run canonical campaign contract tests
cargo test -p milestonex-campaign

# Run all tests
cargo test --workspace
```

### Integration Tests

```bash
# Deploy to testnet for integration testing
make deploy-testnet

# Test contract invocation
stellar contract invoke --id "$CONTRACT_ID" --source test_account --network testnet -- version
```

### Test Coverage

- Aim for high test coverage on critical paths
- Test both success and failure scenarios
- Include edge cases and boundary conditions

## Reporting Issues

### Bug Reports

When reporting a bug, please include:

- A clear description of the problem
- Steps to reproduce the issue
- Expected behavior vs. actual behavior
- Environment information (OS, Rust version, Soroban CLI version)
- Relevant logs or error messages
- Screenshots if applicable

### Feature Requests

When requesting a feature, please include:

- A clear description of the proposed feature
- The motivation behind the feature
- Potential implementation approach (if known)
- Examples or use cases

### Security Issues

**Do not report security issues publicly.** Instead, please send them to:

- Email: security@milestonex.io
- Or use GitHub's private vulnerability reporting feature

## Getting Help

- Check existing [GitHub Issues](https://github.com/MillestoneX/MilestoneX-Contracts/issues)
- Read the [documentation](docs/)
- Join our community discussions
- Reach out to maintainers

## Recognition

Contributors are recognized in:
- The project's contributors list
- Release notes for significant contributions
- Project documentation for major features

Thank you for contributing to MilestoneX! Your contributions help make on-chain crowdfunding more accessible and transparent.
