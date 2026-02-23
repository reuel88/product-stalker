# 0002. Currency Normalization — Dual Price Strategy

**Date:** 2026-02-20
**Status:** Accepted

## Context

Users track products across retailers that price in different currencies. Two problems arise:

1. **Cross-currency comparison** — "Which retailer has the lowest price?" requires a common currency.
2. **Daily change accuracy** — A "today vs yesterday" percentage must reflect real price changes, not exchange rate movement between the two check times.

## Decision

### 1. Store both original and normalized prices

Each availability check persists two price representations:

| Column | Purpose |
|--------|---------|
| `price_minor_units` / `price_currency` | Exact retailer price (source of truth) |
| `normalized_price_minor_units` / `normalized_currency` | Converted to user's preferred currency at check time |

Normalization uses `ExchangeRateService` (Frankfurter API, cached in `exchange_rates` table, refreshed every 24 hours).

### 2. Re-normalize daily comparisons with today's rate

`DailyPriceComparison` does **not** compare stored normalized values directly. Instead, `renormalize_averages()` converts both today's and yesterday's per-retailer averages using today's exchange rate. This isolates real price changes from rate movement.

### 3. Per-retailer averages use original prices

Each retailer prices in a single currency, so per-retailer daily averages are computed from `price_minor_units`. This avoids injecting exchange rate noise into retailer-level comparisons.

## Consequences

### What becomes easier

- **Accurate cross-currency lowest-price** — The frontend's `getEffectivePrice()` prefers normalized price, enabling apples-to-apples comparison across currencies.
- **Stable daily percentage** — Re-normalization means the daily indicator only moves when a retailer actually changes its price.
- **Auditable history** — Original prices are always preserved; normalized values can be recomputed.

### What becomes harder

- **Storage cost** — Every availability row carries two price columns.
- **Historical normalized values drift** — The stored `normalized_price_minor_units` reflects the rate at check time, not the current rate. This is acceptable because cross-currency comparisons on historical data are not a current requirement.

### Key files

- `crates/core/src/services/exchange_rate_service.rs` — rate fetching and caching
- `crates/domain/src/services/availability/checker.rs` — `normalize_price()` during checks
- `crates/domain/src/services/availability/comparison.rs` — `renormalize_averages()` for daily comparison
- `apps/desktop/src/modules/products/price-utils.ts` — `getEffectivePrice()` on frontend
- `crates/domain/src/services/currency.rs` — currency exponent lookup
