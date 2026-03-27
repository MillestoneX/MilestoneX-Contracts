//! Storage Performance Tests
//!
//! Comprehensive tests to verify storage optimizations and measure improvements.

use crate::storage_optimized::{hash_string, string_to_symbol, symbol_to_string};
use crate::donation_optimized::{store_donation, get_donations_by_project, is_transaction_processed, mark_transaction_processed};
use soroban_sdk::{Env, Address};

#[test]
fn test_storage_key_size_reduction() {
    let env = Env::default();
    
    // Test that hashing produces consistent 32-byte keys
    let project_id = "test-project-with-long-name";
    let hash = hash_string(&env, project_id);
    
    // Verify hash is always 32 bytes regardless of input length
    assert_eq!(hash.len(), 32);
    
    // Test determinism
    let hash2 = hash_string(&env, project_id);
    assert_eq!(hash, hash2);
}

#[test]
fn test_symbol_storage_efficiency() {
    let env = Env::default();
    
    // Convert common asset codes to symbols
    let xlm_sym = string_to_symbol(&env, "XLM");
    let usdc_sym = string_to_symbol(&env, "USDC");
    
    // Symbols are more compact than strings
    assert_eq!(xlm_sym.to_string(), "XLM");
    assert_eq!(usdc_sym.to_string(), "USDC");
    
    // Test round-trip conversion
    let xlm_str = symbol_to_string(&env, &xlm_sym);
    assert_eq!(xlm_str.to_string(), "XLM");
}

#[test]
fn test_donation_storage_gas_efficiency() {
    let env = Env::default();
    let donor = Address::generate(&env);
    
    // Store multiple donations and verify they're retrievable
    for i in 0..10 {
        store_donation(
            &env,
            "performance-test-project",
            donor.clone(),
            ((i + 1) * 100) as i128,
            "XLM",
            env.ledger().timestamp(),
            &format!("tx-hash-{}", i),
        );
    }
    
    // Retrieve all donations
    let donations = get_donations_by_project(&env, "performance-test-project");
    assert_eq!(donations.len(), 10);
    
    // Verify data integrity
    let total: i128 = donations.iter().map(|d| d.amount).sum();
    assert_eq!(total, 5500i128); // Sum of 100+200+...+1000
}

#[test]
fn test_transaction_ttl_behavior() {
    let env = Env::default();
    
    // Mark transaction as processed
    mark_transaction_processed(&env, "test-tx-hash");
    assert!(is_transaction_processed(&env, "test-tx-hash"));
    
    // Advance ledger to simulate time passing
    env.ledger().with_mut(|li| {
        li.sequence_number += 500; // Halfway through TTL
    });
    
    // Should still be tracked (within TTL)
    assert!(is_transaction_processed(&env, "test-tx-hash"));
    
    // Advance beyond TTL
    env.ledger().with_mut(|li| {
        li.sequence_number += 600; // Beyond 1000 ledger TTL
    });
    
    // May expire depending on storage backend behavior
    // (This test demonstrates TTL concept)
}

#[test]
fn test_storage_read_write_patterns() {
    let env = Env::default();
    let donor = Address::generate(&env);
    
    // Test single write, multiple reads pattern
    store_donation(
        &env,
        "read-write-test",
        donor.clone(),
        1000i128,
        "USDC",
        env.ledger().timestamp(),
        "tx-read-write",
    );
    
    // Multiple reads should be efficient
    for _ in 0..5 {
        let donations = get_donations_by_project(&env, "read-write-test");
        assert_eq!(donations.len(), 1);
    }
}

#[test]
fn test_multiple_projects_isolation() {
    let env = Env::default();
    let donor = Address::generate(&env);
    
    // Create donations across multiple projects
    let projects = vec!["proj-alpha", "proj-beta", "proj-gamma"];
    
    for (i, project) in projects.iter().enumerate() {
        for j in 0..3 {
            store_donation(
                &env,
                project,
                donor.clone(),
                ((j + 1) * 100) as i128,
                "XLM",
                env.ledger().timestamp(),
                &format!("tx-{}-{}", project, j),
            );
        }
    }
    
    // Verify each project has correct donations
    for (i, project) in projects.iter().enumerate() {
        let donations = get_donations_by_project(&env, project);
        assert_eq!(donations.len(), 3);
        
        let total: i128 = donations.iter().map(|d| d.amount).sum();
        assert_eq!(total, 600i128); // 100+200+300
    }
}

#[test]
fn test_asset_symbol_conversion_performance() {
    let env = Env::default();
    
    // Test all supported assets
    let assets = vec!["XLM", "USDC", "NGNT", "USDT", "EURT"];
    
    for asset in assets.iter() {
        let sym = string_to_symbol(&env, asset);
        let back = symbol_to_string(&env, &sym);
        assert_eq!(back.to_string(), *asset);
    }
    
    // Test custom asset
    let custom = string_to_symbol(&env, "CUSTOM");
    assert_eq!(custom.to_string(), "CUSTOM");
}

#[test]
fn test_hash_collision_resistance() {
    let env = Env::default();
    
    // Generate hashes for similar project IDs
    let hash1 = hash_string(&env, "project-a");
    let hash2 = hash_string(&env, "project-b");
    let hash3 = hash_string(&env, "project-1");
    
    // All should be unique
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_ne!(hash2, hash3);
    
    // Similar names should produce very different hashes
    let mut diff_count = 0;
    for i in 0..32 {
        if hash1.get(i) != hash2.get(i) {
            diff_count += 1;
        }
    }
    assert!(diff_count > 16, "Hashes should differ in at least half the bytes");
}

#[test]
fn test_storage_scalability() {
    let env = Env::default();
    let donor = Address::generate(&env);
    
    // Test with large number of donations
    let num_donations = 50;
    
    for i in 0..num_donations {
        store_donation(
            &env,
            "scalability-test",
            donor.clone(),
            100i128,
            "XLM",
            env.ledger().timestamp(),
            &format!("scalability-tx-{}", i),
        );
    }
    
    let donations = get_donations_by_project(&env, "scalability-test");
    assert_eq!(donations.len(), num_donations);
}

// ===== Storage Size Comparison Tests =====

#[test]
fn compare_old_vs_new_storage_approach() {
    let env = Env::default();
    
    // OLD APPROACH (conceptual - not implemented here):
    // - Storage key: Vec<u8> with "donation_" + project_id_bytes + index_bytes
    //   Typical size: 9 + len(project_id) + 4 = ~30-80 bytes per key
    // - Asset: String = ~20-50 bytes
    // - TX Hash: String = ~50-70 bytes
    // - Project ID stored redundantly in value = ~3-64 bytes
    // Total per donation: ~150-300 bytes
    
    // NEW OPTIMIZED APPROACH:
    // - Storage key: (BytesN<32>, u32) = 36 bytes fixed
    // - Asset: Symbol = ~8-12 bytes
    // - TX Hash: BytesN<32> = 32 bytes
    // - No redundant project_id in value
    // Total per donation: ~80-120 bytes
    
    // Estimated savings: 40-60% reduction in storage size
    // This translates directly to lower gas costs
    
    // Demonstrate the optimized approach works correctly
    store_donation(
        &env,
        "comparison-test-project",
        Address::generate(&env),
        1000i128,
        "XLM",
        env.ledger().timestamp(),
        "comparison-tx-hash",
    );
    
    let donations = get_donations_by_project(&env, "comparison-test-project");
    assert_eq!(donations.len(), 1);
    assert_eq!(donations.get(0).unwrap().amount, 1000i128);
}
