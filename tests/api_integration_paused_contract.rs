use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

mod tip_router {
    soroban_sdk::contractimport!(file = "target/wasm32v1-none/release/tip_router.wasm");
}

fn create_token_contract<'a>(env: &Env) -> (StellarAssetClient<'a>, TokenClient<'a>, Address) {
    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let asset_client = StellarAssetClient::new(env, &sac.address());
    let token_client = TokenClient::new(env, &sac.address());
    (asset_client, token_client, sac.address())
}

#[test]
fn test_api_integration_simulation_paused_contract_rejected() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let tip_router_address = env.register(tip_router::WASM, ());
    let tip_router_client = tip_router::Client::new(&env, &tip_router_address);

    let (asset_client, token_client, _token_address) = create_token_contract(&env);

    let tipper = Address::generate(&env);
    let creator = Address::generate(&env);

    let tip_amount = 1_000;
    asset_client.mint(&tipper, &tip_amount);
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &tip_amount, &live_until);

    // Admin pauses contract
    let admin = Address::generate(&env);
    tip_router_client.pause(&admin);

    // Api tries to submit tip while paused - should fail
    let result = tip_router_client.try_send_tip(&tipper, &creator, &tip_amount, &token_client.address);
    assert!(result.is_err());

    // Verify no funds moved
    assert_eq!(token_client.balance(&tipper), tip_amount);
    assert_eq!(token_client.balance(&creator), 0);
    assert_eq!(token_client.balance(&tip_router_address), 0);

    println!("API Integration Simulation: Paused contract rejection - PASSED");
}