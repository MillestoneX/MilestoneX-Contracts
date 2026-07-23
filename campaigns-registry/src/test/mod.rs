pub mod concurrent_campaigns_tests;
pub mod create_campaign_tests;
pub mod donate_tests;
pub mod status_transition_tests;

use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, String, Vec};

use crate::types::{CampaignMetadata, StellarAsset};

/// Pre-registered contract instance, ready for calls via
/// `env.as_contract(&fx.contract_id, || CampaignsRegistry::method(...))`.
///
/// `env.mock_all_auths()` has already been applied.
pub struct RegistryFixture {
    pub env: Env,
    pub contract_id: Address,
}

pub(crate) fn with_registry() -> RegistryFixture {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::CampaignsRegistry);
    RegistryFixture { env, contract_id }
}

/// Metadata for a campaign accepting native XLM, 1 day to the deadline,
/// no minimum donation.
pub(crate) fn default_metadata(env: &Env) -> CampaignMetadata {
    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(Address::generate(env)),
    });

    CampaignMetadata {
        goal_amount: 1_000,
        end_time: env.ledger().timestamp() + 86_400,
        accepted_assets: assets,
        min_donation_amount: 0,
    }
}

/// Advance the ledger timestamp to `timestamp`.
pub(crate) fn set_ledger_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set_timestamp(timestamp);
}
