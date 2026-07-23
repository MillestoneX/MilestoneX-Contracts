# Contract State Machine & Entrypoint Authorization Matrix

**Location:** `docs/state-machine.md`  
**Companion Docs:** `docs/events.md`, `docs/deployment.md`  
**Last Updated:** July 2026  
**Purpose:** Centralized reference for all contract state transitions, authorization rules, and mutation gates

---

## Overview

The MilestoneX campaign contract contains **11 mutating entrypoints** across three domains:

1. **Campaign Lifecycle** (5 functions)
2. **Donor Operations** (2 functions)
3. **Milestone Release** (2 functions)
4. **Admin Controls** (2 functions)

Each entrypoint enforces a consistent security posture:
- **Authorization gate** (creator or donor-specific `require_auth()`)
- **Freeze gate** (global contract freeze flag blocks all writes except freeze/unfreeze/initialize)
- **Status gate** (campaign state validation)
- **Reentrancy lock** (temporary storage lock for cross-contract calls)

This document maps every entrypoint to these gates for rapid security review and contributor onboarding.

---

## Entrypoint Authorization Matrix

| Entrypoint | Auth | Freeze Gate | Status Gate(s) | Reentrancy Lock | Storage Mutations | Events |
|---|---|---|---|---|---|---|
| **initialize** | Creator `require_auth()` | ❌ None | Campaign not initialized, valid goal/deadline/assets/milestones | ❌ None | ✅ Campaign + Milestones (persistent) | `campaign_initialized` |
| **donate** | Donor `require_auth()` | ✅ Yes | Campaign `Active` OR `GoalReached` + timestamp < deadline + amount ≥ min_donation | ✅ Yes | ✅ Campaign, TotalRaised, AssetRaised, DonorData, DonorAssetDonation, DonationCount, UniqueDonorCount, Milestones (auto-unlock) | `donation_received`, `milestone_unlocked` (per unlocked), `campaign_goal_reached` (if goal reached) |
| **claim_refund** | Donor `require_auth()` | ✅ Yes | Campaign `Cancelled` OR `Ended` (unpaid goals) + within 30-day refund window + donor record exists + refund not already claimed | ✅ Yes | ✅ DonorData (mark refunded), token transfers per asset | `refund_claimed`, `asset_refund` (per asset) |
| **end_campaign** | Creator `require_auth()` | ✅ Yes | Campaign initialized + status ∈ {`Active`, `GoalReached`} | ❌ None | ✅ Campaign status → `Ended`, concluded_at_ledger | `campaign_ended` |
| **cancel_campaign** | Creator `require_auth()` | ✅ Yes | Campaign initialized + status ∈ {`Active`, `GoalReached`, `Ended`} | ❌ None | ✅ Campaign status → `Cancelled`, concluded_at_ledger | `campaign_cancelled` |
| **extend_deadline** | Creator `require_auth()` | ✅ Yes | Campaign initialized + status ∈ {`Active`, `GoalReached`} + new deadline > now + new deadline ≤ now + 10 years | ❌ None | ✅ Campaign end_time | `deadline_extended` |
| **release_milestone** | Creator `require_auth()` | ✅ Yes | Campaign initialized + milestone index valid + status `Unlocked` + prior milestones `Released` | ✅ Yes | ✅ Milestone status → `Released`, released_amount, released_at, released_to, ReleaseCount | `milestone_released` (per asset) |
| **release_milestone_multi_asset** | Creator `require_auth()` | ✅ Yes | Campaign initialized + milestone index valid + status `Unlocked` + recipient ≠ contract + total_raised > 0 | ✅ Yes | ✅ Milestone status → `Released`, AssetRaised (per-asset decrements), TotalRaised, ReleaseCount | `milestone_released` (per asset) |
| **freeze** | Creator `require_auth()` | ❌ None | Campaign initialized | ❌ None | ✅ Frozen flag → true | `contract_frozen` |
| **unfreeze** | Creator `require_auth()` | ❌ None | Campaign initialized | ❌ None | ✅ Frozen flag → false | `contract_unfrozen` |
| **upgrade** | Creator `require_auth()` | ✅ Yes | Campaign initialized | ❌ None | ❌ None (WASM update managed by Soroban runtime) | `contract_upgraded` |

**Legend:**
- ✅ = Present / Required
- ❌ = Absent / Not applicable
- **Auth**: Authorization requirement (`require_auth()` caller)
- **Freeze Gate**: Blocked if `is_frozen(&env) == true`?
- **Status Gate**: Campaign state preconditions
- **Reentrancy Lock**: Temporary storage mutex to prevent cross-contract re-entrancy
- **Storage Mutations**: Which persistent or temporary storage keys are written
- **Events**: Events published to Stellar Horizon

---

## Campaign Status Transitions

```
                      ┌────────────────────┐
                      │    Initialize      │
                      │   (Creator auth)   │
                      └─────────┬──────────┘
                                │
                    ┌───────────▼──────────┐
                    │  Active (initial)    │
                    │  Accepts donations   │
                    └───────────┬──────────┘
                                │
                ┌───────────────┼──────────────────┐
                │               │                  │
                │               │ (donate reaches  │
                │               │  goal_amount)    │
                ▼               ▼                  │
        ┌──────────────┐  ┌──────────────┐        │
        │  Cancelled   │  │ GoalReached  │        │
        │  (Creator)   │  │ (Auto-trnsn) │        │
        │ Refund open  │  │ Accepts ∅    │        │
        │   30 days    │  │ donations    │        │
        └──────────────┘  └──────┬───────┘        │
                                 │                │
                     ┌───────────┴────────────┐   │
                     │ (deadline passes,      │   │
                     │  or end_campaign)      │   │
                     ▼                        ▼   ▼
                ┌──────────────┐         ┌──────────────┐
                │  Cancelled   │─────────│    Ended     │
                │              │ (admin) │              │
                │ Refund open  │         │ Refund open  │
                │              │         │ (30 days, if │
                │              │         │  goal unmet) │
                └──────────────┘         └──────────────┘

Terminal States: Ended, Cancelled
Accepting Donations: Active, GoalReached (until deadline)
Refund-Eligible: Cancelled (always), Ended (if goal not reached)
```

**Key Invariants:**
- `initialize` only succeeds before any prior initialization
- `cancel_campaign` always succeeds before terminal state is reached (can cancel from `Ended`)
- `end_campaign` only works while `Active` or `GoalReached`
- Deadline is enforced per-transaction: all mutations gate on `now < campaign.end_time`

---

## Milestone Status Transitions

```
Locked ─────────────► Unlocked ────────────► Released
         (auto-unlock             (explicit,
          on donation             creator
          reaching                only)
          target)
```

**Key Invariants:**
- Milestone auto-unlocks when `campaign.raised_amount >= milestone.target_amount`
- Manual release requires `release_milestone()` or `release_milestone_multi_asset()`
- Release order is enforced: previous milestone must be `Released` before current
- No skipping: attempting to release milestone N while milestone N-1 is `Unlocked` panics
- Milestone unlock is **idempotent**: re-reaching target does not re-emit `milestone_unlocked`

---

## Freeze Module Specification

**Location:** `campaign/src/storage.rs:355–375`

### Purpose
Global admin safety mechanism to block all contract mutations during emergency or upgrade scenarios.

### Data Storage
- **Key:** `DataKey::Frozen` (persistent, ~60-day TTL)
- **Type:** `bool`
- **Default:** `false` (contract not frozen until `freeze()` is called)

### Freeze Check Pattern
```rust
// Every mutating entrypoint (except freeze/unfreeze/initialize) includes:
if is_frozen(&env) {
    panic_with_error!(&env, Error::ContractFrozen);
}
```

This check fires **after** authorization but **before** any state mutations, ensuring:
1. Creator authorization is validated first (fail-fast on auth)
2. Freeze is checked second (fail-fast on global lock)
3. Business logic gates come last (status, reentrancy)

### Freeze Invariants
- ✅ **Freeze is "sticky"** while active — all writes rejected until `unfreeze()` is called
- ✅ **Freeze/unfreeze are self-operations** — can be called while frozen (otherwise admin locked out)
- ✅ **Initialize is never frozen** — contract can be initialized after being frozen
- ✅ **Atomicity** — freeze state written to persistent storage; survives ledger sequence
- ✅ **TTL managed** — automatically refreshed to ~60 days on each access

### Freeze Matrix
| Operation | Blocked When Frozen? | Rationale |
|---|---|---|
| initialize | ❌ No | Pre-initialization; contract state doesn't yet exist |
| donate | ✅ Yes | Mutation; prevent donations during emergency |
| claim_refund | ✅ Yes | Mutation; prevent refund claims during freeze |
| end_campaign | ✅ Yes | Mutation; campaign state locked |
| cancel_campaign | ✅ Yes | Mutation; campaign state locked |
| extend_deadline | ✅ Yes | Mutation; campaign metadata locked |
| release_milestone | ✅ Yes | Mutation; prevents unauthorized fund release |
| release_milestone_multi_asset | ✅ Yes | Mutation; prevents unauthorized fund release |
| freeze | ❌ No | Self-operation; must succeed to lock contract |
| unfreeze | ❌ No | Self-operation; must succeed to unlock contract |
| upgrade | ✅ Yes | Mutation (WASM update); prevent upgrade during freeze |

---

## Reentrancy Protection

**Location:** `campaign/src/storage.rs:300–313`

### Purpose
Prevent re-entrant call chains during cross-contract token transfers.

### Data Storage
- **Key:** `DataKey::ReentrancyLock` (temporary, ~7-day TTL)
- **Type:** `bool` (presence indicates lock held)
- **Behavior:** `acquire_lock()` panics with `Error::ReentrantCall` if already present; `release_lock()` removes the key

### Protected Entrypoints
1. **donate** — Protect donation + milestone unlock + storage updates
2. **claim_refund** — Protect multi-asset refund transfers  
3. **release_milestone** — Protect single-asset release
4. **release_milestone_multi_asset** — Protect multi-asset release + accounting updates

### Patterns
```rust
// Standard acquire-release pattern:
pub fn donate(env: Env, donor: Address, amount: i128, asset: AssetInfo) {
    acquire_lock(&env);  // ← Panics if already locked
    
    // ... authorization, status gates ...
    
    // ... token transfers, storage mutations ...
    
    release_lock(&env);  // ← Always called (explicit, not try-finally)
}
```

### Key Properties
- **Per-call mutex** — Lock is acquired at entry, released at exit
- **Temporary storage** — Auto-expires after ~7 days (contract lifetime is much longer, so not critical for cleanup)
- **Panic on conflict** — `Error::ReentrantCall` surfaced to caller
- **CEI pattern in multi-asset** — State written before transfers to prevent double-release via re-entrant call on same milestone

---

## Event Emission Checklist

All events are **immutable audit logs** published to Stellar Horizon. See `docs/events.md` for full event schema.

### Entrypoint → Events

| Entrypoint | Events Emitted | Conditions |
|---|---|---|
| initialize | `campaign_initialized` (1x) | Once per successful initialization |
| donate | `donation_received` (1x) | Every donation; `milestone_unlocked` (0..n) if milestones unlocked; `campaign_goal_reached` (1x) if goal reached on this call |
| claim_refund | `refund_claimed` (1x) + `asset_refund` (0..n) | 1x refund claimed per donor; 1x per asset refunded (multi-asset campaigns may emit multiple) |
| end_campaign | `campaign_ended` (1x) | Once per end call |
| cancel_campaign | `campaign_cancelled` (1x) | Once per cancel call |
| extend_deadline | `deadline_extended` (1x) | Once per extend call |
| release_milestone | `milestone_released` (1x) | Once per release; amount clamped to contract balance |
| release_milestone_multi_asset | `milestone_released` (0..n) | Per asset released (dust amounts < 1 base unit skipped) |
| freeze | `contract_frozen` (1x) | Once per freeze call |
| unfreeze | `contract_unfrozen` (1x) | Once per unfreeze call |
| upgrade | `contract_upgraded` (1x) | Once per upgrade call |

**Important:** Events are **idempotent** in the sense that re-calling a function may re-emit the same event structure, but downstream systems should use **transaction hash + event index** for deduplication, not just event topic.

---

## Authorization & Access Control

### Creator-Only Functions
The following functions require `campaign.creator.require_auth()`:
- `end_campaign`
- `cancel_campaign`
- `extend_deadline`
- `release_milestone` (via wrapper in `lib.rs`)
- `release_milestone_multi_asset` (via wrapper in `lib.rs`)
- `freeze`
- `unfreeze`
- `upgrade`

**Rationale:** Only the campaign creator should be able to control campaign lifecycle and fund release.

### Donor-Specific Functions
The following functions require the caller (donor) to call `require_auth()`:
- `donate` — Donor authorizes their donation
- `claim_refund` — Donor authorizes refund claim

**Rationale:** Individual donors control their own contributions and refund claims.

### Public/Read-Only Functions
The following functions require no authorization:
- All `get_*` view functions (campaign status, total raised, donor info, milestone info)

**Rationale:** These are read-only and do not mutate state.

### Authorization Check Order
Every mutating entrypoint follows this pattern:
1. **Authorization gate first** (fail-fast on invalid caller)
2. **Freeze gate second** (fail-fast on global lock)
3. **Status/business logic gates third** (campaign state validation)
4. **Storage mutations last** (only after all gates pass)

---

## Pre-Upgrade Contract Migration Checklist

When deploying a new contract WASM hash via `upgrade()`:

- [ ] **Freeze the contract** (call `freeze()` before initiating upgrade deployment)
  - Prevents concurrent donations/refunds during upgrade window
  - Gives operators a clean "pause" point for migration scripting

- [ ] **Verify all milestones released or campaign not Active**
  - If campaign is still `Active` or `GoalReached`, ensure no pending milestone releases
  - Optionally end campaign early via `end_campaign()`

- [ ] **Backup storage state** (operator responsibility, not contract-enforced)
  - Snapshot all persistent storage keys before deploying new WASM
  - Save campaign data, milestone records, donor records, and asset accounting

- [ ] **Test new WASM against storage schema**
  - Ensure new code can read old `DataKey` enum variants (backwards compatibility)
  - If schema changed, add migration logic in new `initialize_v2()` or equivalent

- [ ] **Deploy new WASM** (call `upgrade(new_wasm_hash)`)
  - Contract stays frozen during deployment; caller auth check still enforced
  - Soroban runtime updates the contract code atomically

- [ ] **Emit `contract_upgraded` event** (automatic on upgrade call)
  - Topics: `("campaign", "contract_upgraded")`
  - Data includes old admin address, new WASM hash, timestamp

- [ ] **Unfreeze the contract** (call `unfreeze()`)
  - Restore normal operations; donors can resume donations and refunds

- [ ] **Monitor first few operations post-upgrade**
  - Watch for any panic/error spikes in new code paths
  - Validate event emission matches expected schema in `docs/events.md`

---

## Storage & TTL Management

### Persistent Storage (Campaign Lifetime)
All persistent keys are bumped to **~60-day TTL** on every access:

| Key | Type | Bumped On | Purpose |
|---|---|---|---|
| `CampaignData` | Struct | Every read/write | Campaign metadata + status |
| `MilestoneData(u32)` | Vec | Per-milestone ops | Milestone targets, release status |
| `DonorData(Address)` | Struct | Donation, refund | Donor contribution history |
| `TotalRaised` | i128 | Donation, release | Global funding counter |
| `AssetRaised(Address)` | i128 | Donation, release | Per-asset funding (for proportional release math) |
| `DonorAssetDonation(Address, Address)` | i128 | Donation | Per-donor per-asset contribution (for pro-rata refunds) |
| `DonationCount` | u64 | Donation | Total donations accepted |
| `UniqueDonorCount` | u32 | Donation | Distinct donor count |
| `ReleaseCount` | u64 | Release | Total releases completed |
| `Frozen` | bool | Freeze/unfreeze | Global freeze flag |

### Temporary Storage (Short-lived)
Temporary keys have **~7-day TTL** and are used for transient state:

| Key | Type | Expires After | Purpose |
|---|---|---|---|
| `ReentrancyLock` | bool | ~7 days | Per-call mutex (released after function exit) |
| `ContractStatus` | u32 | ~7 days | Transient campaign status flag (rarely used) |

---

## Testing Entrypoints by Issue Number

Each entrypoint is tracked in the GitHub issue system. Test files in `campaign/src/test/` validate both success and failure paths:

| Entrypoint | Primary Issue(s) | Test File(s) |
|---|---|---|
| initialize | #175, #194 | `integration_tests.rs` (success), `negative_path_tests.rs` (validation) |
| donate | #194, #195, #198, #242, #243 | `integration_tests.rs`, `invariant_tests.rs` |
| claim_refund | #211, #242, #243 | `claim_refund_tests.rs`, `refund_eligibility_tests.rs` |
| end_campaign | #212, #243 | `concluded_ledger_tests.rs`, `negative_path_tests.rs` |
| cancel_campaign | #214, #243 | `concluded_ledger_tests.rs`, `negative_path_tests.rs` |
| extend_deadline | #215, #243 | `negative_path_tests.rs` |
| release_milestone | #207, #242, #244 | `release_milestone_tests.rs` |
| release_milestone_multi_asset | #208, #242, #244 | `release_milestone_tests.rs` |
| freeze | #246 | `negative_path_tests.rs` (freeze guard tests) |
| unfreeze | #246 | `negative_path_tests.rs` (freeze guard tests) |
| upgrade | #246 | `negative_path_tests.rs` (freeze guard tests) |

---

## Inline Documentation Cross-References

All entrypoint function signatures in `campaign/src/lib.rs` and submodules include doc comments that reference this matrix:

```rust
/// Issue #207 – `release_milestone` function
///
/// See `docs/state-machine.md` for authorization matrix and freeze/reentrancy gates.
pub fn release_milestone(env: Env, milestone_index: u32, recipient: Address) { ... }
```

**Pattern:** Each entrypoint doc comment should include:
1. Issue number(s) for ownership tracking
2. One-line purpose statement
3. **Reference to `docs/state-machine.md` for detailed security posture**
4. Pre/post conditions (panics)

---

## Quick Reference: "Who can do what, when?"

### Campaign Active & Goal Not Reached
| Actor | Can... | Cannot... |
|---|---|---|
| Any donor | Donate (auth required) | End campaign, release milestones, claim refund |
| Creator | End campaign, cancel campaign, extend deadline, release milestones, freeze/unfreeze, upgrade | Donate (different caller) |

### Campaign GoalReached (Goal Met)
| Actor | Can... | Cannot... |
|---|---|---|
| Any donor | Donate until deadline (auth required) | Claim refund, release milestones |
| Creator | End campaign, cancel campaign, extend deadline, release milestones, freeze/unfreeze, upgrade | Donate |

### Campaign Ended (Deadline Passed or Ended Early)
| Actor | Can... | Cannot... |
|---|---|---|
| Any donor | Claim refund (if within 30-day window + goal not met, auth required) | Donate, release milestones |
| Creator | Cancel campaign, freeze/unfreeze, upgrade | Extend deadline, release milestones (must end first) |

### Campaign Cancelled (Creator Action)
| Actor | Can... | Cannot... |
|---|---|---|
| Any donor | Claim refund (always, auth required) | Donate, release milestones |
| Creator | Freeze/unfreeze, upgrade | End campaign (already terminal), release milestones |

### Contract Frozen (Admin Lock)
| Actor | Can... | Cannot... |
|---|---|---|
| Any donor | (No mutations) | Donate, claim refund |
| Creator | Unfreeze (always), initialize new contract | Donate (as donor), release milestones, end/cancel campaign, extend deadline, upgrade |

---

## Appendix: Error Codes & Meanings

See `campaign/src/types.rs` for the canonical error enum. Key codes for state machine validation:

| Code | Name | Trigger | Recovery |
|---|---|---|---|
| 1 | `AlreadyInitialized` | `initialize()` called twice | Create a new contract instance |
| 2 | `NotInitialized` | Any mutation before `initialize()` | Call `initialize()` first |
| 3 | `Unauthorized` | Caller not creator/donor | Provide correct authority |
| 4 | `CampaignEnded` | Donation past deadline | Wait for refund window or retry before deadline |
| 5 | `CampaignNotActive` | Status not Active/GoalReached | Cancel campaign or end campaign first if intended |
| 20 | `InvalidMilestones` | Milestones not strictly ascending | Initialize with valid milestone sequence |
| 22 | `InvalidCampaignTransition` | Invalid status state change | Transition only follows valid path (see diagram) |
| 50 | `RefundNotPermitted` | Campaign not Cancelled/Ended or goal met | End or cancel campaign first |
| 60 | `ReentrantCall` | Re-entrant mutation detected | Likely cross-contract call issue; retry transaction |
| 80 | `ContractFrozen` | Contract is frozen | Call `unfreeze()` to re-enable mutations |

---

## References

- **docs/events.md** — Event schema and topic structure
- **docs/deployment.md** — Deployment workflow and bootstrap steps
- **campaign/src/types.rs** — Error enum and state definitions
- **campaign/src/storage.rs** — Storage layer and freeze/lock implementation
- **campaign/src/lib.rs** — Entrypoint definitions and test module
