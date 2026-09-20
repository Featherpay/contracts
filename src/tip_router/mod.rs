//! Tip routing: `send_tip`, `send_tip_with_fee`, `pause`/`unpause`.
//!
//! This module intentionally contains **no logic** on Day 1. The threat model
//! (`docs/decisions/threat-model.md`) is a prerequisite for implementing
//! `send_tip` (see `PLAN.md` M1). The struct below is the empty contract
//! skeleton that makes the scaffold buildable; `soroban contract build` must
//! pass before any fund-movement code lands.

use soroban_sdk::{contract, contractimpl};

/// The TipRouter contract. Fund-routing logic arrives in M1 after the threat
/// model in `docs/decisions/threat-model.md` is approved.
#[contract]
pub struct TipRouter;

#[contractimpl]
impl TipRouter {}
