//! Optimized Donation Storage
//!
//! Uses compact storage keys and efficient data structures to reduce gas costs.

use crate::storage_optimized::{
    hash_string, symbol_to_string, string_to_symbol, DonationRecord, StorageKey,
    donation_key, project_count_key, tx_hash_key,
};
use soroban_sdk::{Address, Env, String, Symbol, BytesN, Vec};

/// Store a donation using optimized storage format
/// 
/// # Optimization Benefits:
/// - Uses hashed project_id for fixed-size keys (32 bytes vs variable)
/// - Stores asset as Symbol instead of String (saves ~20-40 bytes per donation)
/// - Stores tx_hash as BytesN<32> instead of String (saves ~20-60 bytes)
/// - Removes redundant project_id from stored struct
pub fn store_donation(
    env: &Env,
    project_id: &str,
    donor: Address,
    amount: i128,
    asset: &str,
    timestamp: u64,
    tx_hash: &str,
) {
    let project_id_hash = hash_string(env, project_id);
    
    // Get current count (read once)
    let index = get_donation_count(env, &project_id_hash);
    
    // Create optimized donation record
    let asset_symbol = string_to_symbol(env, asset);
    let tx_hash_bytes = hash_string(env, tx_hash);
    
    let record = DonationRecord {
        donor,
        amount,
        asset: asset_symbol,
        timestamp,
        tx_hash: tx_hash_bytes,
    };
    
    // Store with compact key
    let key = donation_key(project_id_hash.clone(), index);
    env.storage().persistent().set(&key, &record);
    
    // Increment count (single write, not read-modify-write)
    env.storage().persistent().set(
        &project_count_key(project_id_hash),
        &(index + 1),
    );
}

/// Get donation count for a project
fn get_donation_count(env: &Env, project_id_hash: &BytesN<32>) -> u32 {
    env.storage()
        .persistent()
        .get(&project_count_key(project_id_hash.clone()))
        .unwrap_or(0)
}

/// Get all donations for a project (optimized read path)
pub fn get_donations_by_project(env: &Env, project_id: &str) -> Vec<DonationRecord> {
    let project_id_hash = hash_string(env, project_id);
    let count = get_donation_count(env, &project_id_hash);
    
    let mut donations = Vec::new(env);
    for i in 0..count {
        if let Some(record) = env.storage().persistent().get::<_, DonationRecord>(
            &donation_key(project_id_hash.clone(), i)
        ) {
            donations.push_back(record);
        }
    }
    
    donations
}

/// Check if transaction was processed (with TTL support)
pub fn is_transaction_processed(env: &Env, tx_hash: &str) -> bool {
    let tx_hash_hash = hash_string(env, tx_hash);
    env.storage()
        .temporary()
        .has(&tx_hash_key(tx_hash_hash))
}

/// Mark transaction as processed with TTL
/// 
/// # TTL Implementation:
/// - Uses temporary storage (cheaper than persistent)
/// - Automatically expires after ledger-specific TTL
/// - Reduces long-term storage bloat
pub fn mark_transaction_processed(env: &Env, tx_hash: &str) {
    let tx_hash_hash = hash_string(env, tx_hash);
    let key = tx_hash_key(tx_hash_hash);
    
    // Use temporary storage with TTL
    env.storage().temporary().set(&key, &true);
    
    // Extend TTL to give time for duplicate detection
    // Typical block time is ~5 seconds, so 1000 ledgers = ~1.4 hours
    let ttl = env.ledger().get_live_until_ledger();
    let new_ttl = env.ledger().sequence() + 1000; // Keep for 1000 ledgers
    if new_ttl > ttl {
        env.storage().temporary().extend_ttl(&key, new_ttl);
    }
}

/// Extended donation record with project_id for API compatibility
#[derive(Clone)]
pub struct DonationWithProject {
    pub donor: Address,
    pub amount: i128,
    pub asset: String,
    pub project_id: String,
    pub timestamp: u64,
    pub tx_hash: String,
}

impl DonationWithProject {
    /// Convert from optimized DonationRecord
    pub fn from_record(record: &DonationRecord, project_id: &str, tx_hash_str: &str, env: &Env) -> Self {
        Self {
            donor: record.donor.clone(),
            amount: record.amount,
            asset: symbol_to_string(env, &record.asset),
            project_id: String::from_str(env, project_id),
            timestamp: record.timestamp,
            tx_hash: String::from_str(env, tx_hash_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_store_and_retrieve_donation() {
        let env = Env::default();
        let donor = Address::generate(&env);
        
        store_donation(
            &env,
            "test-project",
            donor.clone(),
            1000i128,
            "XLM",
            env.ledger().timestamp(),
            "tx-hash-123",
        );
        
        let donations = get_donations_by_project(&env, "test-project");
        assert_eq!(donations.len(), 1);
        assert_eq!(donations.get(0).unwrap().amount, 1000i128);
    }

    #[test]
    fn test_transaction_tracking() {
        let env = Env::default();
        
        assert!(!is_transaction_processed(&env, "new-tx"));
        mark_transaction_processed(&env, "new-tx");
        assert!(is_transaction_processed(&env, "new-tx"));
    }

    #[test]
    fn test_multiple_donations_same_project() {
        let env = Env::default();
        let donor = Address::generate(&env);
        
        for i in 0..5 {
            store_donation(
                &env,
                "multi-donation-project",
                donor.clone(),
                ((i + 1) * 100) as i128,
                "USDC",
                env.ledger().timestamp(),
                &format!("tx-{}", i),
            );
        }
        
        let donations = get_donations_by_project(&env, "multi-donation-project");
        assert_eq!(donations.len(), 5);
        
        // Verify amounts
        let total: i128 = donations.iter().map(|d| d.amount).sum();
        assert_eq!(total, 1500i128); // 100+200+300+400+500
    }
}
