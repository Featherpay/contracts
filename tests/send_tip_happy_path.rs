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
fn test_cross_contract_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let (asset_client, token_client, _token_address) = create_token_contract(&env);

    let tipper = Address::generate(&env);
    let contract_addr = Address::generate(&env);
    let _recipient = Address::generate(&env);

    // Mint some tokens to tipper
    asset_client.mint(&tipper, &1000);

    // Approve contract to spend tipper's tokens
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &contract_addr, &1000, &live_until);

    // Check allowance
    let allowance = token_client.allowance(&tipper, &contract_addr);
    assert_eq!(allowance, 1000);

    // Now simulate what the contract does: call transfer_from as the contract
    // We need to authorize as the contract_addr
    // With mock_all_auths, this should work
    token_client.transfer_from(&contract_addr, &tipper, &contract_addr, &100);

    // Check balances
    assert_eq!(token_client.balance(&tipper), 900);
    assert_eq!(token_client.balance(&contract_addr), 100);
    assert_eq!(token_client.allowance(&tipper, &contract_addr), 900);
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

#[test]
fn test_send_tip_happy_path() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    // Approve the tip router contract to spend tipper's tokens
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    let amount = 100;

    // Perform the tip
    tip_router_client.send_tip(&tipper, &creator, &amount, &token_client.address);

    // Verify tipper balance decreased by exactly amount
    assert_eq!(token_client.balance(&tipper), 900);

    // Verify creator balance increased by exactly amount
    assert_eq!(token_client.balance(&creator), 100);

    // Verify contract balance is zero (funds never stuck)
    assert_eq!(token_client.balance(&tip_router_client.address), 0);
}

#[test]
fn test_send_tip_emits_tip_sent_event() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    // Approve the tip router contract to spend tipper's tokens
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    let amount = 250;

    // Perform the tip
    tip_router_client.send_tip(&tipper, &creator, &amount, &token_client.address);

    // Get events and verify TipSent event was emitted
    let events = env.events().all();
    // Filter events from our contract only
    let tip_router_events = events.filter_by_contract(&tip_router_address);
    let event_list = tip_router_events.events();
    assert_eq!(event_list.len(), 1);

    // Check the event type is Contract event
    assert_eq!(
        event_list[0].type_,
        soroban_sdk::xdr::ContractEventType::Contract
    );
}

#[test]
fn test_send_tip_multiple_tips_accumulate() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    // Approve the tip router contract to spend tipper's tokens
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    // First tip
    tip_router_client.send_tip(&tipper, &creator, &100, &token_client.address);
    let events1 = env.events().all();
    let count1 = events1
        .filter_by_contract(&tip_router_address)
        .events()
        .len();
    assert_eq!(token_client.balance(&tipper), 900);
    assert_eq!(token_client.balance(&creator), 100);
    assert_eq!(token_client.balance(&tip_router_client.address), 0);

    // Second tip
    tip_router_client.send_tip(&tipper, &creator, &200, &token_client.address);
    let events2 = env.events().all();
    let count2 = events2
        .filter_by_contract(&tip_router_address)
        .events()
        .len();
    assert_eq!(token_client.balance(&tipper), 700);
    assert_eq!(token_client.balance(&creator), 300);
    assert_eq!(token_client.balance(&tip_router_client.address), 0);

    // Sum of events from each call should be 2
    assert_eq!(count1 + count2, 2);
}

#[test]
fn test_send_tip_different_tippers() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, asset_client, tipper1, creator, tip_router_address) =
        setup_test(&env);

    // Approve for tipper1
    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper1, &tip_router_address, &1000, &live_until);

    // Create a second tipper
    let tipper2 = Address::generate(&env);
    asset_client.mint(&tipper2, &500);
    token_client.approve(&tipper2, &tip_router_address, &500, &live_until);

    // Tip from tipper1
    tip_router_client.send_tip(&tipper1, &creator, &100, &token_client.address);
    assert_eq!(token_client.balance(&tipper1), 900);
    assert_eq!(token_client.balance(&creator), 100);

    // Tip from tipper2
    tip_router_client.send_tip(&tipper2, &creator, &200, &token_client.address);
    assert_eq!(token_client.balance(&tipper2), 300);
    assert_eq!(token_client.balance(&creator), 300);

    // Contract balance remains zero
    assert_eq!(token_client.balance(&tip_router_client.address), 0);
}
