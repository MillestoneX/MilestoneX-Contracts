# Security Policy

## Key Derivation Function (KDF) Migration (Issue #XX)

As of version X.X.X, the vault key derivation has been upgraded from SHA-256 to **Argon2id** with per-instance salts. This prevents dictionary and rainbow-table attacks on operator passwords.

**For users:** When you next load or create a vault with a password, the new KDF will be used automatically. Existing SHA-256-derived vaults must be re-encrypted by creating a new vault and migrating keys.

## Reporting a Vulnerability

OrbitChain-Contracts contains Soroban smart contracts handling crowdfunding and fund management. Security vulnerabilities can have serious financial consequences. Please report them responsibly.

**Do NOT open a public issue to report a security vulnerability.**

### How to Report

- **GitHub Private Advisories**: Use [GitHub's private vulnerability reporting](https://github.com/OrbitChainLabs/OrbitChain-Contracts/security/advisories/new)
- **Email**: Contact the maintainers listed in the repository README

### What to Include

- Description of the vulnerability and its potential impact
- Steps to reproduce
- Affected contract(s) and function(s)
- Proof-of-concept (if available)
- Suggested fix (optional)

### Response Timeline

| Stage | Timeline |
|-------|----------|
| Acknowledgement | 48 hours |
| Initial triage | 5 business days |
| Fix or mitigation | 30 days for critical issues |

### Scope

High-priority security areas for this project:

- Smart contract logic errors (Soroban/Stellar)
- Arithmetic overflow/underflow in fund calculations
- Unauthorized access to admin or contributor functions
- Reentrancy or state manipulation vulnerabilities
- Incorrect access control on contract invocations

Thank you for helping keep OrbitChain and its users safe.
