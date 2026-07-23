//! MilestoneX multi-campaign registry contract.
//!
//! `milestonex-campaign` is single-campaign-per-instance by design (see the
//! root `README.md`'s "Contract Canonicalization" section). This contract
//! is additive: it supports many campaigns in one instance, each addressed
//! by a `u64` campaign ID returned from `create_campaign`. It does not
//! replace or modify `milestonex-campaign`, which remains deployed and
//! canonical for the single-campaign use case.
//!
//! See `docs/multi-campaign.md` for the migration plan and current scope
//! limitations relative to the single-campaign contract (no milestones,
//! refunds, multi-asset release, or freeze/upgrade controls in this first
//! version).

#![no_std]
// `Events::publish` is deprecated in soroban-sdk 26.x in favour of the
// `#[contractevent]` macro; see `event.rs` for the same suppression
// `campaign/src/event.rs` already uses.
#![allow(deprecated)]

pub mod contract;
pub mod event;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use types::{AssetInfo, CampaignMetadata, CampaignRecord, CampaignStatusResponse};

pub use contract::MAX_CAMPAIGNS_PER_INSTANCE;

#[contract]
pub struct CampaignsRegistry;

#[contractimpl]
impl CampaignsRegistry {
    /// Create a new campaign; returns its assigned ID.
    pub fn create_campaign(env: Env, creator: Address, metadata: CampaignMetadata) -> u64 {
        contract::create_campaign(&env, creator, metadata)
    }

    /// Donate to the campaign identified by `campaign_id`.
    pub fn donate(env: Env, donor: Address, campaign_id: u64, amount: i128, asset: AssetInfo) {
        contract::donate(&env, donor, campaign_id, amount, asset)
    }

    /// End a campaign early. Requires creator authorization.
    pub fn end_campaign(env: Env, campaign_id: u64) {
        contract::end_campaign(&env, campaign_id)
    }

    /// Cancel a campaign. Requires creator authorization.
    pub fn cancel_campaign(env: Env, campaign_id: u64) {
        contract::cancel_campaign(&env, campaign_id)
    }

    /// Read a campaign's full record. Returns `None` if `campaign_id` is unknown.
    pub fn get_campaign(env: Env, campaign_id: u64) -> Option<CampaignRecord> {
        storage::get_campaign(&env, campaign_id)
    }

    /// Get a campaign's current status with computed `days_remaining`.
    pub fn get_campaign_status(env: Env, campaign_id: u64) -> CampaignStatusResponse {
        contract::get_campaign_status(&env, campaign_id)
    }

    /// Total number of campaigns created on this instance (highest assigned ID).
    pub fn campaign_count(env: Env) -> u64 {
        storage::next_campaign_id(&env)
    }
}
