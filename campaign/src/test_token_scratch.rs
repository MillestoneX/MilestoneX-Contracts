#[cfg(test)]
mod test {
    use soroban_sdk::{Env, Address, token::StellarAssetClient, token::Client};
    #[test]
    fn test_mint() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(admin).address();
        let admin_client = StellarAssetClient::new(&env, &token);
        let user = Address::generate(&env);
        admin_client.mint(&user, &1000);
        let client = Client::new(&env, &token);
        assert_eq!(client.balance(&user), 1000);
    }
}
