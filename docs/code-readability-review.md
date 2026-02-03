# Code Readability Review - Product Stalker

**Date:** 2026-02-03
**Readability Score:** 7.5/10

## Summary

This is a well-structured Tauri desktop application with a React/TypeScript frontend and a Rust backend. The codebase demonstrates good architectural patterns with clear separation of concerns across commands, services, and repositories layers. However, there are several opportunities to improve readability, reduce code duplication, and enhance maintainability.

---

## Critical Issues

### 1. Significant Code Duplication in ProductsView Dialogs

**File:** `apps/desktop/src/modules/products/ui/views/products-view.tsx`
**Lines:** 176-333

The Create and Edit dialogs share nearly identical form structures (~80 lines each), violating the DRY principle.

**Why it hurts readability:** Developers must compare both dialogs to understand if they are truly the same or have subtle differences. Changes to form fields require updates in multiple places.

**Suggested improvement:** Extract to a shared `ProductFormDialog` component:

```tsx
interface ProductFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  formData: CreateProductInput;
  onFormChange: (data: CreateProductInput) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  submitLabel: string;
  submittingLabel: string;
}

function ProductFormDialog({
  open,
  onOpenChange,
  title,
  description,
  formData,
  onFormChange,
  onSubmit,
  isSubmitting,
  submitLabel,
  submittingLabel,
}: ProductFormDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <ProductForm formData={formData} onChange={onFormChange} />
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={onSubmit} disabled={isSubmitting}>
            {isSubmitting ? submittingLabel : submitLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

---

### 2. Repeated UUID Parsing Pattern in Commands

**Files:**
- `apps/desktop/src-tauri/src/commands/products.rs` (lines 64-65, 96-97, 115-116)
- `apps/desktop/src-tauri/src/commands/availability.rs` (lines 42-43, 55-56, 69-70)

The same UUID parsing pattern with identical error handling is repeated 6+ times across command files.

**Why it hurts readability:** The repetitive pattern obscures the actual command logic and increases the chance of inconsistent error messages.

**Suggested improvement:** Add a utility function:

```rust
// Add to error.rs or a new utils.rs module
pub fn parse_uuid(id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))
}

// Usage becomes cleaner:
pub async fn get_product(id: String, db: State<'_, DbState>) -> Result<ProductResponse, AppError> {
    let uuid = parse_uuid(&id)?;
    let product = ProductService::get_by_id(db.conn(), uuid).await?;
    Ok(ProductResponse::from(product))
}
```

---

### 3. Long Function with Multiple Responsibilities in ScraperService

**File:** `apps/desktop/src-tauri/src/services/scraper_service.rs`
**Lines:** 128-178

The `extract_availability` function handles three different JSON structures (direct product, ProductGroup, and @graph array) in a single 50-line function with deeply nested logic.

**Why it hurts readability:** The function does too much, making it hard to understand what scenarios it handles at a glance.

**Suggested improvement:** Split into focused helper methods:

```rust
/// Extract availability from a JSON-LD value, trying multiple known structures
fn extract_availability(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
    Self::try_extract_from_product(json)
        .or_else(|| Self::try_extract_from_product_group(json, variant_id))
        .or_else(|| Self::try_extract_from_graph(json, variant_id))
        .or_else(|| Self::try_extract_from_array(json, variant_id))
}

fn try_extract_from_product(json: &serde_json::Value) -> Option<String> {
    if Self::is_product_type(json) {
        Self::get_availability_from_product(json)
    } else {
        None
    }
}

fn try_extract_from_product_group(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
    if Self::is_product_group_type(json) {
        Self::get_availability_from_product_group(json, variant_id)
    } else {
        None
    }
}

fn try_extract_from_graph(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
    json.get("@graph")
        .and_then(|g| g.as_array())
        .and_then(|graph| Self::find_availability_in_items(graph, variant_id))
}

fn try_extract_from_array(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
    json.as_array()
        .and_then(|arr| Self::find_availability_in_items(arr, variant_id))
}

fn find_availability_in_items(items: &[serde_json::Value], variant_id: Option<&str>) -> Option<String> {
    for item in items {
        if let Some(avail) = Self::try_extract_from_product(item) {
            return Some(avail);
        }
        if let Some(avail) = Self::try_extract_from_product_group(item, variant_id) {
            return Some(avail);
        }
    }
    None
}
```

---

## Suggestions for Improvement

### 1. Magic Numbers in AvailabilityBadge Time Formatting

**File:** `apps/desktop/src/modules/products/ui/components/availability-badge.tsx`
**Lines:** 53-56

```tsx
// Current
const diffMs = now.getTime() - date.getTime();
const diffMins = Math.floor(diffMs / 60000);
const diffHours = Math.floor(diffMs / 3600000);
const diffDays = Math.floor(diffMs / 86400000);

// Suggested
const MS_PER_MINUTE = 60_000;
const MS_PER_HOUR = 3_600_000;
const MS_PER_DAY = 86_400_000;

const diffMs = now.getTime() - date.getTime();
const diffMins = Math.floor(diffMs / MS_PER_MINUTE);
const diffHours = Math.floor(diffMs / MS_PER_HOUR);
const diffDays = Math.floor(diffMs / MS_PER_DAY);
```

---

### 2. Duplicated Test Database Setup Code

**Files:**
- `apps/desktop/src-tauri/src/services/product_service.rs` (lines 144-152)
- `apps/desktop/src-tauri/src/services/availability_service.rs` (lines 83-98)
- `apps/desktop/src-tauri/src/repositories/product_repository.rs` (lines 95-103)
- `apps/desktop/src-tauri/src/repositories/availability_check_repository.rs` (lines 77-94)

The `setup_test_db()` function is duplicated across multiple test modules.

**Suggested improvement:** Create a shared test utilities module:

```rust
// src/test_utils.rs (or #[cfg(test)] pub mod test_utils in lib.rs)
#[cfg(test)]
pub mod test_utils {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};

    pub async fn setup_test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);
        // Create all tables...
        conn
    }

    pub async fn create_test_product(conn: &DatabaseConnection, url: &str) -> Uuid {
        // Shared helper
    }
}
```

---

### 3. Long HTTP Header Block in ScraperService

**File:** `apps/desktop/src-tauri/src/services/scraper_service.rs`
**Lines:** 49-64

The 15 header lines in `fetch_page` obscure the function's main logic.

**Suggested improvement:** Extract headers to a const or builder method:

```rust
impl ScraperService {
    fn build_client() -> Result<reqwest::Client, AppError> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .default_headers(Self::browser_headers())
            .build()
            .map_err(AppError::from)
    }

    fn browser_headers() -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", Self::USER_AGENT.parse().unwrap());
        headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap());
        // ... other headers
        headers
    }
}
```

---

### 4. Component Props Could Use JSDoc Comments

**File:** `apps/desktop/src/modules/products/ui/components/products-table.tsx`
**Lines:** 42-47

```tsx
// Current
interface ProductsTableProps {
  products: ProductResponse[];
  isLoading?: boolean;
  onEdit?: (product: ProductResponse) => void;
  onDelete?: (product: ProductResponse) => void;
}

// Suggested
interface ProductsTableProps {
  /** List of products to display in the table */
  products: ProductResponse[];
  /** Show loading skeleton when true */
  isLoading?: boolean;
  /** Called when user clicks edit action for a product */
  onEdit?: (product: ProductResponse) => void;
  /** Called when user clicks delete action for a product */
  onDelete?: (product: ProductResponse) => void;
}
```

---

### 5. ProductsView Has Too Many State Variables

**File:** `apps/desktop/src/modules/products/ui/views/products-view.tsx`
**Lines:** 45-56

The component manages 7 pieces of state related to dialogs and forms.

**Suggested improvement:** Use a discriminated union for dialog state:

```tsx
type DialogState =
  | { type: 'closed' }
  | { type: 'create' }
  | { type: 'edit'; product: ProductResponse }
  | { type: 'delete'; product: ProductResponse };

const [dialogState, setDialogState] = useState<DialogState>({ type: 'closed' });
```

---

## Positive Observations

| Area | Observation |
|------|-------------|
| **Architecture** | Excellent layered architecture (Commands → Services → Repositories) with clear separation of concerns |
| **Error Handling** | Well-documented `AppError` enum with error codes and helpful methods like `is_not_found()` |
| **Constants** | Well-organized into separate files (api.ts, messages.ts, queryKeys.ts, ui.ts) |
| **Test Coverage** | Comprehensive unit and integration tests with clear naming conventions |
| **Type Safety** | Good use of TypeScript strict types and Rust's type system |
| **Documentation** | `CLAUDE.md` provides excellent onboarding documentation |
| **Naming** | Self-descriptive function names throughout the codebase |

---

## File-Specific Ratings

| File | Rating | Key Observation |
|------|--------|-----------------|
| `scraper_service.rs` | Good | Well-tested but could benefit from function extraction |
| `product_service.rs` | Excellent | Clean validation logic, good test coverage |
| `products-view.tsx` | Needs Improvement | Dialog duplication should be addressed |
| `products-table.tsx` | Good | Clean table implementation, good skeleton pattern |
| `useProducts.ts` | Good | Concise hook with clear return value |
| `error.rs` | Excellent | Comprehensive error handling with good ergonomics |
| `availability_check.rs` | Excellent | Good enum design with helpful methods |
| `constants/*.ts` | Excellent | Well-organized, easy to maintain |

---

## Action Items

### High Priority
- [ ] Extract shared `ProductFormDialog` component from `products-view.tsx`
- [ ] Create UUID parsing utility to eliminate repetition in commands

### Medium Priority
- [ ] Refactor `extract_availability` in `scraper_service.rs` for clarity
- [ ] Create shared test utilities module in Rust

### Low Priority
- [ ] Add JSDoc comments to component props
- [ ] Extract magic numbers to named constants in `availability-badge.tsx`
- [ ] Consider using discriminated union for dialog state in `ProductsView`
