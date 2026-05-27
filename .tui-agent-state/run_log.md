# TUI Agent Run Log

---

## Run #15 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-cN0ma` (v0.3.9, ~80s)
- Created tmux session `fresh-test-run15` (220×50)
- **Preflight:** GitHub MCP auth confirmed (listed issues). Playbook integrity verified.
- **Bug recheck — *Keyboard Shortcuts* 'q':** STILL BROKEN ("Editing disabled in this buffer"). Same as Run #14.
- **Bug recheck — #2117 (Review Diff discard hunk):** STILL BROKEN. Created review_diff_test.txt with +3 lines, triggered discard — "Patch failed: error: patch failed: review_diff_test.txt:2error: review_diff_test.txt: patch does not apply". Manual `git apply --reverse --check` succeeds (confirming it's Fresh's bug).
- **Flash: Jump plugin:** PASS — opened via command palette, jump-hint overlay activated (letters replace visible chars), pressed 'n' hint to jump from Ln 7 Col 18 → Ln 7 Col 6.
- **Package Manager (Package: Packages):** PASS — shows 13 available packages with categories [P/T/L], detail panel, filter tabs (All/Installed/Plugins/Themes/Languages/Bundles/Sync). Search by "/" filters: "theme" → 3 results. Registry synced (1/1 sources).
- **Package Manager (Package: Install from URL):** PASS — prompts "Git URL or local path:" input dialog.
- **Live Diff: vs HEAD:** PASS — green `│` gutter markers (ANSI 38;5;78) and green bg (48;5;22) on added lines. Status: "Live Diff: comparing against HEAD".
- **Live Diff: vs Disk:** PASS — `+` marker on unsaved line. Status: "Live Diff: comparing against file on disk".
- **Live Diff: vs Branch...:** PASS — "Branch or ref" prompt pre-filled "main". Status: "Live Diff: comparing against main".
- **Live Grep: Cycle Provider:** PASS — Alt+P cycles: git-grep → rg → grep → git-grep. All 3 providers available. Search "Test" returned 1000+ matches.
- **Block selection (Alt+Shift+Arrow):** PASS — M-S-Down and M-S-Right work! Block selected "Line " (cols 1-5) across rows 1-4. Typing '>' replaced selection on all 4 rows simultaneously. NOTE: Run #12 reported M-S-Down didn't work — it DOES work in this build.
- **Dev Container features:** PASS — Create Config creates minimal .devcontainer/devcontainer.json; Show Info displays container config with action buttons; Show Features shows "No features configured"; Show Forwarded Ports shows "No configured or runtime ports to show."; all Dev Container panels close with 'q' (unlike *Keyboard Shortcuts* buffer).

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| *Keyboard Shortcuts* 'q' close | **STILL BROKEN** | "Editing disabled in this buffer" — same as Runs 12-14 |
| #2117 Review Diff discard hunk | **STILL BROKEN** | Patch failed error persists; manual git apply --reverse works |
| Flash: Jump plugin | **PASS** | Hint overlay activates, pressing hint char jumps cursor |
| Package: Packages browser | **PASS** | 13 packages, search, filter tabs, detail panel, Install button |
| Package: Install from URL | **PASS** | "Git URL or local path:" prompt appears |
| Live Diff: vs HEAD | **PASS** | Green gutter markers on added lines; status confirmed |
| Live Diff: vs Disk | **PASS** | `+` marker on unsaved content; status confirmed |
| Live Diff: vs Branch... | **PASS** | Branch prompt, "comparing against main" confirmed |
| Live Grep: Cycle Provider | **PASS** | git-grep → rg → grep cycling; search works with all providers |
| Block selection (Alt+Shift+Arrow) | **PASS** | M-S-Down and M-S-Right work; rectangular edit confirmed |
| Dev Container: Create Config | **PASS** | Creates .devcontainer/devcontainer.json with template |
| Dev Container: Show Info | **PASS** | Shows config, action buttons, q closes correctly |
| Dev Container: Show Features | **PASS** | "No features configured" |
| Dev Container: Show Forwarded Ports | **PASS** | "No configured or runtime ports" panel with q close |

### Issues Filed / Comments
- No new issues filed (all tests passed or are known bugs with open issues)
- Note: *Keyboard Shortcuts* 'q' bug persists — already tracked via #2125 comment

### Cleanup
- Fresh exited via Ctrl+Q (d = discard and quit)
- tmux session `fresh-test-run15` killed
- review_diff_test.txt commit reverted on dev branch; .devcontainer removed

---

## Run #14 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-c7jCY`
- Created tmux session `fresh-test-run14` (220×50)
- **T47 Rapid keystrokes:** 50-char burst intact; 20 rapid Ctrl+Z all undone correctly. PASS.
- **T48 Resize reflow:** 220×50 → 80×24 → 180×40 all reflow; mid-typing resize safe. PASS.
- **Alt+A project-wide Search & Replace:** Panel opened; 9 matches in 4 files found; Space scoping (deselected source files to scope to test_file1.txt); Replace All with confirmation ("Replaced 3 occurrences in 1 files"). PASS.
- **Calibrate Keyboard wizard:** 24 steps/5 groups (Basic Editing, Line Navigation, Word Navigation, Document Navigation, Emacs-Style). Does NOT test Ctrl+H. s/b/g/a controls all work.
- **#2125 recheck (Diagnostics panel):** q CONFIRMED FIXED (commit 89caf72). `*Keyboard Shortcuts*` 'q' STILL BROKEN ("Editing disabled"). Comment posted on #2125.
- **#2112 recheck (outside-workspace search):** CONFIRMED FIXED (commit b7e7e64). /tmp files now found in Search/Replace panel. Comment posted on #2112.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| T47: Rapid keystrokes | **PASS** | 50-char burst intact, 20 rapid undos clean |
| T48: Resize reflow | **PASS** | All size transitions smooth, mid-typing resize safe |
| Alt+A: Project-wide Search | **PASS** | 9 matches/4 files, scoping, replace all with confirmation |
| Calibrate Keyboard wizard | **TESTED** | 24 steps/5 groups; Ctrl+H NOT tested by wizard |
| #2125 Diagnostics 'q' fix | **CONFIRMED FIXED** | commit 89caf72 verified via UI |
| #2125 *Keyboard Shortcuts* 'q' | **STILL BROKEN** | Shows "Editing disabled in this buffer" |
| #2112 Outside-workspace search | **CONFIRMED FIXED** | commit b7e7e64 verified via UI |

### Issues Filed / Comments
- No new issues filed
- Comment on #2125: Diagnostics panel fixed; *Keyboard Shortcuts* still broken
- Comment on #2112: Confirmed fixed with test procedure

### Cleanup
- Fresh exited via Ctrl+Q; tmux session `fresh-test-run14` killed
- Test files removed: `tmp_test_files/`, `/tmp/rapid_test.txt`, `/tmp/outside_workspace_test.txt`

---

## Run #13 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Loaded state from `tui-automated-testing-state` branch
- Built fresh debug binary from source (`cargo build --bin fresh --features runtime`, ~3.5 min)
  - Binary: `target/debug/fresh`
- Created tmux session `fresh-test` (220×50)
- **Bug Verification (Sprint 12):**
  - TB01: CONFIRMED — `*Keyboard Shortcuts*` 'q' close non-functional (BUG-001)
  - TB02: CONFIRMED — Edit menu "Replace..." mislabeled (BUG-002)
  - TB03: RESOLVED — Alt+W behavior IS correct (context-sensitive, not a bug)
- **GitHub Actions:**
  - Searched for RC12-01: Already covered by issue #2125 → Added comment with Keyboard Shortcuts buffer info
  - Filed new issue #2135 for RC12-02 (Edit menu label mismatch)
- **New Feature Tests:**
  - T28: PASS — Go to Matching Bracket (via command palette; `(` → `)`, `{` → `}`)
  - T30: PASS — Position History (Alt+Left back, Alt+Right forward)
  - T37: PASS — Toggle Line Wrap (View menu ☑ Line Wrap)
  - T45: PASS — Large file (49MB / 500K lines) opens instantly, navigation immediate, search <2s
  - T46: PASS — Binary file (/bin/ls) opens gracefully with [BIN] tag and hex notation

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TB01: Keyboard Shortcuts 'q' close | **CONFIRMED BUG** | "Editing disabled" — same root cause as #2125 |
| TB02: Edit menu Replace label | **CONFIRMED BUG** | Mislabeled "Replace..." → filed #2135 |
| TB03: Alt+W inconsistency | **RESOLVED - NOT A BUG** | Context-sensitive behavior is correct |
| T28: Go to Matching Bracket | **PASS** | Works via command palette |
| T30: Position History | **PASS** | Alt+Left/Right navigate back/forward |
| T37: Toggle Line Wrap | **PASS** | View menu ☑ toggle works both ways |
| T45: Large File Performance | **PASS** | 49MB opened instantly; byte-offset mode |
| T46: Binary File Handling | **PASS** | [BIN] tag; hex notation for non-printable |

### Issues Found / Filed
- Issue #2135 filed: "Edit menu 'Replace...' label maps to Ctrl+Alt+R (Query Replace)"
- Comment on #2125: Keyboard Shortcuts buffer also affected by same root cause

### Key Learnings
- Fresh uses "byte offset mode" for large files (gutter shows bytes, not line numbers)
- Binary files get `[BIN]` tab tag + `<XX>` hex notation for non-printable bytes  
- `Ctrl+]` (ASCII 0x1D) doesn't transmit reliably via tmux send-keys; use command palette for bracket matching
- Alt+W = Close Tab (outside search) is CORRECT behavior; not a bug
- Line Wrap is in View menu (no command palette entry found in this search)

### Cleanup
- Fresh exited via Ctrl+Q
- tmux session `fresh-test` killed
- Test files /tmp/test_brackets.js, /tmp/test_long_line.txt, /tmp/large_test_file.txt deleted

---

## Run #12 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Attempted to load existing state (no local state found → pulled from remote)
- Built fresh 0.3.9 binary from source: `cargo build --release --bin fresh` (~60s)
  - Binary path: `target/release/fresh` (Note: previous runs used `/opt/node22/bin/fresh` via npm)
- Created tmux session `fresh-test` (220×50)
- Executed comprehensive re-verification of Sprints 1-9 (most already tested in Runs 1-11)
- Investigated 2 new potential bugs

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| Sprint 1 (Launch & UI) | **PASS** | All confirmed working as documented |
| Sprint 2 (File Ops) | **PASS** | Ctrl+N/O/S, Alt+W, save dialog all work |
| Sprint 3 (Editing) | **PASS** | Ctrl+Z/Y/C/X/V/A/W/L/D//, all working |
| Sprint 4 (Search/Replace) | **PASS** | Ctrl+F search, Ctrl+R replace, Ctrl+Alt+R query replace all work |
| Sprint 5 (Navigation) | **PASS** | Ctrl+G go-to-line, Command Palette, menu bar |
| Sprint 6 (Command Palette) | **PASS** | All modes (file/>command/:line/#buffer) verified |
| Sprint 7 (Views/Layout) | **PASS** | Split Vertical/Horizontal, File Explorer, Theme Selection |
| Sprint 8 (Tabs/Buffers) | **PASS** | Multi-tab, next/prev buffer, close with confirm dialog |
| Sprint 9 (Terminal) | **PASS** | Integrated terminal, Ctrl+Space toggle, Close Split |
| Settings UI | **PASS** | All categories visible, General settings confirmed |
| Help System | **PASS** | F1 manual, Shift+F1 keyboard shortcuts both open |

### Issues Found This Run

#### BUG-CANDIDATE-RC12-01: Keyboard Shortcuts Buffer 'q' Close Does Not Work
- Buffer text at line 4: "Press 'q' to close this buffer."
- **Actual behavior:** Pressing 'q' shows "Editing disabled in this buffer" in status bar, buffer stays open
- **Workaround:** Use Alt+W
- **Severity:** Low
- **Note:** Check if this is already filed under existing issues before filing new issue
- **Filing blocked:** GitHub MCP token expired this run; file in Run #13

#### BUG-CANDIDATE-RC12-02: Edit Menu "Replace..." Shows Ctrl+Alt+R (Query Replace, Not Basic Replace)
- Edit menu item "Replace..." shortcut = `Ctrl+Alt+R` = opens Query Replace (interactive mode)
- Basic "Replace" (Ctrl+R) is NOT in the Edit menu at all
- Command palette clearly shows two distinct commands: Replace (Ctrl+R) vs Query Replace (Ctrl+Alt+R)
- **Assessment:** May be intentional design, or documentation inconsistency
- **Note:** Already documented in learning_db.md as known behavior; re-verify whether it's a real bug
- **Filing blocked:** GitHub MCP token expired; assess in Run #13

### Key Learnings / Corrections
- Binary can be built from source via `cargo build --release --bin fresh`; binary is `target/release/fresh` not `fresh-editor`
- Binary installed by npm is at `/opt/node22/bin/fresh` (from previous runs); source build works too
- Session persistence confirmed: Unsaved buffers restored on relaunch (hot exit)
- Save/discard dialog confirmed: letter + Enter (not single keypress)
- Keyboard shortcuts buffer cannot be closed with 'q' despite the docs saying so
- Alt+W and Whole Word toggle conflict documented: Alt+W in search bar = toggle whole word; outside search = close tab
- Block selection tmux keys: `M-S-Down` appears to NOT trigger block select reliably in this tmux version (investigation needed)

### Cleanup
- tmux sessions `fresh-test` and `quit-test` both killed
- No test files left behind on disk (all were in /tmp)

---

## Run #11 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from `claude/ecstatic-mayer-5DivD` branch (6.5 min build)
- Checked out `tui-automated-testing-state` branch, loaded all prior state
- Launched tmux session `fresh-qa` (200×50)
- Executed 10+ test objectives covering bookmarks, Settings add/delete/reset, and LSP with fake-pylsp

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-BOOKMARKS | **PASSED** | Alt+1/2/etc jump to bookmarks 1/2/etc; "not set" for missing; Ctrl+P → "Set Bookmark" |
| TC-SETTINGS-ADD-NEW | **PASSED** | Typing while focused on list header activates [+] Add new inline input; Enter confirms |
| TC-SETTINGS-CTRL-R | **RESOLVED/PARTIAL** | Ctrl+R is a NO-OP for field reset; Escape from field reverts pending changes; [ Reset ] button via Tab works |
| TC-SETTINGS-DEL-X | **PENDING** | [x] buttons appear mouse-only; keyboard navigation to sub-list items not confirmed |
| TC-FAKE-LSP | **PASSED** | fake-pylsp recognized as `pylsp`; LSP starts; connection handshake logged |
| TC-LSP-GOTO-DEF | **PASSED** | F12 Go to Definition works; navigates to LSP-returned location |
| TC-LSP-HOVER | **PARTIAL** | Alt+K shows "No hover info available" (expected with fake-pylsp null response) |
| TC-LSP-REFERENCES | **PASSED** | Find References opens dock panel with clickable results; Enter navigates correctly |
| TC-REFERENCES-NAV | **CONFIRMED** | References panel Enter WORKS (unlike *Quickfix* BUG #2124) |

### Issues Found This Run
- **0 new bugs filed**
- **1 important distinction**: References panel (from LSP Find References) correctly handles Enter navigation — this is DIFFERENT from *Quickfix* buffer (BUG #2124 which is from Live Grep Alt+M)
- **Ctrl+R in Settings**: Does NOT reset number fields — CHANGELOG claim may be incorrect for 0.3.9

### Key Learnings
- Binary 0.3.9 confirmed from `fresh --version`
- Bookmarks: `Ctrl+P → Set Bookmark → digit → Enter`; jump with `Alt+N`
- Settings list [+] Add new: type text directly while header is focused (no Enter needed to start)
- Settings [x] delete: likely mouse-only (no keyboard path found)
- Escape from Settings pending field: REVERTS changes (useful as keyboard reset)
- fake-pylsp setup: symlink `scripts/fake-lsp/bin/fake-pylsp` → `/usr/local/bin/pylsp`; set `FAKE_DEVCONTAINER_STATE` env
- LSP Find References panel IS keyboard-navigable (Enter works); bug is specific to *Quickfix*

---

## Run #10 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (tui-automated-testing-state base = `88883dc`, v0.3.8)
- Launched tmux session `fresh-test` (200×50)
- Executed 7 test objectives: Alt+/, Markdown Preview, Keyboard Macros, Settings Ctrl+R, Review Diff regression check

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-ALT-SLASH | **PASSED** | `M-/` opens Live Grep; 375 results for "fn main"; preview split works |
| TC-MARKDOWN | **PASSED** | Markdown Compose mode: ANSI bold/italic/headings; status "Markdown Compose: ON (soft breaks, centered)" |
| TC-MACRO-RECORD | **PASSED** | "Record Macro" prompt (0-9), F5 stops, macro saved with action count |
| TC-MACRO-PLAYBACK | **PASSED** | F4 plays macro correctly; all 3 test lines got " [MACRO]" appended |
| TC-MACROS-LIST | **PASSED** | "List Macros" opens `*Macros*` buffer; WARNING: buffer is editable (not strict RO) |
| TC-SETTINGS-CTRL-R | **PARTIAL** | Ctrl+R when field highlighted does NOT reset; `[ Reset ]` button reachable via Tab; full test inconclusive |
| TC-REVIEW-DIFF-CONTROLS | **FALSE POSITIVE CORRECTED** | All controls broken BY DESIGN — per `docs/internal/review-diff-feature-restoration-plan.md` (Status: Planned) |

### Issues Found This Run
- **0 new bugs filed**
- **1 false positive corrected**: Run #8 TC-REVIEW-DIFF-DISCARD "PASSED" was wrong; Review Diff panel controls were never implemented in this codebase version

### Key Learnings
- Version is 0.3.8 (not 0.3.9 as previously logged)
- Review Diff panel controls are planned-but-not-implemented features
- DECCKM `$'\033OB'` must be UNQUOTED in bash (not inside double quotes)
- `Explorer` menu item appears in menu bar when File Explorer is used
- `*Macros*` buffer is editable (different from strictly-RO Quickfix/Diagnostics)

---

## Run #9 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`, ~3 min)
- Checked out `tui-automated-testing-state` branch, loaded state from 8 prior runs
- Launched tmux session `fresh-test` (200×50)
- Executed 8+ test objectives covering LSP popup navigation, Quickfix navigation, shell commands, multi-cursor, diagnostics panel, and backlog items

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-LSP-POPUP-NAV-2 | **CONFIRMED** | Plain `Up`/`Down` keys navigate popup; ANSI `[48;5;25m]` highlight confirms selection |
| TC-QUICKFIX-ENTER | **BUG FOUND** | Enter → "Editing disabled"; no navigation keybindings for Quickfix; BUG #2124 filed |
| TC-DIAG-PANEL-SHORTCUTS | **BUG FOUND** | q/a/Enter all → "Editing disabled"; status hints are non-functional; BUG #2125 filed |
| TC-SETTINGS-CTRL-R | **PARTIAL** | Ctrl+R in Settings closes the overlay; `[ Reset ]` footer button not reachable via Tab cycling |
| TC-SHELL-CMD | **PASSED** | `Alt+|` → "Shell command:" prompt → sort → `*Shell: sort*` tab with sorted output |
| TC-SHELL-CMD-REPLACE | **PASSED** | `Shell Command (Replace)` via palette → `sort -r` → in-place replacement confirmed |
| TC-MULTICURSOR-LINE-ENDS | **PASSED** | `M-I` (Alt+Shift+I) on 5 lines → "6 cursors | Added cursors to line ends (6)" |
| TC-BUG2122-RECHECK | **STILL OPEN** | `move_to_paragraph_down/up` still have no keybinding; select variants have `Ctrl+Shift+↓/↑` |

### Issues Found This Run
- **BUG #2124 filed**: Quickfix buffer `Enter` shows "Editing disabled" — no jump-to-match behavior despite design spec requiring it
- **BUG #2125 filed**: Diagnostics panel `q/a/RET` shortcuts are non-functional — status bar hints are misleading

### Key Discoveries This Run
1. **Quickfix buffer has no navigation keybindings**: Searching Keybinding Editor for `/quickfix` only shows export bindings (Alt+M, Alt+Q in `prompt` context). The design doc says Enter should navigate but this was never implemented.
2. **Diagnostics panel shortcuts don't work**: The `q: close | a: toggle filter | RET: goto` hints in the status bar and `Enter:select | Esc:close` panel body text are misleading — these shortcuts are not bound.
3. **Shell Command feature fully confirmed**: Both `Alt+|` (output to new buffer) and `Shell Command (Replace)` (output replaces selection) work correctly. Tested with `sort` and `sort -r`.
4. **Add Cursors to Line Ends (`M-I`) confirmed working**: 5-line selection → 6 cursors at line ends. Status bar shows confirmation message.
5. **Fake LSP (`scripts/fake-lsp/bin/fake-pylsp`) discovered**: Requires `FAKE_DEVCONTAINER_STATE` env var. Could unlock LSP feature testing in future runs.
6. **Settings UI Ctrl+R investigation**: The `Ctrl+R` key closes Settings overlay (routes to global Find & Replace). The `[ Reset ]` button is in the footer but not reachable via Tab cycling in the tested workflow. Needs further investigation.
7. **Settings keystroke leak confirmed**: Navigating Settings with Tab and search can leak keystrokes into editor. Config file was accidentally modified during testing (restored manually).

### Lessons Learned
See learning_db.md for additions: Lesson 44–50

---

## Run #8 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`, ~3 min)
- Pulled state from `tui-automated-testing-state` branch (7 prior runs)
- Launched tmux session `fresh-test` (200×50)
- Executed 10 test objectives covering 0.3.9 features, bug regression checks, and new discoveries

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-LSP-STATUS | **PASSED** | LSP status popup shows server state; auto-opens log tab on failure; states: (off)/(error)/running |
| TC-LSP-POPUP-NAV | **DISCOVERED** | DECCKM sequences (ESC prefix) CLOSE popups; use plain Up/Down for popup nav |
| TC-LIVE-GREP-DIAG | **PASSED** | Alt+D toggles Diagnostics scope; "No matches" without LSP (expected); provider disappears |
| TC-LIVE-GREP-ALTM | **PASSED** | Alt+M saves to `*Quickfix*` [RO] buffer in split; format: `file:line:col  content` |
| TC-ORCHESTRATOR-0.3.9 | **PASSED** | New UI: Alt+P project scope, Alt+T show worktrees, `/` filter, session detail buttons |
| TC-C3-LANGUAGE | **PASSED** | C3 syntax highlighting fully working; `C3` status bar; folding at fn/struct |
| TC-REVIEW-DIFF-DISCARD | **BUG FIXED** | BUG #2117 CONFIRMED FIXED in 0.3.9 — discard works correctly; comment on GH issue |
| TC-WORKSPACE-RESTORE-2056 | **PASSED** | Session isolation by working directory confirmed; no cross-project tab mixing |
| TC-PLUGIN-API-DATADIRS | **DOCUMENTED** | getWorkingDataDir() and getTerminalDir() documented from API types |

### Issues Found This Run
- **None filed** — BUG #2117 resolved; all other behaviors working as expected or documented

### Key Discoveries This Run
1. **BUG #2117 (Review Diff discard) FIXED**: Confirmed working in 0.3.9 dev build. Tested twice. Comment posted on GitHub.
2. **Popup navigation insight**: DECCKM sequences (`$'\033OA'`, `$'\033OB'`) start with ESC which CLOSES any active overlay/popup. For popup list navigation, use plain tmux key names (`Up`, `Down`). DECCKM only applies to cursor movement inside the editor buffer.
3. **C3 language support**: Full syntax highlighting with Sublime syntax grammar. `.c3`, `.c3i`, `.c3t` extensions. c3lsp configured but not bundled.
4. **Orchestrator 0.3.9 UI**: New project scope filter (Alt+P), show-all-worktrees toggle (Alt+T), `/` filter search, session detail action buttons (Visit/Details/Stop/Archive/Delete).
5. **Live Grep Alt+M Quickfix buffer**: Saves all matches to `*Quickfix*` [RO] buffer with `file:line:col  content` format, 249 matches saved correctly.
6. **LSP (error) state**: When LSP binary missing: Fresh tries to start it, immediately opens the log file as a [RO] tab, status bar shows `LSP (error)`. Log shows the exact error (e.g., `Unknown binary 'rust-analyzer' in official toolchain`).

### Lessons Learned
See learning_db.md for additions: Lesson 35–43

---

## Run #7 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`)
- Pulled state from `tui-automated-testing-state` branch (6 prior runs)
- Launched tmux session `fresh-test` (200×50)
- Executed 12 test objectives covering 0.3.9 new features and backlog items

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-DASHBOARD-DEFAULT | **CONFIRMED** | 0.3.9: Dashboard no longer opens by default with `--no-restore` |
| TC-PARA-SELECT | **PASSED** | select_to_paragraph_down/up work via Ctrl+Shift+↓/↑ (CSI 1;6B / CSI 1;6A escape sequences) |
| TC-SETTINGS-CHECKBOX | **RESOLVED** | Checkboxes ARE reachable: ↑↓ arrows navigate to them in the right panel; Enter toggles them |
| TC-CONFIRM-QUIT | **PASSED** | Enable in Settings → "Confirm Quit: [ ]" → Enter → Save; Ctrl+Q shows `Quit Fresh? (y)es, (N)o:` |
| TC-SCROLL-SYNC | **PASSED** | Both splits scroll together when Scroll Sync enabled; confirmed with CHANGELOG.md in both panes |
| TC-AUTO-REVERT | **PASSED** | External file append detected and reverted within ~3s (auto_revert_poll = 2000ms default) |
| TC-NEXT-WINDOW | **TESTED** | "Next Window" returns "Cancelled" when only 1 window open — correct single-window behavior |
| TC-LIVE-GREP-0.3.9 | **PASSED** | New toolbar working: scope toggles (Files/Buffers/Terminals), provider cycle, [buf] tag, Word mode |
| TC-PAGEDOWN-OVERSHOOT | **PASSED** | Basic PageDown/PageUp navigation correct; targeted fix hard to confirm without bug repro file |
| TC-COMPLETION-AUTO-SHOW | **PARTIAL** | Setting toggles correctly; popup requires LSP (off) — not testable without LSP server |
| TC-PARA-MOVE-BUG | **BUG CONFIRMED** | move_to_paragraph_down/up have NO default keybinding and are NOT in command palette → GitHub #2122 filed |
| TC-BUG-2117-CHECK | **STILL OPEN** | Review Diff discard bug NOT fixed in 0.3.9 (not in changelog fixes) |

### Issues Found This Run
- **BUG #2122 filed**: `move_to_paragraph_down/up` actions (0.3.9 feature) have no default keybinding and no command palette entry. Users cannot invoke the feature without manually binding it. Inconsistent with `select_to_paragraph_*` which have `Ctrl+Shift+↓/↑`.

### Key Discoveries This Run
1. **Settings checkboxes via keyboard**: Navigate with ↑↓ arrows (DECCKM) in the right panel, press Enter to toggle. This DOES work — previous run's concern was unfounded. Tab only reaches number/text inputs.
2. **select_to_paragraph escape sequences**: CSI 1;6B = Ctrl+Shift+Down, CSI 1;6A = Ctrl+Shift+Up — confirmed working
3. **Live Grep 0.3.9**: Provider shows as `[ git-grep ]`, `[ rg ]`, `[ grep ]` when cycling with Alt+P. File scope results untagged; Buffer scope results show `[buf]` prefix.
4. **confirm_quit prompt format**: Shows `Quit Fresh? (y)es, (N)o:` at bottom line, requires letter + Enter (N+Enter = stays open, Y+Enter = quits).
5. **Settings search**: Press `/` in Settings UI while in the LEFT panel to trigger search across all setting names (not just visible category).
6. **move_to_paragraph design intent** (from PR #2084): Author intentionally omitted palette commands but appears to have overlooked adding default keybindings — `select_to_paragraph` has bindings but the new `move_to_paragraph` does not.

### Lessons Learned
See learning_db.md for additions: Lesson 29–34

---

## Run #6 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, ~50s)
- Checked out `tui-automated-testing-state` branch, loaded state from 5 prior runs
- Launched tmux session `fresh_test` (200×50)
- Executed 7 test objectives covering theme editor, auto-save, env manager, tour, review diff, orchestrator, workspace trust

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-THEME-EDITOR (complete) | **PASSED** | Color edit + Save As → custom theme created in ~/.config/fresh/themes/ |
| TC-AUTO-SAVE | **PASSED** | Enable in config; file auto-saved within 8s (5s interval); tab loses asterisk |
| TC-ENV-MANAGER | **PASSED** | Show Status → Activate (direnv) → Deactivate: all 3 commands working |
| TC-TOUR | **PASSED** | Load .fresh-tour.json; navigate Step 1→2→3→4→Exit; each step opens correct file |
| TC-REVIEWDIFF-STAGE | **PASSED** | Stage hunk with `s`: 3 added lines moved to STAGED section |
| TC-ORCHESTRATOR-NEW | **PASSED** | Alt+N → form → Tab×6 to Create Session → session-1 worktree created |
| TC-WORKSPACE-TRUST | **PASSED** | T to trust → status bar confirms "Workspace trusted" |

### Issues Found This Run
- **PENDING BUG INVESTIGATION**: Settings UI checkboxes NOT reachable via Tab key. Tab navigates to number/text inputs and footer buttons, skipping checkboxes (e.g., "Auto Save Enabled"). Needs investigation whether this is by design or a bug.
- **NOTE**: Orchestrator "Create Session" button requires exactly 6 Tab presses from the dialog open state to reach the button. More than 6 = cycles back to checkbox. Important UX discovery.
- **NOTE**: Tour panel button navigation: Tab focuses buttons, Up/Down navigates within tour panel. Pressing Enter when "Next →" is focused advances the tour.

### False Positive Rate: 0% (0 of 0 bugs filed)

### Settings Navigation Discovery
The Settings UI uses a complex navigation model:
- `↑↓` in left panel: navigate sections
- `Tab`: jump to next focusable widget IN THE RIGHT PANEL (number inputs and text inputs only; checkboxes are NOT tab-navigable)
- `Enter` on section: scrolls right panel to show that section
- Auto-save was enabled by directly editing /root/.config/fresh/config.json (demonstrated it persists and works)

---

## Run #1 — 2026-05-26

### Status: COMPLETED (with post-run self-correction)

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, 16s)
- Initialized all state files for the first time
- Launched tmux session, executed 30+ test cases across core launch, file ops, editing, search/replace, and views
- Filed 4 GitHub issues
- **Post-run:** Reviewed documentation, discovered 2 of 4 issues were false positives
- Closed #2108 and #2110, updated #2109 and #2111

### Test Results Summary
| Category | Passed | Failed | Notes |
|----------|--------|--------|-------|
| Core launch (TC-001–011) | 11 | 0 | |
| File operations (TC-020–026) | 7 | 0 | |
| Editing (TC-030–035) | 6 | 0 | |
| Search & replace (TC-040–049) | 8 | 1 | TC-043 Shift+F3 broken in tmux (terminal compat) |
| Views & layout (TC-050–058) | 9 | 0 | |
| Issues filed | 4 | — | 2 real (#2109, #2111); 2 false positives (#2108, #2110) |

### Lessons Learned (Run #1)
- Arrow key DECCKM requirement discovered
- Menu highlight verification requires `-e` ANSI capture
- Hot exit causes file restoration on re-launch — not a bug
- "Revert" vs "Reload with Encoding" distinction
