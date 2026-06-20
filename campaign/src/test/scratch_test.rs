use soroban_sdk::{Env, Address};
use crate::test::release_milestone_tests::*;

#[test]
fn scratch() {
    let env = Env::default();
    env.mock_all_auths();
    // ...
}
