//! Tip routing: `send_tip`, `send_tip_with_fee`, `pause`/`unpause`.

use soroban_sdk::{contract, contracterror, contractevent, contractimpl, token, Address, Env, Map};

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

/// Event emitted when a tip with fee is successfully sent.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipSentWithFee {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub fee_amount: i128,
    pub creator_amount: i128,
    pub treasury: Address,
    pub timestamp: u64,
}

/// Paused state key for contract storage.
const PAUSED_KEY: &str = "paused";

/// Error codes for the TipRouter contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Amount must be positive.
    InvalidAmount = 1,
    /// Tipper and creator cannot be the same address.
    SelfTip = 2,
    /// Tipper has insufficient balance.
    InsufficientBalance = 3,
    /// Tipper has not granted sufficient allowance.
    InsufficientAllowance = 4,
    /// Asset address is invalid or unsupported.
    InvalidAsset = 5,
    /// Contract is paused.
    ContractPaused = 6,
    /// Fee basis points must be > 0 and < 10_000.
    InvalidFeeBps = 7,
    /// Fee would consume entire amount (creator_amount = 0).
    FeeConsumesEntireAmount = 8,
    /// Unauthorized caller.
    Unauthorized = 9,
}

/// The TipRouter contract.
#[contract]
pub struct TipRouter;

#[contractimpl]
impl TipRouter {
    /// Check if the contract is paused.
    fn is_paused(env: &Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }

    /// Set the paused state.
    fn set_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&PAUSED_KEY, &paused);
    }

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
        // Reject if contract is paused
        if Self::is_paused(&env) {
            env.panic_with_error(Error::ContractPaused);
        }

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

    /// Pauses the contract — only callable by admin.
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        Self::set_paused(&env, true);
    }

    /// Unpauses the contract — only callable by admin.
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        Self::set_paused(&env, false);
    }
}
