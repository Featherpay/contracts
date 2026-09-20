# Featherpay Contracts — Project Plan

## 1. Purpose
This repo holds the Soroban smart contract that powers every tip on Featherpay. Rather than `api` building a plain wallet-to-wallet transfer, every tip is routed through this contract: it receives the payment from the tipper's embedded wallet and forwards it to the creator in the same atomic transaction, optionally taking a protocol fee along the way. This is a **required, active component** of the core product, not an optional or deferred layer.

## 2. Scope
In scope:
- `TipRouter` contract — receives a tip payment and routes it to the intended creator atomically
- Fee handling — an optional protocol fee taken and routed to a Featherpay treasury account in the same transaction
- Event emission for every tip (for indexing/analytics, not long-term on-chain storage)
- Safety mechanisms: pause/circuit-breaker, upgrade path, reentrancy-equivalent safety checks

Out of scope:
- Key custody/signing → `wallet-service` (the tipper's embedded wallet still signs the call into this contract — the contract itself never holds a user's private key)
- Fiat conversion, batching decisions, ledger accounting → `api`
- UI → `frontend`

## 3. Why a Contract, Not a Direct Transfer
A direct wallet-to-wallet transfer is simpler, but a routing contract buys:
- **Atomicity** — the tip payment and (optional) fee split happen in a single transaction; there's no window where a fee transfer could succeed while the creator payment fails, or vice versa
- **A single, auditable choke point** — every tip flows through one well-tested piece of logic, rather than `api` needing to correctly construct fee-splitting logic across multiple transactions
- **On-chain verifiability** — every tip is independently verifiable by anyone querying the contract's emitted events, not just Featherpay's internal ledger
- **A foundation for future features** (split-tips, on-chain leaderboards) without re-architecting the payment path later

## 4. Milestones

### M0 — Foundations & Threat Model (Week 1-2)
- Repo scaffold, CI (build, test, clippy, fmt on every PR)
- Threat model specific to a funds-routing contract: what happens if the contract is called with a malformed recipient, a zero amount, an unsupported asset, or is invoked in a way that tries to leave funds stuck in the contract
- Local Soroban sandbox environment working end-to-end

### M1 — TipRouter Contract v1 (Week 2-5)
- `send_tip(tipper: Address, creator: Address, amount: i128, asset: Address)`
  - Pulls `amount` of `asset` from `tipper` (via the token's `transfer_from`/allowance pattern)
  - Forwards the full amount to `creator` in the same transaction
  - Emits a `TipSent` event (tipper, creator, amount, asset, timestamp)
- Authorization: the call must be authorized by the tipper (via `require_auth`), matching the signature `wallet-service` produces on `api`'s behalf
- Unit tests: happy path, insufficient balance, zero amount, self-tip, unsupported asset, malformed recipient address

### M2 — Fee Routing (Week 5-7)
- `send_tip_with_fee(tipper, creator, amount, fee_bps, treasury)`
  - Splits `amount` atomically: `amount * (1 - fee_bps/10000)` to `creator`, remainder to `treasury`
  - Rounding handled explicitly and tested (fee math must never let a tiny tip round to zero fee in a way that's exploitable, and must never take more than `fee_bps` implies)
- Configurable fee rate, settable only by an admin-controlled function, itself gated and tested
- Tests: fee boundary conditions, minimum viable tip size given fee rounding, malicious fee_bps values rejected

### M3 — Safety Mechanisms (Week 7-9)
- **Pause/circuit-breaker**: an admin-gated `pause()`/`unpause()` that stops `send_tip` from executing, for use if a vulnerability is discovered post-launch
- **Upgrade path**: decide and implement whether the contract is upgradeable (WASM hash swap under admin auth) or immutable-per-deployment — document the tradeoff explicitly in `docs/decisions/`
- **Stuck-funds check**: verify by design and by test that the contract can never end a transaction holding a balance it didn't intend to hold (no partial-failure state leaves funds trapped in the contract itself)

### M4 — Integration with `api` (Week 9-10)
- `api` updated to build calls to `send_tip`/`send_tip_with_fee` instead of plain transfers (see `featherpay/api` PLAN.md for its side of this integration)
- Feature-flagged rollout: `api` can fall back to a direct transfer path if a critical issue is found in the contract post-launch, without needing a contract redeploy to recover
- End-to-end testnet testing across `wallet-service` → `contracts` → `api` → `frontend`

### M5 — Security Hardening (Week 10-12)
- Internal review pass against common Soroban pitfalls (authorization checks, integer overflow/underflow in fee math, storage bloat/rent, upgrade-authorization gaps)
- Fix findings, write regression tests for each
- Freeze the contract interface ahead of audit

### M6 — External Audit (Week 12-16, calendar time depends on auditor availability)
- Engage a Soroban/Rust-experienced auditing firm — this is a funds-routing contract handling every tip on the platform, and gets the same audit priority as `wallet-service`
- Triage and fix findings; publish the audit report

### M7 — Testnet Launch (Week 16-18)
- Deploy to Stellar testnet behind the feature flag
- Run real (testnet) tip volume through it via the `frontend`/`api` beta, watching pause/circuit-breaker readiness and fee-math correctness under real usage

### M8 — Mainnet Launch (Week 18+, gated on audit completion + legal readiness)
- Deploy with conservative per-tip and daily volume caps enforced at the contract level initially, raised gradually as confidence builds
- Circuit breaker rehearsed (not just implemented) before go-live

## 5. Key Design Decisions (and why)

| Decision | Choice | Rationale |
|---|---|---|
| Routing model | Contract receives then forwards atomically, in a single transaction | Guarantees the tip and any fee split succeed or fail together — no partial-payment states |
| Fee handling | Computed and split inside the contract, not as a separate transaction from `api` | Atomicity and a single auditable source of truth for how fees are calculated |
| Authorization | Tipper's embedded wallet signs the call via `require_auth`, same as it would sign a plain transfer | No new custody model introduced — `wallet-service` still does exactly what it did before, just signing a contract call instead of a transfer |
| Upgradability | Decide explicitly in M3, document in `docs/decisions/` | A funds-routing contract's upgrade policy is a first-order security decision, not a detail to leave implicit |
| Rollback path | `api` retains a feature-flagged fallback to direct transfers | A funds-routing contract bug shouldn't be able to halt the entire product if `api` can revert to the simpler path while a fix ships |

## 6. Risks & Open Questions
- **Funds-in-contract risk**: Because the contract briefly receives funds before forwarding them, any bug that fails to forward correctly is a direct funds-at-risk bug, not just a logic bug — this raises the bar on M1/M2 testing and M6 audit scope relative to a pure record-keeping contract.
- **Fee rounding at micropayment scale**: A $0.50 tip with a small fee percentage introduces integer rounding that needs careful, explicit handling — get this wrong and either the treasury or the creator is systematically shorted by fractions of a cent that add up at volume.
- **Upgradability vs. trust**: An upgradeable contract is faster to patch but asks users to trust an admin key; an immutable contract is more trustworthy but can't be fixed without a full migration. This needs a clear decision and clear communication to creators, not a silent default.
- **Regulatory**: A contract that atomically takes a fee cut on every payment strengthens the case that Featherpay is operating as a payments intermediary — this should be part of the same legal review already flagged for `wallet-service` and `api`, not a separate afterthought.

## 7. Definition of Done (per milestone)
A milestone is done when: code merged to `main`, test coverage for new logic ≥ 95% given this contract routes real funds, CI green, and — starting at M2 — any change to fee math has an explicit rounding/edge-case test added alongside it.