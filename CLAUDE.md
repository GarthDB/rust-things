# rust-things3 — Claude Code Notes

## Code Intelligence (ferrograph)

`ferrograph index .` builds `.ferrograph` (gitignored). Use before refactors:

```bash
ferrograph status .                          # node/edge overview
ferrograph callers <node_id> -d .ferrograph  # who calls this?
ferrograph blast <node_id> -d .ferrograph    # what breaks if this changes?
ferrograph dead -d .ferrograph               # unreachable code
```

Run `ferrograph search "name" -d .ferrograph` to find node IDs by symbol name.
Particularly important before touching shared types like `TaskFilters`.

## SemVer constraint — do NOT modify `TaskFilters`

`TaskFilters` (in `libs/things3-core/src/models.rs`) is a stable public struct since
1.0.0 with no `#[non_exhaustive]`. Adding public fields is a breaking change. All
new filter capabilities must live as **private fields on `TaskQueryBuilder`** only.

## Builder-only pattern for new filters

All new query predicates follow this shape:

1. Private field on `TaskQueryBuilder`, gated: `#[cfg(feature = "advanced-queries")]`
2. Public builder method, also gated
3. Applied in `execute()` after `query_tasks()` returns, never in `build()` / `TaskFilters`
4. When any post-filter is active, `execute()` strips `limit`/`offset` before the DB call
   and re-applies pagination in Rust after filtering

## Feature flag: `advanced-queries`

Gates all query execution APIs: `query_tasks()`, `TaskQueryBuilder::execute()`, and
every builder-only predicate. Unit tests that touch gated fields must be gated too.

```bash
cargo test -p things3-core --lib                              # without feature
cargo test -p things3-core --features advanced-queries --lib  # with feature
cargo clippy -p things3-core --lib --features advanced-queries -- -D warnings
```

## Public API surface lives in `lib.rs`

Every `pub` item in a module must have an explicit `pub use` in the crate's `lib.rs`.
No glob re-exports (`pub use foo::*`). If you add a new public type, add it to the
`pub use` list in `lib.rs`; if it should be internal, use `pub(crate)` from the start.

## Default to `pub(crate)`

New items that don't need to be part of the crate's public API must use `pub(crate)`,
not `pub`. This is enforced by `unreachable_pub = "warn"` in `[workspace.lints.rust]`.

## `#[non_exhaustive]` policy

All new public enums, error enums, and public structs with fields that may grow must
carry `#[non_exhaustive]`. This is a free, backwards-compatible change that prevents
downstream `match` exhaustiveness from breaking when new variants or fields are added.

Grandfathered exceptions (do NOT add `#[non_exhaustive]`):
- `TaskFilters` — frozen stable public struct since 1.0.0; extension via builder pattern only
