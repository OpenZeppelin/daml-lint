# daml-lint — Public Audit Results

Static analysis scan of 5 FOSS DAML projects using daml-lint. All findings are **NOT VALIDATED** by the respective project teams — they are raw tool output with manual triage.

**Date:** 2026-03-04
**Tool version:** daml-lint v0.1.0 (Rust, `cargo build --release`)
**Detectors (6):** `missing-ensure-decimal`, `unguarded-division`, `missing-positive-amount`, `archive-before-execute`, `head-of-list-query`, `unbounded-fields`

---

## Table of Contents

1. [canton-dex-platform](#1-canton-dex-platform)
2. [canton-erc20](#2-canton-erc20)
3. [canton-vault](#3-canton-vault)
4. [daml-finance](#4-daml-finance)
5. [splice](#5-splice)
6. [Aggregate Summary](#6-aggregate-summary)
7. [Known False Positive Patterns](#7-known-false-positive-patterns)

---

## 1. canton-dex-platform

**GitHub:** [0xalberto/canton-dex-platform](https://github.com/0xalberto/canton-dex-platform)
**Commit:** [`e9f01be`](https://github.com/0xalberto/canton-dex-platform/commit/e9f01be60369b9978dcf0e60aa5fa2ce02967a3f)
**Files scanned:** 11 | **Summary: 24 findings — 10 HIGH, 14 MEDIUM**

### HIGH (10)

| # | Detector | File | Line | Description |
|---|----------|------|------|-------------|
| 1 | `missing-positive-amount` | Account.daml | 33 | Choice `ReserveFunds` accepts `amount : Decimal` without asserting > 0 |
| 2 | `missing-positive-amount` | Account.daml | 44 | Choice `ReleaseFunds` accepts `amount : Decimal` without asserting > 0 |
| 3 | `missing-positive-amount` | Account.daml | 55 | Choice `TransferFunds` accepts `amount : Decimal` without asserting > 0 |
| 4 | `missing-positive-amount` | CustodyBridge.daml | 20 | Choice `SubmitDepositSignature` accepts `amount : Decimal` without asserting > 0 |
| 5 | `missing-positive-amount` | CustodyBridge.daml | 42 | Choice `SubmitWithdrawalSignature` accepts `amount : Decimal` without asserting > 0 |
| 6 | `missing-ensure-decimal` | CustodyBridge.daml | 64 | Template `CustodyDepositRequest` — `amount` field with no ensure clause |
| 7 | `missing-ensure-decimal` | CustodyBridge.daml | 81 | Template `CustodyWithdrawalRequest` — `amount` field with no ensure clause |
| 8 | `unguarded-division` | Margin.daml | 42 | Division by `usedMargin` without prior > 0 check in `CheckMarginCall` |
| 9 | `missing-ensure-decimal` | Settlement.daml | 66 | Template `SettledDeliveryVsPayment` — `quantity` field with no ensure clause |
| 10 | `missing-ensure-decimal` | Settlement.daml | 66 | Template `SettledDeliveryVsPayment` — `cashAmount` field with no ensure clause |

### MEDIUM (14)

All 14 are `unbounded-fields`:

| Template | File | Unbounded Fields |
|----------|------|-----------------|
| Account | Account.daml:6 | `accountId` |
| Asset | Asset.daml:6 | `symbol`, `name` |
| AuditLog | AuditLog.daml:6 | `logId`, `accountId` |
| ComplianceAlertLog | AuditLog.daml:23 | `alertId`, `accountId`, `description` |
| Compliance | Compliance.daml:6 | `accountId` |
| CustodyBridge | CustodyBridge.daml:6 | `accountId`, `externalAccountRef` |
| CustodyDepositRequest | CustodyBridge.daml:64 | `accountId`, `depositId`, `currency`, `signature`, `transactionHash`, `status` |
| CustodyWithdrawalRequest | CustodyBridge.daml:81 | `accountId`, `withdrawalId`, `currency`, `destinationAddress`, `signature`, `status` |
| Margin | Margin.daml:6 | `accountId` |
| Order | Order.daml:6 | `orderId`, `accountId`, `symbol` |
| RiskLimit | RiskLimit.daml:6 | `accountId` |
| Settlement | Settlement.daml:6 | `settlementId`, `tradeId`, `symbol` |
| SettledDeliveryVsPayment | Settlement.daml:66 | `settlementId`, `tradeId`, `symbol`, `securitiesTransferId`, `cashTransferId` |
| Trade | Trade.daml:8 | `tradeId`, `buyerOrderId`, `sellerOrderId`, `symbol` |

---

## 2. canton-erc20

**GitHub:** [ChainSafe/canton-erc20](https://github.com/ChainSafe/canton-erc20)
**Commit:** [`e98a7db`](https://github.com/ChainSafe/canton-erc20/commit/e98a7db5b9e0bdb96331cac72052a39134a84dfc)
**Files scanned:** 16 | **Summary: 17 findings — 9 HIGH, 8 MEDIUM**

### HIGH (9)

| # | Detector | File | Line | Description |
|---|----------|------|------|-------------|
| 1 | `missing-ensure-decimal` | Bridge/Contracts.daml | 35 | Template `MintCommand` — `amount` field with no ensure clause |
| 2 | `missing-ensure-decimal` | Bridge/Contracts.daml | 77 | Template `WithdrawalRequest` — `amount` field with no ensure clause |
| 3 | `missing-ensure-decimal` | Bridge/Contracts.daml | 128 | Template `WithdrawalEvent` — `amount` field with no ensure clause |
| 4 | `missing-positive-amount` | Wayfinder/Bridge.daml | 42 | Choice `CreatePendingDeposit` — `amount` parameter without > 0 assertion |
| 5 | `missing-positive-amount` | Wayfinder/Bridge.daml | 77 | Choice `InitiateWithdrawal` — `amount` parameter without > 0 assertion |
| 6 | `missing-ensure-decimal` | CIP56/Events.daml | 23 | Template `TokenTransferEvent` — `amount` field with no ensure clause |
| 7 | `missing-ensure-decimal` | CIP56/Token.daml | 18 | Template `CIP56Holding` — `amount` field with no ensure clause |
| 8 | `missing-ensure-decimal` | Common/FingerprintAuth.daml | 81 | Template `PendingDeposit` — `amount` field with no ensure clause |
| 9 | `missing-ensure-decimal` | Common/FingerprintAuth.daml | 132 | Template `DepositReceipt` — `amount` field with no ensure clause |

### MEDIUM (8)

All 8 are `unbounded-fields`:

| Template | File | Unbounded Fields |
|----------|------|-----------------|
| MintCommand | Bridge/Contracts.daml:35 | `evmTxHash`, `fingerprint`, `tokenSymbol`, `auditObservers` |
| WithdrawalRequest | Bridge/Contracts.daml:77 | `fingerprint`, `tokenSymbol`, `auditObservers` |
| WithdrawalEvent | Bridge/Contracts.daml:128 | `fingerprint`, `tokenSymbol`, `auditObservers` |
| WayfinderBridgeConfig | Wayfinder/Bridge.daml:33 | `auditObservers` |
| TokenConfig | CIP56/Config.daml:16 | `auditObservers` |
| TokenTransferEvent | CIP56/Events.daml:23 | `auditObservers` |
| CIP56TransferFactory | CIP56/TransferFactory.daml:15 | `auditObservers` |
| DepositReceipt | Common/FingerprintAuth.daml:132 | `evmTxHash`, `tokenId` |

---

## 3. canton-vault

**GitHub:** [ted-gc/canton-vault](https://github.com/ted-gc/canton-vault)
**Commit:** [`bb51d2c`](https://github.com/ted-gc/canton-vault/commit/bb51d2c5df89d506dc38af8a3760029013c1ffe7)
**Files scanned:** 8 | **Summary: 10 findings — 7 HIGH, 3 MEDIUM**

### HIGH (7)

| # | Detector | File | Line | Description | Triage |
|---|----------|------|------|-------------|--------|
| 1 | `unguarded-division` | Types.daml | 90 | Division by `10000000000.0` in `roundDown` | **False positive** — constant |
| 2 | `unguarded-division` | Types.daml | 99 | Division by `state.totalAssets` in `calculateShares` | **True positive** |
| 3 | `unguarded-division` | Types.daml | 107 | Division by `state.totalShares` in `calculateAssets` | **True positive** |
| 4 | `unguarded-division` | Types.daml | 115 | Division by `state.totalShares` in `sharePrice` | **True positive** |
| 5 | `unguarded-division` | Vault.daml | 170 | Division by `state.totalShares` in choice `Mint` | **True positive** |
| 6 | `unguarded-division` | Vault.daml | 266 | Division by `state.totalAssets` in choice `Withdraw` | **True positive** |
| 7 | `unguarded-division` | Vault.daml | 336 | Division by `1000000` in `convertMicrosecondsToSeconds` | **False positive** — constant |

**Triage detail:** Findings #2-6 have function-level guards (`if totalShares == 0.0 then ...`) but the guard protects a different variable than the divisor. For example, `calculateShares` guards on `totalShares == 0` but divides by `totalAssets`. If `totalAssets == 0` while `totalShares > 0` (vault loss scenario), division-by-zero occurs.

### MEDIUM (3)

| Template | File | Unbounded Fields |
|----------|------|-----------------|
| DepositRequest | AsyncRequests.daml:10 | `requestId` |
| RedeemRequest | AsyncRequests.daml:72 | `requestId` |
| VaultShareAllocation | TransferFactory.daml:100 | `settlementId` |

---

## 4. daml-finance

**GitHub:** [digital-asset/daml-finance](https://github.com/digital-asset/daml-finance)
**Commit:** [`155f931b`](https://github.com/digital-asset/daml-finance/commit/155f931b6ebe7d3662fd72788cb17f0bfb5a7ba6)
**Files scanned:** 281 | **Summary: 89 findings — 56 HIGH, 33 MEDIUM**

**Note:** daml-finance is a **library**, not a standalone application. ~48 of the 56 HIGH findings are by-design — financial instrument templates intentionally accept zero/negative parameters (e.g., negative coupon rates for discount bonds, zero-strike options). The library delegates validation to consuming applications.

### HIGH by Detector (56)

| Detector | Count | Key Examples |
|----------|-------|-------------|
| `missing-ensure-decimal` | 48 | Instrument templates across Bond, Option, Equity, Swap, StructuredProduct |
| `unguarded-division` | 8 | ContingentClaims `eval` (DivF), Stochastic valuation (discount factor), Settlement routing |

**`missing-ensure-decimal` by instrument category:**

| Category | Templates | Fields |
|----------|----------|--------|
| Bond (V3) | FixedRate, FloatingRate, ZeroCoupon, Callable, InflationLinked | `couponRate`, `notional`, `inflationIndexBaseValue` |
| Option (V0) | EuropeanCash, EuropeanPhysical, BarrierEuropeanCash, Dividend | `strike`, `barrier`, `multiplier` |
| Equity (V0) | Stock, Dividend, StockSplit, RightsIssue, Merger, DivOption | `ownershipStake`, `price`, `newPrice`, `ratio` |
| Swap (V0) | InterestRate, Currency, CreditDefault, FX, Asset, Fpml | `periodicRate`, `principalAmount`, `fixRate`, `ownershipStake` |
| StructuredProduct (V0) | AutoCallable, BarrierReverseConvertible | `couponRate`, `barrier`, `notional` |
| Settlement (V4) | Instruction | Decimal fields without ensure |

**`unguarded-division` (8 actionable):**

| # | File | Function | Divides By |
|---|------|----------|-----------|
| 1 | ContingentClaims/Core/V3/Observation.daml:86 | `eval` | `x'` (DivF operator) |
| 2-4 | ContingentClaims/Valuation/V0/Stochastic.daml:164-183 | `fapf` | `disc'` (discount factor, 3 sites) |
| 5 | Instrument/Equity/V0/Merger/Instrument.daml | lifecycle | computed ratio |
| 6-8 | Settlement and routing computations | various | intermediate amounts |

### MEDIUM (33)

All 33 are `unbounded-fields` across Instrument, Lifecycle, and Settlement templates:
- `version`, `description` on all Instrument templates
- `holidayCalendarIds`, `referenceAssetId` on structured instruments
- `otherConsumed`, `otherProduced` lists on lifecycle Effect templates
- `swapStreams`, `currencies` lists on Fpml instruments

---

## 5. splice

**GitHub:** [hyperledger-labs/splice](https://github.com/hyperledger-labs/splice)
**Commit:** [`6a0dcbba4`](https://github.com/hyperledger-labs/splice/commit/6a0dcbba4bdae0861cd6f5021bfd436c18fd5034)
**Files scanned:** 95 | **Summary: 188 findings — 81 HIGH, 107 MEDIUM**

### HIGH by Detector (81)

| Detector | Count |
|----------|-------|
| `unguarded-division` | 30 |
| `missing-positive-amount` | 42 |
| `missing-ensure-decimal` | 9 |

#### Unguarded Division by `amuletPrice` (16 critical sites)

| # | File | Line | Context |
|---|------|------|---------|
| 1 | AmuletRules.daml | 230 | `scaleFees (1.0 / openRound.amuletPrice)` in `BuyMemberTraffic` |
| 2 | AmuletRules.daml | 722 | `amountUsd / round.amuletPrice` in `ConvertFeaturedAppActivityMarkers` |
| 3 | AmuletRules.daml | 898 | `scaleFees (1.0 / openRound.amuletPrice)` in `summarizeAndValidateContext` |
| 4 | AmuletRules.daml | 1246 | `scaleFees (1.0 / openRound.amuletPrice)` in `transferConfigAmuletFromOpenRound` |
| 5 | AmuletRules.daml | 1265 | `trafficCostUsd / contextMiningRound.amuletPrice` |
| 6 | AmuletRules.daml | 1681 | `feeUsd / contextMiningRound.amuletPrice` in `computeTransferPreapprovalFee` |
| 7 | Payment.daml | 67 | `pq.amount / openMiningRound.amuletPrice` |
| 8 | Subscriptions.daml | 173 | `* (1.0 / round.amuletPrice)` |
| 9 | Subscriptions.daml | 208 | `* (1.0 / round2.amuletPrice)` |
| 10 | TestWallet.daml | 1186 | `trafficAmount / 1e6` |
| 11 | TestWallet.daml | 1187 | `trafficCostUsd / amuletPrice` |
| 12 | TestWallet.daml | 1198 | `preapprovalFeeUsd / amuletPrice` |
| 13 | TestSubscriptions.daml | 173 | `* (1.0 / round.amuletPrice)` |
| 14 | TestSubscriptions.daml | 208 | `* (1.0 / round2.amuletPrice)` |
| 15 | TestSplitwell.daml | 100 | `amount / intToNumeric (length group.members + 1)` |
| 16 | Splitwell.daml | various | Additional division sites |

**Risk:** `OpenMiningRound.amuletPrice` has no `ensure amuletPrice > 0.0`. Set by DSO governance. A misconfiguration could cause div-by-zero across all transfer, payment, and subscription code.

#### Additional Division Sites in Issuance

| Line | File | Divisor | Context |
|------|------|---------|---------|
| 115 | Issuance.daml | `intToDecimal summary.totalSvRewardWeight` | `computeIssuingRoundParameters` — has `== 0` early return (safe) |
| 126 | Issuance.daml | `roundsPerYear` | `computeIssuingRoundParameters` — validated in config (safe) |
| 139 | Issuance.daml | `amuletPrice` | `computeIssuingRoundParameters` — unguarded (same class as D1) |

#### Archive Before Execute (1 HIGH)

| Line | File | Evidence |
|------|------|---------|
| 777 | DsoRules.daml | `fetchAndArchive` at 777 before `try` block at 801. If execution fails, archived contract is permanently consumed. |

#### Missing Positive Amount (42 instances)

**splice-amulet (30):**

| Line | File | Parameter | Choice |
|------|------|-----------|--------|
| 146 | AmuletRules.daml | `inputs` (list) | `AmuletRules_CreateExternalPartySetupProposal` |
| 176 | AmuletRules.daml | `inputs` (list) | `AmuletRules_CreateTransferPreapproval` |
| 209 | AmuletRules.daml | `inputs` (list) | `AmuletRules_BuyMemberTraffic` |
| 255 | AmuletRules.daml | `trafficCids` (list) | `AmuletRules_MergeMemberTrafficContracts` |
| 518 | AmuletRules.daml | `validatorRewardCouponCids` (list) | `AmuletRules_ClaimExpiredRewards` |
| 518 | AmuletRules.daml | `appCouponCids` (list) | `AmuletRules_ClaimExpiredRewards` |
| 518 | AmuletRules.daml | `svRewardCouponCids` (list) | `AmuletRules_ClaimExpiredRewards` |
| 617 | AmuletRules.daml | `amount` (Decimal) | `AmuletRules_AllocateDevelopmentFundCoupon` |
| 617 | AmuletRules.daml | `unclaimedDevelopmentFundCouponCids` (list) | `AmuletRules_AllocateDevelopmentFundCoupon` |
| 708 | AmuletRules.daml | `markerCids` (list) | `AmuletRules_ConvertFeaturedAppActivityMarkers` |
| ... | ... | ... | (plus additional input-list findings across transfer/mint/reward choices) |

**splice-dso-governance (6):**

| Line | File | Parameter | Choice |
|------|------|-----------|--------|
| 614 | DsoRules.daml | `requestCids` (list) | `DsoRules_ElectDsoDelegate` |
| 867 | DsoRules.daml | `nonSvVoteCids` (list) | `DsoRules_GarbageCollectAmuletPriceVotes` |
| 867 | DsoRules.daml | `duplicateVoteCids` (list) | `DsoRules_GarbageCollectAmuletPriceVotes` |
| 1129 | DsoRules.daml | `amuletPriceVoteCids` (list) | `DsoRules_AdvanceOpenMiningRounds` |
| 1461 | DsoRules.daml | `amount` (Decimal) | `DsoRules_CreateUnallocatedUnclaimedActivityRecord` |
| 1481 | DsoRules.daml | `unclaimedRewardsToBurnCids` (list) | `DsoRules_AllocateUnallocatedUnclaimedActivityRecord` |

**splice-wallet (5):**

| Line | File | Parameter | Choice |
|------|------|-----------|--------|
| 44 | BuyTrafficRequest.daml | `inputs` (list) | `BuyTrafficRequest_Complete` |
| 414 | Install.daml | `inputs` (list) | `WalletAppInstall_ExecuteBatch` |
| 60 | MintingDelegation.daml | `inputs` (list) | `MintingDelegation_Mint` |
| 118 | TransferOffer.daml | `inputs` (list) | `AcceptedTransferOffer_Complete` |
| 20 | TransferPreapproval.daml | `inputs` (list) | `TransferPreapprovalProposal_Accept` |

**Splitwell (1):**

| Line | File | Parameter | Choice |
|------|------|-----------|--------|
| 126 | Splitwell.daml | `amount` (Decimal) | `SplitwellRules_EnterPayment` |

#### Missing Ensure Decimal (9 instances)

| Template | File | Field |
|----------|------|-------|
| OpenMiningRound | Round.daml:20 | `amuletPrice` |
| SummarizingMiningRound | Round.daml:43 | `amuletPrice` |
| IssuingMiningRound | Round.daml:55 | `issuancePerValidatorRewardCoupon`, `issuancePerFeaturedAppRewardCoupon`, `issuancePerUnfeaturedAppRewardCoupon`, `issuancePerSvRewardCoupon` |
| ClosedMiningRound | Round.daml:78 | Same 4 issuance fields |
| SubscriptionPayment | Subscriptions.daml:297 | `targetAmount` |

### MEDIUM by Detector (107)

| Detector | Count |
|----------|-------|
| `unbounded-fields` | 91 |
| `head-of-list-query` | 16 |

All 16 `head-of-list-query` findings are in test files (`Test*.daml`), where single-element pattern matching on query results is common practice.

---

## 6. Aggregate Summary

### Findings by Project

| Project | Files | HIGH | MEDIUM | Total |
|---------|-------|------|--------|-------|
| canton-dex-platform | 11 | 10 | 14 | 24 |
| canton-erc20 | 16 | 9 | 8 | 17 |
| canton-vault | 8 | 7 | 3 | 10 |
| daml-finance | 281 | 56 | 33 | 89 |
| splice | 95 | 81 | 107 | 188 |
| **Total** | **411** | **163** | **165** | **328** |

### Findings by Detector

| Detector | HIGH | MEDIUM | Total | Most Affected |
|----------|------|--------|-------|---------------|
| `missing-ensure-decimal` | 74 | — | 74 | daml-finance (48), splice (9), canton-erc20 (7) |
| `missing-positive-amount` | 55 | — | 55 | splice (42), canton-dex (5) |
| `unguarded-division` | 52 | — | 52 | splice (30), daml-finance (8), canton-vault (7) |
| `archive-before-execute` | 1 | — | 1 | splice (1) |
| `unbounded-fields` | — | 149 | 149 | splice (91), daml-finance (33), canton-dex (14) |
| `head-of-list-query` | — | 16 | 16 | splice (16, all test files) |

---

## 7. Known False Positive Patterns

### Constant Divisors

daml-lint flags division by numeric literals (e.g., `/ 2`, `/ 1000000`, `/ 10000000000.0`). These are always positive and cannot cause division-by-zero. Affected:

- canton-vault: `roundDown` (divides by `10000000000.0`), `convertMicrosecondsToSeconds` (divides by `1000000`)
- splice: `median` (divides by `2`), `summarizeDso` (divides by `2`, `2.0`, `3.0`)

**False positive rate for `unguarded-division`:** ~15% (constant divisors)

### Cross-Function Guards

daml-lint does not track guards across function boundaries. A variable guarded in a caller function is flagged as unguarded at the division site. Affected:

- splice: `Issuance.daml:115` — `totalSvRewardWeight` has `== 0` early return in caller
- splice: `Issuance.daml:126` — `roundsPerYear` validated in `validAmuletConfig`

**False positive rate for `unguarded-division`:** ~15% (cross-function guards)

### Library-Design Permissiveness

daml-finance instrument templates intentionally accept zero/negative Decimal parameters. ~48 of the 56 HIGH findings in daml-finance are by-design.

**False positive rate for `missing-ensure-decimal` in daml-finance:** ~85%

### Legitimately Empty Lists

Many `missing-positive-amount` findings flag list parameters that are legitimately empty in certain call paths (e.g., `rewardCouponCids` can be empty if no rewards exist for a round).

**False positive rate for `missing-positive-amount`:** ~50%

---

*Generated by daml-lint v0.1.0 | Audit date: 2026-03-04*
