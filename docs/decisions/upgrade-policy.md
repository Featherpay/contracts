# Upgrade Policy Decision

## Decision: Immutable Contract (Option A)

**Status: Accepted**

## Rationale

The TipRouter contract will be deployed as **immutable** (Option A). Once deployed, the WASM bytecode cannot be changed. Any bug fix or feature requires deploying a new contract address and migrating state.

### Tradeoffs Considered

| Factor | Immutable (A) | Upgradeable (B) |
|--------|---------------|-----------------|
| **Auditability** | ✅ Single, frozen codebase audited once | ❌ Requires re-audit on every upgrade |
| **Trust Model** | ✅ Users verify once, trust forever | ❌ Users must trust admin key not to rug |
| **Emergency Response** | ❌ Slow - new deploy + migration | ✅ Fast - single admin transaction |
| **Operational Complexity** | ✅ Simple - no upgrade logic | ❌ Complex - upgrade auth, migration scripts |
| **Regulatory** | ✅ Clear - code is law | ⚠️ Ambiguous - admin control = custodial risk |

## Why Immutable?

1. **Financial Contracts Demand Predictability**: TipRouter moves real money. Users and regulators need certainty that the logic they audited is the logic that runs forever.

2. **Admin Key Risk**: An upgradeable contract with admin-controlled WASM swap is effectively a custodial bridge. If the admin key is compromised, all funds can be stolen via malicious upgrade. Immutable removes this attack vector entirely.

3. **Audit Finality**: A single audit covers the contract for its entire lifetime. Upgradeable contracts require re-audit on every change, creating ongoing cost and risk.

4. **Simplicity**: No upgrade logic, no migration scripts, no version tracking in the contract itself. The contract does one thing and does it forever.

## Mitigation for Bugs

If a critical bug is discovered post-deploy:

1. **Pause Circuit Breaker**: Admin can immediately pause the contract via `pause()` to stop all fund movement.
2. **Deploy Fixed Contract**: Deploy new immutable TipRouter v2 with bug fix.
3. **Migrate State**: 
   - For tips: Tips are ephemeral (no long-term state). Just route new tips to v2.
   - For fee config: Re-set `fee_bps` on v2 via `set_fee_bps()`.
   - For paused state: Ensure v2 starts unpaused.
4. **Coordinate Cutover**: Update `featherpay/api` config to point to new contract ID. This is a single config change + redeploy.

## Upgrade Procedure (Emergency Only)

```
1. Admin calls pause() on v1
2. Deploy v2 (fixed WASM)
3. Admin calls set_fee_bps() on v2 with current fee
4. Update api config: TIP_ROUTER_CONTRACT_ID=v2
5. Redeploy api
6. Verify end-to-end tip flow on v2
7. Monitor for 24h
8. (Optional) Call unpause() on v1 to allow any stuck tips to complete, then pause again
```

## What About `set_fee_bps`?

The `fee_bps` is stored in contract storage and can be updated by admin. This is **not** a code upgrade - it's a parameter change within the same immutable logic. The fee calculation logic itself is immutable.

## Rejection of Option B

Upgradeable via WASM hash swap was rejected because:
- Introduces custodial risk (admin key = full fund control)
- Violates "code is law" principle for financial infrastructure
- Creates ongoing audit burden
- Adds complexity with marginal operational benefit (emergency pause + redeploy is fast enough)

## Future Reconsideration

This decision can be revisited if:
- Stellar introduces native contract upgrade with timelock/multisig
- Regulatory framework clarifies upgradeable contract treatment
- Operational experience shows pause+redeploy is insufficient

For now, **immutable is the only acceptable choice for a funds-routing contract**.