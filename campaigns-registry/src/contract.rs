//! Campaign registry business logic, wired into the contract impl in
//! `lib.rs` as thin entrypoint methods on `CampaignsRegistry`.
//!
//! Every function here takes a `campaign_id` and scopes its reads/writes to
//! that one campaign via `storage::get_campaign_or_panic` /
//! `storage::set_campaign`, so operations on one campaign never affect
//! another.

use crate::event;
use crate::storage::{get_campaign_or_panic, get_donor, next_campaign_id, set_campaign, set_donor, set_next_campaign_id};
use crate::types::{
    validate_campaign_transition, AssetInfo, CampaignMetadata, CampaignRecord, CampaignStatus,
    CampaignStatusResponse, DonorRecord, Error,
};
use soroban_sdk::{panic_with_error, Address, Env, String};

/// Maximum number of campaigns a single contract instance will create.
///
/// Bounds per-instance growth so instance-wide operations (e.g. a future
/// paginated listing endpoint) stay tractable. `create_campaign` panics with
/// `Error::MaxCampaignsExceeded` once reached; deploy a new contract
/// instance to create more.
pub const MAX_CAMPAIGNS_PER_INSTANCE: u64 = 10_000;

/// Create a new campaign, returning its assigned ID (starting at 1).
///
/// # Panics
/// - `Error::InvalidGoalAmount` if `metadata.goal_amount <= 0`
/// - `Error::InvalidEndTime` if `metadata.end_time` <= current ledger timestamp
/// - `Error::InvalidAssets` if `metadata.accepted_assets` is empty
/// - `Error::InvalidAssetCode` if any accepted asset has an invalid code
/// - `Error::MaxCampaignsExceeded` if `MAX_CAMPAIGNS_PER_INSTANCE` is reached
pub fn create_campaign(env: &Env, creator: Address, metadata: CampaignMetadata) -> u64 {
    creator.require_auth();

    if metadata.goal_amount <= 0 {
        panic_with_error!(env, Error::InvalidGoalAmount);
    }

    let now = env.ledger().timestamp();
    if metadata.end_time <= now {
        panic_with_error!(env, Error::InvalidEndTime);
    }

    if metadata.accepted_assets.is_empty() {
        panic_with_error!(env, Error::InvalidAssets);
    }
    for asset in metadata.accepted_assets.iter() {
        if !asset.has_valid_code() {
            panic_with_error!(env, Error::InvalidAssetCode);
        }
    }

    let next_id = next_campaign_id(env);
    if next_id >= MAX_CAMPAIGNS_PER_INSTANCE {
        panic_with_error!(env, Error::MaxCampaignsExceeded);
    }
    let id = next_id + 1;

    let campaign = CampaignRecord {
        id,
        creator: creator.clone(),
        goal_amount: metadata.goal_amount,
        raised_amount: 0,
        end_time: metadata.end_time,
        status: CampaignStatus::Active,
        accepted_assets: metadata.accepted_assets,
        min_donation_amount: metadata.min_donation_amount,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: now,
        concluded_at_ledger: None,
        donation_count: 0,
    };

    set_campaign(env, &campaign);
    set_next_campaign_id(env, id);

    event::campaign_created(env, id, &creator);

    id
}

/// Returns `true` when `asset` is in `campaign.accepted_assets`.
fn is_asset_accepted(env: &Env, asset: &AssetInfo, campaign: &CampaignRecord) -> bool {
    match asset {
        AssetInfo::Stellar(addr) => campaign
            .accepted_assets
            .iter()
            .any(|a| a.issuer == Some(addr.clone())),
        AssetInfo::Native => {
            let xlm_code = String::from_str(env, "XLM");
            campaign
                .accepted_assets
                .iter()
                .any(|a| a.asset_code == xlm_code)
        }
    }
}

/// Donate to a specific campaign, identified by `campaign_id`.
///
/// Internal accounting only: like `milestonex-campaign::donate`, this
/// updates the campaign's recorded balance and does not itself perform a
/// SEP-41 token transfer into contract custody.
///
/// # Panics
/// - `Error::CampaignNotFound` if no campaign exists with this ID
/// - `Error::CampaignNotActive` unless status is `Active` or `GoalReached`
/// - `Error::CampaignEnded` if the campaign's deadline has passed
/// - `Error::DonationTooSmall` if `amount <= 0` or below `min_donation_amount`
/// - `Error::AssetNotAccepted` if `asset` is not in the campaign's accepted list
/// - `Error::Overflow` if `raised_amount` or a counter would overflow
pub fn donate(env: &Env, donor: Address, campaign_id: u64, amount: i128, asset: AssetInfo) {
    donor.require_auth();

    let mut campaign = get_campaign_or_panic(env, campaign_id);

    if !campaign.status.accepts_donations() {
        panic_with_error!(env, Error::CampaignNotActive);
    }
    if env.ledger().timestamp() >= campaign.end_time {
        panic_with_error!(env, Error::CampaignEnded);
    }
    if amount <= 0 || (campaign.min_donation_amount > 0 && amount < campaign.min_donation_amount) {
        panic_with_error!(env, Error::DonationTooSmall);
    }
    if !is_asset_accepted(env, &asset, &campaign) {
        panic_with_error!(env, Error::AssetNotAccepted);
    }

    campaign.raised_amount = campaign
        .raised_amount
        .checked_add(amount)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
    campaign.donation_count = campaign
        .donation_count
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));

    if campaign.status == CampaignStatus::Active && campaign.raised_amount >= campaign.goal_amount
    {
        campaign.status = CampaignStatus::GoalReached;
        event::campaign_status_changed(
            env,
            campaign_id,
            CampaignStatus::Active,
            CampaignStatus::GoalReached,
        );
    }

    set_campaign(env, &campaign);

    let mut donor_record = get_donor(env, campaign_id, &donor).unwrap_or_else(|| DonorRecord {
        donor: donor.clone(),
        total_donated: 0,
        donation_count: 0,
        last_donation_time: 0,
        last_donation_ledger: 0,
    });
    donor_record.total_donated = donor_record
        .total_donated
        .checked_add(amount)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
    donor_record.donation_count = donor_record
        .donation_count
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
    donor_record.last_donation_time = env.ledger().timestamp();
    donor_record.last_donation_ledger = env.ledger().sequence();
    set_donor(env, campaign_id, &donor, &donor_record);

    event::donation_received(env, campaign_id, &donor, amount, campaign.raised_amount);
}

/// End a campaign early (before its deadline). Requires creator authorization.
///
/// # Panics
/// - `Error::CampaignNotFound` if no campaign exists with this ID
/// - `Error::InvalidCampaignTransition` if the campaign is already `Cancelled`
pub fn end_campaign(env: &Env, campaign_id: u64) {
    let mut campaign = get_campaign_or_panic(env, campaign_id);
    campaign.creator.require_auth();

    let from = campaign.status;
    validate_campaign_transition(env, &from, &CampaignStatus::Ended)
        .unwrap_or_else(|e| panic_with_error!(env, e));

    campaign.status = CampaignStatus::Ended;
    campaign.concluded_at_ledger = Some(env.ledger().sequence());
    set_campaign(env, &campaign);

    event::campaign_status_changed(env, campaign_id, from, CampaignStatus::Ended);
}

/// Cancel a campaign. Requires creator authorization.
///
/// # Panics
/// - `Error::CampaignNotFound` if no campaign exists with this ID
/// - `Error::InvalidCampaignTransition` if the campaign is already `Cancelled`
pub fn cancel_campaign(env: &Env, campaign_id: u64) {
    let mut campaign = get_campaign_or_panic(env, campaign_id);
    campaign.creator.require_auth();

    let from = campaign.status;
    validate_campaign_transition(env, &from, &CampaignStatus::Cancelled)
        .unwrap_or_else(|e| panic_with_error!(env, e));

    campaign.status = CampaignStatus::Cancelled;
    campaign.concluded_at_ledger = Some(env.ledger().sequence());
    set_campaign(env, &campaign);

    event::campaign_status_changed(env, campaign_id, from, CampaignStatus::Cancelled);
}

/// Get a campaign's current status with computed `days_remaining`.
/// No auth required (read-only view).
///
/// # Panics
/// - `Error::CampaignNotFound` if no campaign exists with this ID
#[must_use]
pub fn get_campaign_status(env: &Env, campaign_id: u64) -> CampaignStatusResponse {
    let campaign = get_campaign_or_panic(env, campaign_id);

    let now = env.ledger().timestamp();
    let days_remaining = if now < campaign.end_time {
        ((campaign.end_time - now) / 86_400) as i64
    } else {
        -(((now - campaign.end_time) / 86_400) as i64)
    };

    CampaignStatusResponse {
        status: campaign.status,
        days_remaining,
    }
}
