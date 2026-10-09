# TipRouter Contract Interface

This document describes the public interface of the TipRouter Soroban smart contract.

## Contract Information

- **Contract ID**: Set via `TIP_ROUTER_CONTRACT_ID` environment variable
- **Network**: Stellar (Public / Testnet)
- **Asset**: USDC (or other 6-decimal asset) - address provided at call time

---

## Functions

### `send_tip`

Sends a tip from `tipper` to `creator` atomically.

#### Signature
```rust
pub fn send_tip(env: Env, tipper: Address, creator: Address, amount: i128, asset: Address)
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `tipper` | `Address` | The address sending the tip (must authorize the call) |
| `creator` | `Address` | The address receiving the tip |
| `amount` | `i128` | The amount to tip in atomic units (must be > 0) |
| `asset` | `Address` | The token contract address for the asset being tipped |

#### Authorization
Requires `tipper.require_auth()` — the tipper's embedded wallet must sign the transaction.

#### Preconditions
1. `amount > 0` (rejects zero/negative with `Error::InvalidAmount`)
2. `tipper != creator` (rejects self-tips with `Error::SelfTip`)
3. Tipper has sufficient balance of `asset`
4. Tipper has granted sufficient allowance to the contract for `asset`
5. Contract is not paused

#### Error Cases
| Error | Code | Condition |
|-------|------|-----------|
| `InvalidAmount` | 1 | `amount <= 0` |
| `SelfTip` | 2 | `tipper == creator` |
| `InsufficientBalance` | 3 | Tipper balance < amount (token contract error) |
| `InsufficientAllowance` | 4 | Tipper allowance < amount (token contract error) |
| `ContractPaused` | 6 | Contract is paused |

#### Events
Emits `TipSent` event after successful transfer:
```rust
pub struct TipSent {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub timestamp: u64,
}
```

#### Atomicity
The transfer is atomic: `tipper` → contract → `creator` in one transaction. Contract balance is zero before and after.

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --source tipper \
  --network testnet \
  -- send_tip \
  --tipper $TIPPER_ADDRESS \
  --creator $CREATOR_ADDRESS \
  --amount 1000000 \
  --asset $USDC_CONTRACT_ID
```

---

### `send_tip_with_fee`

Sends a tip with a protocol fee split between creator and treasury.

#### Signature
```rust
pub fn send_tip_with_fee(
    env: Env,
    tipper: Address,
    creator: Address,
    amount: i128,
    asset: Address,
    fee_bps: u32,
    treasury: Address,
)
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `tipper` | `Address` | The address sending the tip (must authorize) |
| `creator` | `Address` | The address receiving the creator portion |
| `amount` | `i128` | Total tip amount in atomic units (must be > 0) |
| `asset` | `Address` | The token contract address for the asset |
| `fee_bps` | `u32` | Protocol fee in basis points (1–9999) |
| `treasury` | `Address` | Address receiving the protocol fee |

#### Fee Calculation
```
fee_amount = (amount * fee_bps) / 10_000  // integer division, round down
creator_amount = amount - fee_amount
```
Both `fee_amount > 0` and `creator_amount > 0` required.

#### Authorization
Requires `tipper.require_auth()`.

#### Preconditions
All `send_tip` preconditions plus:
1. `fee_bps > 0` and `fee_bps < 10_000` (rejects with `Error::InvalidFeeBps`)
2. `creator_amount > 0` after fee calculation (rejects with `Error::FeeConsumesEntireAmount`)

#### Error Cases
| Error | Code | Condition |
|-------|------|-----------|
| `InvalidAmount` | 1 | `amount <= 0` |
| `SelfTip` | 2 | `tipper == creator` |
| `InsufficientBalance` | 3 | Tipper balance < amount |
| `InsufficientAllowance` | 4 | Tipper allowance < amount |
| `ContractPaused` | 6 | Contract is paused |
| `InvalidFeeBps` | 7 | `fee_bps == 0` or `fee_bps >= 10_000` |
| `FeeConsumesEntireAmount` | 8 | Fee rounds to >= amount |

#### Events
Emits `TipSentWithFee` event:
```rust
pub struct TipSentWithFee {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub fee_amount: i128,
    pub creator_amount: i128,
    pub treasury: Address,
    pub timestamp: u64,
}
```

#### Atomicity
Atomic three-way transfer: `tipper` → contract → `creator` (creator_amount) + `treasury` (fee_amount). Contract balance zero before/after.

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --source tipper \
  --network testnet \
  -- send_tip_with_fee \
  --tipper $TIPPER_ADDRESS \
  --creator $CREATOR_ADDRESS \
  --amount 1000000 \
  --asset $USDC_CONTRACT_ID \
  --fee_bps 100 \
  --treasury $TREASURY_ADDRESS
```

---

### `pause`

Pauses the contract — stops all tip processing.

#### Signature
```rust
pub fn pause(env: Env, admin: Address)
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Admin address (must authorize) |

#### Authorization
Requires `admin.require_auth()`.

#### Effects
- Sets paused state to `true`
- All subsequent `send_tip` and `send_tip_with_fee` calls fail with `Error::ContractPaused`
- Does not affect already-settled tips

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- pause \
  --admin $ADMIN_ADDRESS
```

---

### `unpause`

Unpauses the contract — resumes tip processing.

#### Signature
```rust
pub fn unpause(env: Env, admin: Address)
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Admin address (must authorize) |

#### Authorization
Requires `admin.require_auth()`.

#### Effects
- Sets paused state to `false`
- `send_tip` and `send_tip_with_fee` resume normal operation

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- unpause \
  --admin $ADMIN_ADDRESS
```

---

### `set_fee_bps`

Sets the default protocol fee basis points (admin only).

#### Signature
```rust
pub fn set_fee_bps(env: Env, admin: Address, fee_bps: u32)
```

#### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Admin address (must authorize) |
| `fee_bps` | `u32` | New default fee in basis points (0–9999) |

#### Authorization
Requires `admin.require_auth()`.

#### Validation
Rejects `fee_bps >= 10_000` with `Error::InvalidFeeBps`.

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- set_fee_bps \
  --admin $ADMIN_ADDRESS \
  --fee_bps 100
```

---

### `get_fee_bps`

Gets the current default protocol fee basis points.

#### Signature
```rust
pub fn get_fee_bps(env: Env) -> u32
```

#### Returns
Current `fee_bps` value (default: 0).

#### Example Soroban CLI Invocation
```bash
soroban contract invoke \
  --id $TIP_ROUTER_CONTRACT_ID \
  --network testnet \
  -- get_fee_bps
```

---

## Events

### `TipSent`
Emitted by `send_tip` on success.
```rust
pub struct TipSent {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub timestamp: u64,
}
```

### `TipSentWithFee`
Emitted by `send_tip_with_fee` on success.
```rust
pub struct TipSentWithFee {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub asset: Address,
    pub fee_amount: i128,
    pub creator_amount: i128,
    pub treasury: Address,
    pub timestamp: u64,
}
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1 | `InvalidAmount` | Amount must be positive |
| 2 | `SelfTip` | Tipper and creator cannot be the same |
| 3 | `InsufficientBalance` | Tipper has insufficient balance |
| 4 | `InsufficientAllowance` | Tipper has not granted sufficient allowance |
| 5 | `InvalidAsset` | Asset address is invalid or unsupported |
| 6 | `ContractPaused` | Contract is paused |
| 7 | `InvalidFeeBps` | Fee basis points must be > 0 and < 10,000 |
| 8 | `FeeConsumesEntireAmount` | Fee would leave creator with 0 |
| 9 | `Unauthorized` | Caller not authorized |

---

## Security Considerations

1. **Authorization**: All fund-movement functions require `tipper.require_auth()`. Admin functions require `admin.require_auth()`.
2. **Reentrancy**: State updates (none in this contract) happen before external calls. Token transfers are the last operation.
3. **Integer Safety**: All arithmetic uses checked operations. Fee calculation uses `checked_mul` and `checked_div`.
4. **Paused State**: Stored in instance storage with appropriate TTL extension.
5. **Immutability**: Contract is deployed as immutable WASM. Bugs require new deployment + migration.

---

## Integration with `featherpay/api`

The `featherpay/api` service builds unsigned XDR transactions calling these functions:

1. **Unsigned XDR Construction**: `api` uses Stellar SDK to build `invokeContractFunction` operations
2. **Signing**: Unsigned XDR sent to `wallet-service` for tipper's embedded wallet signature
3. **Submission**: Signed XDR submitted via Soroban RPC
4. **Reconciliation**: `api` verifies `TipSent`/`TipSentWithFee` events on-chain match DB records

### Sequence Number Handling
- `api` fetches fresh sequence number for each tip on FIRST attempt
- On retry (transient network error), SAME XDR is reused (same sequence number)
- This prevents double-submission with different sequence numbers

### Fee Decision Logic
```typescript
if (config.protocolFeeBps > 0 && config.tipRouterTreasuryPublicKey) {
  unsignedXdr = contracts.buildSendTipWithFeeCall({ feeBps, treasuryPublicKey, ... });
} else {
  unsignedXdr = contracts.buildSendTipCall({ ... });
}
```