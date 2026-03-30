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

/// Event emitted when a multi-signature withdrawal proposal is created.
#[derive(Clone)]
pub struct WithdrawalProposalCreated {
    pub proposal_id: u64,
    pub proposer: Address,
    pub recipient: Address,
    pub amount: i128,
    pub asset: String,
    pub threshold: u32,
    pub expires_at: u64,
    pub timestamp: u64,
}

/// Event emitted when a proposal receives an approval.
#[derive(Clone)]
pub struct WithdrawalProposalApproved {
    pub proposal_id: u64,
    pub approver: Address,
    pub approval_count: u32,
    pub threshold: u32,
    pub timestamp: u64,
}

/// Event emitted when a proposal gets executed.
#[derive(Clone)]
pub struct WithdrawalProposalExecuted {
    pub proposal_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub asset: String,
    pub executed_by: Address,
    pub timestamp: u64,
}

/// Event emitted when a proposal is cancelled before execution.
#[derive(Clone)]
pub struct WithdrawalProposalCancelled {
    pub proposal_id: u64,
    pub cancelled_by: Address,
    pub timestamp: u64,
}

/// Event emitted when a pending proposal expires.
#[derive(Clone)]
pub struct WithdrawalProposalExpired {
    pub proposal_id: u64,
    pub timestamp: u64,
}

/// Event emitted whenever multisig config is changed.
#[derive(Clone)]
pub struct WithdrawalMultisigConfigUpdated {
    pub admin: Address,
    pub threshold: u32,
    pub single_sig_limit: i128,
    pub proposal_ttl_secs: u64,
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

impl WithdrawalProposalCreated {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.proposal_id, self.proposer.clone()),
            (
                self.proposal_id,
                self.proposer.clone(),
                self.recipient.clone(),
                self.amount,
                self.asset.clone(),
                self.threshold,
                self.expires_at,
                self.timestamp,
            ),
        );
    }
}

impl WithdrawalProposalApproved {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.proposal_id, self.approver.clone()),
            (
                self.proposal_id,
                self.approver.clone(),
                self.approval_count,
                self.threshold,
                self.timestamp,
            ),
        );
    }
}

impl WithdrawalProposalExecuted {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.proposal_id, self.executed_by.clone()),
            (
                self.proposal_id,
                self.recipient.clone(),
                self.amount,
                self.asset.clone(),
                self.executed_by.clone(),
                self.timestamp,
            ),
        );
    }
}

impl WithdrawalProposalCancelled {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.proposal_id, self.cancelled_by.clone()),
            (self.proposal_id, self.cancelled_by.clone(), self.timestamp),
        );
    }
}

impl WithdrawalProposalExpired {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.proposal_id,),
            (self.proposal_id, self.timestamp),
        );
    }
}

impl WithdrawalMultisigConfigUpdated {
    pub fn emit(&self, env: &Env) {
        env.events().publish(
            (self.admin.clone(), self.threshold),
            (
                self.admin.clone(),
                self.threshold,
                self.single_sig_limit,
                self.proposal_ttl_secs,
                self.timestamp,
            ),
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

/// Event type identifier for WithdrawalProposalCreated
pub const EVENT_WITHDRAWAL_PROPOSAL_CREATED: &[u8] = b"withdrawal_proposal_created";

/// Event type identifier for WithdrawalProposalApproved
pub const EVENT_WITHDRAWAL_PROPOSAL_APPROVED: &[u8] = b"withdrawal_proposal_approved";

/// Event type identifier for WithdrawalProposalExecuted
pub const EVENT_WITHDRAWAL_PROPOSAL_EXECUTED: &[u8] = b"withdrawal_proposal_executed";

/// Event type identifier for WithdrawalProposalCancelled
pub const EVENT_WITHDRAWAL_PROPOSAL_CANCELLED: &[u8] = b"withdrawal_proposal_cancelled";

/// Event type identifier for WithdrawalProposalExpired
pub const EVENT_WITHDRAWAL_PROPOSAL_EXPIRED: &[u8] = b"withdrawal_proposal_expired";

/// Event type identifier for WithdrawalMultisigConfigUpdated
pub const EVENT_WITHDRAWAL_MULTISIG_CONFIG_UPDATED: &[u8] = b"withdrawal_multisig_config_updated";
