# Featherpay Contracts

**The Soroban smart contract that routes every tip on Featherpay.**

## Table of Contents

- [Overview](#overview)
- [Why a Contract, Not a Direct Transfer](#why-a-contract-not-a-direct-transfer)
- [Architecture](#architecture)
- [Contract Interface](#contract-interface)
- [Fee Handling](#fee-handling)
- [Safety Mechanisms](#safety-mechanisms)
- [Repo Structure](#repo-structure)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Testing](#testing)
- [Deploying](#deploying)
- [Security](#security)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [FAQ](#faq)
- [License](#license)

---

## Overview

When a tipper sends a tip on Featherpay, [`featherpay/api`](https://github.com/featherpay/api) doesn't build a plain wallet-to-wallet transfer — it builds a call into this contract's `send_tip` (or `send_tip_with_fee`) function. [`featherpay/wallet-service`](https://github.com/featherpay/wallet-service) signs that call on the tipper's behalf, exactly as it would sign a plain transfer — this contract introduces no new custody model. The contract then, in one atomic transaction:

1. Pulls the tip amount from the tipper
2. Forwards it to the creator
3. If a fee is configured, splits off the protocol's cut to a treasury account
4. Emits a `TipSent` event for indexing and analytics

## Why a Contract, Not a Direct Transfer

- **Atomicity** — the tip and any fee split succeed or fail together; there's no window where one leg completes without the other
- **A single, auditable choke point** — every tip flows through one well-tested contract, rather than `api` reconstructing fee logic across multiple transactions
- **On-chain verifiability** — anyone can independently verify tip activity from the contract's emitted events, not just Featherpay's internal ledger
- **Room to grow** — split-tips, on-chain leaderboards, and similar features build naturally on this contract later, without re-architecting the payment path

## Architecture

```
┌────────────┐   signed call    ┌──────────────────┐
│  api          │ ───────────────► │   TipRouter        │
│  (builds the   │                  │   contract           │
│   send_tip call)│                  │   (this repo)         │
└────────────┘                  │                       │
                                    │  1. pull from tipper  │
        ┌───────────────┐          │  2. forward to creator│
        │  wallet-service  │ ◄────── │  3. split fee (opt.)  │
        │  (signs on the    │  auth  │  4. emit TipSent      │
        │   tipper's behalf) │        └──────────┬───────────┘
        └───────────────┘                       │
                                                    ▼
                                          ┌──────────────────┐
                                          │  Creator + (opt.)   │
                                          │  treasury accounts   │
                                          └──────────────────┘
```

The contract never holds a private key and never initiates a transaction on its own — it only executes when called with a validly authorized signature from the tipper's embedded wallet.

## Contract Interface

| Function | Description |
|---|---|
| `send_tip(tipper, creator, amount, asset)` | Routes a tip from tipper to creator atomically, no fee |
| `send_tip_with_fee(tipper, creator, amount, asset, fee_bps, treasury)` | Same, but splits off `fee_bps` basis points to `treasury` in the same transaction |
| `pause()` / `unpause()` | Admin-gated circuit breaker — halts `send_tip*` execution if an issue is found |
| `set_fee_bps(fee_bps)` | Admin-gated update to the default protocol fee rate |

Full parameter types and emitted event schemas are documented in `docs/interface.md`.

## Fee Handling

Fee calculation happens inside the contract, not in `api`, so there is exactly one place fee math can go wrong and exactly one place it's tested. Rounding at micropayment scale is handled explicitly and tested at the boundary — see [PLAN.md](./PLAN.md) M2 for the specific edge cases (minimum viable tip size given rounding, fee-value validation, rejecting malformed `fee_bps` values).

## Safety Mechanisms

- **Circuit breaker** — an admin-gated `pause()` stops all tip routing immediately if a vulnerability is discovered post-launch, without needing a redeploy
- **Feature-flagged fallback in `api`** — if a critical contract issue is found, `api` can revert to a plain direct-transfer path while a fix ships, so the product doesn't have to fully halt
- **Upgrade policy** — documented explicitly in `docs/decisions/`, since whether this contract is upgradeable is a first-order security/trust decision, not an implementation detail

## Repo Structure

```
contracts/
├── src/
│   ├── tip_router/         # send_tip, send_tip_with_fee, pause/unpause
│   ├── fee/                  # Fee calculation and rounding logic
│   └── admin/                  # Admin-gated configuration functions
├── tests/
│   ├── happy_path/
│   ├── fee_edge_cases/
│   └── safety/                  # Pause behavior, malformed-input handling
├── docs/
│   ├── interface.md             # Generated contract interface reference
│   └── decisions/                 # Upgradability and other first-order design decisions
├── scripts/                        # Deploy + interaction scripts (Soroban CLI wrappers)
├── PLAN.md
└── README.md
```

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, with the `wasm32-unknown-unknown` target)
- [Soroban CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli)

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli
```

## Getting Started

```bash
git clone https://github.com/featherpay/contracts.git
cd contracts

soroban contract build
soroban network start local
```

## Testing

```bash
cargo test --workspace              # Unit + integration tests
cargo tarpaulin --workspace --out Html   # Coverage
```

Because this contract routes real funds, CI enforces:
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` on every PR
- ≥ 95% coverage on new logic
- A mandatory fee-rounding/edge-case test alongside any change to `src/fee/`

## Deploying

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/tip_router.wasm \
  --source <your-identity> \
  --network testnet

soroban contract invoke \
  --id <contract-id> \
  --source <your-identity> \
  --network testnet \
  -- send_tip --tipper ... --creator ... --amount ... --asset ...
```

See `scripts/` for wrapped versions of common deploy/invoke flows.

## Security

- **Not yet audited.** See [PLAN.md](./PLAN.md) for the audit milestone and timeline — this is a hard gate before mainnet, given the contract handles every tip's funds.
- Any PR touching `src/tip_router/` or `src/fee/` requires two reviewer approvals.
- Found a vulnerability? **Do not open a public issue.** Email `security@featherpay.io`.

## Roadmap

See [PLAN.md](./PLAN.md) for the full milestone breakdown: contract v1 → fee routing → safety mechanisms → `api` integration → security hardening → external audit → testnet → mainnet.

## Contributing

1. Fork and branch from `main`
2. Any change touching fund movement or fee math needs an explicit edge-case test — PRs without one will not be merged
3. Run `cargo fmt` and `cargo clippy` before opening a PR
4. Changes to `src/tip_router/` or `src/fee/` require two reviewer approvals, matching the bar used in `wallet-service`

## FAQ

**Does this contract ever hold a user's private key?**
No. It only ever executes a call that was already signed by the tipper's embedded wallet via `wallet-service`. This contract introduces no new custody model — it's the payment logic, not the signing logic.

**What happens if the contract has a bug after launch?**
The `pause()` circuit breaker halts routing immediately, and `api` can fall back to a feature-flagged direct-transfer path so tipping doesn't have to stop entirely while a fix ships.

**Why not just calculate the fee in `api` and send two transactions?**
Two transactions can't be guaranteed atomic — one could succeed while the other fails, leaving an inconsistent state. Doing it in one contract call removes that failure mode entirely.

## License
MIT
