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

#[test]
fn test_exact_copy() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let (tip_router_client, token_client, _asset_client, tipper, creator, tip_router_address) =
        setup_test(&env);

    let live_until = env.ledger().sequence() + 1000;
    token_client.approve(&tipper, &tip_router_address, &1000, &live_until);

    let amount = 250;

    // Perform the tip
    tip_router_client.send_tip(&tipper, &creator, &amount, &token_client.address);

    // Get events and verify TipSent event was emitted
    let events = env.events().all();
    let all_events = events.events();
    println!("DEBUG: Total events: {}", all_events.len());
    for (i, event) in all_events.iter().enumerate() {
        println!(
            "DEBUG Event {}: contract={:?}, type={:?}",
            i, event.contract_id, event.type_
        );
    }

    // Filter events from our contract only
    let tip_router_events = events.filter_by_contract(&tip_router_address);
    let event_list = tip_router_events.events();
    println!("DEBUG: Filtered events: {}", event_list.len());
    assert_eq!(event_list.len(), 1);

    // Check the event type is Contract event
    assert_eq!(
        event_list[0].type_,
        soroban_sdk::xdr::ContractEventType::Contract
    );
}
