use crate::validation::{validate_project_id, ProjectIdValidationResult, ValidationError};
use soroban_sdk::{Address, Env, String, Vec};

/// Represents a donation recorded on-chain
/// 
/// # Fields
/// * `donor` - The address of the donor
/// * `amount` - The amount donated
/// * `asset` - The asset type donated (e.g., "XLM", "USDC")
/// * `project_id` - The project this donation is for
/// * `timestamp` - When the donation was recorded
/// * `tx_hash` - The transaction hash of the donation
#[derive(Clone)]
pub struct Donation {
    pub donor: Address,
    pub amount: i128,
    pub asset: String,
    pub project_id: String,
    pub timestamp: u64,
    pub tx_hash: String,
}

impl Donation {
    /// Create a new Donation instance
    pub fn new(
        donor: Address,
        amount: i128,
        asset: String,
        project_id: String,
        timestamp: u64,
        tx_hash: String,
    ) -> Donation {
        Donation {
            donor,
            amount,
            asset,
            project_id,
            timestamp,
            tx_hash,
        }
    }

    /// Store the donation in contract storage
    /// Key format: "donation_{project_id}_{index}"
    pub fn store(&self, env: &Env, project_id: &String, index: u32) {
        let key = donation_key(env, project_id, index);
        env.storage().instance().set(&key, self);
    }

    /// Retrieve a donation from contract storage by project_id and index
    pub fn load(env: &Env, project_id: &String, index: u32) -> Option<Donation> {
        let key = donation_key(env, project_id, index);
        env.storage().instance().get(&key)
    }
}

/// Generate a storage key for a donation
fn donation_key(env: &Env, project_id: &String, index: u32) -> Vec<u8> {
    // Key format: donation_{project_id}_{index}
    let mut key = Vec::new(env);
    let prefix = b"donation_";
    for byte in prefix.iter() {
        key.push_back(*byte);
    }
    
    // Append project_id bytes
    for byte in project_id.to_bytes().iter() {
        key.push_back(*byte);
    }
    
    key.push_back(b'_');
    
    // Append index as bytes (4 bytes for u32)
    let index_bytes = index.to_le_bytes();
    for byte in index_bytes.iter() {
        key.push_back(*byte);
    }
    
    key
}

/// Storage key for the donation count per project
fn donation_count_key(env: &Env, project_id: &String) -> Vec<u8> {
    let mut key = Vec::new(env);
    let prefix = b"donation_count_";
    for byte in prefix.iter() {
        key.push_back(*byte);
    }
    
    for byte in project_id.to_bytes().iter() {
        key.push_back(*byte);
    }
    
    key
}

/// Get the donation count for a project
pub fn get_donation_count(env: &Env, project_id: &String) -> u32 {
    let key = donation_count_key(env, project_id);
    env.storage().instance().get::<_, u32>(&key).unwrap_or(0)
}

/// Increment and store the donation count for a project
pub fn increment_donation_count(env: &Env, project_id: &String) -> u32 {
    let key = donation_count_key(env, project_id);
    let current_count = get_donation_count(env, project_id);
    let new_count = current_count + 1;
    env.storage().instance().set(&key, &new_count);
    new_count
}

/// Get all donations for a project
pub fn get_donations_by_project(env: &Env, project_id: &String) -> Vec<Donation> {
    let count = get_donation_count(env, project_id);
    let mut donations = Vec::new(env);
    
    for i in 0..count {
        if let Some(donation) = Donation::load(env, project_id, i) {
            donations.push_back(donation);
        }
    }
    
    donations
}

// ===== Transaction Hash Tracking for Duplicate Prevention =====

/// Storage key for transaction hash tracking
/// Key format: "tx_hash_{hash}"
fn tx_hash_key(env: &Env, tx_hash: &String) -> Vec<u8> {
    let mut key = Vec::new(env);
    let prefix = b"tx_hash_";
    for byte in prefix.iter() {
        key.push_back(*byte);
    }
    
    for byte in tx_hash.to_bytes().iter() {
        key.push_back(*byte);
    }
    
    key
}

/// Check if a transaction hash has already been processed
pub fn is_transaction_processed(env: &Env, tx_hash: &String) -> bool {
    let key = tx_hash_key(env, tx_hash);
    env.storage().instance().has(&key)
}

/// Mark a transaction hash as processed
pub fn mark_transaction_processed(env: &Env, tx_hash: &String) {
    let key = tx_hash_key(env, tx_hash);
    env.storage().instance().set(&key, &true);
}

/// Validate donation data
/// 
/// Returns true if the donation data is valid
pub fn validate_donation(env: &Env, donor: &Address, amount: i128, asset: &String, project_id: &String) -> bool {
    // Validate amount is positive
    if amount <= 0 {
        return false;
    }
    
    // Validate asset is not empty
    if asset.to_bytes().len() == 0 {
        return false;
    }
    
    // Validate project_id format
    let validation_result = validate_project_id(env, project_id);
    if !validation_result.is_valid() {
        return false;
    }
    
    true
}

/// Validate donation data and return detailed error
/// 
/// Returns Ok(()) if valid, or Err with specific validation error
pub fn validate_donation_with_error(
    env: &Env,
    donor: &Address,
    amount: i128,
    asset: &String,
    project_id: &String,
) -> Result<(), ValidationError> {
    // Validate amount is positive
    if amount <= 0 {
        return Err(ValidationError::InvalidFormat);
    }
    
    // Validate asset is not empty
    if asset.to_bytes().len() == 0 {
        return Err(ValidationError::InvalidFormat);
    }
    
    // Validate project_id format
    match validate_project_id(env, project_id) {
        ProjectIdValidationResult::Valid => Ok(()),
        ProjectIdValidationResult::Empty => Err(ValidationError::EmptyProjectId),
        ProjectIdValidationResult::TooShort => Err(ValidationError::ProjectIdTooShort),
        ProjectIdValidationResult::TooLong => Err(ValidationError::ProjectIdTooLong),
        ProjectIdValidationResult::InvalidCharacters => Err(ValidationError::InvalidProjectIdCharacters),
        ProjectIdValidationResult::InvalidFormat => Err(ValidationError::InvalidProjectIdFormat),
    }
}
