//! Optimized Storage Utilities
//!
//! Provides efficient storage patterns for Soroban contracts:
//! - Compact storage keys using symbols
//! - Efficient data encoding
//! - Reduced redundancy
//! - TTL support for temporary data

use soroban_sdk::{contracttype, symbol_short, Env, String, Symbol, Vec, BytesN};

/// Storage key symbols for efficient lookup
#[contracttype]
pub enum StorageKey {
    /// Donation data: (project_id_hash, index) -> DonationRecord
    Donation(BytesN<32>, u32),
    /// Project donation count: project_id_hash -> u32
    ProjectCount(BytesN<32>),
    /// Transaction hash tracking: tx_hash_hash -> bool
    TxHash(BytesN<32>),
    /// Asset configuration
    AssetConfig(Symbol),
    /// Admin address
    Admin(Symbol),
}

/// Optimized donation record that doesn't store project_id redundantly
#[derive(Clone)]
#[contracttype]
pub struct DonationRecord {
    pub donor: Address,
    pub amount: i128,
    pub asset: Symbol,  // Use Symbol instead of String for common assets
    pub timestamp: u64,
    pub tx_hash: BytesN<32>,  // Store as bytes instead of String
}

/// Hash a string to create compact storage keys
pub fn hash_string(env: &Env, data: &str) -> BytesN<32> {
    // Use Soroban's built-in hashing
    env.crypto().sha256(&data.as_bytes())
}

/// Generate optimized donation storage key
pub fn donation_key(project_id_hash: BytesN<32>, index: u32) -> StorageKey {
    StorageKey::Donation(project_id_hash, index)
}

/// Generate project count key
pub fn project_count_key(project_id_hash: BytesN<32>) -> StorageKey {
    StorageKey::ProjectCount(project_id_hash)
}

/// Generate transaction hash key
pub fn tx_hash_key(tx_hash_hash: BytesN<32>) -> StorageKey {
    StorageKey::TxHash(tx_hash_hash)
}

/// Convert String to Symbol for common asset codes (more efficient)
pub fn string_to_symbol(env: &Env, s: &str) -> Symbol {
    // For common assets, use short symbols
    match s {
        "XLM" => symbol_short!("XLM"),
        "USDC" => symbol_short!("USDC"),
        "NGNT" => symbol_short!("NGNT"),
        "USDT" => symbol_short!("USDT"),
        "EURT" => symbol_short!("EURT"),
        _ => {
            // For other assets, try to create a symbol or fall back to default
            if s.len() <= 9 {
                Symbol::try_from_small_str(s).unwrap_or(symbol_short!("UNKNOWN"))
            } else {
                symbol_short!("CUSTOM")
            }
        }
    }
}

/// Convert Symbol back to String when needed
pub fn symbol_to_string(env: &Env, sym: &Symbol) -> String {
    String::from_str(env, &sym.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_hash_string_deterministic() {
        let env = Env::default();
        let hash1 = hash_string(&env, "test-project");
        let hash2 = hash_string(&env, "test-project");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_unique() {
        let env = Env::default();
        let hash1 = hash_string(&env, "project-a");
        let hash2 = hash_string(&env, "project-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_symbol_conversion() {
        let env = Env::default();
        let sym = string_to_symbol(&env, "XLM");
        assert_eq!(sym.to_string(), "XLM");
        
        let str = symbol_to_string(&env, &sym);
        assert_eq!(str.to_string(), "XLM");
    }
}
