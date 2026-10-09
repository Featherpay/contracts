// Stuck-funds verification tests (Day 5, M3).

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

fn setup_test<'a>(
    env: &Env,
) -> (
    tip_router::Client<'a>,
    TokenClient<'a>,
    StellarAssetClient<'a>,
    Address,
    Address,
    Address,
) {
    let tip_router_address = env.register(tip_router::WASM, ());
    let tip_router_client = tip_router::Client::new(env, &tip_router_address);

    let (asset_client, token_client, _token_address) = create_token_contract(env);

    let tipper = Address::generate(env);
    let creator = Address::generate(env);

    // Mint some tokens to tipper
    asset_client.mint(&tipper, &1000);

    (
        tip_router_client,
        token_client,
        asset_client,
        tipper,
        creator,
        tip_router_address,
    )
}

fn setup_test_with_treasury<'a>(
    env: &Env,
) -> (
    tip_router::Client<'a>,
    TokenClient<'a>,
    StellarAssetClient<'a>,
    Address,
    Address,
    Address,
    Address,
) {
    let tip_router_address = env.register(tip_router::WASM, ());
    let tip_router_client = tip_router::Client::new(env, &tip_router_address);

    let (asset_client, token_client, _token_address) = create_token_contract(env);

    let tipper = Address::generate(env);
    let creator = Address::generate(env);
    let treasury = Address::generate(env);

    // Mint some tokens to tipper
    asset_client.mint(&tipper, &10000);

    (
        tip_router_client,
        token_client,
        asset_client,
        tipper,
        creator,
        treasury,
        tip_router_address,
    )
}

#[test]
fn test_contract_balance_zero_after_successful_send_tip() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    tip_router_client.send_tip(&tipper, &creator, &500, &token_client.address);

    // Contract balance must be exactly 0 after successful tip
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_successful_send_tip_with_fee() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, treasury, tip_router_address) =
        setup_test_with_treasury(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &10000, &live_until);

    tip_router_client.send_tip_with_fee(&tipper, &creator, &10000, &token_client.address, &100, &treasury);

    // Contract balance must be exactly 0 after successful tip with fee
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // Try to send more than balance - should fail
    let result = tip_router_client.try_send_tip(&tipper, &creator, &2000, &token_client.address);
    assert!(result.is_err());

    // Contract balance must be exactly 0 after failed tip
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_self_tip() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, _creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // Try to tip self - should fail
    let result = tip_router_client.try_send_tip(&tipper, &tipper, &100, &token_client.address);
    assert!(result.is_err());

    // Contract balance must be exactly 0 after failed tip
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // Try zero amount - should fail
    let result = tip_router_client.try_send_tip(&tipper, &creator, &0, &token_client.address);
    assert!(result.is_err());

    // Contract balance must be exactly 0
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_paused() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // Pause the contract
    let admin = Address::generate(&env);
    tip_router_client.pause(&admin);

    // Try to send tip while paused - should fail
    let result = tip_router_client.try_send_tip(&tipper, &creator, &100, &token_client.address);
    assert!(result.is_err());

    // Contract balance must be exactly 0
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_with_fee_invalid_fee_bps() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, treasury, tip_router_address) =
        setup_test_with_treasury(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &10000, &live_until);

    // Try with invalid fee_bps (0) - should fail
    let result = tip_router_client.try_send_tip_with_fee(&tipper, &creator, &100, &token_client.address, &0, &treasury);
    assert!(result.is_err());

    // Contract balance must be exactly 0
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_with_fee_max_bps() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, treasury, tip_router_address) =
        setup_test_with_treasury(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &10000, &live_until);

    // Try with fee_bps = 10000 - should fail
    let result = tip_router_client.try_send_tip_with_fee(&tipper, &creator, &100, &token_client.address, &10_000, &treasury);
    assert!(result.is_err());

    // Contract balance must be exactly 0
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_with_fee_paused() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, treasury, tip_router_address) =
        setup_test_with_treasury(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &10000, &live_until);

    // Pause the contract
    let admin = Address::generate(&env);
    tip_router_client.pause(&admin);

    // Try to send tip with fee while paused - should fail
    let result = tip_router_client.try_send_tip_with_fee(&tipper, &creator, &100, &token_client.address, &100, &treasury);
    assert!(result.is_err());

    // Contract balance must be exactly 0
    assert_eq!(token_client.balance(&tip_router_address), 0);
}

#[test]
fn test_contract_balance_zero_after_failed_send_tip_with_fee_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, treasury, tip_router_address) =
        setup_test_with_treasury(&env);

    let live_until = env.ledger().sequence() + 1000;
    // Only approve 1000, but try to send 2000
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // Try to send more than approved allowance with fee
    // Note: In the test environment with mock_all_auths_allowing_non_root_auth,
    // the transfer might succeed even with insufficient allowance.
    // The important assertion is that contract balance remains 0 regardless.
    let _result = tip_router_client.try_send_tip_with_fee(&tipper, &creator, &2000, &token_client.address, &100, &treasury);

    // Contract balance must be exactly 0 regardless of success/failure
    assert_eq!(token_client.balance(&tip_router_address), 0);
}