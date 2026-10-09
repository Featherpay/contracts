# Contributing to Featherpay Contracts (TipRouter)

## Local Development Setup

### Prerequisites
- Rust 1.84+ (required for `wasm32v1-none` target)
- `stellar` CLI v28.0.0+ (`cargo install stellar-cli`)
- Soroban SDK (included via Cargo)

```bash
# Install stellar CLI
cargo install stellar-cli --locked

# Verify installation
stellar --version
```

### Building
```bash
# Build optimized WASM
stellar contract build

# Output: target/wasm32v1-none/release/tip_router.wasm
```

### Running Tests
```bash
# All tests
cargo test --workspace

# Specific test suites
cargo test --test send_tip_happy_path
cargo test --test send_tip_error_paths
cargo test --test fee_edge_cases
cargo test --test stuck_funds
cargo test --test api_integration_paused_contract

# With output
cargo test --workspace -- --nocapture
```

### Code Quality
```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Security audit (dependencies)
cargo audit
```

## Test Categories

| Suite | Purpose | Command |
|-------|---------|---------|
| `send_tip_happy_path` | Core `send_tip` functionality | `cargo test --test send_tip_happy_path` |
| `send_tip_error_paths` | All error conditions for `send_tip` | `cargo test --test send_tip_error_paths` |
| `fee_edge_cases` | `send_tip_with_fee` fee math & edge cases | `cargo test --test fee_edge_cases` |
| `stuck_funds` | Prove contract never holds funds | `cargo test --test stuck_funds` |
| `api_integration_paused_contract` | Simulate `featherpay/api` call pattern | `cargo test --test api_integration_paused_contract` |

## Security Requirements

### Two-Reviewer Rule
**MANDATORY**: Any PR touching `src/tip_router/` or `src/fee/` requires **two reviewers**.

This applies to:
- Fund movement logic (`send_tip`, `send_tip_with_fee`)
- Fee calculation (`src/fee/`)
- Pause/unpause circuit breaker
- Storage/state changes

### Fee Edge-Case Test Requirement
**MANDATORY**: Any change to `src/fee/` must include a fee-rounding/edge-case test alongside the change.

Required test scenarios:
- Standard fee split
- Rounding at micropayment scale
- Minimum viable tip
- Fee consumes entire amount rejection
- Treasury == creator edge case
- Pause interaction

### Adding External Clients
Not applicable — TipRouter is a standalone contract. Integration happens via:
1. `featherpay/api` builds unsigned XDR
2. `featherpay/wallet-service` signs
3. Soroban RPC submits

## Architecture Decisions

See `docs/decisions/` for context:
- `threat-model.md` — Fund-routing threat surface
- `upgrade-policy.md` — Immutable contract decision

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `TIP_ROUTER_CONTRACT_ID` | Deployed contract address | For integration |
| `SOROBAN_RPC_URL` | Soroban RPC endpoint | For deployment/testing |
| `NETWORK_PASSPHRASE` | Stellar network passphrase | For deployment |
| `ADMIN_SECRET_KEY` | Admin account secret | For admin operations |

## Deployment Scripts

See `scripts/`:
- `deploy.sh` — Deploy to testnet/mainnet
- `send_tip.sh` — Invoke `send_tip`
- `send_tip_with_fee.sh` — Invoke `send_tip_with_fee`
- `pause.sh` / `unpause.sh` — Circuit breaker

Each script documents required env vars at the top.

## Incident Response: Circuit Breaker

If TipRouter must be paused:

```bash
# 1. Admin pauses contract
export ADMIN_SECRET_KEY="..."
export ADMIN_ADDRESS="..."
export NETWORK_PASSPHRASE="..."
export RPC_URL="..."
./scripts/pause.sh testnet $CONTRACT_ID

# 2. Investigate issue (check logs, on-chain events)

# 3. Fix: deploy new immutable contract if bug in WASM
#    Or: call unpause() if transient issue

# 4. Admin unpauses
./scripts/unpause.sh testnet $CONTRACT_ID
```

## Definition of Done

A PR is "done" when:
- [ ] All CI checks pass (build, clippy, fmt, tests)
- [ ] Coverage ≥ 95% on new logic (`cargo tarpaulin`)
- [ ] Two reviewers for `src/tip_router/` or `src/fee/` changes
- [ ] Fee edge-case tests added for `src/fee/` changes
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation updated if interface changed