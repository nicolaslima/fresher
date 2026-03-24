# Investigation: E2E Test Does Not Reproduce Large File Highlighting Bug

## Resolution

**There was no discrepancy.** Both tmux and the e2e test behaved identically:
0 highlight spans at EOF on both first and second Ctrl+End visits.

The doc's claim that "tmux shows 210 spans, green strings visible" on the first
Ctrl+End was incorrect. Re-testing at commit `c606febc` with debug logging
confirmed that tmux also gets `slice_bytes_len=0` and all-white text at EOF.

### Root cause

`find_parse_resume_point` used `query_range(0, desired_start + 1)` — an
unbounded search that found a checkpoint at byte ~12KB (from the initial
top-of-file parse on file open). It then called
`buffer.slice_bytes(12169..11387835)`, but the buffer uses lazy loading:
only ~24KB near the cursor (at EOF) was loaded by `ensure_visible`. The
range `12169..~11371835` contained `Unloaded` buffer pieces, so
`get_text_range` (immutable) returned `None`, and `slice_bytes` returned
an empty vec via `.unwrap_or_default()`.

With 0 bytes of content, `full_parse` produced 0 spans.

### The fix (commit 88717d1, re-applied in fe7fb71)

Two changes to `find_parse_resume_point`:

1. **Bounded checkpoint search**: `query_range(desired_start - MAX_PARSE_BYTES, desired_start + 1)`
   instead of `query_range(0, desired_start + 1)`. With no nearby checkpoint,
   the fallback starts fresh at `desired_start`, and `slice_bytes` only needs
   ~13KB of data that IS loaded by `ensure_visible`.

2. **Enable checkpoint creation in fallback**: Changed `create_checkpoints=false`
   to `true`, so the first visit creates checkpoints near EOF. The second visit
   reuses them, parsing only ~13KB instead of starting fresh again.

### Key data (after fix)

| Step | spans | bytes_parsed | highlight_colors |
|------|-------|-------------|-----------------|
| First Ctrl+End | 204 | 12,886 | 2 (green, blue) |
| Ctrl+Home | 278 | 12,402 | 5 |
| Second Ctrl+End | 204 | 12,886 | 2 |

The test now asserts that both visits to EOF produce highlighting and that
the second visit parses < 1MB (was 11MB before the fix).

## Files involved

- `crates/fresh-editor/src/primitives/highlight_engine.rs` — `TextMateEngine::full_parse`, `find_parse_resume_point`
- `crates/fresh-editor/tests/e2e/syntax_highlighting_embedded_offset.rs` — test `test_large_file_highlighting_survives_navigation`
- `crates/fresh-editor/src/model/buffer.rs` — `Buffer::load_from_file`, `slice_bytes`, `get_text_range` (returns None for Unloaded pieces)
- `crates/fresh-editor/src/view/viewport.rs` — `ensure_visible` loads ~24KB around cursor via `get_text_range_mut`
