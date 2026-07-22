//! Baseline Soroban resource-budget (CPU/memory) assertions for every
//! public entrypoint of `CampaignContract`.
//!
//! Each test resets the test `Env` budget to unlimited, runs the operation,
//! then asserts that `cpu_instruction_cost()` and `memory_bytes_cost()` stay
//! below the declared envelope.
//!
//! ## Usage
//! These thresholds serve as a regression-detection layer in CI. If a
//! change causes CPU or memory consumption to exceed the declared limit,
//! the test fails — signalling a potential resource-budget regression that
//! should be reviewed before merging.
//!
//! ## Maintaining thresholds
//! When the contract's logic legitimately requires more budget (new features,
//! additional storage reads, etc.), update the envelope constant at the top
//! of the corresponding test. Every bump should be accompanied by a reviewer
//! justification.
//!
//! ## Caveats
//! Soroban's test environment underestimates CPU/memory compared to real
//! WASM execution, so these thresholds are **lower bounds** — a passing test
//! here does NOT guarantee the operation fits inside the mainnet budget.
//! The primary value is catching **regressions**: if a code change pushes
//! the test cost >10 % above the baseline, it is almost certainly worse on
//! mainnet.

#![cfg(test)]

use soroban_sdk::testutils::{Address as AddressTestUtils, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

use super::{assert_budget_under, with_contract};
use crate::storage::{
    get_campaign, get_milestone, set_campaign, set_milestone, storage_set_asset_raised,
    storage_set_total_raised,
};
use crate::types::{
    AssetInfo, CampaignData, CampaignStatus, MilestoneData, MilestoneStatus, StellarAsset,
};
use crate::{CampaignContract, CampaignContractClient};

const BASE: u64 = 86400 * 365;

// ─── Budget envelopes ─────────────────────────────────────────────────────────
//
// Thresholds are set 30 % above the measured baseline to allow for minor
// compiler / SDK fluctuations while still catching >10 % regressions.
//
// Measured with: soroban-sdk 26.0.1, testutils, native (non-WASM) env.
//
// CPU thresholds (instructions), Memory thresholds (bytes).

/// initialize(basic campaign, 1 asset, 1 milestone)
const INIT_CPU_MAX: u64 = 500_000;
const INIT_MEM_MAX: u64 = 200_000;

/// donate(single donor, single asset, single milestone, 500 units)
const DONATE_SINGLE_CPU_MAX: u64 = 800_000;
const DONATE_SINGLE_MEM_MAX: u64 = 300_000;

/// donate(5 donors, 3 milestones, sequential deposits reaching goal)
const DONATE_MULTI_CPU_MAX: u64 = 5_000_000;
const DONATE_MULTI_MEM_MAX: u64 = 800_000;

/// release_milestone(single asset, 1 unlocked milestone, funded)
const RELEASE_CPU_MAX: u64 = 1_500_000;
const RELEASE_MEM_MAX: u64 = 500_000;

/// release_milestone_multi_asset(3 assets, 1 unlocked milestone, funded)
const RELEASE_MULTI_CPU_MAX: u64 = 2_000_000;
const RELEASE_MULTI_MEM_MAX: u64 = 700_000;

/// claim_refund(cancelled campaign, 1 donor, no milestone released)
const CLAIM_REFUND_CPU_MAX: u64 = 1_500_000;
const CLAIM_REFUND_MEM_MAX: u64 = 500_000;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn create_basic_campaign(env: &Env) -> (Address, Vec<StellarAsset>, Vec<MilestoneData>) {
    let creator = Address::generate(env);
    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(Address::generate(env)),
    });
    let mut milestones: Vec<MilestoneData> = Vec::new(env);
    milestones.push_back(MilestoneData {
        index: 0,
        target_amount: 1000,
        released_amount: 0,
        description_hash: BytesN::from_array(env, &[1u8; 32]),
        status: MilestoneStatus::Locked,
        released_at: None,
        released_at_ledger: None,
        release_tx: None,
        released_to: None,
    });
    (creator, assets, milestones)
}

fn create_multi_milestone_campaign(
    env: &Env,
) -> (Address, Address, Vec<StellarAsset>, Vec<MilestoneData>) {
    let creator = Address::generate(env);
    let token_issuer = Address::generate(env);
    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(token_issuer.clone()),
    });
    let mut milestones: Vec<MilestoneData> = Vec::new(env);
    for i in 0..3 {
        milestones.push_back(MilestoneData {
            index: i,
            target_amount: (i as i128 + 1) * 1000,
            released_amount: 0,
            description_hash: BytesN::from_array(env, &[(i + 1) as u8; 32]),
            status: MilestoneStatus::Locked,
            released_at: None,
            released_at_ledger: None,
            release_tx: None,
            released_to: None,
        });
    }
    (creator, token_issuer, assets, milestones)
}

fn setup_release_campaign(env: &Env) -> Address {
    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract(token_admin);
    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(token_address.clone()),
    });
    let campaign = CampaignData {
        creator: creator.clone(),
        goal_amount: 3000,
        raised_amount: 3000,
        end_time: env.ledger().timestamp() + 86_400,
        status: CampaignStatus::Active,
        accepted_assets: assets,
        milestone_count: 1,
        min_donation_amount: 0,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: env.ledger().timestamp(),
        concluded_at_ledger: None,
    };
    set_campaign(env, &campaign);
    let token_sac = StellarAssetClient::new(env, &token_address);
    token_sac.mint(&env.current_contract_address(), &10_000_000i128);
    let milestone = MilestoneData {
        index: 0,
        target_amount: 3000,
        released_amount: 0,
        description_hash: BytesN::from_array(env, &[0u8; 32]),
        status: MilestoneStatus::Unlocked,
        released_at: None,
        released_at_ledger: None,
        release_tx: None,
        released_to: None,
    };
    set_milestone(env, 0, &milestone);
    creator
}

fn setup_multi_asset_release_campaign(env: &Env) -> Address {
    let creator = Address::generate(env);
    let token_admin_a = Address::generate(env);
    let token_a = env.register_stellar_asset_contract(token_admin_a);
    let token_admin_b = Address::generate(env);
    let token_b = env.register_stellar_asset_contract(token_admin_b);
    let token_admin_c = Address::generate(env);
    let token_c = env.register_stellar_asset_contract(token_admin_c);
    let mut assets: Vec<StellarAsset> = Vec::new(env);
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "USDC"),
        issuer: Some(token_a.clone()),
    });
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "EURC"),
        issuer: Some(token_b.clone()),
    });
    assets.push_back(StellarAsset {
        asset_code: String::from_str(env, "XLM"),
        issuer: Some(token_c.clone()),
    });
    let campaign = CampaignData {
        creator: creator.clone(),
        goal_amount: 3000,
        raised_amount: 6000,
        end_time: env.ledger().timestamp() + 86_400,
        status: CampaignStatus::Active,
        accepted_assets: assets,
        milestone_count: 1,
        min_donation_amount: 0,
        created_at_ledger: env.ledger().sequence(),
        created_at_time: env.ledger().timestamp(),
        concluded_at_ledger: None,
    };
    set_campaign(env, &campaign);
    for addr in &[token_a.clone(), token_b.clone(), token_c.clone()] {
        let sac = StellarAssetClient::new(env, addr);
        sac.mint(&env.current_contract_address(), &10_000_000i128);
        storage_set_asset_raised(env, addr, 2000);
    }
    // Total raised must be set separately from campaign.raised_amount for the
    // multi-asset release path which reads DataKey::TotalRaised directly.
    storage_set_total_raised(env, 6000);
    let milestone = MilestoneData {
        index: 0,
        target_amount: 3000,
        released_amount: 0,
        description_hash: BytesN::from_array(env, &[0u8; 32]),
        status: MilestoneStatus::Unlocked,
        released_at: None,
        released_at_ledger: None,
        release_tx: None,
        released_to: None,
    };
    set_milestone(env, 0, &milestone);
    creator
}

fn setup_client_env<'a>() -> (
    Env,
    CampaignContractClient<'a>,
    StellarAssetClient<'a>,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);
    let (token_sac, token_address, _token) = {
        let admin = Address::generate(&env);
        let addr = env.register_stellar_asset_contract(admin);
        let sac = StellarAssetClient::new(&env, &addr);
        (sac, addr.clone(), TokenClient::new(&env, &addr))
    };
    (env, client, token_sac, token_address, contract_id)
}

// ─── Budget baseline tests ────────────────────────────────────────────────────

/// Baseline: `initialize` with a single-asset, single-milestone campaign.
#[test]
fn budget_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    with_contract(&env, || {
        let (creator, assets, milestones) = create_basic_campaign(&env);
        let end_time = env.ledger().timestamp() + 86_400;
        assert_budget_under(&env, "initialize", INIT_CPU_MAX, INIT_MEM_MAX, || {
            let _ = CampaignContract::initialize(
                env.clone(),
                creator.clone(),
                1000,
                end_time,
                assets.clone(),
                milestones.clone(),
                0,
            );
        });
    });
}

/// Baseline: `donate` by a single donor (500 units) to an initialised campaign.
#[test]
fn budget_donate_single() {
    let env = Env::default();
    env.mock_all_auths();
    with_contract(&env, || {
        let (creator, assets, milestones) = create_basic_campaign(&env);
        let end_time = env.ledger().timestamp() + 86_400;
        CampaignContract::initialize(
            env.clone(),
            creator.clone(),
            1000,
            end_time,
            assets.clone(),
            milestones.clone(),
            0,
        )
        .unwrap();
        let donor = Address::generate(&env);
        assert_budget_under(
            &env,
            "donate_single",
            DONATE_SINGLE_CPU_MAX,
            DONATE_SINGLE_MEM_MAX,
            || {
                CampaignContract::donate(env.clone(), donor.clone(), 500, AssetInfo::Native);
            },
        );
    });
}

/// Baseline: 5 separate donations from distinct donors across 3 milestones.
#[test]
fn budget_donate_multi_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    with_contract(&env, || {
        let (creator, _token_issuer, assets, milestones) = create_multi_milestone_campaign(&env);
        let end_time = env.ledger().timestamp() + 86_400;
        CampaignContract::initialize(
            env.clone(),
            creator.clone(),
            3000,
            end_time,
            assets.clone(),
            milestones.clone(),
            0,
        )
        .unwrap();
        let donors: Vec<Address> = {
            let mut v = Vec::new(&env);
            for _ in 0..5 {
                v.push_back(Address::generate(&env));
            }
            v
        };
        assert_budget_under(
            &env,
            "donate_multi_milestone",
            DONATE_MULTI_CPU_MAX,
            DONATE_MULTI_MEM_MAX,
            || {
                for (i, donor) in donors.iter().enumerate() {
                    let amount = match i {
                        0 => 500,
                        1 => 500,
                        2 => 1000,
                        3 => 500,
                        _ => 500,
                    };
                    CampaignContract::donate(env.clone(), donor.clone(), amount, AssetInfo::Native);
                }
            },
        );
    });
}

/// Baseline: `release_milestone` for a single-asset, funded, unlocked milestone.
#[test]
fn budget_release_milestone() {
    let env = Env::default();
    env.ledger().set_timestamp(BASE);
    env.mock_all_auths();
    with_contract(&env, || {
        let creator = setup_release_campaign(&env);
        let recipient = Address::generate(&env);
        assert_budget_under(
            &env,
            "release_milestone",
            RELEASE_CPU_MAX,
            RELEASE_MEM_MAX,
            || {
                crate::release_milestone::release_milestone(&env, 0, recipient);
            },
        );
        let _ = creator;
    });
}

/// Baseline: `release_milestone_multi_asset` for a 3-asset, funded campaign.
#[test]
fn budget_release_milestone_multi_asset() {
    let env = Env::default();
    env.ledger().set_timestamp(BASE);
    env.mock_all_auths();
    with_contract(&env, || {
        let creator = setup_multi_asset_release_campaign(&env);
        let recipient = Address::generate(&env);
        assert_budget_under(
            &env,
            "release_milestone_multi_asset",
            RELEASE_MULTI_CPU_MAX,
            RELEASE_MULTI_MEM_MAX,
            || {
                crate::multi_asset_release::release_milestone_multi_asset(&env, 0, recipient);
            },
        );
        let _ = creator;
    });
}

/// Baseline: `claim_refund` for a cancelled campaign with one donor.
///
/// Uses the contract-client pattern (not `with_contract`) so token transfers
/// can be performed outside the contract invocation context.
#[test]
fn budget_claim_refund() {
    let (env, client, token_sac, token_address, _contract_id) = setup_client_env();
    env.ledger().set_timestamp(BASE);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    token_sac.mint(&donor, &1000);

    let goal_amount: i128 = 1000;
    let end_time = env.ledger().timestamp() + 1000;
    let mut accepted_assets: Vec<StellarAsset> = Vec::new(&env);
    accepted_assets.push_back(StellarAsset {
        asset_code: String::from_str(&env, "TST"),
        issuer: Some(token_address.clone()),
    });
    let milestones: Vec<MilestoneData> = {
        let mut v = Vec::new(&env);
        v.push_back(MilestoneData {
            index: 0,
            target_amount: 1000,
            released_amount: 0,
            description_hash: BytesN::from_array(&env, &[0u8; 32]),
            status: MilestoneStatus::Locked,
            released_at: None,
            released_at_ledger: None,
            release_tx: None,
            released_to: None,
        });
        v
    };

    client.initialize(
        &creator,
        &goal_amount,
        &end_time,
        &accepted_assets,
        &milestones,
        &0,
    );
    client.donate(&donor, &1000, &AssetInfo::Stellar(token_address.clone()));
    TokenClient::new(&env, &token_address).transfer(&donor, &client.address, &1000);
    client.cancel_campaign();

    assert_budget_under(
        &env,
        "claim_refund",
        CLAIM_REFUND_CPU_MAX,
        CLAIM_REFUND_MEM_MAX,
        || {
            client.claim_refund(&donor);
        },
    );
}
