use soroban_sdk::{Address, Env, Vec, token};
use crate::types::{Error, MilestoneStatus, StellarAsset};
use crate::storage::{get_campaign, get_milestone, set_milestone, storage_get_total_raised};

/// Issue #208 â€“ Multi-asset milestone release
///
/// Releases milestone funds proportionally across all accepted assets.
/// Each asset transferred separately; all succeed or all revert (atomic via Soroban).
/// Release amount per asset = (asset_raised / total_raised) * milestone_release_amount
pub fn release_milestone_multi_asset(env: &Env, milestone_index: u32, recipient: Address) {
    let campaign = get_campaign(env).unwrap_or_else(|| {
        soroban_sdk::panic_with_error!(env, Error::NotInitialized)
    });

    campaign.creator.require_auth();

    let mut milestone = get_milestone(env, milestone_index).unwrap_or_else(|| {
        soroban_sdk::panic_with_error!(env, Error::MilestoneNotFound)
    });

    if milestone.status != MilestoneStatus::Unlocked {
        soroban_sdk::panic_with_error!(env, Error::InvalidMilestoneTransition);
    }

    let total_raised = storage_get_total_raised(env);
    let milestone_release_amount = milestone.target_amount - milestone.released_amount;

    if total_raised > 0 {
        for asset in campaign.accepted_assets.iter() {
            if let Some(issuer) = asset.issuer.clone() {
                let token_client = token::Client::new(env, &issuer);
                let asset_raised = token_client.balance(&env.current_contract_address());
                // Proportional: (asset_raised / total_raised) * milestone_release_amount
                let asset_release = (asset_raised * milestone_release_amount) / total_raised;
                if asset_release > 0 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &recipient,
                        &asset_release,
                    );
                }
            }
        }
    }

    milestone.released_amount = milestone.target_amount;
    milestone.status = MilestoneStatus::Released;
    set_milestone(env, milestone_index, &milestone);

    env.events().publish(
        ("milestone", "multi_asset_released"),
        (milestone_index, milestone_release_amount, recipient),
    );
}
