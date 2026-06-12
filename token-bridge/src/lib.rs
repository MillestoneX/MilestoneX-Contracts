//! Token bridge contract for cross-chain asset transfers.
//!
//! Provides a minimal Soroban contract skeleton with `hello` (ping) and `version`
//! endpoints. Extend this contract to implement full cross-chain bridging logic.

#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TokenBridgeContract;

#[contractimpl]
impl TokenBridgeContract {
    pub fn hello(env: Env) -> soroban_sdk::Symbol {
        soroban_sdk::Symbol::new(&env, "token_bridge")
    }

    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }
}
