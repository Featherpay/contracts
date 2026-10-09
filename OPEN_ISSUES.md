# Open Issues Before Mainnet

This document tracks work remaining before TipRouter can launch on mainnet.

## M6 - External Security Audit (HARD GATE)

- [ ] **External security audit**
  - Engage reputable Soroban auditor (e.g., Trail of Bits, Spearbit, OtterSec)
  - Audit scope: `src/tip_router/`, `src/fee/`, `src/admin/`
  - Must address: reentrancy, integer overflow, authorization, storage
  - **Blocker**: No mainnet deployment without clean audit report
  - Budget: $50k-$100k
  - Timeline: 4-6 weeks

## M7 - Testnet Beta Under Real Volume

- [ ] **Deploy to Stellar testnet**
  - Use `scripts/deploy.sh` with testnet credentials
  - Verify contract ID, fee config, admin controls

- [ ] **Integration testing with real `featherpay/api` and `wallet-service`**
  - End-to-end tip flow: API → wallet-service → contract
  - Test idempotency, retries, paused contract handling
  - Verify reconciliation job matches on-chain events

- [ ] **Real tip volume load testing**
  - Simulate 100+ tips/minute
  - Measure: latency, gas costs, contract balance
  - Verify no stuck funds under load

- [ ] **Fee economics validation**
  - Measure actual processor + anchor + protocol fees
  - Validate "near-zero fees" claim at $1 tip scale
  - Adjust `PROTOCOL_FEE_BPS` if needed

## M8 - Mainnet Deployment with Caps

- [ ] **Mainnet deployment with initial volume caps**
  - Deploy immutable contract to mainnet
  - Set conservative `PROTOCOL_FEE_BPS` (e.g., 50 bps = 0.5%)
  - Set `WITHDRAWAL_DAILY_CAP_CENTS` low initially ($100/day/creator)
  - Monitor for 30 days before increasing caps

- [ ] **Production monitoring & alerting**
  - Contract balance alerts (should always be 0)
  - Failed transaction alerts
  - Pause circuit breaker activation alerts
  - Reconciliation discrepancy alerts

- [ ] **Disaster recovery runbooks**
  - Contract pause procedure
  - New contract deployment + migration procedure
  - Emergency contact list (auditors, Stellar Foundation, etc.)

## Technical Debt / Improvements

- [ ] **Batch on-chain settlement** (gas optimization)
  - Currently: each tip = separate `send_tip` call
  - Future: meta-transactions or batch settlement
  - Requires contract upgrade (new immutable deployment)

- [ ] **Multi-asset support beyond USDC**
  - EURC, other stablecoins
  - Asset validation in contract

- [ ] **Time-weighted average price (TWAP) for fee calculation**
  - Current: fixed fee_bps
  - Future: dynamic fee based on gas/processor costs

- [ ] **Formal verification**
  - Use Soroban's formal verification tools
  - Prove: contract balance always 0, no reentrancy

## Documentation

- [ ] **Complete API reference** (auto-generated from contract)
- [ ] **Integration guide for `featherpay/api`**
- [ ] **Runbooks for operations team**
- [ ] **Architecture decision records (ADRs)** for key choices

## Dependencies

- [ ] **featherpay/api** production-ready (M6)
- [ ] **featherpay/wallet-service** production-ready (M7)
- [ ] **Stellar anchor partner** production agreement (M7)
- [ ] **Stellar network** stability (no protocol upgrades breaking contract)

---

## Priority Legend

- 🔴 **Blocking** - Must complete before next milestone
- 🟡 **Important** - Should complete before next milestone
- 🟢 **Nice to have** - Can defer

---

## Audit Checklist (for external auditor)

When engaging an auditor, provide this checklist:

### Authorization
- [ ] `send_tip` requires `tipper.require_auth()`
- [ ] `send_tip_with_fee` requires `tipper.require_auth()`
- [ ] `pause`/`unpause`/`set_fee_bps` require `admin.require_auth()`
- [ ] No path bypasses authorization

### Integer Safety
- [ ] Fee calculation uses checked arithmetic (`checked_mul`, `checked_div`)
- [ ] Amount validation: `amount > 0`, `fee_bps < 10_000`
- [ ] `creator_amount > 0` enforced

### Storage
- [ ] Instance storage TTL extended on every read/write
- [ ] `paused` and `fee_bps` persisted correctly

### Reentrancy
- [ ] State changes (none in this contract) before external calls
- [ ] Token transfers are last operations
- [ ] Events emitted after transfers

### Events
- [ ] `TipSent` includes: tipper, creator, amount, asset, timestamp
- [ ] `TipSentWithFee` includes: tipper, creator, amount, asset, fee_amount, creator_amount, treasury, timestamp
- [ ] Events only emitted after successful transfers

### Upgradeability
- [ ] Contract is immutable (no `upgrade` function)
- [ ] Admin can only pause/unpause, set fee_bps

---

**Last Updated:** 2026-10-09
**Next Review:** Before M6 audit kickoff