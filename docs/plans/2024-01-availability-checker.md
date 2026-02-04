# Product Availability Checker - Implementation Plan

## Overview

Add web scraping functionality to check product availability from Shopify stores using Schema.org JSON-LD structured data.

**Target**: https://www.sotsu.com/products/flipaction-elite-16?variant=46518950953186

**Detection Method**: Parse `<script type="application/ld+json">` for `"availability"` field:
- `http://schema.org/InStock` → In Stock
- `http://schema.org/OutOfStock` → Out of Stock
- `http://schema.org/BackOrder` → Back Order

---

## Design Decisions

1. **Separate `availability_checks` table** - Allows historical tracking, separation of concerns
2. **Schema.org parser** - Start simple, Shopify stores consistently use this format
3. **Store errors in DB** - Track failed checks vs never checked
4. **Manual checking only (Phase 1)** - Background tasks deferred to Phase 2

---

## Implementation

### Phase 1: Backend (Rust) ✓ COMPLETE

#### 1.1 Add Dependencies
**File**: `apps/desktop/src-tauri/Cargo.toml`
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
scraper = "0.23"
```

#### 1.2 Database Migration
**New File**: `src-tauri/src/migrations/m20240103_000001_create_availability_checks_table.rs`

Schema:
- `id` (UUID, PK)
- `product_id` (UUID, FK → products.id, CASCADE DELETE)
- `status` (TEXT: in_stock, out_of_stock, back_order, unknown)
- `raw_availability` (TEXT, nullable - original schema.org value)
- `error_message` (TEXT, nullable - for failed checks)
- `checked_at` (TEXT, timestamp)

Indexes on `product_id` and `checked_at`.

**Update**: `migrations/mod.rs`, `migrations/migrator.rs`

#### 1.3 Entity
**New File**: `src-tauri/src/entities/availability_check.rs`
- `AvailabilityStatus` enum with `from_schema_org()` parser
- SeaORM entity with relation to Product

**Update**: `entities/mod.rs`, `entities/prelude.rs`

#### 1.4 Error Handling
**Update**: `src-tauri/src/error.rs`
- Add `Http(reqwest::Error)` variant
- Add `Scraping(String)` variant

#### 1.5 Repository
**New File**: `src-tauri/src/repositories/availability_check_repository.rs`
- `create()` - Store check result
- `find_latest_for_product()` - Get most recent check
- `find_all_for_product()` - Get history with optional limit
- `delete_for_product()` - Cleanup (cascade handled by FK)

**Update**: `repositories/mod.rs`

#### 1.6 Scraper Service
**New File**: `src-tauri/src/services/scraper_service.rs`
- `check_availability(url)` - Fetch page and parse
- `parse_schema_org(html)` - Extract availability from JSON-LD
- User-Agent header, 30s timeout
- Unit tests for HTML parsing

#### 1.7 Availability Service
**New File**: `src-tauri/src/services/availability_service.rs`
- `check_product(conn, product_id)` - Orchestrate check and store result
- `get_latest(conn, product_id)` - Get latest check
- `get_history(conn, product_id, limit)` - Get check history

**Update**: `services/mod.rs`

#### 1.8 Commands
**New File**: `src-tauri/src/commands/availability.rs`
- `AvailabilityCheckResponse` DTO
- `check_availability(product_id)` - Manual trigger
- `get_latest_availability(product_id)` - Get latest
- `get_availability_history(product_id, limit)` - Get history

**Update**: `commands/mod.rs`, `lib.rs` (register in invoke_handler)

---

### Phase 2: Frontend (React/TypeScript) ✓ COMPLETE

#### 2.1 Types
**Update**: `src/modules/products/types.ts`
```typescript
export type AvailabilityStatus = "in_stock" | "out_of_stock" | "back_order" | "unknown";

export interface AvailabilityCheckResponse {
  id: string;
  product_id: string;
  status: AvailabilityStatus;
  raw_availability: string | null;
  error_message: string | null;
  checked_at: string;
}
```

#### 2.2 Constants
**Update**: `src/constants/api.ts` - Add command names
**Update**: `src/constants/queryKeys.ts` - Add query key factories
**Update**: `src/constants/messages.ts` - Add availability messages

#### 2.3 Hook
**New File**: `src/modules/products/hooks/useAvailability.ts`
- `useAvailability(productId)` - Get latest + check mutation
- `useAvailabilityHistory(productId, limit)` - Get history

#### 2.4 UI Component
**New File**: `src/modules/products/ui/components/availability-badge.tsx`
- Color-coded badge (green/red/yellow/gray)
- Check button with spinner
- Last checked timestamp

#### 2.5 Products Table
**Update**: `src/modules/products/ui/components/products-table.tsx`
- Add "Availability" column with badge and check button

---

## File Summary

### New Files (8)
| File | Purpose |
|------|---------|
| `src-tauri/src/migrations/m20240103_000001_create_availability_checks_table.rs` | Migration |
| `src-tauri/src/entities/availability_check.rs` | Entity |
| `src-tauri/src/repositories/availability_check_repository.rs` | Repository |
| `src-tauri/src/services/scraper_service.rs` | HTML scraping |
| `src-tauri/src/services/availability_service.rs` | Business logic |
| `src-tauri/src/commands/availability.rs` | Tauri commands |
| `src/modules/products/hooks/useAvailability.ts` | React hooks |
| `src/modules/products/ui/components/availability-badge.tsx` | UI component |

### Modified Files (16)

| File | Changes |
| --- | --- |
| `Cargo.toml` | Add reqwest, scraper |
| `src-tauri/src/error.rs` | Add Http, Scraping errors |
| `src-tauri/src/entities/mod.rs` | Export entity |
| `src-tauri/src/entities/prelude.rs` | Export types |
| `src-tauri/src/entities/product.rs` | Add relation to availability checks |
| `src-tauri/src/repositories/mod.rs` | Export repository |
| `src-tauri/src/services/mod.rs` | Export services |
| `src-tauri/src/commands/mod.rs` | Export commands |
| `src-tauri/src/lib.rs` | Register commands |
| `src-tauri/src/migrations/mod.rs` | Add migration |
| `src-tauri/src/migrations/migrator.rs` | Register migration |
| `src/constants/api.ts` | Add commands |
| `src/constants/queryKeys.ts` | Add query keys |
| `src/constants/messages.ts` | Add availability messages |
| `src/modules/products/types.ts` | Add types |
| `src/modules/products/ui/components/products-table.tsx` | Add availability column |

---

## Verification

1. **Build**: `pnpm build` in apps/desktop
2. **Run migrations**: Automatic on app start
3. **Test scraping**: Add the Sotsu product, click "Check" button
4. **Verify status**: Should show availability based on Schema.org data (varies by variant - Space Black may be "In Stock", Silver may be "Back Order")
5. **Test error handling**: Add product with invalid URL, verify "Unknown" status with error message

---

## Future Enhancements

- ~~Background periodic checking with tokio tasks~~ ✓ COMPLETE
- ~~Desktop notifications when products become available~~ ✓ COMPLETE
- ~~Bulk "Check All" operation~~ ✓ COMPLETE
- Support for non-Shopify sites (different parsing strategies) - *Partially complete: headless browser now handles bot-protected sites*
- ~~Price tracking from Schema.org data~~ ✓ COMPLETE

---

## Phase 4: Enhanced Automation ✓ COMPLETE

Implemented bulk checking, desktop notifications for back-in-stock products, and background periodic checking.

### 4.1 Bulk "Check All" Operation

**Backend:**
- `AvailabilityService::check_all_products()` - Checks all products with 500ms rate limiting
- `BulkCheckResult` and `BulkCheckSummary` types for response
- `is_back_in_stock()` helper for status transition detection
- `check_all_availability` Tauri command

**Frontend:**
- `useCheckAllAvailability` hook in `useAvailability.ts`
- "Check All" button in products-view.tsx with loading spinner
- Toast messages for bulk operation results

### 4.2 Desktop Notifications (Back in Stock)

**Backend:**
- Status transition detection in both single and bulk check operations
- Notifications sent via `tauri_plugin_notification` when:
  - Single product check: product transitions to `in_stock` from any other status
  - Bulk check: any products are back in stock (aggregated notification)
- Respects `enable_notifications` setting

### 4.3 Background Periodic Checking

**Database Migration:** `m20240104_000001_add_background_check_settings.rs`
- `background_check_enabled` (boolean, default: false)
- `background_check_interval_minutes` (integer, default: 60)

**Backend:**
- `background/mod.rs` and `background/availability_checker.rs`
- Spawned on app startup in `lib.rs`
- Checks settings every 60 seconds when disabled
- When enabled, performs bulk check at configured interval
- Sends desktop notifications for back-in-stock products

**Frontend:**
- Updated `Settings` interface with new fields
- "Background Checking" card in settings-view.tsx
- Toggle switch for enable/disable
- Interval selector: 15 min, 30 min, 1 hour, 4 hours, daily

### Files Modified/Created

**New Files:**

| File | Purpose |
|------|---------|
| `src-tauri/src/background/mod.rs` | Background module |
| `src-tauri/src/background/availability_checker.rs` | Periodic checker task |
| `src-tauri/src/migrations/m20240104_000001_add_background_check_settings.rs` | Settings migration |

**Modified Backend Files:**

| File | Changes |
|------|---------|
| `src-tauri/src/lib.rs` | Added background module, spawn checker on startup |
| `src-tauri/src/services/availability_service.rs` | Added bulk check, back-in-stock detection |
| `src-tauri/src/services/mod.rs` | Export new types |
| `src-tauri/src/commands/availability.rs` | Added check_all_availability with notifications |
| `src-tauri/src/entities/setting.rs` | Added background check settings |
| `src-tauri/src/repositories/setting_repository.rs` | Updated for new settings |
| `src-tauri/src/services/setting_service.rs` | Updated for new settings |
| `src-tauri/src/commands/settings.rs` | Updated DTOs |
| `src-tauri/src/migrations/mod.rs` | Added new migration |
| `src-tauri/src/migrations/migrator.rs` | Registered new migration |

**Modified Frontend Files:**

| File | Changes |
|------|---------|
| `src/constants/api.ts` | Added CHECK_ALL_AVAILABILITY |
| `src/constants/messages.ts` | Added bulk check messages |
| `src/modules/products/types.ts` | Added BulkCheckResult, BulkCheckSummary |
| `src/modules/products/hooks/useAvailability.ts` | Added useCheckAllAvailability hook |
| `src/modules/products/ui/views/products-view.tsx` | Added Check All button |
| `src/modules/settings/hooks/useSettings.ts` | Added background check settings |
| `src/modules/settings/ui/views/settings-view.tsx` | Added Background Checking card |

### Verification

1. **Bulk Check**: Click "Check All" button, verify all products update with toast message
2. **Notifications**: Set a product to out_of_stock, check when it's in_stock, verify desktop notification
3. **Background Checking**: Enable in settings, set interval to 15 minutes for testing, verify products auto-update

---

## Phase 3: Headless Browser Support (for Cloudflare-protected sites) ✓ COMPLETE

### Problem

Some sites use JavaScript-based bot protection (Cloudflare, PerimeterX, etc.) that cannot be bypassed with HTTP headers alone. These protections:
- Return a 403 "Just a moment..." challenge page
- Require JavaScript execution to pass verification
- Example: `templeandwebster.com.au`

### Solution Options

#### Option A: Rust Headless Browser (Recommended)

Use `headless_chrome` or `chromiumoxide` crate to run a real browser:

**Pros:**
- Full JavaScript execution
- Passes most bot detection
- Can reuse browser instance for multiple checks

**Cons:**
- Requires Chrome/Chromium installed on user's system
- Larger binary size
- More memory usage

**Dependencies:**
```toml
headless_chrome = "1.0.20"
# or
chromiumoxide = "0.8.0"
```

**Implementation:**
1. Add `ScraperStrategy` enum: `Http` | `Headless`
2. Detect Cloudflare challenge in response (check for "Just a moment..." or 403 with challenge)
3. Retry with headless browser if challenge detected
4. Cache browser instance for reuse

#### Option B: External Browser via WebDriver

Use Selenium/WebDriver protocol with user's existing browser.

**Pros:**
- No bundled browser needed
- User controls which browser

**Cons:**
- Requires user to have ChromeDriver/GeckoDriver installed
- More complex setup

#### Option C: Document as Limitation

Mark sites with heavy bot protection as unsupported.

**Pros:**
- No additional complexity
- Smaller binary

**Cons:**
- Limited site support

### Recommended Approach

1. **Detect Cloudflare challenges** - Check for 403 + "Just a moment..." in response
2. **Show clear error message** - "This site has bot protection. Headless browser support coming soon."
3. **Implement Option A** - Use `headless_chrome` crate with lazy browser initialization
4. **Add user setting** - Toggle to enable/disable headless browser (for users without Chrome)

### Implementation Plan

#### 3.1 Update Error Handling
**File**: `src-tauri/src/error.rs`
- Add `BotProtection(String)` variant for detected challenges

#### 3.2 Challenge Detection
**File**: `src-tauri/src/services/scraper_service.rs`
```rust
fn is_cloudflare_challenge(status: u16, body: &str) -> bool {
    status == 403 && (
        body.contains("Just a moment...") ||
        body.contains("cf-browser-verification") ||
        body.contains("_cf_chl_opt")
    )
}
```

#### 3.3 Headless Browser Service
**New File**: `src-tauri/src/services/headless_service.rs`
- `HeadlessService` with lazy Chrome initialization
- `fetch_page(url)` - Navigate and wait for content
- Timeout handling (longer than HTTP - 60s)
- Browser cleanup on app exit

##### 3.3.1 Design Decisions

Design decisions for `HeadlessService` and `fetch_page` behavior. Each decision below includes expected configuration knobs and failure modes for implementers.

**1. Platform-specific browser binary detection**

- **Decision**: How `HeadlessService` locates Chrome/Chromium on Windows, macOS, and Linux. Check `CHROME_PATH` (or equivalent) env first; then common install paths (e.g. Windows: Program Files, Registry; macOS: `/Applications/Google Chrome.app`; Linux: `google-chrome`, `chromium`, `chromium-browser` via `which` or standard paths).
- **Configuration knobs**: `chrome_path` (optional override), `prefer_chromium` (bool, try Chromium before Chrome on Linux).
- **Failure modes**: Binary not found → fail initialization with clear error; wrong architecture → same; insufficient permissions → same. No silent fallback to a wrong binary.

**2. Browser download/installation fallback policy**

- **Decision**: When no suitable binary is found, choose one of: **fail** (return error, clear message + install link; recommended for Phase 3), **prompt** (show dialog with install link), or **auto-download** (download Chromium to app data; higher complexity). Document trade-offs: fail is simplest; auto-download improves UX but adds size/network/versioning concerns.
- **Configuration knobs**: `browser_fallback_policy`: `fail` | `prompt` | `auto_download` (if supported later).
- **Failure modes**: User dismisses prompt → treat as fail for that session; auto-download: disk space/network failure → fail with message; version mismatch after download → fail or retry download.

**3. Concurrency model for `fetch_page`**

- **Decision**: Use a **single browser process with multiple tabs** (recommended for Phase 3): one Chrome instance, one tab per in-flight `fetch_page`. Schedule requests via a semaphore or bounded queue so that at most N tabs are open. Alternative: multiple browser instances only if single-process limits are hit (document as future option).
- **Configuration knobs**: `max_concurrent_tabs` (max tabs open at once), `browser_instance_mode`: `single` | `multi` (if both supported later).
- **Failure modes**: Tab limit reached → queue or return "busy" / capacity error; browser crash → fail in-flight requests and recreate browser on next use; starvation → document queue policy (FIFO).

**4. Resource limits and throttling**

- **Decision**: Enforce max concurrent headless sessions/tabs (e.g. `max_tabs`). When at capacity, apply **backpressure**: either queue with a max depth and reject new requests with "capacity exceeded", or wait. Optionally **evict** idle tabs (e.g. LRU or oldest-first) when approaching limit to avoid OOM.
- **Configuration knobs**: `max_tabs`, `max_queue_depth` (if queuing), `tab_idle_ttl_sec` (close idle tabs after N seconds), `eviction_policy`: `lru` | `oldest` | `none`.
- **Failure modes**: OOM → reduce `max_tabs` or enable eviction; slow responses under load → backpressure + clear errors; queue full → return "capacity exceeded" (or similar) to caller.

**5. CAPTCHA and interactive bot challenges**

- **Decision**: When the headless page requires CAPTCHA or interactive verification (e.g. Cloudflare challenge that does not auto-resolve), **surface to callers** via a structured result (e.g. `ChallengeRequired { url, kind }`) so the UI can show "manual check required" or open the URL in the user's browser. Optionally support **fallback to manual review**: open URL in default browser or mark product as "manual check required".
- **Configuration knobs**: `on_challenge`: `return_error` | `open_external_browser` | `manual_review`.
- **Failure modes**: Challenge never resolved → return structured error; external browser not available when `open_external_browser` → fall back to `return_error`; caller ignores challenge result → document that UI must handle `ChallengeRequired`.

#### 3.4 Unified Scraper
**Update**: `src-tauri/src/services/scraper_service.rs`
- Try HTTP first (fast path)
- Detect challenge → retry with headless
- Return `BotProtection` error if headless unavailable/fails

#### 3.5 Frontend
**Update**: Error messages to explain bot protection detection
**Update**: Settings to toggle headless browser support

### Verification

1. Test with `templeandwebster.com.au` product
2. Verify Shopify sites still work with fast HTTP path
3. Test fallback when Chrome not installed
4. Verify browser cleanup on app close

### Implementation Summary

Phase 3 was implemented with the following components:

**Backend (Rust):**

| File | Purpose |
|------|---------|
| `src-tauri/src/services/headless_service.rs` | HeadlessService with lazy Chrome initialization |
| `src-tauri/src/migrations/m20250203_000001_add_headless_browser_setting.rs` | Database migration for headless browser setting |

**Key Features Implemented:**
- **HeadlessService** - Lazy browser initialization, reuses single browser instance with multiple tabs
- **Platform-specific browser detection** - Finds Chrome, Chromium, or Edge on Windows, macOS, and Linux
- **Anti-detection measures** - Hides `navigator.webdriver` property to bypass basic bot detection
- **CAPTCHA detection** - Detects unsolvable CAPTCHAs and returns user-friendly error messages
- **Automatic fallback** - HTTP scraper tries first (fast path), falls back to headless browser on 403/challenge detection
- **Frontend toggle** - Settings page includes "Use Headless Browser" switch for users to enable/disable
- **Graceful degradation** - Clear error messages when Chrome/browser not installed

**Modified Files:**

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Added `headless_chrome` dependency |
| `src-tauri/src/services/mod.rs` | Export HeadlessService |
| `src-tauri/src/services/scraper_service.rs` | Integrated headless fallback, challenge detection |
| `src-tauri/src/entities/setting.rs` | Added `headless_browser_enabled` field |
| `src-tauri/src/repositories/setting_repository.rs` | Updated for new setting |
| `src-tauri/src/services/setting_service.rs` | Updated for new setting |
| `src-tauri/src/commands/settings.rs` | Updated DTOs |
| `src-tauri/src/migrations/mod.rs` | Added new migration |
| `src-tauri/src/migrations/migrator.rs` | Registered new migration |
| `src/modules/settings/ui/views/settings-view.tsx` | Added headless browser toggle |

---

## Phase 5: Price Tracking ✓ COMPLETE

### Overview

Added price tracking to the existing availability checker by extracting `offers.price` and `offers.priceCurrency` from Schema.org JSON-LD data.

### Design Decisions

1. **Storage**: Extended `availability_checks` table with price columns (not a separate table)
   - Price and availability are collected together from the same scrape
   - Simplifies queries and follows existing patterns

2. **Price format**: Store as `INTEGER` cents + `TEXT` currency code
   - Avoids floating-point precision issues
   - Efficient for comparison (price drop detection)
   - Example: $789.00 USD → `price_cents: 78900`, `price_currency: "USD"`

3. **Price drop detection**: Any decrease from previous check triggers notification
   - Matches existing "back in stock" notification pattern

### Implementation

#### 5.1 Database Migration

**File**: `src-tauri/src/migrations/m20250205_000001_add_price_tracking.rs`

Added three columns to `availability_checks`:
- `price_cents` (INTEGER, nullable) - price in smallest currency unit
- `price_currency` (TEXT, nullable) - ISO 4217 code (USD, EUR, AUD)
- `raw_price` (TEXT, nullable) - original Schema.org value for debugging

#### 5.2 Scraper Service Updates

**File**: `src-tauri/src/services/scraper_service.rs`

- Added `PriceInfo` struct with `price_cents`, `price_currency`, `raw_price`
- Updated `ScrapingResult` to include `price: PriceInfo`
- Added `parse_price_to_cents()` helper - handles formats like "789.00", "1,234.56", "$789.00"
- Added `get_price_from_offer()` - extracts price from Schema.org offers
- Updated parsing functions to extract price alongside availability

#### 5.3 Service Layer Updates

**File**: `src-tauri/src/services/availability_service.rs`

- Updated `BulkCheckResult` with `price_cents`, `price_currency`, `previous_price_cents`, `is_price_drop`
- Updated `BulkCheckSummary` with `price_drop_count`
- Added `is_price_drop()` method for price drop detection
- Updated notifications to include price drop alerts

#### 5.4 Repository Updates

**File**: `src-tauri/src/repositories/availability_check_repository.rs`

- Added `CreateCheckParams` struct for cleaner parameter passing
- Updated `create()` to accept price parameters
- Added `find_previous_price()` for price drop detection

#### 5.5 Command Layer Updates

**File**: `src-tauri/src/commands/availability.rs`

- Updated `AvailabilityCheckResponse` DTO with `price_cents`, `price_currency`, `raw_price`

#### 5.6 Frontend Updates

**Types** (`src/modules/products/types.ts`):
- Updated `AvailabilityCheckResponse` with price fields
- Updated `BulkCheckResult` and `BulkCheckSummary` with price tracking fields

**Utilities** (`src/lib/utils.ts`):
- Added `formatPrice()` helper using `Intl.NumberFormat`

**Products Table** (`src/modules/products/ui/components/products-table.tsx`):
- Added `PriceCell` component
- Added "Price" column after "Availability"

**Messages** (`src/constants/messages.ts`):
- Added `PRICE` section with price-related messages

### Files Summary

**New Files (1):**

| File | Purpose |
|------|---------|
| `src-tauri/src/migrations/m20250205_000001_add_price_tracking.rs` | Migration for price columns |

**Modified Backend Files (7):**

| File | Changes |
|------|---------|
| `src-tauri/src/services/scraper_service.rs` | Added PriceInfo, price extraction from Schema.org |
| `src-tauri/src/entities/availability_check.rs` | Added price_cents, price_currency, raw_price fields |
| `src-tauri/src/repositories/availability_check_repository.rs` | Added CreateCheckParams, find_previous_price() |
| `src-tauri/src/services/availability_service.rs` | Added price drop detection, updated bulk results |
| `src-tauri/src/commands/availability.rs` | Updated response DTO with price fields |
| `src-tauri/src/migrations/mod.rs` | Export new migration |
| `src-tauri/src/migrations/migrator.rs` | Register new migration |
| `src-tauri/src/repositories/mod.rs` | Export CreateCheckParams |

**Modified Frontend Files (5):**

| File | Changes |
|------|---------|
| `src/modules/products/types.ts` | Added price fields to types |
| `src/lib/utils.ts` | Added formatPrice() helper |
| `src/modules/products/ui/components/products-table.tsx` | Added PriceCell and Price column |
| `src/constants/messages.ts` | Added PRICE messages |
| `src/__tests__/mocks/data.ts` | Updated mock data with price fields |

### Verification

1. **Backend Tests**: All 349 Rust tests pass
2. **Clippy**: No warnings
3. **TypeScript**: Type check passes
4. **Frontend Tests**: All 194 tests pass
5. **Manual Testing**:
   - Add a Shopify product (e.g., sotsu.com)
   - Click "Check" - verify price extracted and displayed
   - Verify existing products without price show "-" gracefully
   - Test bulk check - verify price drop count in toast
