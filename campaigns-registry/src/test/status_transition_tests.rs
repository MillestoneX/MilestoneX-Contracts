use super::{default_metadata, with_registry};
use crate::types::CampaignStatus;
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
fn test_end_campaign_transitions_active_to_ended() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::end_campaign(env.clone(), campaign_id);
    });

    let campaign = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_id)
        })
        .unwrap();
    assert_eq!(campaign.status, CampaignStatus::Ended);
    assert!(campaign.concluded_at_ledger.is_some());
}

#[test]
fn test_cancel_after_end_is_allowed() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::end_campaign(env.clone(), campaign_id);
    });
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_id);
    });

    let campaign = env
        .as_contract(&fx.contract_id, || {
            CampaignsRegistry::get_campaign(env.clone(), campaign_id)
        })
        .unwrap();
    assert_eq!(campaign.status, CampaignStatus::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_cancel_after_cancel_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_id);
    });
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_id);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_end_after_cancel_panics() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_id = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_id);
    });
    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::end_campaign(env.clone(), campaign_id);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn test_end_campaign_unknown_id_panics() {
    let fx = with_registry();
    let env = &fx.env;

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::end_campaign(env.clone(), 999);
    });
}

#[test]
fn test_status_transition_on_one_campaign_does_not_affect_sibling() {
    let fx = with_registry();
    let env = &fx.env;
    let campaign_a = create_default_campaign(env, &fx.contract_id);
    let campaign_b = create_default_campaign(env, &fx.contract_id);

    env.as_contract(&fx.contract_id, || {
        CampaignsRegistry::cancel_campaign(env.clone(), campaign_a);
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

    assert_eq!(a.status, CampaignStatus::Cancelled);
    assert_eq!(b.status, CampaignStatus::Active);
}
