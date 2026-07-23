use super::{default_metadata, set_ledger_timestamp, with_registry};
use crate::types::AssetInfo;
use crate::CampaignsRegistry;
use soroban_sdk::{testutils::Address as _, Address};

fn create_default_campaign(env: &soroban_sdk::Env, contract_id: &Address) -> u64 {
    let creator = Address::generate(env);
    let metadata = default_metadata(env);
    env.as_contract(contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata)
    })
}

#[test]
fn test_donate_updates_raised_amount_and_donation_count() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);
    let donor = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_id, 250, AssetInfo::Native);
    });

    let campaign = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_id)
        })
        .expect("campaign should exist");
    assert_eq!(campaign.raised_amount, 250);
    assert_eq!(campaign.donation_count, 1);
}

#[test]
fn test_donate_transitions_to_goal_reached_when_goal_met() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id); // goal_amount = 1_000
    let donor = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_id, 1_000, AssetInfo::Native);
    });

    let campaign = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_id)
        })
        .expect("campaign should exist");
    assert_eq!(campaign.status, crate::types::CampaignStatus::GoalReached);
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn test_donate_unknown_campaign_id_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let donor = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, 999, 100, AssetInfo::Native);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_donate_after_cancel_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_id);
    });

    let donor = Address::generate(env);
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_id, 100, AssetInfo::Native);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_donate_after_deadline_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    // default_metadata sets end_time = now + 86_400.
    set_ledger_timestamp(env, env.ledger().timestamp() + 86_401);

    let donor = Address::generate(env);
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_id, 100, AssetInfo::Native);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_donate_zero_amount_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);
    let donor = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_id, 0, AssetInfo::Native);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_donate_unaccepted_stellar_asset_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);
    let donor = Address::generate(env);
    let random_token = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(
            env.clone(),
            donor,
            campaign_id,
            100,
            AssetInfo::Stellar(random_token),
        );
    });
}

#[test]
fn test_donate_scoped_to_one_campaign_leaves_sibling_campaign_untouched() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_a = create_default_campaign(env, &fx.contract_id);
    let campaign_b = create_default_campaign(env, &fx.contract_id);
    let donor = Address::generate(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::donate(env.clone(), donor, campaign_a, 500, AssetInfo::Native);
    });

    let a = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_a)
        })
        .unwrap();
    let b = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_b)
        })
        .unwrap();

    assert_eq!(a.raised_amount, 500);
    assert_eq!(b.raised_amount, 0);
    assert_eq!(b.donation_count, 0);
}
