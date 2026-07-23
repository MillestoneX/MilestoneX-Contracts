// src/types.rs

use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env, String, Vec};

// ─── Error enum ───────────────────────────────────────────────────────────────

/// Canonical typed error codes for the campaigns-registry contract.
///
/// Codes are stable — never renumber an existing variant; only append new
/// ones. Wherever this contract shares a concept with the single-campaign
/// `milestonex-campaign` contract (see `campaign/src/types.rs::Error`), the
/// discriminant is kept numerically identical so off-chain indexers can use
/// one error lookup table across both contracts. `CampaignNotFound` (27) and
/// `MaxCampaignsExceeded` (28) are new — they fill a gap in campaign's error
/// space (26 `StorageWriteError` .. 30 `InvalidRecipient`) that campaign
/// itself doesn't use, so no collision is introduced if the two error spaces
/// are ever merged.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The campaign deadline has already passed.
    CampaignEnded = 4,
    /// Operation requires the campaign to be `Active` or `GoalReached`.
    CampaignNotActive = 5,
    /// Donated asset is not in the campaign's accepted assets list.
    AssetNotAccepted = 6,
    /// Donation amount is <= 0, or below the campaign's minimum threshold.
    DonationTooSmall = 7,
    /// `goal_amount` must be strictly positive.
    InvalidGoalAmount = 13,
    /// `end_time` must be strictly greater than the current ledger timestamp.
    InvalidEndTime = 14,
    /// A checked arithmetic operation overflowed.
    Overflow = 17,
    /// `accepted_assets` must be non-empty.
    InvalidAssets = 18,
    /// `asset_code` must be non-empty and <= 12 characters (Stellar limit).
    InvalidAssetCode = 19,
    /// The requested campaign status transition is not permitted.
    InvalidCampaignTransition = 22,
    /// No campaign exists with the given ID.
    CampaignNotFound = 27,
    /// `MAX_CAMPAIGNS_PER_INSTANCE` has been reached; no more campaigns can
    /// be created on this contract instance.
    MaxCampaignsExceeded = 28,
}

impl Error {
    /// Returns the stable on-chain wire code for this error variant.
    pub fn as_wire_code(self) -> u32 {
        self as u32
    }
}

/// Canonical wire-code table for every variant of `Error`.
pub const WIRE_CODE_TABLE: &[(Error, u32)] = &[
    (Error::CampaignEnded, 4),
    (Error::CampaignNotActive, 5),
    (Error::AssetNotAccepted, 6),
    (Error::DonationTooSmall, 7),
    (Error::InvalidGoalAmount, 13),
    (Error::InvalidEndTime, 14),
    (Error::Overflow, 17),
    (Error::InvalidAssets, 18),
    (Error::InvalidAssetCode, 19),
    (Error::InvalidCampaignTransition, 22),
    (Error::CampaignNotFound, 27),
    (Error::MaxCampaignsExceeded, 28),
];

#[cfg(test)]
mod error_code_tests {
    #[test]
    fn error_discriminants_are_unique() {
        let codes = super::WIRE_CODE_TABLE;
        for (index, (_, code)) in codes.iter().enumerate() {
            assert!(
                !codes[index + 1..].iter().any(|(_, c)| c == code),
                "Duplicate wire code {} at index {}",
                code,
                index,
            );
        }
    }

    #[test]
    fn as_wire_code_matches_table() {
        for (variant, expected_code) in super::WIRE_CODE_TABLE {
            let actual = variant.as_wire_code();
            assert_eq!(
                actual, *expected_code,
                "as_wire_code() mismatch for {:?}: expected {}, got {}",
                variant, expected_code, actual,
            );
        }
    }

    #[test]
    fn wire_code_table_is_sorted() {
        let codes = super::WIRE_CODE_TABLE;
        for i in 1..codes.len() {
            assert!(
                codes[i - 1].1 <= codes[i].1,
                "WIRE_CODE_TABLE not sorted at index {}: {} > {}",
                i,
                codes[i - 1].1,
                codes[i].1,
            );
        }
    }
}

// ─── Campaign lifecycle ───────────────────────────────────────────────────────

/// Campaign status with documented transition rules.
///
/// Identical state machine to `campaign::types::CampaignStatus`, kept as a
/// local type rather than a shared import — see `docs/multi-campaign.md` for
/// why this contract does not depend on `milestonex-common`'s (currently
/// unused, drifted) `CampaignStatus` definition.
///
/// ```text
/// Active ──► GoalReached ──► Ended ──► Cancelled
///   │              │           ▲
///   └──────────────┴───────────┘  (deadline passes in any non-terminal state)
///   │
///   └──► Cancelled  (creator at any point before Ended)
/// ```
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    /// Campaign is open and accepting donations.
    Active,
    /// Goal amount reached; still accepting donations until deadline.
    GoalReached,
    /// Deadline passed or campaign concluded normally.
    Ended,
    /// Creator cancelled the campaign; terminal state.
    Cancelled,
}

impl CampaignStatus {
    /// Returns `true` when donations are accepted.
    pub fn accepts_donations(self) -> bool {
        matches!(self, Self::Active | Self::GoalReached)
    }
}

/// Validate a proposed status transition; mirrors
/// `campaign::validate_campaign_transition` exactly.
pub fn validate_campaign_transition(
    env: &Env,
    current_status: &CampaignStatus,
    next_status: &CampaignStatus,
) -> Result<(), Error> {
    match (current_status, next_status) {
        (CampaignStatus::Active, CampaignStatus::GoalReached) => Ok(()),
        (CampaignStatus::Active, CampaignStatus::Ended) => Ok(()),
        (CampaignStatus::Active, CampaignStatus::Cancelled) => Ok(()),
        (CampaignStatus::GoalReached, CampaignStatus::Ended) => Ok(()),
        (CampaignStatus::GoalReached, CampaignStatus::Cancelled) => Ok(()),
        (CampaignStatus::Ended, CampaignStatus::Cancelled) => Ok(()),
        _ => panic_with_error!(env, Error::InvalidCampaignTransition),
    }
}

// ─── Asset types ──────────────────────────────────────────────────────────────

/// A Stellar asset descriptor accepted by a campaign.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StellarAsset {
    /// IETF-style asset code (e.g. `"XLM"`, `"USDC"`, `"EURC"`).
    pub asset_code: String,
    /// Token contract address. `None` only for native XLM.
    pub issuer: Option<Address>,
}

impl StellarAsset {
    /// Returns `true` when the asset code is non-empty and <= 12 bytes.
    #[must_use]
    pub fn has_valid_code(&self) -> bool {
        let len = self.asset_code.len();
        len > 0 && len <= 12
    }
}

/// Donation asset selector passed by the donor at call time.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetInfo {
    Native,
    Stellar(Address),
}

// ─── Campaign metadata (create_campaign input) ────────────────────────────────

/// Creation parameters for a new campaign. Passed to `create_campaign`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignMetadata {
    /// Total funding target in stroops / base units.
    pub goal_amount: i128,
    /// UNIX timestamp (seconds) after which new donations are rejected.
    pub end_time: u64,
    /// Ordered list of accepted tokens; must be non-empty.
    pub accepted_assets: Vec<StellarAsset>,
    /// Donations below this amount are rejected. `0` disables the check.
    pub min_donation_amount: i128,
}

// ─── Campaign record (per-ID storage) ─────────────────────────────────────────

/// Per-campaign configuration and runtime state.
///
/// Stored under `DataKey::Campaign(id)` in persistent storage, scoped by
/// campaign ID so many campaigns coexist in one contract instance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignRecord {
    /// Unique campaign ID, assigned sequentially starting at 1.
    pub id: u64,
    /// Address that created the campaign and holds creator privileges.
    pub creator: Address,
    pub goal_amount: i128,
    pub raised_amount: i128,
    pub end_time: u64,
    pub status: CampaignStatus,
    pub accepted_assets: Vec<StellarAsset>,
    pub min_donation_amount: i128,
    pub created_at_ledger: u32,
    pub created_at_time: u64,
    pub concluded_at_ledger: Option<u32>,
    /// Total number of accepted donation calls for this campaign.
    pub donation_count: u64,
}

// ─── Donor record (per campaign, per donor) ───────────────────────────────────

/// Aggregate donation record for a single donor address, scoped to one
/// campaign ID.
///
/// Stored under `DataKey::DonorData(campaign_id, donor_address)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DonorRecord {
    pub donor: Address,
    pub total_donated: i128,
    pub donation_count: u32,
    pub last_donation_time: u64,
    pub last_donation_ledger: u32,
}

// ─── Storage keys ─────────────────────────────────────────────────────────────

/// All persistent storage keys.
///
/// Rule: never remove or renumber variants — doing so silently changes the
/// XDR discriminant and breaks existing on-chain data. Only append new
/// variants.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Next campaign ID to assign; doubles as the total-created counter.
    NextCampaignId,
    /// Campaign record, scoped by ID.
    Campaign(u64),
    /// Donor record, scoped by (campaign_id, donor_address).
    DonorData(u64, Address),
}

// ─── Read-only response types ─────────────────────────────────────────────────

/// Response type for `get_campaign_status`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignStatusResponse {
    pub status: CampaignStatus,
    /// Negative means the deadline has passed.
    pub days_remaining: i64,
}
