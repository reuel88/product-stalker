# 0001. Availability Checker Stays in Tauri Layer

**Date:** 2025-02-08
**Status:** Accepted

## Context

During the Rust workspace refactor, we evaluated whether the background availability checker (`src/background/availability_checker.rs`) should be extracted into a separate domain crate.

The current architecture is:

```
src/background/availability_checker.rs  (Tauri layer - orchestration)
    │
    ├─→ TauriAvailabilityService        (Tauri adapter)
    │       └─→ AvailabilityService     (domain crate - business logic)
    │
    ├─→ SettingService                  (core crate - settings)
    │
    └─→ Tauri APIs
         ├─→ tauri::async_runtime::spawn()
         ├─→ app.notification()
         └─→ app.emit()
```

## Decision

**Keep the availability checker in `src/background/`** within the Tauri application layer.

The business logic for checking product availability is already extracted into `product-stalker-domain::AvailabilityService`. The background checker is purely orchestration code that:

1. Manages polling intervals and scheduling
2. Spawns async tasks via `tauri::async_runtime`
3. Sends desktop notifications via Tauri plugins
4. Emits events to the frontend via `app.emit()`

These are all Tauri-specific concerns that cannot be decoupled without significant abstraction overhead.

## Consequences

### What becomes easier

- **Simpler crate structure** - No need for additional infrastructure crates for ~100 lines of orchestration code
- **Clear separation maintained** - Domain logic ("what to check") stays in domain crate, orchestration ("when to check, how to notify") stays in Tauri layer
- **Lower maintenance burden** - Fewer crates to manage and version

### What becomes harder

- **Testing the scheduler** - The polling loop is tightly coupled to Tauri runtime; testing requires integration tests with the full app context
- **Reuse across apps** - If we build a CLI or web version, the scheduling logic would need to be reimplemented

### Future consideration

If the app grows to have multiple background tasks (price alerts, sync tasks, etc.), consider extracting a generic scheduler infrastructure crate:

```
crates/scheduler/
├── traits.rs      # TaskRunner, NotificationHandler traits
├── polling.rs     # Generic polling loop
└── lib.rs
```

This is premature optimization for a single background task but should be revisited if the pattern repeats.
