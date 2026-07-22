#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Env, String};

use super::with_contract;
use crate::types::{CampaignMetrics, StellarAsset};
use crate::CampaignContract;

fn setup_basic_env(env: &Env) {
    env.mock_all_auths();
    with_contract(env, || {
        let creator = soroban_sdk::Address::generate(env);
        let mut assets: soroban_sdk::Vec<StellarAsset> = soroban_sdk::Vec::new(env);
        assets.push_back(StellarAsset {
            asset_code: String::from_str(env, "XLM"),
            issuer: Some(soroban_sdk::Address::generate(env)),
        });
        let mut milestones: soroban_sdk::Vec<crate::types::MilestoneData> =
            soroban_sdk::Vec::new(env);
        milestones.push_back(crate::types::MilestoneData {
            index: 0,
            target_amount: 1000,
            released_amount: 0,
            description_hash: soroban_sdk::BytesN::from_array(env, &[1u8; 32]),
            status: crate::types::MilestoneStatus::Locked,
            released_at: None,
            released_at_ledger: None,
            release_tx: None,
            released_to: None,
        });

        CampaignContract::initialize(
            env.clone(),
            creator,
            1000,
            env.ledger().timestamp() + 86400,
            assets,
            milestones,
            0,
        )
        .unwrap();
    });
}

#[test]
fn test_metrics_view_returns_defaults_before_any_ops() {
    let env = Env::default();
    setup_basic_env(&env);
    with_contract(&env, || {
        let metrics = CampaignContract::metrics_view(env.clone());
        assert_eq!(metrics.donations_total, 0);
        assert_eq!(metrics.milestones_released_total, 0);
        assert_eq!(metrics.refunds_total, 0);
    });
}

#[test]
fn test_emit_diagnostics_does_not_panic() {
    let env = Env::default();
    setup_basic_env(&env);
    with_contract(&env, || {
        CampaignContract::emit_diagnostics(env.clone());
    });
}

#[test]
fn test_metrics_view_returns_struct() {
    let env = Env::default();
    let metrics = CampaignMetrics::default();
    assert_eq!(metrics.donations_total, 0);
    assert_eq!(metrics.milestones_released_total, 0);
    assert_eq!(metrics.refunds_total, 0);
    assert_eq!(metrics.last_diagnostics_ledger, 0);
}
