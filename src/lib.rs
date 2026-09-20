//! Featherpay TipRouter — the Soroban smart contract that routes every tip on
//! Featherpay.
//!
//! Day 1 scaffold: no contract logic exists yet. The threat model in
//! `docs/decisions/threat-model.md` must be read before any fund-movement code
//! is written (see `PLAN.md` M0).
//!
//! The crate is a workspace-less single contract package targeting
//! `wasm32-unknown-unknown`. Modules:
#![no_std]

pub mod admin; // Admin-gated configuration functions
pub mod fee; // Fee calculation and rounding logic
pub mod tip_router; // send_tip, send_tip_with_fee, pause/unpause
