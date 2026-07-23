// src/storage.rs
//
// All reads and writes are scoped by campaign ID via `DataKey::Campaign(id)`
// / `DataKey::DonorData(id, donor)`, so many campaigns share one contract
// instance without their state colliding.

use crate::types::{CampaignRecord, DataKey, DonorRecord, Error};
use soroban_sdk::{panic_with_error, Address, Env};

/// ~30 days — bump threshold: if remaining TTL < this, extend.
pub const PERSISTENT_BUMP_THRESHOLD: u32 = 518_400;

/// ~60 days — extend to this TTL when bumping persistent entries.
pub const PERSISTENT_BUMP_AMOUNT: u32 = 1_036_800;

/// Bump a persistent key's TTL if it is below the threshold. No-ops safely
/// when the key does not exist — `extend_ttl` panics on missing keys.
#[inline]
fn bump_persistent(env: &Env, key: &DataKey) {
    if env.storage().persistent().has(key) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}

// ─── Campaign ID counter ────────────────────────────────────────────────────

/// Next campaign ID to assign. Returns `0` before the first campaign is
/// created (so the first assigned ID is `1`); also usable as the
/// total-campaigns-created counter.
pub fn next_campaign_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextCampaignId)
        .unwrap_or(0)
}

pub fn set_next_campaign_id(env: &Env, id: u64) {
    env.storage().persistent().set(&DataKey::NextCampaignId, &id);
    bump_persistent(env, &DataKey::NextCampaignId);
}

// ─── Campaigns ───────────────────────────────────────────────────────────────

/// Store the campaign record, scoped by `campaign.id`.
pub fn set_campaign(env: &Env, campaign: &CampaignRecord) {
    let key = DataKey::Campaign(campaign.id);
    env.storage().persistent().set(&key, campaign);
    bump_persistent(env, &key);
}

/// Load a campaign record by ID and refresh its TTL. Returns `None` if no
/// campaign with this ID exists.
#[must_use]
pub fn get_campaign(env: &Env, id: u64) -> Option<CampaignRecord> {
    let key = DataKey::Campaign(id);
    let value = env.storage().persistent().get(&key)?;
    bump_persistent(env, &key);
    Some(value)
}

/// Same as `get_campaign` but panics with `CampaignNotFound` instead of
/// returning `None`.
#[must_use]
pub fn get_campaign_or_panic(env: &Env, id: u64) -> CampaignRecord {
    get_campaign(env, id).unwrap_or_else(|| panic_with_error!(env, Error::CampaignNotFound))
}

// ─── Donors (scoped by campaign_id + donor address) ───────────────────────────

pub fn get_donor(env: &Env, campaign_id: u64, donor: &Address) -> Option<DonorRecord> {
    let key = DataKey::DonorData(campaign_id, donor.clone());
    let value = env.storage().persistent().get(&key)?;
    bump_persistent(env, &key);
    Some(value)
}

pub fn set_donor(env: &Env, campaign_id: u64, donor: &Address, record: &DonorRecord) {
    let key = DataKey::DonorData(campaign_id, donor.clone());
    env.storage().persistent().set(&key, record);
    bump_persistent(env, &key);
}
