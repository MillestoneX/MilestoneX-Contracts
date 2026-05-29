#![no_std]

pub mod storage;
pub mod types;

use soroban_sdk::{contract, contractimpl, Env};

pub const VERSION: u32 = 1;

#[contract]
pub struct CampaignContract;

#[contractimpl]
impl CampaignContract {
    pub fn hello(env: Env) -> soroban_sdk::Symbol {
        soroban_sdk::Symbol::new(&env, "campaign")
    }

    pub fn version() -> u32 {
        VERSION
    }
}
