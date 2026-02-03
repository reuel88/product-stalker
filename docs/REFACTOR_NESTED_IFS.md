# Refactor Nested If Statements

This document tracks nested if statements that should be refactored to use early returns and guard clauses.

## Overview

Per the project's code style guidelines in `CLAUDE.md`:
> Prefer early returns over nested if statements (guard clauses)

## Findings

### High Priority

#### 1. `apps/desktop/src-tauri/src/services/scraper_service.rs` (Lines 130-175)

**Function:** `extract_availability()`

**Issue:** Multiple nested if + if-let patterns inside loops

**Current Pattern:**
```rust
if Self::is_product_type(json) {
    if let Some(avail) = Self::get_availability_from_product(json) {
        return Some(avail);
    }
}
```

**Suggested Fix:**
- Combine conditions using `&&` or use `and_then()` chaining
- Extract repeated patterns into helper functions that return early

---

#### 2. `apps/desktop/src-tauri/src/lib.rs` (Lines 87-99)

**Function:** Autostart configuration block

**Issue:** Nested if-else-if with nested if-let in both branches

**Current Pattern:**
```rust
if settings.launch_at_login && !is_enabled {
    if let Err(e) = autostart_manager.enable() {
        log::error!("Failed to enable autostart: {}", e);
    } else {
        log::info!("Autostart enabled");
    }
} else if !settings.launch_at_login && is_enabled {
    // similar nesting...
}
```

**Suggested Fix:**
- Use early returns for non-matching conditions
- Use `.map_err()` or match expressions instead of nested if-let
- Consider extracting into a helper function

---

### Medium Priority

#### 3. `apps/desktop/src-tauri/src/services/scraper_service.rs` (Lines 181-207)

**Functions:** `is_product_type()` and `is_product_group_type()`

**Issue:** Nested if-else chains inside map closures

**Current Pattern:**
```rust
json.get("@type")
    .map(|t| {
        if let Some(s) = t.as_str() {
            s == "Product"
        } else if let Some(arr) = t.as_array() {
            arr.iter().any(|v| v.as_str() == Some("Product"))
        } else {
            false
        }
    })
    .unwrap_or(false)
```

**Suggested Fix:**
- Replace if-else chain with `match` statement
- Consider using pattern matching at the top level

---

#### 4. `apps/desktop/src-tauri/src/plugins/system_tray.rs` (Lines 58-66)

**Function:** Window visibility toggle in tray click handler

**Issue:** Nested if statements checking window visibility

**Current Pattern:**
```rust
if let Some(window) = tray.app_handle().get_webview_window("main") {
    if window.is_visible().unwrap_or(false) {
        // hide logic
    } else {
        // show logic
    }
}
```

**Suggested Fix:**
- Extract visibility toggle into a helper function
- Use early return if window is None

---

### Low Priority

#### 5. `apps/desktop/src-tauri/src/commands/window.rs` (Lines 64-72)

**Function:** Window transition command

**Issue:** Sequential if-let statements

**Notes:** Already reasonably structured, minor improvements possible with pattern matching or helper functions.

---

## Progress Tracker

| File | Lines | Status |
|------|-------|--------|
| `services/scraper_service.rs` | 130-175 | [ ] Pending |
| `lib.rs` | 87-99 | [ ] Pending |
| `services/scraper_service.rs` | 181-207 | [ ] Pending |
| `plugins/system_tray.rs` | 58-66 | [ ] Pending |
| `commands/window.rs` | 64-72 | [ ] Pending |

## References

- [CLAUDE.md](../CLAUDE.md) - Project code style guidelines
- [Rust by Example - Early Returns](https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html)
