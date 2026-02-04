# Documentation

This directory contains project documentation organized by type.

## Structure

```
docs/
├── decisions/    # Architecture Decision Records (ADRs)
├── guides/       # Setup guides and reference documentation
├── plans/        # Technical specs and implementation plans
└── README.md     # This file
```

## Usage

| Type | When to Use | Lifecycle |
|------|-------------|-----------|
| **Decisions** | Recording architectural choices and their rationale | Permanent (append-only) |
| **Guides** | Setup instructions, maintenance procedures, reference docs | Permanent (update as needed) |
| **Plans** | Detailed implementation specs for features | Archive after completion |

## Workflow

1. **New feature?** Create a GitHub Issue for tracking
2. **Complex implementation?** Add a plan in `docs/plans/` and link from the issue
3. **Architectural choice?** Record in `docs/decisions/` for future reference

## Naming Conventions

- Decisions: `NNNN-short-title.md` (e.g., `0001-use-seaorm-for-database.md`)
- Plans: `YYYY-MM-short-title.md` (e.g., `2026-02-headless-browser-feature.md`)
