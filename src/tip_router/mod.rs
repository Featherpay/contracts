//! Tip routing: `send_tip`, `send_tip_with_fee`, `pause`/`unpause`.

use soroban_sdk::{contract, contracterror, contractevent, contractimpl, token, Address, Env};

/// Event emitted when a tip is successfully sent.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipSent {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub timestamp: u64,
}

/// Error codes for the TipRouter contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Amount must be positive.
    InvalidAmount = 1,
    /// Tipper and creator cannot be the same address.
    SelfTip = 2,
}

/// The TipRouter contract.
#[contract]
pub struct TipRouter;

#[contractimpl]
impl TipRouter {
    /// Sends a tip from `tipper` to `creator` atomically.
    ///
    /// # Arguments
    /// * `tipper` - The address sending the tip (must authorize the call)
    /// * `creator` - The address receiving the tip
    /// * `amount` - The amount to tip (must be > 0)
    /// * `asset` - The token contract address for the asset being tipped
    ///
    /// # Authorization
    /// Requires `tipper.require_auth()` — the tipper's embedded wallet must sign.
    ///
    /// # Events
    /// Emits `TipSent` with tipper, creator, amount, asset, and ledger timestamp.
    pub fn send_tip(env: Env, tipper: Address, creator: Address, amount: i128, asset: Address) {
        // Reject zero or negative amounts
        if amount <= 0 {
            env.panic_with_error(Error::InvalidAmount);
        }

        // Reject self-tips
        if tipper == creator {
            env.panic_with_error(Error::SelfTip);
        }

        // Tipper must authorize the call
        tipper.require_auth();

        // Get the token client for the asset
        let token_client = token::Client::new(&env, &asset);
        let contract_addr = env.current_contract_address();

        // Pull amount from tipper to this contract using transfer (requires tipper auth)
        token_client.transfer(&tipper, &contract_addr, &amount);

        // Forward the full amount to creator using transfer (requires contract auth)
        token_client.transfer(&contract_addr, &creator, &amount);

        // Emit TipSent event with ledger timestamp
        let timestamp = env.ledger().timestamp();
        TipSent {
            tipper,
            creator,
            amount,
            asset,
            timestamp,
        }
        .publish(&env);
    }
}
