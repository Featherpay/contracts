# TipRouter Threat Model

> **Status: living document — written before any contract logic exists (Day 1, M0).**
> Every question here is a hard requirement on the eventual implementation.
> Questions map to regression tests: no answer is "it will never happen";
> each is "the contract must make it impossible / the tests must prove it".

## Scope and trust assumptions

The TipRouter contract routes every tip on Featherpay. It is **not** a custody
solution: it never holds a private key, never initiates a transaction on its
own, and is only ever invoked by a call already signed by the tipper's embedded
wallet (`featherpay/wallet-service`). The contract's invariant is that it is a
stateless-through-the-transaction money router: funds may pass *through* it,
never *rest* in it.

Trust assumptions we design against rather than assume away:

- The tipper's wallet signature is the root authorization primitive (same one
  used for a plain transfer — no new custody model).
- The contract lives on the blockchain and anyone can attempt to call it.
- Transfers of the configured `asset` are to a standard Soroban fungible token
  implementing `transfer`/`transfer_from`/`balance`.
- The admin key is a real, attackable secret — its compromise is a scenario we
  must bound, not bracket.

## Questions the model must answer

### 1. What happens if `send_tip` is called with a zero amount?

**Outcome required:** the call must fail before any state change or transfer.

A zero-amount tip is either a client bug (API sent a bad message) or an
exploit probe (attacker probing whether they can mint free `TipSent` events,
spam the event log, or trigger fee/token edge cases cheaply). It must not
succeed silently, because succeeding would let anyone emit unlimited events and
would mask every downstream error.

**Enforced by:** explicit `amount <= 0` rejection (Day 2 requirement: "must not
compile or execute if `amount <= 0`") with a dedicated `InvalidAmount`-style
error and a test that both balances are unchanged afterward.

### 2. What happens with a malformed or non-existent recipient address?

**Outcome required:** the call must fail cleanly, with no fund movement.

Two distinct cases:

- **Malformed / zero address.** A structurally invalid `Address` is rejected at
  decode time by the framework; a zero-address-like `Address::from_contract_data`
  / empty `AccountId` must be rejected by explicit validation before any
  transfer, because the token layer will happily treat such an address as a
  valid transfer target (money would go somewhere untraceable).
- **Non-existent but well-formed address.** A well-formed `Address` (a valid
  strkey that has never been funded, or a contract address that doesn't
  implement the expected token/balance interface) is hard to distinguish from
  valid *ex ante*. The contract must therefore make the transfer leg itself the
  source of truth: if the token `transfer` fails, the whole transaction reverts
  atomically. The creator is never left "half paid".

**Enforced by:** address validation for the obvious-malformed cases + relying on
atomic revert of the token transfer for the "funded or not" cases; tests assert
both balances unchanged and contract balance zero after each failure.

### 3. What happens if the tipper has insufficient balance or has not granted the required allowance?

**Outcome required:** full transaction revert, zero partial state change.

The pull model is tipper → contract → creator. The withdrawal from the tipper
uses the same allowance/`transfer_from` pattern the tipper's wallet already
uses for plain transfers. If the tipper lacks balance, or the contract's
allowance is insufficient, the token's `transfer_from` call panics and —
because fees, transfers, and event emission are all inside one transaction —
the entire call reverts. There is no window in which the contract has taken
money it cannot forward.

**Enforced by:** the atomicity of a single Soroban transaction plus tests that
(insufficient balance) and (insufficient allowance) both leave every balance
exactly as it was and no `TipSent` event emitted.

### 4. Can the contract ever end a transaction holding a non-zero balance (are funds at risk of getting stuck)?

**Outcome required:** no — this is the cardinal invariant.

The contract's fund-flow shape is receive → forward in the same atomic
transaction. The contract balance of `asset` must be **exactly zero** whenever
a transaction ends, whether that transaction succeeded or failed.

The two ways funds get stuck, and the defenses:

- **Code path where a transfer leg is skipped.** If a bug skipped the forward
  leg, money would sit in the contract. Defense: every leg (pull, forward,
  fee split) must be in the same transaction; there is no "store balance and
  pay later" state machine to get stuck in. Tests in
  `tests/safety/stuck_funds.rs` query the contract's *actual on-chain balance*
  after success and after every failure path and require it to be `0`.
- **Fees that round the creator's share to zero.** If fee math can produce
  `creator_amount = 0`, a tip could be "fully consumed" as fee with nothing
  forwarded. Defense: fee-bps validation that rejects any rate producing
  `creator_amount == 0`, and a minimum-viable-tip rejection (see `src/fee/`
  and Day 4).

**Enforced by:** the stuck-funds test suite, which is a formal-ish proof by
construction: run every success and every error path, then assert contract
balance is zero.

### 5. What is the blast radius if the admin key is compromised?

**Outcome required:** fund movement must NOT be part of the blast radius.

Admin powers under design (M2/M3):

- `set_fee_bps` — changes the default fee rate future calls use.
- `pause` / `unpause` — halts or resumes tip routing.
- `upgrade` (if Option B is chosen, Day 5) — swaps the contract wasm. This is
  the only admin power that approaches fund risk, and only in the sense that a
  compromised admin can deploy contract *code* that then routes funds — the
  admin key alone never moves a user's funds directly; the tipper wallet still
  has to authorize every pull.

Consequences of compromise:

- **Reputation/fee trust (high):** an attacker controlling `set_fee_bps` could
  set a 99.99% fee on new transactions. Mitigation: fee rate held back by
  per-call `fee_bps` validation (never above a sane bound), administrative
  review cadence, and pausing funds flow entirely is *within* a legitimate
  admin's remit anyway.
- **Availability (high):** attacker can `pause()` the contract, halting tips.
  Mitigation: `api`'s feature-flagged fallback to direct transfers keeps the
  product moving during incident response (see `PLAN.md` M4); `unpause` is a
  known key-rotation question.
- **Code replacement (only if upgradeable):** attacker can deploy a new wasm.
  Mitigation (if upgradeable): multi-sig or off-chain governance over the admin
  address, and treat the upgrade+deploy operation as the deployment-grade
  secret it is.
- **Direct fund theft (none):** the admin key cannot invoke `send_tip` on
  behalf of a tipper and cannot withdraw from the contract, because the
  contract holds nothing and every transfer requires the tipper's
  authorization.

**Enforced by:** non-admin-call tests on every admin function, and the fact
that no admin function signature takes a beneficiary or moves balance directly.

### 6. What authorization model prevents an unauthorized caller from invoking `send_tip` on behalf of a tipper?

**Outcome required:** `send_tip` must only execute if the tipper's address
itself has authorized the call.

The model is identical to the one `wallet-service` already uses for plain
transfers, extended one hop:

1. `featherpay/api` builds an unsigned `send_tip(tipper, creator, amount,
   asset)` call.
2. `featherpay/wallet-service` signs it as the tipper (the embedded wallet key
   is the tipper's own key; the contract introduces no second signer, no
   shared secret, no vault key).
3. The contract invokes `tipper.require_auth()` on the **tipper** argument —
   not the contract address, not `env.current_contract_address()` for the
   identity of *who* is authorized. The Soroban auth framework then verifies
   that the signing account/contracts cover the tipper's authorization
   (checking the tipper's `require_auth` against the transaction's signature
   payload for `(tipper, contract=this, fn=send_tip, args=...)`).

Properties this buys:

- **No privescalation:** an attacker cannot call `send_tip` with someone
  else's `tipper` to spend their balance — authorization is bound to the
  concrete `tipper` address, the concrete contract, and the concrete arguments.
- **Replay-safe per intent:** successful reuse requires the tipper wallet to
  sign again; the framework archives (nonce) used auth payloads.
- **Contract-forwarding cannot be weaponized against third parties:** a
  malicious third-party contract that *calls into* TipRouter still cannot make
  a tip move unless the tipper signed that exact call.

**Enforced by:** the Day 3 "unauthorized caller" test — a call built and
submitted *without* tipper authorization must fail the `require_auth` check and
revert, with no partial state change.

## Threat surface summary

| # | Threat | Severity | Primary defense | Proof |
|---|---|---|---|---|
| 1 | Zero/negative amount | Low | Immediate reject | `tests/happy_path` + `tests/safety` |
| 2 | Malformed/zero recipient | Med | Validate, plus atomic revert | `tests/safety` |
| 3 | Insufficient balance/allowance | High | Atomic revert of whole txn | `tests/safety` |
| 4 | Stuck funds (non-zero contract balance) | **Critical** | Single-transaction flow, no hold state | `tests/safety/stuck_funds.rs` |
| 5 | Compromised admin key | High | Admin powers exclude fund movement | Admin non-caller tests |
| 6 | Unauthorized `send_tip` / priv-esc | **Critical** | `tipper.require_auth()` on tipper only | Day 3 unauthorized test |
| 7 | Fee rounding exploits (M2) | Med | Fee-bps validation, min-viable-tip check | `tests/fee_edge_cases` |
| 8 | Paused-state bypass (M3) | High | `ContractPaused` check before any leg | `tests/safety` |

## Standing rules for contributors (see also PLAN.md §7 / Day 7 checklist)

- Every fund-movement change ships with an explicit edge-case/error-path test.
- "It reverts" is not a sufficient claim: the test must show *which* error and
  *that* balances (tipper, creator, contract) are unchanged and contract balance
  is `0`.
- Anything that weakens the cardinal invariant (stuck funds = 0) is a design
  change and needs a Decision record here under `docs/decisions/`.