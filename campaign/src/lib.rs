#![no_std]

pub mod storage;
pub mod types;

use soroban_sdk::{contract, contractimpl, Env, Vec};
use types::{AssetInfo, CampaignData, CampaignStatus, Error, MilestoneData};
use storage::{get_campaign, set_campaign, set_milestone};

pub const VERSION: u32 = 1;

#[contract]
pub struct CampaignContract;

#[contractimpl]
impl CampaignContract {
    /// Initialize a new campaign with strict validation on all inputs.
    ///
    /// # Panics
    /// - `Error::InvalidGoalAmount` if goal_amount <= 0
    /// - `Error::InvalidEndTime` if end_time <= current ledger timestamp
    /// - `Error::InvalidAssets` if accepted_assets is empty
    /// - `Error::InvalidMilestones` if milestones are not sorted ascending by target_amount
    /// - `Error::MilestoneMismatch` if last milestone.target_amount != goal_amount
    pub fn initialize(
        env: Env,
        creator: soroban_sdk::Address,
        goal_amount: i128,
        end_time: u64,
        accepted_assets: Vec<AssetInfo>,
        milestones: Vec<MilestoneData>,
    ) -> Result<(), Error> {
        // Validation 1: goal_amount > 0
        if goal_amount <= 0 {
            panic_with_error(&env, Error::InvalidGoalAmount);
        }

        // Validation 2: end_time > current ledger timestamp
        let current_timestamp = env.ledger().timestamp();
        if end_time <= current_timestamp {
            panic_with_error(&env, Error::InvalidEndTime);
        }

        // Validation 3: accepted_assets non-empty
        if accepted_assets.is_empty() {
            panic_with_error(&env, Error::InvalidAssets);
        }

        // Validation 4 & 5: milestones sorted ascending and last == goal_amount
        if !milestones.is_empty() {
            validate_milestones(&env, &milestones, goal_amount)?;
        }

        // All validations passed, store campaign data
        let campaign = CampaignData {
            creator,
            goal_amount,
            raised_amount: 0,
            end_time,
            status: CampaignStatus::Active,
            accepted_assets,
            milestone_count: milestones.len() as u32,
        };

        set_campaign(&env, &campaign);

        // Store each milestone
        for (index, milestone) in milestones.iter().enumerate() {
            set_milestone(&env, index as u32, milestone);
        }

        Ok(())
    }

    pub fn hello(env: Env) -> soroban_sdk::Symbol {
        soroban_sdk::Symbol::new(&env, "campaign")
    }

    pub fn version() -> u32 {
        VERSION
    }
}

/// Helper function to validate milestone conditions
fn validate_milestones(
    env: &Env,
    milestones: &Vec<MilestoneData>,
    goal_amount: i128,
) -> Result<(), Error> {
    // Check if milestones are sorted ascending by target_amount
    for i in 1..milestones.len() {
        let prev = &milestones.get(i - 1).unwrap();
        let current = &milestones.get(i).unwrap();

        if prev.target_amount >= current.target_amount {
            panic_with_error(env, Error::InvalidMilestones);
        }
    }

    // Check if last milestone.target_amount == goal_amount
    if let Some(last_milestone) = milestones.last() {
        if last_milestone.target_amount != goal_amount {
            panic_with_error(env, Error::MilestoneMismatch);
        }
    } else {
        panic_with_error(env, Error::InvalidMilestones);
    }

    Ok(())
}

/// Helper function to panic with a descriptive error message
fn panic_with_error(env: &Env, error: Error) -> ! {
    match error {
        Error::InvalidGoalAmount => {
            env.panic_with_error(soroban_sdk::Symbol::new(env, "InvalidGoalAmount"))
        }
        Error::InvalidEndTime => {
            env.panic_with_error(soroban_sdk::Symbol::new(env, "InvalidEndTime"))
        }
        Error::InvalidAssets => {
            env.panic_with_error(soroban_sdk::Symbol::new(env, "InvalidAssets"))
        }
        Error::InvalidMilestones => {
            env.panic_with_error(soroban_sdk::Symbol::new(env, "InvalidMilestones"))
        }
        Error::MilestoneMismatch => {
            env.panic_with_error(soroban_sdk::Symbol::new(env, "MilestoneMismatch"))
        }
    }
}

