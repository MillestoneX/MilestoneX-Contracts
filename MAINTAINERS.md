# Maintainers

This document lists the current maintainers of the MilestoneX project and their areas of responsibility.

## Project Maintainers

### Core Team

| Name | GitHub | Responsibilities |
|------|--------|------------------|
| MilestoneX Team | @MillestoneX | Project oversight, architecture, and strategic direction |

### Active Maintainers

| Name | GitHub | Areas of Ownership |
|------|--------|---------------------|
| MilestoneX Team | @MillestoneX | Core contract development, deployment, CI/CD, security |

## Areas of Ownership

### Smart Contracts
- **Canonical Campaign Contract** (`campaign/`)
  - Milestone-based fund release logic
  - Multi-asset donation processing
  - Refund mechanisms
  - Freeze/upgrade controls
  - Reentrancy protection
  - Analytics endpoints

- **Legacy Core Contract** (`crates/contracts/core/`)
  - Maintenance only (no new features)
  - BugFixes for critical issues
  - Migration support to canonical contract

### CLI Tools
- **MilestoneX Tools** (`crates/tools/`)
  - Asset management commands
  - Key management and encryption
  - Wallet signing workflows
  - Network configuration
  - Deployment utilities

### Infrastructure
- **CI/CD Pipelines**
  - GitHub Actions workflows
  - Automated testing
  - Security scanning (cargo-audit, cargo-deny)
  - Deployment automation

- **Documentation**
  - README.md
  - API documentation
  - Deployment guides
  - Contribution guidelines

### Security
- **Security Audits**
  - Contract security reviews
  - Vulnerability assessments
  - Security best practices
  - Incident response

### Token Bridge
- **Cross-Chain Integration** (`token-bridge/`)
  - Stellar asset bridging
  - Cross-chain transaction handling
  - Bridge security

## Becoming a Maintainer

### Criteria for Maintainership

Contributors who demonstrate the following may be invited to become maintainers:

- **Consistent Contributions**: Regular, high-quality contributions over time
- **Domain Expertise**: Deep understanding of specific project areas
- **Community Engagement**: Active participation in reviews, discussions, and issue resolution
- **Reliability**: Dependable response to issues and PRs in their area
- **Alignment**: Strong alignment with project goals and values

### Process

1. **Nomination**: Existing maintainers nominate potential new maintainers
2. **Discussion**: Core team discusses the nomination
3. **Consensus**: Maintainers reach consensus on the invitation
4. **Onboarding**: New maintainer is onboarded with specific responsibilities
5. **Announcement**: New maintainer is announced to the community

### Maintainer Responsibilities

- **Code Review**: Review and merge PRs in their area of ownership
- **Issue Triage**: Respond to and prioritize issues in their domain
- **Release Management**: Participate in release planning and execution
- **Documentation**: Keep documentation up-to-date for their areas
- **Mentorship**: Help onboard and guide new contributors
- **Security**: Promptly address security vulnerabilities in their domain

## Stepping Down

Maintainers who wish to step down should:

1. Notify the core team in advance
2. Document current work and pending items
3. Help identify and onboard a replacement if possible
4. Ensure smooth transition of responsibilities

## Contact

For maintainer-related inquiries:
- **General**: GitHub Discussions
- **Security**: security@milestonex.io
- **Urgent**: Create a GitHub issue with the `maintainer` label

## Governance Decisions

Major decisions affecting the project direction require:
- Consensus among active maintainers
- Documentation of the decision in the repository
- Communication to the community when appropriate

## Emeritus Maintainers

Former maintainers who have stepped down but remain valued contributors:

| Name | GitHub | Period | Notes |
|------|--------|--------|-------|
| *None yet* | - | - | - |

---

*Last updated: 2024-07-20*
