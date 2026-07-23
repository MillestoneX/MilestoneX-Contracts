// `Events::publish` is deprecated in soroban-sdk 26.x in favour of the
// `#[contractevent]` macro. Migrating is tracked as a follow-up, matching
// the same suppression already used in `campaign/src/event.rs`.
#![allow(deprecated)]

use crate::types::CampaignStatus;
use soroban_sdk::{Address, Env, Symbol};

/// Emitted by `create_campaign`.
pub fn campaign_created(env: &Env, campaign_id: u64, creator: &Address) {
    let topics = (
        Symbol::new(env, "campaign_created"),
        env.current_contract_address(),
    );
    env.events().publish(topics, (campaign_id, creator));
}

/// Emitted by `donate`.
pub fn donation_received(
    env: &Env,
    campaign_id: u64,
    donor: &Address,
    amount: i128,
    new_raised_total: i128,
) {
    let topics = (
        Symbol::new(env, "donation_received"),
        env.current_contract_address(),
    );
    env.events()
        .publish(topics, (campaign_id, donor, amount, new_raised_total));
}

/// Emitted by donation-driven and explicit campaign status transitions.
pub fn campaign_status_changed(
    env: &Env,
    campaign_id: u64,
    from: CampaignStatus,
    to: CampaignStatus,
) {
    let topics = (
        Symbol::new(env, "campaign_status_changed"),
        env.current_contract_address(),
    );
    env.events().publish(topics, (campaign_id, from, to));
}
