use crate::event;
#[cfg(feature = "diag")]
use crate::storage::storage_increment_diagnostic_counter;
use crate::storage::{
    acquire_lock, get_campaign, get_milestone, is_frozen, release_lock, set_milestone,
    storage_increment_release_count,
};
#[cfg(feature = "diag")]
use crate::types::CampaignMetrics;
use crate::types::{Error, MilestoneStatus};
use soroban_sdk::{panic_with_error, token, Address, Env};

/// Issue #207 – `release_milestone` function
///
/// Releases funds for an unlocked milestone to the recipient using the primary
/// (first) accepted asset only.
///
/// **Precondition:** The caller (`#[contractimpl]` wrapper) MUST have already
/// verified `creator.require_auth()` before calling this function.
///
/// Validates milestone status is `Unlocked`.
/// Prevents double release — `Released` milestones panic with `MilestoneAlreadyReleased`.
/// Prevents skipping milestones — previous milestone must be Released.
/// Transfers the full release amount from the campaign's primary (first) accepted asset
/// to the recipient.
/// Sets milestone status to `Released`.
/// Emits `milestone_released` event.
/// Respects the freeze flag — panics with `ContractFrozen` if frozen.
/// Rejects multi-asset campaigns — panics with `UseMultiAssetRelease` so the caller
/// routes to `release_milestone_multi_asset`.
///
/// ## Use
///
/// ## Single-asset vs multi-asset
///
/// - Single-asset release: when the campaign accepts exactly one asset (`accepted_assets.len() == 1`). This is the legacy fast path; it transfers the milestone delta in full.
/// - Multi-asset release: when the campaign accepts more than one asset. This proportionally distributes across all assets.
///
/// Calling the wrong one is unidiomatic and will be rejected.
///
/// ## Security
///
/// Issue #242 – Reentrancy protection: acquires lock at entry, releases at exit.
/// Issue #244 – Balance verification: checks contract balance before each transfer.
///
/// # Panics
/// - `Error::NotInitialized` if campaign not initialized
/// - `Error::MilestoneNotFound` if milestone index is out of range
/// - `Error::InvalidMilestoneTransition` if milestone is not `Unlocked`
/// - `Error::PreviousMilestoneNotReleased` if a prior milestone is not yet Released
/// - `Error::MilestoneAlreadyReleased` if milestone is already in Released state
/// - `Error::UseMultiAssetRelease` if the campaign accepts more than one asset
/// - `Error::InsufficientContractBalance` if contract lacks funds for transfer
/// - `Error::ContractFrozen` if contract is frozen
pub fn release_milestone(env: &Env, milestone_index: u32, recipient: Address) {
    // Issue #242 – Reentrancy protection: acquire lock
    acquire_lock(env);

    let campaign =
        get_campaign(env).unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized));

    // Freeze check — reject all mutating operations while frozen
    if is_frozen(env) {
        soroban_sdk::panic_with_error!(env, Error::ContractFrozen);
    }

    // Multi-asset campaigns must use the proportional release path.
    if campaign.accepted_assets.len() > 1 {
        panic_with_error!(env, Error::UseMultiAssetRelease);
    }

    let mut milestone = get_milestone(env, milestone_index)
        .unwrap_or_else(|| panic_with_error!(env, Error::MilestoneNotFound));

    // Prevent double release: milestone already in Released state
    if milestone.status == MilestoneStatus::Released {
        soroban_sdk::panic_with_error!(env, Error::MilestoneAlreadyReleased);
    }

    // Prevent releasing locked milestones (must be Unlocked first)
    if milestone.status != MilestoneStatus::Unlocked {
        panic_with_error!(env, Error::InvalidMilestoneTransition);
    }

    // Prevent skipping milestones: if not milestone 0, previous must be Released
    if milestone_index > 0 {
        let prev_milestone = get_milestone(env, milestone_index - 1)
            .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, Error::MilestoneNotFound));
        if prev_milestone.status != MilestoneStatus::Released {
            soroban_sdk::panic_with_error!(env, Error::PreviousMilestoneNotReleased);
        }
    }

    let release_amount = milestone
        .target_amount
        .checked_sub(milestone.released_amount)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));

    let timestamp = env.ledger().timestamp();

    // Transfer from the primary (first) accepted asset only — releasing from
    // every accepted asset would multiply the payout by the asset count.
    // Campaigns with more than one accepted asset must use
    // `release_milestone_multi_asset`, which distributes proportionally.
    let asset = campaign
        .accepted_assets
        .first()
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized));

    if let Some(issuer) = asset.issuer.clone() {
        let token_client = token::Client::new(env, &issuer);

        // Issue #244 – Query actual contract balance for verification
        let asset_balance = token_client.balance(&env.current_contract_address());

        if asset_balance > 0 && release_amount > 0 {
            // Issue #244 – Verify contract balance is sufficient BEFORE transfer
            if asset_balance < release_amount {
                panic_with_error!(env, Error::InsufficientContractBalance);
            }

            // Clamp to available balance (should never be needed due to check above)
            let transfer_amount = release_amount.min(asset_balance);

            token_client.transfer(
                &env.current_contract_address(),
                &recipient,
                &transfer_amount,
            );

            event::milestone_released(
                env,
                milestone_index,
                transfer_amount,
                asset.asset_code.clone(),
                &recipient,
                timestamp,
            );
        }
    }

    milestone.released_amount = milestone.target_amount;
    milestone.status = MilestoneStatus::Released;
    milestone.released_at = Some(timestamp);
    milestone.released_to = Some(recipient);
    set_milestone(env, milestone_index, &milestone);
    storage_increment_release_count(env);
    #[cfg(feature = "diag")]
    storage_increment_diagnostic_counter(env, |m: &mut CampaignMetrics| {
        m.milestones_released_total += 1;
    });

    // Issue #242 – Release reentrancy lock
    release_lock(env);
}
