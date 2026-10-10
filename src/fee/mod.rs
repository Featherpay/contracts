//! Fee calculation and rounding logic.
//!
//! This module contains all fee math for the TipRouter contract.
//! The tip_router module calls into this for all calculations.

use crate::tip_router::Error;
use soroban_sdk::Env;

/// Calculate fee amount and creator amount from total amount and fee basis points.
///
/// Fee is calculated as: fee_amount = (amount * fee_bps) / 10_000 (integer division, round down)
/// Creator receives: creator_amount = amount - fee_amount
///
/// Returns (fee_amount, creator_amount) or Error if validation fails.
pub fn calculate_fee(_env: &Env, amount: i128, fee_bps: u32) -> Result<(i128, i128), Error> {
    // Validate fee_bps
    if fee_bps == 0 || fee_bps >= 10_000 {
        return Err(Error::InvalidFeeBps);
    }

    // Use checked arithmetic to prevent overflow
    let fee_amount = amount
        .checked_mul(fee_bps as i128)
        .ok_or(Error::InvalidFeeBps)?
        .checked_div(10_000)
        .ok_or(Error::InvalidFeeBps)?;

    let creator_amount = amount.checked_sub(fee_amount).ok_or(Error::InvalidFeeBps)?;

    // Ensure creator gets something
    if creator_amount <= 0 {
        return Err(Error::FeeConsumesEntireAmount);
    }

    Ok((fee_amount, creator_amount))
}

/// Minimum viable tip amount for a given fee_bps.
/// Returns the smallest amount that results in creator_amount > 0.
pub fn minimum_viable_tip(_fee_bps: u32) -> i128 {
    // We need: amount - (amount * fee_bps / 10_000) > 0
    // With integer division, the smallest amount where this holds is 1
    // For any valid fee_bps < 10_000, amount=1 gives fee=0, creator=1 (OK)
    1
}
