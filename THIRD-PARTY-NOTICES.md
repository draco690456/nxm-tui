# Third-Party Notices

This project includes code adapted from third-party sources. The original
copyright and licenses are reproduced below.

## grok-build (xai-org/grok-build)

- **Source:** `crates/codegen/xai-grok-pager-render/src/search` (`mod.rs`, `matcher.rs`)
- **Copyright:** Copyright 2023-2026 SpaceXAI
- **License:** Apache License 2.0
- **Usage:** The text search module (`src/search.rs` — `QueryKind`, `TextMatcher`,
  `next_index_after`, `prev_index_before`) was adapted from the above source.
  The adaptation replaces the compiled-regex field with `Option<regex::Regex>`
  to remove all production `unwrap()`/`expect()` fallbacks.

A copy of the Apache License 2.0 is available at
<https://www.apache.org/licenses/LICENSE-2.0>.
