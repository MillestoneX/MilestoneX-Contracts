#![no_std]
use soroban_sdk::{Address, Env, String};

/// Event emitted when a donation is received
/// 
/// # Fields
/// * `donor` - The address of the donor
/// * `amount` - The amount donated
/// * `asset` - The asset type donated
/// * `project_id` - The project ID this donation is mapped to
/// * `timestamp` - The timestamp of the donation
#[derive(Clone)]
pub struct DonationReceived {
    pub donor: Address,
    pub amount: i128,
    pub asset: String,
    pub project_id: String,
    pub timestamp: u64,
}

/// Event emitted when a withdrawal is processed
/// 
/// # Fields
/// * `recipient` - The address receiving the withdrawal
/// * `amount` - The amount withdrawn
/// * `asset` - The asset type withdrawn
/// * `timestamp` - The timestamp of the withdrawal
#[derive(Clone)]
pub struct WithdrawalProcessed {
    pub recipient: Address,
    pub amount: i128,
    pub asset: String,
    pub timestamp: u64,
}

/// Event emitted when a donation is rejected due to duplicate transaction
/// 
/// # Fields
/// * `tx_hash` - The duplicate transaction hash
/// * `reason` - The rejection reason
/// * `timestamp` - When the duplicate was detected
#[derive(Clone)]
pub struct DonationRejected {
    pub tx_hash: String,
    pub reason: String,
    pub timestamp: u64,
}

impl DonationReceived {
    /// Emit the DonationReceived event to the ledger
    /// 
    /// # Topics (indexed for querying)
    /// - donor: Address of the donor
    /// - project_id: Project ID for grouping donations
    /// 
    /// # Data (full event payload)
    /// - donor: Address of the donor
    /// - amount: Amount donated  
    /// - asset: Asset type donated
    /// - project_id: Project ID this donation is mapped to
    /// - timestamp: When the donation was received
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.donor.clone(), self.project_id.clone()),
            (self.donor.clone(), self.amount, self.asset.clone(), self.project_id.clone(), self.timestamp),
        );
    }
}

impl WithdrawalProcessed {
    /// Emit the WithdrawalProcessed event to the ledger
    /// 
    /// # Topics (indexed for querying)
    /// - recipient: Address of the recipient
    /// - amount: Amount withdrawn
    /// 
    /// # Data (full event payload)
    /// - recipient: Address of the recipient
    /// - amount: Amount withdrawn
    /// - asset: Asset type withdrawn
    /// - timestamp: When the withdrawal was processed
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.recipient.clone(), self.amount),
            (self.recipient.clone(), self.amount, self.asset.clone(), self.timestamp),
        );
    }
}

impl DonationRejected {
    /// Emit the DonationRejected event to the ledger
    /// 
    /// # Topics (indexed for querying)
    /// - tx_hash: The rejected transaction hash
    /// 
    /// # Data (full event payload)
    /// - tx_hash: The duplicate transaction hash
    /// - reason: Rejection reason
    /// - timestamp: When the rejection occurred
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.tx_hash.clone(),),
            (self.tx_hash.clone(), self.reason.clone(), self.timestamp),
        );
    }
}

/// Event type identifier for DonationReceived
/// Used by indexers to identify this event type
pub const EVENT_DONATION_RECEIVED: &[u8] = b"donation_received";

/// Event type identifier for WithdrawalProcessed  
/// Used by indexers to identify this event type
pub const EVENT_WITHDRAWAL_PROCESSED: &[u8] = b"withdrawal_processed";

/// Event type identifier for DonationRejected
/// Used by indexers to identify this event type
pub const EVENT_DONATION_REJECTED: &[u8] = b"donation_rejected";
