# Post-1.0.0 Roadmap

**Last Updated**: May 2026  
**Status**: Active Development  
**Target Audience**: Contributors, users, maintainers

---

## Overview

This document outlines the planned evolution of `rust-things3` after the 1.0.0 stable release. The roadmap is organized into three horizons:

1. **1.x Series** (2026): Minor releases with backward-compatible enhancements
2. **2.0 Release** (2027): Major release with breaking changes  
3. **Long-term Vision** (2027+): Future direction and possibilities

---

## Release Philosophy

### Semantic Versioning Commitment

We strictly follow [Semantic Versioning 2.0.0](https://semver.org/):

- **1.x.y** (Patch): Bug fixes only, no new features
- **1.x.0** (Minor): New features, backward compatible
- **2.0.0** (Major): Breaking changes, API evolution

### Stability Guarantees

**1.x series**:
- ✅ No breaking changes to public APIs
- ✅ Deprecations announced 2 minor versions ahead
- ✅ Security updates backported
- ✅ Bug fixes for critical issues

**2.0 and beyond**:
- Migration guides provided
- Deprecation period honored
- Upgrade path documented

---

## 1.x Series Roadmap

### 1.1.0 (2026-04-26) - Query Enhancements

**Theme**: Powerful querying and filtering via the `advanced-queries` feature flag

#### Shipped Features
- [x] **`TaskQueryBuilder`** — fluent builder with `.execute()` API
- [x] **Natural-language date helpers** — `due_today`, `due_this_week`, `overdue`, etc.
- [x] **Flexible tag filters** — `any_tags` (OR semantics), `exclude_tags` (NOT), `tag_count` (threshold)
- [x] **Fuzzy search** — Levenshtein similarity scoring, `execute_ranked()` returning `Vec<RankedTask>`
- [x] **Saved queries** — JSON-backed `SavedQueryStore` with `to_saved_query()` / `from_saved_query()`
- [x] **Boolean filter expressions** — `FilterExpr` supporting And/Or/Not/Pred combinators

#### Implementation Status
- **Status**: Released — 2026-04-26
- **Breaking**: No
- **Feature Flag**: `advanced-queries`

---

### 1.2.0 (2026-04-27) - Performance & Caching

**Theme**: Cursor-based pagination, streaming results, and smarter cache invalidation via the `batch-operations` feature flag

#### Shipped Features
- [x] **Cursor-based pagination** — `execute_paged()` anchoring on `(creationDate, uuid)` for stable cursors under concurrent edits
- [x] **Streaming API** — `execute_stream()` returning `Pin<Box<dyn Stream<Item = Result<Task>> + Send>>`
- [x] **Predictive cache preloading** — `CachePreloader` trait with `DefaultPreloader` heuristics (inbox↔today, areas→projects)
- [x] **Dependency-based cache invalidation** — evicts only affected cache entries rather than flushing the entire cache
- [x] **`ThingsCacheInvalidationHandler`** — bridges middleware mutation events to targeted cache eviction

#### Implementation Status
- **Status**: Released — 2026-04-27
- **Breaking**: No
- **Feature Flag**: `batch-operations`

---

### 1.3.0 (2026-04-27) - Export Formats

**Theme**: New structured export formats for external tool integration

#### Shipped Features
- [x] **TaskPaper export** — plain-text format for task management apps (`export-taskpaper` feature flag)
- [x] **iCalendar export** — `.ics` files for calendar integration (`export-ical` feature flag)

#### Implementation Status
- **Status**: Released — 2026-04-27
- **Breaking**: No
- **Feature Flag**: `export-taskpaper`, `export-ical`

---

### 1.4.0 (2026-04-28) - Agent Skills & Ecosystem

**Theme**: First-class AI agent integration via a curated skill catalog

#### Shipped Features
- [x] **`things3` foundational agent skill** — full 46-tool catalog reconciled from prior 21-tool list
- [x] **`things3-daily-review` skill** — structured workflow for morning standups and end-of-day reviews
- [x] **Skills catalog README with CI validation**

#### Implementation Status
- **Status**: Released — 2026-04-28
- **Breaking**: No
- **Feature Flag**: None

---

### 1.5.0 (Q2 2026) - AppleScript-First Writes & API Hardening

**Theme**: Promote `AppleScriptBackend` to the default mutation path, adopt `ThingsId` throughout the public API, and harden the public API surface

#### Planned Features
- [ ] **`AppleScriptBackend` as default** — macOS mutations default to AppleScript; direct SQLite writes require `--unsafe-direct-db` opt-in; Linux/CI continues with `SqlxBackend`
- [ ] **`ThingsId` type** — replaces `Uuid` throughout the public API; transparent newtype accepting both RFC-4122 UUIDs and Things-native 21–22-char base62 IDs
- [ ] **Bulk create atomicity** — `bulk_create_tasks` is atomic with rollback on failure
- [ ] **MCP connection resilience** — handler errors no longer drop the JSON-RPC connection
- [ ] **Public API hardening** — explicit `pub use` re-exports in `lib.rs`, `#[non_exhaustive]` on all new public types, `pub → pub(crate)` audit

#### Implementation Status
- **Status**: In Progress (Unreleased)
- **Breaking**: Yes (`ThingsId` replaces `Uuid` in public API)
- **Feature Flag**: None (default behavior change)

---

## 2.0.0 Roadmap (2027)

### Vision

Version 2.0 will be a major evolution, incorporating lessons learned from 1.x usage and community feedback. It will include breaking changes to improve ergonomics, performance, and type safety.

### Tentative Breaking Changes

#### API Evolution

**1. More Granular Error Types**
```rust
// Current (1.x)
pub type Result<T> = std::result::Result<T, ThingsError>;

// Proposed (2.0)
pub enum ThingsDatabaseError { /* ... */ }
pub enum ThingsExportError { /* ... */ }
pub enum ThingsQueryError { /* ... */ }
```

**Benefit**: Better error handling, more specific error context

---

**2. Builder Pattern for Configuration**
```rust
// Current (1.x)
let config = ThingsConfig {
    database_path: Some(path),
    fallback_to_default: true,
    ..Default::default()
};

// Proposed (2.0)
let config = ThingsConfig::builder()
    .database_path(path)
    .fallback_to_default(true)
    .cache(CacheConfig::default())
    .build()?;
```

**Benefit**: More discoverable, type-safe, validated configuration

---

**3. Async Traits**
```rust
// Current (1.x) - Concrete type
pub struct ThingsDatabase { /* ... */ }

// Proposed (2.0) - Trait-based
#[async_trait]
pub trait ThingsDatabase {
    async fn get_task(&self, uuid: Uuid) -> Result<Option<Task>>;
    // ...
}

pub struct SqliteThingsDatabase { /* ... */ }
```

**Benefit**: Testability, mock implementations, alternative backends

---

**4. Improved Type Safety**

~~Proposed for 2.0~~ — **Done in 1.x**: `ThingsId` newtype (a 21–22-char base62 string matching Things 3's native ID format) replaced `Uuid` throughout the public API in v1.x. Separate `ProjectId`, `AreaId`, etc. newtypes could still be considered for 2.0.

```rust
// ThingsId is the ID type throughout the public API
let id: ThingsId = task.uuid.clone();
let task = db.get_task_by_uuid(&id).await?;
```

**Benefit**: Compile-time prevention of mixing IDs, correct round-trip with Things 3 IDs

---

#### New Features (2.0)

- [x] **Write Support** — **Shipped in 1.x** via `AppleScriptBackend`
  - Create, update, complete, delete tasks
  - Full project/area/tag CRUD
  - Bulk operations (complete, move, delete, update dates)
  - AppleScript is the default; direct DB writes opt-in via `--unsafe-direct-db`

- [ ] **Alternative Backends**
  - PostgreSQL support (for server deployments)
  - MySQL support
  - In-memory backend (for testing)
  - Cloud storage backends

- [ ] **GraphQL API** (Optional)
  - Query language for complex data needs
  - Subscription support
  - Schema introspection
  - Playground UI

- [ ] **Enhanced Type System**
  - Phantom types for compile-time guarantees
  - Builder pattern validation
  - State machines for task lifecycles

#### Migration Path

- **Timeline**: 6-month deprecation period in 1.x
- **Tools**: Automated migration tool (`things3-migrate`)
- **Documentation**: Comprehensive 2.0 migration guide
- **Support**: 1.x LTS maintained for 12 months after 2.0 release

---

## Long-term Vision (2027+)

### Possible Features

These are ideas for exploration, not commitments:

#### 3.0+ Possibilities

- **Multi-database Support**: Query across multiple Things databases
- **Time Series Analysis**: Track productivity trends, task velocity
- **AI Integration**: Smart task categorization, priority suggestions
- **Collaboration Features**: Shared tasks, team views (if Things adds support)
- **Mobile Bindings**: Kotlin/Swift wrappers for mobile development
- **Desktop UI**: Tauri-based desktop application
- **Cloud Sync**: Optional cloud backup and sync

#### Ecosystem Growth

- **Community Plugins**: Marketplace for community-contributed plugins
- **Integration Gallery**: Showcase of third-party integrations
- **Educational Content**: Tutorials, courses, workshops
- **Enterprise Support**: Commercial support options for businesses

---

## How to Contribute

### Providing Feedback

We welcome feedback on this roadmap!

- **Feature Requests**: [GitHub Issues](https://github.com/GarthDB/rust-things3/issues)
- **Discussions**: [GitHub Discussions](https://github.com/GarthDB/rust-things3/discussions)
- **Pull Requests**: Implementation proposals welcome

### Prioritization

Features are prioritized based on:

1. **Community Demand**: Highly requested features rise in priority
2. **Impact**: Features benefiting many users prioritized
3. **Complexity**: Quick wins shipped sooner
4. **Breaking Changes**: Batched into major releases

### Helping Out

Want to contribute? Check out:

- **[CONTRIBUTING.md](../CONTRIBUTING.md)**: Contribution guidelines
- **[Good First Issues](https://github.com/GarthDB/rust-things3/labels/good%20first%20issue)**: Beginner-friendly tasks
- **[Help Wanted](https://github.com/GarthDB/rust-things3/labels/help%20wanted)**: Issues needing help

---

## Release Cadence

### Planned Schedule

- **Patch releases** (1.x.y): As needed (bug fixes, security)
- **Minor releases** (1.x.0): Quarterly (Q1, Q2, Q3, Q4)
- **Major releases** (2.0.0): Annually or when significant breaking changes accumulate

### Support Policy

- **Current major version** (1.x): Full support, bug fixes, new features
- **Previous major version** (0.x): Security fixes only, 6 months after 1.0.0
- **Older versions**: No support (upgrade recommended)

---

## Deprecation Policy

### How We Deprecate

1. **Announce**: Deprecation announced in release notes
2. **Warn**: Compiler warnings added (`#[deprecated]`)
3. **Document**: Alternative provided in documentation
4. **Wait**: 2 minor versions (6 months) before removal
5. **Remove**: Removed in next major version

### Example Timeline

```
1.2.0: Feature X announced as deprecated
1.3.0: Feature X still present (warnings)
1.4.0: Feature X still present (warnings)
2.0.0: Feature X removed
```

---

## Communication

### Stay Informed

- **GitHub Releases**: https://github.com/GarthDB/rust-things3/releases
- **CHANGELOG.md**: Detailed change log
- **Blog**: (Future) Development blog
- **Social Media**: (Future) Twitter/Mastodon updates

### Quarterly Updates

We'll provide quarterly roadmap updates:

- **Q1**: January roadmap review
- **Q2**: April roadmap review
- **Q3**: July roadmap review
- **Q4**: October roadmap review

---

## Questions?

- **GitHub Issues**: https://github.com/GarthDB/rust-things3/issues
- **GitHub Discussions**: https://github.com/GarthDB/rust-things3/discussions
- **Email**: (If you set up a project email)

---

## Summary

**Released** (1.1–1.4, April 2026):
- Advanced query builder with fuzzy search, boolean expressions, saved queries
- Cursor-based pagination and streaming results with smarter cache invalidation
- TaskPaper and iCalendar export formats
- First-class AI agent skills catalog

**In Progress** (1.5):
- `AppleScriptBackend` as default on macOS; `ThingsId` type throughout public API
- Public API hardening (`#[non_exhaustive]`, explicit re-exports)

**Mid-term** (2.0):
- API evolution with breaking changes
- Granular error types, async traits, builder-pattern configuration
- Alternative backends (PostgreSQL, in-memory)

**Long-term** (3.0+):
- Advanced features (AI, collaboration, etc.)
- Ecosystem maturity
- Enterprise features

**We're excited for the future of `rust-things3`!** 🚀

