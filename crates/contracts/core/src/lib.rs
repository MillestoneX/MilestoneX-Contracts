#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String};

pub mod assets;
pub mod validation;
pub mod events;
pub mod donation;

#[contract]
pub struct CoreContract;

#[contractimpl]
impl CoreContract {
    pub fn init(_env: Env, _admin: Address) {}

    pub fn ping(_env: Env) -> u32 {
        1
    }

    /// Record a donation and emit the DonationReceived event
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `donor` - The address of the donor
    /// * `amount` - The amount donated
    /// * `asset` - The asset type donated (e.g., "XLM", "USDC")
    /// * `project_id` - The project ID to map this donation to (3-64 chars, alphanumeric with hyphens/underscores)
    /// * `tx_hash` - The transaction hash of the donation
    ///
    /// # Returns
    /// * The donation amount if successful
    /// * 0 if validation fails (check validation error for details)
    pub fn donate(
        env: Env,
        donor: Address,
        amount: i128,
        asset: String,
        project_id: String,
        tx_hash: String,
    ) -> i128 {
        // Validate donation data with detailed error handling
        match donation::validate_donation_with_error(&env, &donor, amount, &asset, &project_id) {
            Ok(()) => {},
            Err(_) => return 0,
        }

        // Get timestamp from ledger
        let timestamp = env.ledger().timestamp();

        // Store the donation on-chain
        let donation = donation::Donation::new(
            donor.clone(),
            amount,
            asset.clone(),
            project_id.clone(),
            timestamp,
            tx_hash,
        );
        
        // Get the index for this donation
        let index = donation::increment_donation_count(&env, &project_id) - 1;
        donation.store(&env, &project_id, index);

        // Emit the DonationReceived event with project_id
        events::DonationReceived {
            donor: donor.clone(),
            amount,
            asset: asset.clone(),
            project_id: project_id.clone(),
            timestamp,
        }
        .emit(&env);

        amount
    }

    /// Get all donations for a project
    pub fn get_donations(env: Env, project_id: String) -> soroban_sdk::Vec<Donation> {
        donation::get_donations_by_project(&env, &project_id)
    }

    /// Process a withdrawal and emit the WithdrawalProcessed event
    pub fn withdraw(
        env: Env,
        recipient: Address,
        amount: i128,
        asset: String,
    ) -> i128 {
        // Emit the WithdrawalProcessed event
        events::WithdrawalProcessed {
            recipient: recipient.clone(),
            amount,
            asset: asset.clone(),
            timestamp: env.ledger().timestamp(),
        }
        .emit(&env);

        amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_init_and_ping() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let result = client.ping();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_address_validation_integration() {
        use crate::validation::*;

        let env = Env::default();
        let valid_address = soroban_sdk::String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
        );

        // Test that validation utilities are accessible
        let result = validate_stellar_address(&env, valid_address);
        assert!(result.is_ok());

        // Test boolean validation
        let valid_address2 = soroban_sdk::String::from_str(
            &env,
            "GAYOLLLUIZE4DZMBB2ZBKGBUBZLIOYU6XFLW37GBP2VZD3ABNXCW4BVA",
        );
        assert!(is_valid_stellar_address(&env, valid_address2));
    }

    // ===== Project ID Donation Mapping Tests =====

    #[test]
    fn test_donate_with_valid_project_id() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let amount = 1000i128;
        let asset = String::from_str(&env, "XLM");
        let project_id = String::from_str(&env, "proj-123");
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.donate(&donor, &amount, &asset, &project_id, &tx_hash);
        assert_eq!(result, amount);
    }

    #[test]
    fn test_donate_with_invalid_project_id_empty() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let amount = 1000i128;
        let asset = String::from_str(&env, "XLM");
        let project_id = String::from_str(&env, ""); // Empty project ID
        let tx_hash = String::from_str(&env, "abc123");

        // Should return 0 for invalid project_id
        let result = client.donate(&donor, &amount, &asset, &project_id, &tx_hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_donate_with_invalid_project_id_too_short() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let amount = 1000i128;
        let asset = String::from_str(&env, "XLM");
        let project_id = String::from_str(&env, "AB"); // Too short (min 3 chars)
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.donate(&donor, &amount, &asset, &project_id, &tx_hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_donate_with_invalid_project_id_invalid_chars() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let amount = 1000i128;
        let asset = String::from_str(&env, "XLM");
        let project_id = String::from_str(&env, "proj@123"); // Invalid character @
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.donate(&donor, &amount, &asset, &project_id, &tx_hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_donate_with_invalid_project_id_starts_with_hyphen() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let amount = 1000i128;
        let asset = String::from_str(&env, "XLM");
        let project_id = String::from_str(&env, "-proj123"); // Starts with hyphen
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.donate(&donor, &amount, &asset, &project_id, &tx_hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_donations_groups_by_project_id() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor1 = Address::generate(&env);
        let donor2 = Address::generate(&env);
        let donor3 = Address::generate(&env);

        // Donate to project A
        let project_a = String::from_str(&env, "project-alpha");
        client.donate(&donor1, &1000i128, &String::from_str(&env, "XLM"), &project_a, &String::from_str(&env, "tx1"));
        client.donate(&donor2, &2000i128, &String::from_str(&env, "USDC"), &project_a, &String::from_str(&env, "tx2"));

        // Donate to project B
        let project_b = String::from_str(&env, "project-beta");
        client.donate(&donor3, &500i128, &String::from_str(&env, "XLM"), &project_b, &String::from_str(&env, "tx3"));

        // Get donations for project A
        let donations_a = client.get_donations(&project_a);
        assert_eq!(donations_a.len(), 2);

        // Get donations for project B
        let donations_b = client.get_donations(&project_b);
        assert_eq!(donations_b.len(), 1);

        // Verify amounts
        assert_eq!(donations_a.get(0).unwrap().amount, 1000i128);
        assert_eq!(donations_a.get(1).unwrap().amount, 2000i128);
        assert_eq!(donations_b.get(0).unwrap().amount, 500i128);
    }

    #[test]
    fn test_donation_project_id_mapping_integrity() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);
        let project_id = String::from_str(&env, "test-project-001");
        let tx_hash = String::from_str(&env, "txhash123");

        // Make donation
        client.donate(&donor, &1500i128, &String::from_str(&env, "XLM"), &project_id, &tx_hash);

        // Retrieve and verify project_id is correctly stored
        let donations = client.get_donations(&project_id);
        assert_eq!(donations.len(), 1);
        
        let stored_donation = donations.get(0).unwrap();
        assert_eq!(stored_donation.project_id, project_id);
        assert_eq!(stored_donation.amount, 1500i128);
        assert_eq!(stored_donation.donor, donor);
    }

    #[test]
    fn test_multiple_projects_isolation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CoreContract);
        let client = CoreContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let donor = Address::generate(&env);

        // Create multiple projects
        let projects = vec![
            String::from_str(&env, "proj-001"),
            String::from_str(&env, "proj-002"),
            String::from_str(&env, "proj-003"),
        ];

        // Donate to each project
        for (i, project_id) in projects.iter().enumerate() {
            let amount = ((i + 1) * 100) as i128;
            let tx_hash = String::from_str(&env, &format!("tx{}", i));
            client.donate(&donor, &amount, &String::from_str(&env, "XLM"), &project_id, &tx_hash);
        }

        // Verify each project has exactly one donation
        for project_id in projects.iter() {
            let donations = client.get_donations(&project_id);
            assert_eq!(donations.len(), 1, "Project should have exactly one donation");
        }
    }
}
