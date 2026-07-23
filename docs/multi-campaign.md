# Multi-Campaign Registry

`campaigns-registry` (`milestonex-campaigns-registry`) supports many
campaigns in a single contract instance, each identified by a `u64`
campaign ID. It answers issue #44: `milestonex-campaign` is
single-campaign-per-instance by design (one `initialize` call per deployed
contract), and there was no supported way to run many campaigns without
deploying a new contract instance per campaign.

This document covers the registry's API, how it relates to
`milestonex-campaign`, the migration plan, and what is deliberately out of
scope for this first version.

## Relationship to `milestonex-campaign`

The root `README.md`'s "Contract Canonicalization" section designates
`campaign/` (`milestonex-campaign`) as the canonical contract for
single-campaign deployments — that decision is unchanged by this work.
`campaigns-registry` is **additive**: it is a separate contract for a
different use case (many campaigns sharing one instance), not a
replacement. Existing `milestonex-campaign` deployments continue to work
exactly as before; nothing in `campaign/` was modified for this feature.

Because this is the addition of a third contract, final rollout (which
instance(s) get deployed where, and whether it becomes the recommended path
for new multi-campaign integrations) should go through the same roadmap
review the original issue asked for — this PR delivers a complete, tested
implementation as the starting point for that review, not a unilateral
architecture decision.

## API

| Function | Description |
|---|---|
| `create_campaign(creator, metadata) -> u64` | Creates a campaign, returns its ID (assigned sequentially starting at 1). |
| `donate(donor, campaign_id, amount, asset)` | Donates to one campaign, identified by ID. |
| `end_campaign(campaign_id)` | Ends a campaign early. Creator-only. |
| `cancel_campaign(campaign_id)` | Cancels a campaign. Creator-only. |
| `get_campaign(campaign_id) -> Option<CampaignRecord>` | Full campaign record, or `None` if the ID is unknown. |
| `get_campaign_status(campaign_id) -> CampaignStatusResponse` | Status plus computed `days_remaining`. |
| `campaign_count() -> u64` | Total campaigns created on this instance. |

`CampaignMetadata` (the `create_campaign` input) carries `goal_amount`,
`end_time`, `accepted_assets`, and `min_donation_amount` — the same
per-campaign configuration `milestonex-campaign::initialize` takes, minus
milestones (see [Scope](#scope-of-this-version)).

## Storage scoping

Every read and write is keyed by campaign ID:

- `DataKey::Campaign(id)` — one `CampaignRecord` per campaign.
- `DataKey::DonorData(id, donor_address)` — one `DonorRecord` per
  (campaign, donor) pair.
- `DataKey::NextCampaignId` — the only instance-wide piece of state; a
  counter that assigns IDs and doubles as `campaign_count()`.

Because every operation loads and stores only the `CampaignRecord` (and, for
`donate`, the one relevant `DonorRecord`) for the ID it was given, operations
on one campaign cannot read or mutate another's state. This is exercised
directly in `campaigns-registry/src/test/concurrent_campaigns_tests.rs`,
which creates 5 campaigns and asserts distinct IDs, independent
`raised_amount`/`donation_count` counters, and independent statuses
(`Active`, `GoalReached`, `Ended`, `Cancelled` all coexisting in one
instance).

## `MAX_CAMPAIGNS_PER_INSTANCE`

`create_campaign` panics with `Error::MaxCampaignsExceeded` once
`campaign_count()` reaches `campaigns_registry::MAX_CAMPAIGNS_PER_INSTANCE`
(currently `10_000`). This is a `pub const`, not configurable per-instance.
It exists to keep future instance-wide operations (e.g. a paginated listing
endpoint) tractable — it is not a resource limit Soroban itself imposes.
Once an instance is full, deploy a new contract instance for further
campaigns.

## Migration plan

**New campaigns:** create them via `campaigns-registry::create_campaign`
against one shared contract instance, instead of deploying and initializing
a fresh `milestonex-campaign` instance per campaign.

**Existing `milestonex-campaign` deployments:** there is no automatic
on-chain migration, and this is intentional rather than an oversight.
Soroban has no primitive for trustlessly moving persistent storage between
two different contract addresses — building one specifically to move
donor balances and campaign state would itself be new, security-sensitive
custody logic, which is out of proportion to what issue #44 asked for.
Concretely:

- Already-deployed `milestonex-campaign` instances keep working exactly as
  they do today. Donors and creators of existing campaigns are unaffected.
- A campaign creator who wants their *next* campaign to live in the shared
  registry calls `campaigns-registry::create_campaign` directly — this is
  a new campaign, not a transfer of an existing one's state or funds.
- There is no path to move an in-progress `milestonex-campaign` campaign's
  raised funds or donor history into `campaigns-registry` mid-lifecycle.
  If that capability is ever needed, it should be scoped as its own
  security-reviewed issue rather than folded into this one.

This is the "hardfork documented" option named in the acceptance criteria:
`campaigns-registry` is a parallel system going forward, not an in-place
upgrade of existing instances.

## Scope of this version

Implemented, matching issue #44's acceptance criteria exactly:
`create_campaign`, `donate`, creator-driven status transitions
(`end_campaign`/`cancel_campaign`, automatic `Active` → `GoalReached` on
donation), ID-scoped reads/writes, `MAX_CAMPAIGNS_PER_INSTANCE`, and an
error space that numerically matches `milestonex-campaign::types::Error`
wherever the two contracts share a concept (see the doc comment on
`campaigns-registry::types::Error`).

**Not implemented, deferred to follow-up issues** — none of these are in
issue #44's acceptance criteria, and each is substantial enough to deserve
its own scoped review rather than being bundled in:

- **Milestones and milestone release.** No `MilestoneData`, no
  `release_milestone`. Registry campaigns track only aggregate
  `raised_amount`.
- **Refunds.** No `claim_refund`; no pro-rata per-asset accounting, since
  that exists in `milestonex-campaign` specifically to support refunds.
- **Multi-asset release**, since there is no release logic at all yet.
- **Freeze / upgrade controls** and the **reentrancy lock** that
  `milestonex-campaign` has. Soroban transactions are already atomic, and
  the registry's per-campaign scoping means one campaign's `donate` cannot
  observe another campaign's in-flight state; the reentrancy lock in
  `milestonex-campaign` is defense-in-depth for cross-contract call chains,
  which registry v1 doesn't yet have a reason to guard against. Revisit if
  a future version adds external token transfers or cross-contract calls.
- **Diagnostics** (`diag` feature / `CampaignMetrics`).
- **Real SEP-41 token custody.** Like `milestonex-campaign::donate`,
  `donate` here updates internal accounting only and does not transfer
  tokens into contract custody. This mirrors an existing, documented
  limitation of the canonical contract — it is not a new limitation
  introduced here.

## Known simplifications vs. `milestonex-campaign`

- **Native/XLM acceptance check.** `milestonex-campaign` resolves `Native`
  donations to a hardcoded "canonical" wrapped-XLM address (documented in
  its own source as a testnet-only placeholder, pending real per-network
  configuration). Since `campaigns-registry` doesn't do token transfers,
  there's no token address to resolve — it accepts a `Native` donation
  whenever the campaign's `accepted_assets` includes an entry with
  `asset_code == "XLM"`, regardless of issuer. Once real token transfers
  are added to the registry, this should be revisited alongside
  `milestonex-campaign`'s own placeholder.
- **No per-asset raised breakdown.** `milestonex-campaign` tracks
  `AssetRaised` per token to support pro-rata refunds. Registry v1 has no
  refunds, so it only tracks the aggregate `raised_amount` per campaign.

## Type unification note

`milestonex-common` defines a `CampaignStatus` and `AssetInfo`, and its own
doc comment describes them as "canonical definitions ... used by both
campaign and core contracts." In the current codebase this isn't accurate:
`campaign/src/types.rs` defines its own `CampaignStatus` (with a
`GoalReached` state `common`'s doesn't have) and its own `AssetInfo` (an
enum selecting `Native` vs. a token address; `common`'s is a
`{code, issuer}` struct of `u32`s). `campaign/Cargo.toml` depends on
`common`, but nothing in `campaign/src` actually imports it.

`campaigns-registry` follows the same practical path `campaign` already
does: local types matching real on-chain behavior, rather than the
aspirational shared crate. It does not depend on `milestonex-common`. This
drift is exactly what issue #44's "Issue 9 (type unification)" dependency
gestures at — at the time of writing, GitHub issue #9 in this repository is
about `donate`'s per-milestone read loop, not type unification, so that
cross-reference in #44 doesn't resolve to an actual tracked issue here.
Flagging it in case a type-unification issue needs to be filed separately.

## Backward compatibility

No existing contract, type, or public function was changed. `campaign/`,
`common/`, `crates/contracts/core/`, `crates/tools/`, and `token-bridge/`
are untouched by this PR. `campaigns-registry` is a new workspace member
and a new, independently deployed contract.
