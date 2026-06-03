#![cfg(test)]

use soroban_sdk::testutils::Address as AddressTestUtils;
use soroban_sdk::{Address, Env, Vec, String, BytesN};

use crate::types::{CampaignStatus, CampaignData, StellarAsset, MilestoneStatus, MilestoneData};
use crate::storage::set_campaign;
use crate::CampaignContract;

fn make_env() -> Env {
    Env::default()
}

fn setup_active_campaign(env: &Env) {
    let creator = Address::generate(env);
    let campaign = CampaignData {
        creator: creator.clone(),
        goal_amount: 1000,
        raised_amount: 0,
        end_time: env.ledger().timestamp() + 100_000,
        status: CampaignStatus::Active,
        accepted_assets: {
            let mut assets: Vec<StellarAsset> = Vec::new(env);
            assets.push_back(StellarAsset {
                asset_code: String::from_str(env, "XLM"),
                issuer: Some(Address::generate(env)),
            });
            assets
        },
        milestone_count: 1,
        min_donation_amount: 0,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: env.ledger().timestamp(),
        concluded_at_ledger: None,
    };
    set_campaign(env, &campaign);
}

#[test]
fn returns_active_status() {
    let env = make_env();
    setup_active_campaign(&env);

    let result = CampaignContract::get_campaign_status(env.clone());

    assert_eq!(result.status, CampaignStatus::Active);
    assert!(result.days_remaining > 0);
}

#[test]
fn returns_ended_status() {
    let env = make_env();
    let creator = Address::generate(&env);
    let campaign = CampaignData {
        creator: creator.clone(),
        goal_amount: 1000,
        raised_amount: 0,
        end_time: env.ledger().timestamp() - 1,
        status: CampaignStatus::Ended,
        accepted_assets: {
            let mut assets: Vec<StellarAsset> = Vec::new(&env);
            assets.push_back(StellarAsset {
                asset_code: String::from_str(&env, "XLM"),
                issuer: Some(Address::generate(&env)),
            });
            assets
        },
        milestone_count: 1,
        min_donation_amount: 0,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: env.ledger().timestamp(),
        concluded_at_ledger: None,
    };
    set_campaign(&env, &campaign);

    let result = CampaignContract::get_campaign_status(env.clone());
    assert_eq!(result.status, CampaignStatus::Ended);
}

#[test]
fn returns_cancelled_status() {
    let env = make_env();
    let creator = Address::generate(&env);
    let campaign = CampaignData {
        creator: creator.clone(),
        goal_amount: 1000,
        raised_amount: 0,
        end_time: env.ledger().timestamp() + 100_000,
        status: CampaignStatus::Cancelled,
        accepted_assets: {
            let mut assets: Vec<StellarAsset> = Vec::new(&env);
            assets.push_back(StellarAsset {
                asset_code: String::from_str(&env, "XLM"),
                issuer: Some(Address::generate(&env)),
            });
            assets
        },
        milestone_count: 1,
        min_donation_amount: 0,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: env.ledger().timestamp(),
        concluded_at_ledger: None,
    };
    set_campaign(&env, &campaign);

    let result = CampaignContract::get_campaign_status(env.clone());
    assert_eq!(result.status, CampaignStatus::Cancelled);
}

#[test]
fn calculates_days_remaining() {
    let env = make_env();
    setup_active_campaign(&env);

    let result = CampaignContract::get_campaign_status(env.clone());
    assert!(result.days_remaining > 0);
}
