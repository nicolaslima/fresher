# Review Picker — Plan

## Goal

Replace the two existing entry points to the review-diff feature
(`start_review_diff` and `start_review_range`) with a single command
**Review** that opens a dedicated **picker** screen. The picker covers
the four highest-leverage UX gaps in the current review-diff:

1. **No "type a revspec" tax for the common case** — auto-detected
   "This PR" preset is preselected; Enter opens the review immediately.
2. **No blind commits** — the picker has a live preview pane that
   re-renders the diff as the user moves through the list.
3. **"Since `<sha>` (N new)"** — the picker remembers the HEAD you
   last saw for this `(branch, base)` and offers `<stored-sha>..HEAD`
   as a preset. The row label literally names the stored SHA, so the
   user sees exactly what the comparison is and doesn't have to
   guess what "last reviewed" or "last close" or "last fetch" means.
   See *Watermarks* below.
4. **Comment-count badges** — saved comments become discoverable from
   the picker, not only after opening a range.

The existing review buffer group (toolbar + sticky header + diff +
comments) is unchanged except for one new 1-row **context ribbon**
between the toolbar and the sticky header that names what is being
reviewed and exposes a `p` keybind to re-open the picker.

## Non-Goals

- **No new keymaps inside the diff/comments panels.** All review-mode
  bindings stay as they are. The single addition is `p` → open picker.
  (`p` was chosen over `g` to keep `g`/`G` available for `gg`/`G`
  top/bottom navigation inside the picker list; `g` in vim-land is a
  prefix, not a command.)
- **No editor-core changes.** The picker is a buffer group built from
  the existing `createBufferGroup` primitive, the same way
  `start_review_branch` already is.
- **No new persistence schema.** The watermark file and the comment
  count come from the existing `<data_dir>/audit/<repo>/` directory.

## Two-screen model

The picker is **not** a left panel attached to the review. It is a
separate buffer group, opened in place of the review (or in place of
the editor when first launched). Same takeover pattern as the command
palette and `start_review_branch`.

```
┌────────────┐   Enter         ┌────────────┐
│   PICKER   │ ──────────────▶ │   REVIEW   │
│            │                 │            │
│            │ ◀────────────── │            │
└────────────┘   press  p       └────────────┘
        │                            │
        │ press q                    │ press q
        ▼                            ▼
    back to editor               back to editor
```

Why two screens (rather than a third permanent pane in the review):

- **Space**: the review already runs diff + comments side-by-side. A
  third permanent pane would starve the diff on terminals narrower
  than ~150 columns.
- **Focus clarity**: the picker is a *deciding* task; the review is a
  *reading* task. `j/k` means different things in each. Mixing the
  two in one layout costs a "which pane am I in?" check on every
  keystroke.
- **Consistency**: command palette, file picker, and the existing
  branch-review screen are all takeover screens. Users already know
  the pattern.
- **No wasted pixels after decision**: once a range is picked, the
  picker disappears and the review gets 100% of the area.

## Picker layout

```
split v 0.02
  fixed   header   h=1     "Review what?   Enter open · q cancel · ? keys"
  split h 0.4
    scrollable list        (presets, commits, branches, custom)
    scrollable preview     (live diff of the row under the cursor)
```

The header is a deliberate 3-item strip, not a full cheatsheet —
`?` inside the picker opens the complete key list. Half-disclosed
cheatsheets are worse than either a full one or a single discovery
hook.

List pane content:

```
 ★ This PR  (main..HEAD)             7 commits  +52/−12   (3)
   Since 7e2a9f1  (7e2a9f1..HEAD)               3 new     (1)
   Working tree                                 16 files  (2)
 ─── COMMITS ─────────────────────────────────────────────
   bca083a  feat: farewell
   9e478d5  feat: f-strings                               (1)
   03637f8  feat(util): sub/mul
 ─── BRANCHES ────────────────────────────────────────────
 > main                                                   (0)
   origin/main
   release/v2                                             (4)
 ─── CUSTOM ──────────────────────────────────────────────
 :  type a revspec…
```

**Glyphs — one channel, one meaning.**

| Glyph  | ASCII fallback | Meaning                                    |
|--------|----------------|--------------------------------------------|
| `★`    | `*`            | Auto-detected default (cursor lands here)  |
| `>`    | `>`            | Current branch                             |
| `(N)`  | `(N)`          | Saved-comment count under this range key   |

Notes:
- `(0)` is hidden, not rendered dim. Dimming relies on color/attribute
  support that screen readers and some terminals mangle; omission is
  unambiguous.
- The count is rendered as text `(N)` rather than a unicode dot so
  it's parseable by screen readers and by users who pipe the picker
  through `tmux`/`screen`/SSH sessions that mangle BMP glyphs.
- The `★` and `>` glyphs have ASCII fallbacks selected by the same
  terminal-capability check the rest of the app uses.
- "Last commit" is not a preset row; it's just the first entry under
  COMMITS and would be redundant as a separate preset.

Layout flips to vertical (list on top, preview below) when
`viewport.width < 100`.

**Focused-pane indicator.** `Tab` toggles focus between list and
preview. The focused pane is marked by a bolded/colored title bar and
a left-edge `▌` (fallback `|`) margin rail. Without this, "which pane
does `PageDown` scroll?" is a mystery-key-per-press cost.

## Picker behaviour

### Smart default — `★ This PR`

On open, the cursor lands on `★ This PR`. The "from" is resolved in
this order, falling back through to the last entry that succeeds:

1. `git rev-parse --abbrev-ref @{u}` — upstream of current branch
2. `git merge-base HEAD <default-branch>` — `main`, then `master`,
   then `trunk`
3. `HEAD~1` — last commit only

The `<default-branch>` is whatever `git symbolic-ref refs/remotes/origin/HEAD`
points at, with `main`/`master`/`trunk` fallbacks if the remote head
isn't set.

If the resolved range is empty (you are sitting on the default branch
with no upstream divergence), the row is shown disabled and the
cursor falls through to `Working tree`.

### Live preview

Every cursor move in the list pane debounces a `git diff <from>..<to>`
(trailing debounce, 150 ms) and re-renders the right pane using the
same `buildListLines` / `parseDiffOutput` pipeline the real review
uses. Per-range cache; cleared when the picker closes. Result:
scrolling through commits feels instant on revisits, and what the
user sees in the preview is byte-identical to what they get on Enter.

**Fast-scroll behaviour** (holding `j` down the list):

- The previously-rendered diff stays on screen — no blanking, no
  "Loading…" flash per row.
- The preview pane's title gets a trailing `…` to indicate "this
  shows the *previous* row's diff while the next one computes."
- An in-flight `git diff` is cancelled when the cursor moves again,
  so only the last settled position actually fetches.
- Once the user pauses past the debounce window, the pane swaps in
  the new diff and drops the `…` from the title.

The trailing-edge debounce (rather than leading) is deliberate: it
means scrolling through 20 rows issues *one* `git diff`, not 20.

### Working tree — what's in the diff

The `Working tree` row resolves to `git diff HEAD` (staged +
unstaged combined). File counts and `+N/−M` stats in the picker and
ribbon are computed against the same command, so they never
disagree with the diff the user actually sees.

### Watermarks — "Since `<sha>`"

**Label.** The row is titled `Since <short-sha>` where the SHA is
substituted at render time from the stored tip. The label names the
exact thing the comparison is built from — no verb ("reviewed",
"closed", "pushed") for the user to second-guess. If the stored SHA
is `7e2a9f1`, the row reads `Since 7e2a9f1` and the resolved range
is `7e2a9f1..HEAD`.

The preview pane and ribbon both carry the full revspec
(`7e2a9f1..HEAD`) so the baseline is visible in two places: the
list label names the anchor point, the preview shows what's in the
range.

**Key.** Watermarks are keyed by `(branch, base)`, not by branch
alone. Reviewing `main..feature/x` and reviewing
`origin/main..feature/x` on the same branch are two different
activities and should advance independent markers.

```json
{
  "watermarks": {
    "feature/x": {
      "main":        { "tip": "abc123", "updated_at": "2026-04-16T…" },
      "origin/main": { "tip": "abc123", "updated_at": "2026-04-16T…" }
    }
  }
}
```

**Write policy.** The watermark for `(branch, base)` is rewritten
only on review close (`q` or `stop_review_diff`). `p` (pick another
slice) does *not* advance the watermark: it's a navigation action,
not a "done" action, and the user may come right back. A SIGKILL'd
review leaves the watermark at its prior value, which is the honest
answer to "where was I up to?"

**Read.** When the picker opens, for each watermark entry whose
`tip` differs from `HEAD` **and** whose `tip` is still reachable
from `HEAD` (i.e. ancestor), render a `Since <short-sha> (N new)`
row resolving to `<tip>..HEAD`. Show at most one such row (the
`(branch, base)` that matches the currently-defaulting base).

**Dangling watermarks (force-push / rebase).** If `tip` is *not*
reachable from `HEAD` — typical after a rebase/force-push — the
stored SHA no longer names a meaningful range on the current
history. Fall back to `git merge-base <tip> HEAD`; if non-empty,
render the row with the merge-base SHA in the label
(`Since <merge-base-short>`) and `<merge-base>..HEAD` as the range.
If the merge-base is empty or equals `HEAD`, drop the row entirely.
The label always matches the SHA actually used in the diff — we
never show a row whose name differs from its comparison baseline.

**No watermark yet.** Freshly-checked-out branch, never reviewed:
hide the row rather than render a confusing `Since HEAD (0 new)`.

This is the unique-value-prop feature. Most reviewers come back to a
PR after the author pushes follow-up commits; today they have to find
the old SHA themselves.

### Comment badges

On picker open, list `<data_dir>/audit/<repo>/*.json` once, parse the
`comments.length` from each, and key the counts by review key
(`worktree`, `range-<from>__<to>`). Render `(N)` next to any list row
whose resulting range key has a non-zero count; rows with zero
comments render nothing at all. O(files), tens of ms even with
hundreds of saved reviews. Re-scanned on picker open and on `r`
(refresh); re-entering the picker via `p` invalidates only the
preview cache, so badges reflect comments added during the review
that was just closed.

### Keys (picker mode)

```
j / k / Up / Down       move list cursor
gg / G                  jump to top / bottom of list
Enter                   open the row's range as a review
Tab                     toggle focus between list and preview
PageDown / PageUp       scroll focused pane
:                       focus the custom-revspec field
r                       refresh (re-scan branches/commits/badges)
?                       show the full keymap
q  /  Esc               cancel; close picker, return to where you came from
```

Note: `g` by itself is *not* bound — it is reserved as the prefix for
`gg` (top) and similar future two-key nav commands, matching vim
conventions.

### Custom revspec (`:`)

Pressing `:` moves focus to the one-line revspec input at the bottom
of the list pane. Behaviour:

- The input is a plain text field (no autocomplete in v1).
- Enter parses the input with `git rev-parse` (two dots, three dots,
  and single-ref forms supported). On success, the preview pane
  refreshes and a second Enter opens the review.
- On parse failure, the field border turns red and a one-line reason
  (`unknown revision 'foo'`) is shown in place of the preview pane's
  status row; the input is *not* cleared so the user can edit.
- History: the last 20 successful revspecs are kept in
  `<data_dir>/audit/<repo>/revspec_history.json` and exposed with
  `Up`/`Down` inside the input (mirrors readline).
- `Esc` inside the input returns focus to the list without clearing.

## Review screen — the one new row

`REVIEW_LAYOUT` adds one fixed-height node:

```
split v 0.02
  fixed toolbar  h=2
  split v 0.02
    fixed ribbon h=1               ← NEW
    split v 0.02
      fixed sticky h=1
      split h 0.75
        scrollable diff
        scrollable comments
```

Ribbon content (mode-aware):

| Mode      | Ribbon text                                                      |
|-----------|------------------------------------------------------------------|
| worktree  | `Working tree · 16 files · +40/−77 · 0 comments         p: pick` |
| range     | `★ main..HEAD · 2 files · +10/−1 · 0 comments           p: pick` |
| commit    | `bca083a feat: farewell · 1 file · +3/−0 · 0 comments   p: pick` |
| since     | `7e2a9f1..HEAD · 3 new · +12/−4 · 1 comment             p: pick` |

Always visible. The "what am I reviewing?" question never requires a
keystroke to answer.

**Notation.** Keybindings are rendered inline as `<key>: <label>`
(matching the existing toolbar convention), not as bracketed tokens
like `[g]` — brackets read as buttons and we are never clickable in
this mode.

**Truncation.** On narrow terminals the ribbon must fit a single
line. Priority order when trimming from widest to narrowest:

1. The keybind hint (`p: pick`) is **pinned** to the right edge and
   never truncated.
2. The stats block (`N files · +A/−B · M comments`) stays intact
   down to ~40 columns of ribbon width.
3. The identifier on the left (range spec / commit subject) is
   middle-ellipsized (`feat(a…): sub/mul`) before the stats are
   dropped.
4. Below ~40 columns the stats collapse to `±Δ` (net line count) and
   then to nothing, in that order.

`p` (mnemonic: **p**ick another range) closes the review group and
opens the picker. Initial picker selection is the row corresponding
to the range you just left, so `p`-then-Enter is a no-op refresh.

## Code surface

| Concern                          | Where it lives                            | New / reused |
|----------------------------------|-------------------------------------------|--------------|
| Picker buffer group + layout     | new `audit_picker.ts` (sibling to `audit_mode.ts`) | **new**      |
| List rendering (presets/commits/branches/custom) | `audit_picker.ts`                  | **new**      |
| Live preview rendering           | `audit_picker.ts`, calls existing `parseDiffOutput` + `buildListLines` | **new** wrapper, **reuses** existing |
| Per-range diff cache             | `audit_picker.ts`                          | **new**      |
| `★ This PR` resolution           | `audit_picker.ts` helper                   | **new**      |
| Comment-count scan               | `audit_picker.ts` helper, reads `getDataDir() / audit / <repo> / *.json` | **new** (tiny) |
| Watermarks read / write          | `audit_picker.ts` (read), `audit_mode.ts` `stop_review_diff` (write) | **new** (tiny) |
| Ribbon row                       | `audit_mode.ts`: extend `REVIEW_LAYOUT`, add `buildRibbonEntries()` + truncation helper | **modified** |
| `p: pick` keybind                | `audit_mode.ts`: add `p` to `review-mode` keymap; new handler `review_open_picker` | **modified** (~3 lines) |
| Open review with picked range    | reuses `bootstrapRangeReview` (`audit_mode.ts:3886`) and the worktree path of `start_review_diff` | **reused** |

## Lifecycle

1. User runs **Review** (single command; replaces both `Review Diff`
   and `Review Range (Commit or Branch)`).
2. Picker buffer group opens. Default-detection runs; cursor lands on
   the auto-selected row. Comment-count scan runs. Branches and
   recent commits enumerate. Preview pane shows the default's diff.
3. User browses with `j`/`k`. Preview pane debounce-updates.
4. Enter on a row → close picker group → open review group with that
   range. Saved comments load from `<data_dir>/audit/<repo>/<key>.json`
   exactly as they do today.
5. Inside review: ribbon reflects the slice. Reviewer reads,
   comments, navigates as today.
6. `p` from review → close review group, open picker with the
   current range pre-selected. No watermark change (user may return
   to the same slice). Comments are persisted continuously already,
   so nothing is lost.
7. `q` from review → advance watermark for `(branch, base)` to
   current `HEAD`; close review.
8. `q` from picker → close picker; return to the editor (no review
   was opened, no watermark change).

## What goes away

- **`start_review_range`** and its single-prompt UI (the picker
  replaces it). The `cmd.review_range` i18n keys also drop.
- **The "type a revspec" friction** for users who want anything other
  than HEAD. Power users still have `:` inside the picker.
- **The "I have to open it to know if I have comments"** dance — the
  comment-count badges expose this in the picker.
- **The "what's new since I last looked at this branch?" hunt** —
  the `Since <sha>` row resolves the range for you, and names the
  exact SHA it's comparing against.
- **The "what am I reviewing again?" check** — the ribbon names it.

## Out of scope (good follow-ups, not blockers)

- **Rebase-aware comment matching**: today comments roll forward only
  when the underlying lines still exist; a fingerprint match on
  `(file, surrounding-3-line-hash)` would survive minor rewrites.
  Independently useful; not required for the picker.
- **Resolved / unresolved comment state**: a third state beyond
  exists/deleted. Belongs in the comments panel, not the picker.
- **Per-line `git blame` in the diff**: useful in multi-author
  branches; orthogonal.
- **Mouse support in the picker** (click row to preview, double-click
  to open). Easy to add later.

## Risks and open questions

- **Preview fetch cost**: `git diff main..HEAD` on a large monorepo
  can take seconds. Mitigations described under *Live preview*: keep
  the previous render visible with a `…` title marker, trailing-edge
  150 ms debounce, per-range cache, cancel any in-flight fetch when
  the cursor moves again.
- **Freshly-checked-out branch**: there is no watermark yet. Hide
  the `Since <sha>` row rather than render a confusing
  `Since HEAD (0 new)`.
- **Force-pushed / rebased branch**: the stored `tip` is no longer
  reachable from `HEAD`. Handled by the "Dangling watermarks" rule
  under *Watermarks*: fall back to `merge-base`, else drop the row.
- **Default detection on detached HEAD**: no upstream, no branch.
  Fall through to merge-base with default branch; if that also
  fails, the default becomes "Working tree" rather than a broken
  range.
- **Picker on a non-git directory**: the picker should refuse to
  open with a single-line "Not a git repo" message, the same way the
  current review-diff already handles `emptyState === 'not_git'`.
- **Watermark race**: two concurrent sessions reviewing the same
  `(branch, base)` both writing on close. Last write wins; acceptable
  because the watermark is advisory, not a lock. Worth a note in the
  code comment so no one is tempted to add file locking.

## Phasing

Two user-visible ship points:

- **Ship A — Ribbon only.** Phase 1 below. No `Review` command, no
  picker, no behaviour change beyond "surface what the review
  already knows in a new 1-row ribbon." Ships standalone; it's a
  pure UI addition that doesn't depend on any of the picker work.
- **Ship B — Picker replaces the old entries.** Phases 2–4
  together. This is all-or-nothing: the `Review` command lands, the
  `start_review_range` entry point is removed, and users see the
  picker flow. Cannot partially ship because phases 2–4 depend on
  each other for the picker to feel coherent (preview without
  badges, or badges without watermarks, ships an obviously
  unfinished screen).

Follow-up ship points, each independently useful:

- **Ship C** — Phase 5. Commit list section.
- **Ship D** — Phase 6. Branch list section.

Phase-by-phase sequencing:

1. Ribbon row in the existing review (no behaviour change).
2. Picker buffer group with **presets only** (`★ This PR`, `Working
   tree`, `:custom`). Live preview wired in. Replaces
   `start_review_range`.
3. Comment-count badges on preset rows.
4. Watermark write on close + `Since <sha>` preset.
5. Commit list section + per-row badges.
6. Branch list section.
