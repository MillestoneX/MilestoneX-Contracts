use super::{default_metadata, with_registry};
use crate::storage;
use crate::CampaignsRegistry;
use soroban_sdk::{testutils::Address as _, Address, String, Vec};

#[test]
fn test_create_campaign_assigns_id_one_to_first_campaign() {
    let fx = with_registry();
    let env = &fx.env;
    let creator = Address::generate(env);
    let metadata = default_metadata(env);

    let id = env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata)
    });

    assert_eq!(id, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_campaign_zero_goal_amount_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let creator = Address::generate(env);
    let mut metadata = default_metadata(env);
    metadata.goal_amount = 0;

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_create_campaign_past_end_time_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let creator = Address::generate(env);
    let mut metadata = default_metadata(env);
    metadata.end_time = env.ledger().timestamp();

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_create_campaign_empty_accepted_assets_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let creator = Address::generate(env);
    let mut metadata = default_metadata(env);
    metadata.accepted_assets = Vec::new(env);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_create_campaign_empty_asset_code_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let creator = Address::generate(env);
    let mut metadata = default_metadata(env);
    let mut assets = Vec::new(env);
    assets.push_back(crate::types::StellarAsset {
        asset_code: String::from_str(env, ""),
        issuer: Some(Address::generate(env)),
    });
    metadata.accepted_assets = assets;

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn test_create_campaign_beyond_max_campaigns_per_instance_panics() {
    let fx = with_registry();
    let env = &fx.env;

    // Directly seed the counter at the cap rather than calling
    // `create_campaign` 10,000 times, which would make this test
    // prohibitively slow without testing anything the boundary check
    // itself doesn't already cover.
    env.as_contract(&fx.contract_id, || {
        storage::set_next_campaign_id(env, crate::MAX_CAMPAIGNS_PER_INSTANCE);
    });

    let creator = Address::generate(env);
    let metadata = default_metadata(env);
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::create_campaign(env.clone(), creator, metadata);
    });
}
