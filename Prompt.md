# Featherpay Contracts — 7-Day Intensive Development Prompt

> **Goal:** Reach ~90% completion of the `TipRouter` Soroban smart contract — covering M0 through M3 of the project plan — so that external contributors have a robust, tested, and well-documented foundation to build on.
>
> **Stack:** Rust · Soroban SDK · Stellar testnet · `cargo test` · `cargo clippy` · `cargo tarpaulin`
>
> **Key constraint:** Every tip on Featherpay is routed through this contract. The bar for correctness is high. No fund-movement change is complete without an explicit edge-case test alongside it.

---

## Day 1 — Repo Scaffold, CI, and Threat Model (M0)

**Objective:** Get a working build, a green CI pipeline, and a written threat model before a single line of contract logic exists.

```
You are building `featherpay/contracts` — the Soroban smart contract that routes every tip on Featherpay.

Before writing any contract logic, complete the following:

1. Scaffold the repo with this directory structure:
   contracts/
   ├── src/
   │   ├── tip_router/     # send_tip, send_tip_with_fee, pause/unpause
   │   ├── fee/            # fee calculation and rounding logic
   │   └── admin/          # admin-gated configuration functions
   ├── tests/
   │   ├── happy_path/
   │   ├── fee_edge_cases/
   │   └── safety/
   ├── docs/
   │   ├── interface.md
   │   └── decisions/
   ├── scripts/
   ├── PLAN.md
   └── README.md

2. Add a Cargo.toml that targets `wasm32-unknown-unknown` and includes the Soroban SDK as a dependency (pinned to a specific version).

3. Set up CI (GitHub Actions) that runs on every PR:
   - `cargo build --target wasm32-unknown-unknown --release`
   - `cargo test --workspace`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`

4. Write `docs/decisions/threat-model.md` covering the specific threat surface of a funds-routing contract:
   - What happens if `send_tip` is called with a zero amount?
   - What happens with a malformed or non-existent recipient address?
   - What happens if the tipper has insufficient balance or has not granted the required allowance?
   - Can the contract ever end a transaction holding a non-zero balance (i.e., are funds at risk of getting stuck)?
   - What is the blast radius if the admin key is compromised?
   - What authorization model prevents an unauthorized caller from invoking `send_tip` on behalf of a tipper?

5. Confirm `soroban contract build` succeeds on the empty scaffold before moving on.

Do not write any `send_tip` logic yet. The threat model must exist before the contract does.
```

---

## Day 2 — `send_tip`: Core Contract Logic and Happy-Path Tests (M1, Part 1)

**Objective:** Implement `send_tip` with correct authorization and fund transfer, backed by thorough happy-path tests.

```
Implement `send_tip` in `src/tip_router/` for the Featherpay TipRouter contract.

Function signature:
  send_tip(env: Env, tipper: Address, creator: Address, amount: i128, asset: Address)

Requirements:
1. The call must be authorized by the tipper via `env.current_contract_address()` and `tipper.require_auth()`. The tipper's embedded wallet (managed by `featherpay/wallet-service`) will sign this call — the contract introduces no new custody model.
2. Pull `amount` of `asset` from `tipper` to the contract, then immediately forward the full amount to `creator` in the same transaction. The contract must never end the transaction holding a balance.
3. Emit a `TipSent` event after a successful transfer: fields are `tipper`, `creator`, `amount`, `asset`, and the ledger timestamp.
4. The contract must not compile or execute if `amount <= 0`.

Write tests in `tests/happy_path/` covering:
- A standard tip between two distinct addresses succeeds and emits the correct `TipSent` event
- The tipper's balance decreases by exactly `amount`
- The creator's balance increases by exactly `amount`
- The contract's own balance is zero after the transaction completes (funds never stuck)

Use Soroban's test utilities (`soroban_sdk::testutils`) for all tests. No mocks — test against real token contract behavior in the local sandbox.

Run `cargo test --workspace` and `cargo clippy -- -D warnings` before considering this done.
```

---

## Day 3 — `send_tip` Error Paths and Authorization Tests (M1, Part 2)

**Objective:** Harden `send_tip` against all failure cases described in the threat model.

```
Continuing the TipRouter contract in `src/tip_router/`:

Add exhaustive error-path tests in `tests/happy_path/` and `tests/safety/` for `send_tip`:

1. **Zero amount** — calling `send_tip` with `amount = 0` must panic or return a contract error; it must not silently succeed.
2. **Negative amount** — `amount < 0` must be rejected.
3. **Insufficient balance** — if the tipper does not have enough of `asset`, the transaction must revert cleanly with no partial state change.
4. **Insufficient allowance** — if the tipper has not granted the contract sufficient allowance, the transaction reverts.
5. **Self-tip** — `tipper == creator` should be explicitly rejected (a tipper cannot route a tip to themselves).
6. **Unauthorized caller** — a call where the tipper has NOT authorized the transaction must fail the `require_auth` check and revert.
7. **Unsupported or invalid asset** — passing a malformed asset address must revert without leaving any state change.
8. **Malformed addresses** — both `tipper` and `creator` with invalid/zero addresses must be rejected before any fund movement.

For each error case:
- The test must assert the specific error type or panic message, not just that "something went wrong."
- The test must assert that no partial fund movement occurred (both balances unchanged after a failed call).

Run `cargo tarpaulin --workspace` and confirm coverage on `src/tip_router/` is ≥ 95% before considering this day done.
```

---

## Day 4 — Fee Routing: `send_tip_with_fee` and Rounding Logic (M2)

**Objective:** Implement fee splitting inside the contract with watertight rounding and edge-case tests.

```
Implement `send_tip_with_fee` and the fee calculation module in `src/fee/` for the TipRouter contract.

Function signature:
  send_tip_with_fee(env: Env, tipper: Address, creator: Address, amount: i128, asset: Address, fee_bps: u32, treasury: Address)

Requirements:
1. Fee math lives entirely in `src/fee/`, not inlined in `src/tip_router/`. The tip_router module calls into fee/ for all calculations.
2. The fee calculation must be:
   - `fee_amount = (amount * fee_bps) / 10_000`  (integer division, round down)
   - `creator_amount = amount - fee_amount`
3. In one atomic transaction: pull `amount` from tipper → send `creator_amount` to creator → send `fee_amount` to treasury. All three legs succeed or the whole transaction reverts. The contract's own balance must be zero after the transaction.
4. Emit a `TipSent` event including both `creator_amount` and `fee_amount` as separate fields.
5. Reject invalid `fee_bps` values:
   - `fee_bps = 0` on `send_tip_with_fee` should be rejected (use `send_tip` instead)
   - `fee_bps >= 10_000` (100% or more) must be rejected
   - Validate that `creator_amount > 0` after fee calculation — if the fee rounds up to consume the entire amount, reject the call.

Write tests in `tests/fee_edge_cases/` covering:
- Standard fee split: correct amounts reach creator and treasury, contract balance is zero
- Rounding at micropayment scale: a $0.50 tip (50 cents) with a 1% fee (100 bps) — assert exact amounts
- Minimum viable tip: the smallest `amount` that still results in `creator_amount > 0` given a given `fee_bps`
- `fee_bps = 0` is rejected
- `fee_bps = 10_000` is rejected
- `fee_bps = 10_001` is rejected
- An amount so small that after fee calculation `creator_amount = 0` is rejected
- `treasury == creator` is an explicit edge case — should it be allowed? Document and test the chosen behavior.

Admin-gated fee configuration in `src/admin/`:
- `set_fee_bps(env: Env, admin: Address, fee_bps: u32)` — stores a default fee rate; only callable by a stored admin address
- Test that a non-admin call to `set_fee_bps` is rejected
- Test that a valid admin call updates the stored rate

Run `cargo test --workspace` and `cargo clippy -- -D warnings`. Coverage on `src/fee/` must be ≥ 95%.
```

---

## Day 5 — Safety Mechanisms: Pause/Circuit Breaker and Stuck-Funds Verification (M3)

**Objective:** Implement the admin-gated circuit breaker and prove by test that funds cannot get stuck in the contract.

```
Implement safety mechanisms in the TipRouter contract.

**1. Pause / Circuit Breaker (`src/tip_router/` + `src/admin/`)**

Add `pause()` and `unpause()` functions:
  pause(env: Env, admin: Address)
  unpause(env: Env, admin: Address)

Requirements:
- Both functions require `admin.require_auth()` and must revert if called by a non-admin.
- When the contract is paused, any call to `send_tip` or `send_tip_with_fee` must revert immediately with a clear `ContractPaused` error — no fund movement, no event emission.
- When unpaused, `send_tip` and `send_tip_with_fee` resume normal behavior.
- The paused state is stored in contract storage and persists across transactions.

Write tests in `tests/safety/`:
- `send_tip` succeeds when not paused
- `send_tip` reverts with `ContractPaused` when paused, with no balance change
- `send_tip_with_fee` reverts when paused
- `unpause` restores normal behavior and the next `send_tip` succeeds
- A non-admin call to `pause()` is rejected (tipper/creator cannot trigger the circuit breaker)
- A non-admin call to `unpause()` is rejected

**2. Stuck-Funds Verification**

Write a dedicated test suite in `tests/safety/stuck_funds.rs` that proves by construction:
- After a successful `send_tip`, the contract's balance of `asset` is exactly 0
- After a successful `send_tip_with_fee`, the contract's balance of `asset` is exactly 0
- After a failed `send_tip` (any error path from Day 3), the contract's balance is exactly 0
- After a failed `send_tip_with_fee` (any error path), the contract's balance is exactly 0

Each test must query the contract's actual on-chain balance (not just assert no transfer happened) to prove funds are not silently held.

**3. Upgrade Policy Decision**

Write `docs/decisions/upgrade-policy.md` documenting the chosen upgrade approach:
- Option A: Immutable contract — deployed WASM is final; a bug requires a new contract address and migration
- Option B: Upgradeable via WASM hash swap under admin authorization

Choose one, document the rationale and tradeoffs (auditability, trust model, emergency response speed), and if Option B, implement `upgrade(env, admin, new_wasm_hash)` with a test that a non-admin call is rejected.

Run `cargo test --workspace`, `cargo clippy -- -D warnings`, and `cargo tarpaulin`. Overall coverage must be ≥ 95% across the workspace.
```

---

## Day 6 — Interface Documentation, `docs/interface.md`, and Integration Readiness (M3 → M4)

**Objective:** Produce complete, accurate interface documentation and make the contract ready for `featherpay/api` to integrate against.

```
The TipRouter contract logic is complete. Today's focus is on making it integration-ready for `featherpay/api` and future contributors.

**1. Generate and write `docs/interface.md`**

Document every public function of the TipRouter contract:
- Full function signature with typed parameters
- Description of what the function does
- Authorization requirements (who must sign, and how)
- Preconditions that must be true for the call to succeed
- Error cases and what each one means
- Emitted events: name, fields, and their types
- Example Soroban CLI invocation for each function

**2. Update `scripts/` with deploy and interaction helpers**

Write shell scripts (wrapping `soroban contract` CLI commands) for:
- `scripts/deploy.sh` — deploy to a given network (testnet/mainnet) with required arguments
- `scripts/send_tip.sh` — invoke `send_tip` with example parameters
- `scripts/send_tip_with_fee.sh` — invoke `send_tip_with_fee` with example parameters
- `scripts/pause.sh` / `scripts/unpause.sh` — admin circuit breaker invocation
- Each script must include a usage comment at the top explaining required env vars (identity, network, contract ID)

**3. Integration test: simulate the `featherpay/api` call pattern**

Write a test in `tests/happy_path/api_integration_sim.rs` that simulates exactly how `featherpay/api` will call the contract:
- Build an unsigned `send_tip` call (as `api` would)
- Authorize it as the tipper's embedded wallet (as `wallet-service` would sign it)
- Submit to the local sandbox
- Assert: creator receives the correct amount, `TipSent` event is emitted with all expected fields, contract balance is zero

This test is documentation as much as verification — it shows contributors exactly what `api` is expected to do.

**4. Review `README.md` against the implementation**

Check the contract's `README.md` for any discrepancies between the documented interface and the actual implementation:
- Correct any parameter names, types, or descriptions that have drifted
- Add a "Status" section noting what is complete vs. what is out of scope pending audit/mainnet

Commit all documentation and scripts. Run `cargo test --workspace` one final time to confirm nothing broke.
```

---

## Day 7 — Security Hardening Pass, Final CI Gate, and Contributor Handoff (M5, Partial)

**Objective:** Run a structured internal security review, fix any findings, and leave the repo in a state where an external contributor can pick up any open milestone confidently.

```
Today is a hardening and handoff day for the TipRouter contract. No new features. Focus entirely on correctness, security, and contributor experience.

**1. Internal Security Review**

Work through each item in this checklist and fix any issues found:

Authorization gaps:
- [ ] Every function that moves funds calls `require_auth()` on the correct signer before any state change
- [ ] Admin functions (`pause`, `unpause`, `set_fee_bps`, `upgrade` if implemented) reject non-admin callers before doing anything
- [ ] There is no code path in `send_tip` or `send_tip_with_fee` where `require_auth` is bypassed

Integer safety:
- [ ] All fee arithmetic (`amount * fee_bps / 10_000`) is checked for overflow — use checked arithmetic, not raw `*` on `i128`
- [ ] Negative `fee_bps` is impossible by type (u32), and zero/max values are rejected at runtime
- [ ] `creator_amount` and `fee_amount` are both > 0 before any transfer is attempted

Storage rent:
- [ ] Any data written to contract storage (paused flag, admin address, fee_bps) uses the appropriate storage tier (instance vs. persistent) with appropriate TTL extension calls

Reentrancy-equivalent:
- [ ] State updates (marking the tip as processed, if any internal state is kept) happen before external token transfers, not after

Event correctness:
- [ ] `TipSent` events include all fields required by `featherpay/api`'s reconciliation job: tipper, creator, amount, asset, fee_amount (0 if no fee), timestamp
- [ ] Events are only emitted after all transfers have succeeded

**2. Write a Regression Test for Every Finding**

For each issue found in Step 1 — even if it was a near-miss or a documentation gap — add a test that would have caught it. A security fix without a regression test is incomplete.

**3. Final Coverage Check**

Run `cargo tarpaulin --workspace --out Html`. Every module in `src/` must be at ≥ 95% coverage. If any module is below that threshold, write the missing tests now.

**4. Contributor Handoff: CONTRIBUTING.md**

Write `CONTRIBUTING.md` covering:
- How to set up the local dev environment (Rust toolchain, Soroban CLI, local network)
- How to run the test suite and what each test category covers
- The two-reviewer requirement for any PR touching `src/tip_router/` or `src/fee/`
- The requirement to add a fee-rounding/edge-case test alongside any change to `src/fee/`
- How to invoke the circuit breaker in a real incident
- A pointer to `docs/decisions/` for architectural context
- What "done" means for a PR (all CI checks green, coverage ≥ 95% on new logic, second reviewer for fund-movement changes)

**5. Open Issues**

Create a `OPEN_ISSUES.md` that honestly lists what remains before mainnet:
- External security audit (M6) — hard gate, not optional
- Testnet beta under real tip volume (M7)
- Integration testing with the real `featherpay/api` and `wallet-service` (not simulated)
- Mainnet deployment with initial volume caps (M8)

The repo should be in a state where a new contributor can clone it, read `CONTRIBUTING.md` and `docs/`, understand exactly what is built, what is tested, and what remains to do — without needing to ask anyone.
```

---

## Summary: What 90% Complete Looks Like

After these 7 days, the `featherpay/contracts` repo will have:

| Area | Status |
|---|---|
| Repo scaffold and CI | ✅ Complete |
| Threat model documented | ✅ Complete |
| `send_tip` — happy path and all error paths | ✅ Complete |
| `send_tip_with_fee` — fee math, rounding, edge cases | ✅ Complete |
| Admin-gated fee configuration | ✅ Complete |
| Pause/circuit-breaker | ✅ Complete |
| Stuck-funds proof by test | ✅ Complete |
| Upgrade policy decided and documented | ✅ Complete |
| `docs/interface.md` generated | ✅ Complete |
| Deploy and interaction scripts | ✅ Complete |
| Internal security hardening pass | ✅ Complete |
| ≥ 95% test coverage across workspace | ✅ Complete |
| CONTRIBUTING.md for new contributors | ✅ Complete |
| External audit | 🔲 Remaining (M6 — hard gate before mainnet) |
| Testnet beta under real volume | 🔲 Remaining (M7) |
| Mainnet deployment | 🔲 Remaining (M8) |
