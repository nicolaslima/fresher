# How Rendering Works in Fresh

## Why it isn't trivial

Like all text editors, Fresh takes an on-disk file containing raw bytes, and presents it as an editable document on the screen. Laying out content on the screen is not trivial. What you see on the screen is certainly related to bytes in the file, but this relation is more than a simple 1:1 mapping between byte offsets and screen x/y coordinates. At the very basic level - only a small part of a document may actually be included in the current viewport. The layout needs to account for line break symbols that interrupt the flow (must parse the text to decide visual position), and this also depends on whether line wrapping is enabled or long lines must disappear off the edge of the screen. Already we add another exception to the 1:1 mapping - even inside a contiguous region of file bytes, only some portions are actually visible while other bytes are skipped. Furthermore, text files come in many different encodings, some - like ASCII - are single byte per visual character, others are variable-byte coded or allow characters to have variable width (or to be stacked visually into a single cell).

All this is to say: there is no simple mapping between file offsets and position on screen. Rendering must consider:

- Visual width and stacking of characters
- Line breaks
- Line wrapping modes

There are many more things that rendering must take into account:

- Current screen size
- Scroll position - which part of the document is currently in view
- Scrollbar position rendering (handle / "thumb" size and position)
- Styling based on content (syntax highlighting, markdown preview, etc.)
- Styling based on metadata (e.g. highlighting selected text or search results, marking LSP diagnostics)
- Selective hiding or showing of content (code folding, hiding markdown markup, etc.)
- Cursor position (including multiple cursors)
- Gutter indicators such as: indicators for code folding, modified lines, errors or warnings from LSP
- Split views - showing multiple documents (buffers) at the same time

And there are performance constraints:

- User operations should feel snappy and immediate: opening & saving files, editing, scrolling, jumping to remote locations, etc.
- Scrolling should feel smooth
- Indicators, styles and highlighting should move around together with the text - inserting lines or characters should shift the style stuff as well - without laggy artifacts
- RAM and CPU usage should be reasonable (in Fresh I made it a priority to keep RAM usage low)

We need to also consider features like efficient undo/redo, editing via multiple cursors, search/replace, etc. These are not directly related to rendering but must be taken into account, otherwise performance can suffer.

Fresh also provides a plugin API for extending the editor without modifying the core code. There is tension in what we allow plugins to do:
- Plugin API should have primitives that encourage high-performance rather than chatty or wasteful loops to make it easy to write good plugins.
- At the same time, plugins shouldn't cause the editor to become laggy, or hang or crash.

Fresh supports remote editing of files over SSH, and I made it a goal to support huge files also when editing them over SSH. Sending all the file's content back and forth is not generally a good idea.

So how does Fresh provide all of the above?

## TextBuffer - the data storage layer

One of my goals when creating Fresh was to *instantly* open huge files (many GBs). This is accomplished by only loading those parts of the file needed for editing. If the user never views or edits entire multi-GB regions, there's no reason to read them from disk or to store them in memory. Fresh uses a *piece tree* to keep track of which parts of the file are only on-disk, loaded in memory, or edited/inserted (content that is in memory and was not yet saved to the file). The piece tree only stores offsets, pointers, and line ending indexes (more about this later). The actual data buffers are separately allocated. This allows different trees to point to the same underlying data without excessive data copying. For quick diffing (finding modified regions), and for fast bulk-undo and failure tolerance, the piece tree is a persistent data structure - every modification leaves the old tree root untouched.

## Large vs small files

Initially all files were treated equally - lazy loading of buffers from disk as the user navigates to different scroll areas of the file. Later I found this approach has a few annoying drawbacks.

For fast syntax highlighting in huge files, Fresh doesn't read the entire file and feed it into the highlighter just to show a small range of text somewhere far into the file. Instead, in the lazy-loading mode Fresh loads a few kbs before the top of the viewport to start the highlighter grammar with *some* context (imagine that just before the first line on the screen, in the previous line there was an open " mark - if the highlighter doesn't know about it, it will render incorrectly). This approach does yield high performance opening (or jumping around) in a huge multi-GB file - but the highlighting sometimes breaks down, missing important grammar context, for example if the extra window before the start of the viewport falls inside a string.

Another problem with lazy-loading chunks of the file is that we can't know the current line numbers.

To deal with these issues I finally gave in, and built two internal modes: small files and large files. In small file mode, Fresh does load everything into memory. In large file mode, the lazy-loading approach is used. Syntax highlighting in large files favors speed over correctness. And instead of line numbers, large files show byte offsets in the gutter. As users sometimes want actual line numbers even in huge files, I added a feature allowing you to force a full-file scan that builds a line number index in memory (it reads the entire file but doesn't retain it in memory).

Features that require the entire file's content to be in-memory - such as LSP, and line-accurate scrollbar handles - are disabled in large-file mode. The large file threshold is configurable. Generally, every file you normally edit as code - almost never >1MB - is small enough to be a "small file".

## Encoding

Text files come in various encodings: UTF-8 or ASCII are commonly used, but we also support others like UTF-16, Windows-1250/1251/1252, GBK and GBK18030, ShiftJis, and others. In each of these systems, byte values have different meanings and there are different rules (sometimes bytes are control bytes that affect the meaning of other bytes). For simplicity, Fresh decodes all files at the low level TextBuffer layer, and stores all buffer data as UTF-8.

Due to the fact that some bytes affect the meaning of future bytes, not all encodings can be partially loaded (for lazy large file chunking). Some encodings can be easily read even from the middle of a file - these are called "resynchronizable", because even if you've lost the prefix of the stream you can still find a byte that unambiguously resets the encoder state (in ASCII every byte is independent, in unicode there is a well-defined way to find an unambiguous reset though it may take a few bytes to reach it). Other encodings are not resynchornizable (GB18030, GBK, ShiftJis, EucKr) which means: lazy chunk loading is not possible for these files because we must read ALL the data from the beginning of the file to know how to interpret a byte in the middle.

## The Interval Tree and Marker System

Several editor features attach metadata to text regions. Search results, LSP diagnostic higlights & gutter indicators show up visually when rendering and should stay attached to the same text region even as the user inserts or deletes text.

Example: at first the word "thing" is being highlighted, because the user searched for it:

```
     int x = thing;
             ^^^^^
```

Then, the user inserts text before the higlighted area, the higlight should move:

```
     int xabc = thing;
                ^^^^^
```

It would be wrong for it to stay put:

```
     int xabc = thing;
             ^^^^^
```

This means that higlights can't be assigned an absolute offset in the file or even a line number + column. We need some system to keep the higlight position in sync with the text it's supposed to highlight.

This is where the interval tree comes into play. An interval tree is a data structure that tracks marker IDs, and maintains the offset of each marker and efficiently shifts them around, accounting for insertions or deletions in the text. The interval tree nodes contain absolute offsets of markers. For fast offset shifting (text editing) operations, nodes contain a lazy "pending delta", that all descendent nodes will apply during query time (these deltas can be pushed down when found during queries). Once all deltas are clean (pushed down to the leafs) absolute offsets makes tree rebalancing a simple operation that doesn't need to update node contents.

The goal of this system is that text editing - inserting and removing characters in the middle of the text - automatically shifts all attached metadata, at low cost.

When rendering the viewport, we traverse the interval tree of markers starting at the viewport's start offset, and collect marker IDs as long as we still need to fill up the viewport. These marker IDs are then looked up to gather various metadata values (such as a bunch of LSP diagnostics in an LSP-specific map).

## Mapping screen position back to source offset

The cursor position tracks a byte offset in the source buffer. Inserting or deleting characters affect the source and are introduced into the piece tree. When a user clicks in the middle of some line, we need to translate the visual position (row and column on the screen) to a source offset, so we can move the cursor.

To perform this reverse mapping (visual position -> byte offset), we maintain a list of rendered lines and their constituent "parts" (visual elements), calcualted during the last render. Each element in a line has both a source offset and visual column offset. Using this information we can map both ways.

#### **The Rendering Pipeline**

To render a viewport, Fresh maintains an absolute byte offset representing the top of the visible window. The engine processes the text through a six-stage pipeline until the viewport is filled:

1. **Input Source Text:** Raw UTF-8 bytes are read from the TextBuffer.
2. **Tokenizer:** The raw bytes are converted into base tokens. Line endings (LF/CRLF) become line break tokens, spaces and tabs become whitespace tokens, non-text bytes become binary tokens, and contiguous standard text becomes text tokens.
3. **View Transformer:** Plugins are permitted to alter the token stream. This allows for content transformation or the injection of virtual text, such as Git blame headers or diff filler lines.
4. **Wrapping:** The pipeline enforces line limits. Line break tokens are inserted if a line exceeds a predefined safety threshold (preventing memory exhaustion from abnormally long lines, such as a 1GB single-line JSON file) or if soft wrapping is enabled and the line exceeds the viewport width.
5. **Line Generation:** The pipeline generates ViewLine structures. These structures contain a bi-directional map linking source byte offsets to visual column offsets. The visual-to-byte mapping calculates source locations during cursor movement, while the byte-to-visual mapping calculates screen positions and handles horizontal scrolling.
6. **Styling and Rendering:** Styles, decorations, and cursors are applied to the visual lines using interval trees.

#### **Markers, Overlays, and Highlighting**

Applying metadata—such as syntax highlighting, user selections, and decorations—requires tracking specific text ranges. Because text edits shift these ranges, Fresh uses an interval tree to maintain marker information.
The interval tree stores markers by position and ID. Text insertions or deletions fed into the interval tree automatically shift the positions of all affected markers, removing the need for manual coordinate recalculation. Overlays are built on top of this system by pairing start and end markers to represent self-adjusting ranges.
During rendering, the set of markers applying to the current viewport is extracted into a sorted array. This avoids executing O(\\log n) interval tree lookups for every byte offset processed.
Highlighting is applied using three distinct methods:

* **Syntax Highlighting:** Recalculated per frame for the current viewport and a preceding context window. Fresh utilizes the syntect library to apply Textmate-based grammars.
* **Reference Highlighting:** When the cursor rests on a symbol, all visible occurrences of that word are highlighted. These occurrences are registered as overlays in the interval tree. Because overlays auto-adjust during edits, the highlights remain accurate without recalculation, only invalidating when the cursor moves to a different word.
* **Semantic Highlighting:** The Language Server Protocol (LSP) provides precise highlighting tokens based on language semantics. These are translated into overlays. Fresh utilizes the LSP "range" API for the current viewport and the "full" document API, which supports delta updates to report only what has changed during edits.




-------------


# Rendering Pipeline Evolution in `sinelaw/fresh` — Last ~6 Months

~340 commits touched rendering code (`crates/fresh-editor/src/view/`, rendering primitives, `crates/fresh-winterm/`) between mid-October 2025 and 2026-04-14. Summary of themes, with representative SHAs.

## 1. Major Architectural Changes

### Syntax highlighting: O(n²) re-parse → marker-based checkpoints + convergence
The biggest architectural change in the window. A multi-commit VSCode-style checkpoint system replaced full re-parses on every keystroke:

- `02b15bc` — ParseState/ScopeStack checkpoints every 4 KB (embedded CSS-in-HTML survives viewport jumps)
- `594c513` — *"Replace checkpoint Vec with marker-based system and convergence"*. MarkerList-backed, O(log n) adjustments. Message: *"v1 discarded all checkpoints after every edit, causing 20KB+ re-parses per keystroke in large files."*
- `5c94850` — partial-cache convergence: re-parse only until stored ParseState matches current (*"~256–512 bytes per keystroke, not the entire ~22KB viewport"*)
- `974e497` — merged convergence walk and highlight walk into a single pass
- `04396d6` — fixed span-cache offset drift (*"highlight colors drifting left by 1 position per keystroke"*)
- `6016dd9` — fixed unbounded checkpoint search on large files (found checkpoints at byte 12 KB when viewport was at 11 MB)

### Layout / rendering decoupling
- `d034dc1` *"decouple layout from rendering to fix macro replay bugs"* — split `render_buffer_in_split` into `compute_buffer_layout` + `draw_buffer_in_split`; added `Editor::recompute_layout` for macro playback, since the stale layout cache was breaking synchronous macro actions.
- `1d8a454` / `55b1c6d` / `a78f54d` — moved cursors + line-number state out of global `EditorState` into per-split `BufferViewState` (preceded by `1e78388` which fixed state bleed between splits).

### New `fresh-winterm` crate for Windows (`7d6faa9`)
Bypasses crossterm mouse handling entirely, reads VT input directly via `ReadConsoleInputW`, handles `wRepeatCount` and UTF-16 surrogates, detects corrupt mouse sequences (ESC dropped by Windows console), removes InputParser's timeout-based ESC disambiguation. Followed by several reverts (`3c8b2be`, `4854535`, `aa7c3fc`, `fe7ef1a`) — a rough landing.

### Compose-mode + concealment (`d72c439`, Feb 14)
Introduced conceal ranges, plugin-driven soft breaks, centered compose-mode rendering, and new plugin APIs (`addConceal/addSoftBreak/setLayoutHints`). Later `2b496b5` filters conceals per view_mode so two splits on the same buffer render differently.

### Review-diff pipeline rewrite (recent, Apr 13)
- `ca66b7f` — *"unify files+diff into a single scrollable stream"*
- `6f89b82` — *"collapse via host fold infra — no buffer rebuild"* (reuses new byte-offset fold infra)
- `97e1e6f` earlier replaced a JS-plugin diff gutter with native Rust rendering; `188fe31` cut `diff_since_saved` from O(lines²) to O(edit_size).

## 2. Bug Fixes by Category

**Cursor visibility / hardware cursor** (very active):
- `f6ebb01` bar/underline cursor invisible (REVERSED hid the thin hardware cursor)
- `59e92bc` block cursor invisible in zellij (double-inversion: REVERSED + hardware block)
- `ba5795b` + `9dbc4e6` GUI (ratatui-wgpu) has no hardware cursor → stamp a post-render REVERSED cell
- `16c00197` block selections rendered twice

**Color / theme**: `166fba5` BOLD leaking into dropdown borders (Style::reset vs default); `84c3f7a` selection wiping syntax fg; `2822abf` WCAG contrast in 256-color mode (bg + selection both mapped to black); `c961f45` u8 overflow in grayscale math yielding black for RGB 248; `425f24b` Windows Terminal truecolor detection via `WT_SESSION`.

**Wrapping / reflow**: `f22c16f` double-counted indent squishing single-char-per-line wrapping; `e093150` tabs measured with `char_width` returning 0; `0c166a1` popup-copy extracting unwrapped text with selection coords in wrapped space.

**Off-by-one / half-open range**: `817cbb7` — *"the fix is one character: `>=` → `>`"* — `extend_to_line_end` overlays bleeding onto next row due to half-open range interpretation.

**Viewport math**: `a7a8b44` (logical-vs-visual lines in scroll limit); `3fe80cc`/`fe70b03`/`c8af1bc` three follow-ups resyncing `top_view_line_offset` after view_data rebuild.

**Unicode / terminal quirks**: `05a6ab8` Unicode boundary panic in keybinding editor; `59e92bc` zellij; `7d6faa9` Windows mouse.

## 3. Performance Improvements

- `8cc8667` — O(1) cursor replacing linear `.find()` span lookups in `compute_char_style` (*"~5.6% CPU"*)
- `14e24ec` — debounced semantic highlighting 150ms (was 57% CPU)
- `506620d` — minified-JSON 100% CPU fix via `match_indices` + 1 MB cutoff
- `141295f` — `BracketHighlightOverlay::calculate_nesting_depth()` 16KB-chunked, capped at 1 MB: 24.7s hang on a 438 MB file, *"~16000× fewer buffer reads"*. `d764eac` enforces the cap globally.
- `a2809df` — `debug_span!` → `trace_span!`: was writing *"~91 KB (234 syscalls) to the log on every cursor movement"*
- `b533962` — removed `path.canonicalize()` from decoration cache on every render frame (fixes #886 on rclone mounts); cached raw user config (was re-read ~20×/s)
- `f76a182` — parallel line scanning for exact large-file line numbers (no more estimates)
- `d0409d5` — cached `scrollbar_visual_row_counts()` per frame

## 4. Most Interesting / Non-trivial Bugs

Several stand out as especially pedagogical:

**`d0409d5` — the PageDown-scroll OOM**. Wrapping transform entered an infinite loop when `line_indent > available_width/2` made `remaining_width` perpetually 0. The commit message walks through why only specific width/indent combinations triggered it, and why a `CrosstermBackend::Fullscreen` test couldn't reproduce it (`autoresize()` overrode the width). The fix both corrects (force-emit one grapheme to break the loop) and defends (guard `width < 2`).

**The `817cbb7` one-character fix** — tracking whitespace bleeding on the next row back to a half-open-range misinterpretation. Smallest interesting fix in the window.

**Checkpoint convergence iteration saga** (`02b15bc` → `594c513` → `5c94850` → `974e497` → `04396d6` → `6016dd9`). Multi-day iteration: ship feature, discover double-parse, merge passes, find span-cache drift on insert, discover unbounded checkpoint search on large files. Each fix driven by perf counters (`HighlightStats`) the author added alongside.

**GUI/hardware-cursor double-inversion**: four related commits (`59e92bc`, `f6ebb01`, `ba5795b`, `9dbc4e6`) chase the same class of bug — REVERSED conflicting with the terminal's own cursor — in different combinations (zellij, bar/underline style, wgpu backend, EOL).

**Windows VT input (`7d6faa9`)** — introduces an entire new crate because `crossterm::EnableMouseCapture` replaces the whole console mode and removes VT input. Had to preserve Quick Edit, handle UTF-16 surrogates, detect corrupt mouse sequences, add a 30-s heartbeat to counteract *"silent ConPTY drift"*.

**Compose-mode ripple effects**: `d72c439` introduced a new decoration axis (conceal ranges + plugin soft breaks) that had consequences all the way into scroll math (`ae37f5f`, six weeks later: wheel "absorbed" at every long-wrap list-item boundary) and split sync (`97ff449`: *"compute during sync was impossible because view lines aren't available yet"* — solved with a `sync_scroll_to_end` deferred flag).

**Large-file handling as a coherent strategy** — spanning line-number estimation → byte offsets (`f76a182`), folding (`2f913de`, `08886c4`), bracket scanning (`141295f`), syntax highlighting (`6016dd9`), and diff (`188fe31`). The theme: every render path must degrade to O(viewport), never O(file). `08886c4`'s predecessor was estimating line numbers via `byte/80` on large files — *"wildly wrong results"*.

**Theme-editor redesign (`993a149`)** — full rewrite with inline styling + *"flicker-free rendering"* via atomic content+color updates, plus `8a34466` hiding diagnostic overlays during preview.
