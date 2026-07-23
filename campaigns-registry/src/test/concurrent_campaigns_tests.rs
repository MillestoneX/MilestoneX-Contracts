//! Acceptance-criteria coverage for issue #44: 5+ concurrent campaigns in
//! one contract instance, with distinct IDs, counters, and statuses.

use super::{default_metadata, with_registry};
use crate::types::{AssetInfo, CampaignStatus};
use crate::CampaignsRegistry;
use soroban_sdk::{testutils::Address as _, Address};

#[test]
fn test_five_concurrent_campaigns_get_distinct_sequential_ids() {
    let fx = with_registry();
    let env = &fx.env;

    let mut ids: [u64; 5] = [0; 5];
    for id in ids.iter_mut() {
        let creator = Address::generate(env);
        let metadata = default_metadata(env);
        *id = env.as_contract(&fx.contract_id, || {
            CampaignsRegistry::create_campaign(env.clone(), creator, metadata)
        });
    }

    assert_eq!(ids, [1, 2, 3, 4, 5]);
    assert_eq!(
        env.as_contract(&fx.contract_id, || CampaignsRegistry::campaign_count(env.clone())),
        5
    );
}

#[test]
fn test_five_concurrent_campaigns_keep_distinct_creators_and_raised_counters() {
    let fx = with_registry();
    let env = &fx.env;

    // `core::array::from_fn` builds a fixed-size array from a generator
    // closure without requiring `Address: Copy` or `Default` (which it
    // isn't) — no growable `Vec` needed in this `#![no_std]` crate's tests.
    let campaigns: [(Address, u64); 5] = core::array::from_fn(|_| {
        let creator = Address::generate(env);
        let metadata = default_metadata(env);
        let id = env.as_contract(&fx.contract_id, || {
            CampaignsRegistry::create_campaign(env.clone(), creator.clone(), metadata)
        });
        (creator, id)
    });

    // Donate a distinct amount to each campaign.
    for (i, (_, campaign_id)) in campaigns.iter().enumerate() {
        let donor = Address::generate(env);
        let amount: i128 = 100 * (i as i128 + 1);
        env.as_contract(&fx.contract_id, || {
            CampaignsRegistry::donate(env.clone(), donor, *campaign_id, amount, AssetInfo::Native);
        });
    }

    // Each campaign's counters reflect only its own donation — donating to
    // campaign N must not perturb any other campaign's raised_amount,
    // donation_count, or creator.
    for (i, (creator, campaign_id)) in campaigns.iter().enumerate() {
        let campaign = env
            .as_contract(&fx.contract_id, || {
                CampaignsRegistry::get_campaign(env.clone(), *campaign_id)
            })
            .expect("campaign should exist");
        assert_eq!(campaign.raised_amount, 100 * (i as i128 + 1));
        assert_eq!(campaign.donation_count, 1);
        assert_eq!(&campaign.creator, creator);
    }
}

#[test]
fn test_five_concurrent_campaigns_have_independent_statuses() {
    let fx = with_registry();
    let env = &fx.env;

    let mut ids: [u64; 5] = [0; 5];
    for id in ids.iter_mut() {
        let creator = Address::generate(env);
        let metadata = default_metadata(env);
        *id = env.as_contract(&fx.contract_id, || {
            CampaignsRegistry::create_campaign(env.clone(), creator, metadata)
        });
    }

    // Campaign 0: fully funded -> GoalReached.
    let donor = Address::generate(env);
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, ids[0], 1_000, AssetInfo::Native);
    });
    // Campaign 1: ended early by its creator.
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::end_campaign(env.clone(), ids[1]);
    });
    // Campaign 2: cancelled by its creator.
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), ids[2]);
    });
    // Campaigns 3 and 4 are left untouched — still Active.

    let status_of = |id: u64| -> CampaignStatus {
        env.as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), id)
        })
        .expect("campaign should exist")
        .status
    };

    assert_eq!(status_of(ids[0]), CampaignStatus::GoalReached);
    assert_eq!(status_of(ids[1]), CampaignStatus::Ended);
    assert_eq!(status_of(ids[2]), CampaignStatus::Cancelled);
    assert_eq!(status_of(ids[3]), CampaignStatus::Active);
    assert_eq!(status_of(ids[4]), CampaignStatus::Active);
}
