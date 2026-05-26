# TUI Agent Run Log

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
| Search/replace (TC-040–047) | 6 | 1 partial | TC-042 Enter behavior |
| Views/layout (TC-050–054) | 5 | 0 | |

### Issues Filed
| Issue | Final Status | Verdict |
|-------|-------------|---------|
| #2108 Revert fails | **Closed** | False positive — wrong menu item tested |
| #2109 Ctrl+H terminal compat | **Open** | Real issue — terminal sends 0x08 as Backspace |
| #2110 File opens modified | **Closed** | False positive — hot exit feature by design |
| #2111 Search F3 navigation | **Open** | Needs re-test with search bar closed |

### False Positive Rate: 50% (2 of 4)

---

## LESSONS LEARNED — Run #1

These are concrete, actionable lessons. The next agent MUST read this section before starting.

### Lesson 1: Read documentation BEFORE touching the keyboard
**What happened:** We tested for 2 hours before reading `docs/features/`. Two of our four bug reports were for documented, intentional features (hot exit, Revert prompt logic).

**Rule:** At the start of every run, spend the first 5 minutes reading:
- `docs/features/` — authoritative feature list and keybindings
- `docs/configuration/keyboard.md` — actual keybinding table
- `CHANGELOG.md` entries for the current version — features that look surprising are often announced here

**Do NOT file a bug until you have confirmed it is not documented behavior.**

---

### Lesson 2: Verify menu item selection with ANSI capture
**What happened:** We navigated to what we thought was "Revert" but actually triggered "Reload with Encoding...". We filed a bug about Revert's behavior based on the wrong command's error message.

**Rule:** Whenever testing a menu item:
1. Open the menu and navigate to the target item
2. Run `tmux capture-pane -t SESSION -p -e` (note the `-e` flag for ANSI)
3. Grep for `[48;5;25m` to confirm WHICH item is currently highlighted
4. Only then press Enter

**The plain `-p` capture hides the selection highlight. Always use `-e` for menu verification.**

---

### Lesson 3: Know the key divergences from VS Code before testing
**What happened:** We assumed Fresh uses VS Code keybindings throughout and filed issues when shortcuts behaved differently.

**The known intentional divergences from VS Code:**

| Key | VS Code | Fresh |
|-----|---------|-------|
| `Ctrl+W` | Close tab | **Select word under cursor** |
| `Ctrl+H` | Find & Replace | Intended: Find & Replace; Actual in terminals: Backspace (compatibility issue) |
| `Ctrl+R` | Recent files | **Find & Replace** (reliable) |
| `Ctrl+B` | Toggle sidebar | **Toggle File Explorer** |
| `Ctrl+E` | (various) | Appears to open File Explorer (not confirmed as toggle) |

**Do not file a bug for key differences until checking `docs/configuration/keyboard.md`.**

---

### Lesson 4: tmux send-keys sends multiple keys as literal text
**What happened:** The command `tmux send-keys -t SESSION "S-Left S-Left S-Left" ""` typed the literal text "S-Left S-Left S-Left" into the buffer, corrupting the test file.

**Rule:** ALWAYS send one key per send-keys call:
```bash
# CORRECT
tmux send-keys -t SESSION "S-Left" ""; sleep 0.2
tmux send-keys -t SESSION "S-Left" ""; sleep 0.2
tmux send-keys -t SESSION "S-Left" ""; sleep 0.2

# WRONG — sends literal text
tmux send-keys -t SESSION "S-Left S-Left S-Left" ""
```

If you accidentally corrupt the test file, use `C-z` repeated times or `File > Revert`.

---

### Lesson 5: Hot exit affects every test run — account for it
**What happened:** We opened a file with a clean test, made edits, discarded them, made more edits across multiple tests, then quit. The next launch showed the file as "modified" because hot exit preserved the session state from the final state before quit.

**Rules:**
- When testing "initial launch" behavior, always use `fresh --no-restore` to skip hot exit restoration
- When testing hot exit itself, do it deliberately (see TC-NEW-002/003 in test_plan.md)
- After a test run that made edits, note that the next run will start with restored state

---

### Lesson 6: Reproduce bugs at least twice before filing
**What happened:** All four bugs were filed after single observations. Two turned out to be false positives that a second look would have caught.

**Rule:** Before filing a GitHub issue:
1. Reproduce the behavior at least twice in separate tmux sessions
2. Check the docs (Lesson 1)
3. Verify via ANSI capture where applicable (Lesson 2)
4. Ask: "Could this be a documented feature?" before assuming it's a bug

---

### Lesson 7: Check for existing GitHub issues with broader search terms
**What happened:** We searched with specific phrases like "revert unsaved modifications" but hot exit and Ctrl+H issues might have existing issues under different terms.

**Rule:** Search with at least 3 different query variations before concluding no duplicate exists. Use: feature name, symptom description, key combination involved.

---

## Run #2 — 2026-05-26
### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (needed to rebuild; `cargo build --release`, ~2m23s)
- Note: did NOT read docs first (lesson from Run #1 not followed — no false positives resulted this time)
- Launched tmux session (220×50), re-explored and extended coverage of core features
- Filed 2 new GitHub issues (#2112, #2113) — both verified bugs, no false positives

### Key Technical Discovery
**CRITICAL for tmux automation:** Fresh uses DECCKM (application cursor key mode). Arrow keys MUST be sent as:
- Up: `$'\033OA'`, Down: `$'\033OB'`, Right: `$'\033OC'`, Left: `$'\033OD'`
- Using plain `Up`/`Down` tmux key names sends VT100 sequences (`\033[A`) which are IGNORED.
- Delete key: `$'\033[3~'` (not `DC` tmux key name)

### Test Results Summary
| Feature | Status | Notes |
|---------|--------|-------|
| Launch with --no-restore | PASS | Confirmed hot-exit bypass |
| Arrow key navigation | PASS | **DECCKM mode required** |
| Backspace/Delete | PASS | BSpace works; Delete = `\033[3~` |
| Home/End | PASS | `Home`/`End` tmux keys work |
| Page Up/Down | PASS | `PPage`/`NPage` tmux keys work |
| Text typing | PASS | Characters insert correctly |
| Undo/Redo | PASS | Ctrl+Z / Ctrl+Y multi-step |
| Save (Ctrl+S) | PASS | Status: "Saved"; tab asterisk removed |
| New file (Ctrl+N) | PASS | Creates [No Name] tab |
| Open file (Ctrl+O) | PASS | File browser with Show Hidden / Detect Encoding |
| Close Tab (Alt+W) | PASS | Note: Alt+W = "Close Tab"; different from TC-010 "Close Buffer" |
| Quit (Ctrl+Q) | PASS | Unsaved-changes prompt verified |
| Search (Ctrl+F) | PASS | Case/WholeWord/Regex options; match count in status |
| Go to line (Ctrl+G) | PASS | Prompt stays open after Enter; Escape closes |
| Search/Replace in-project file | PASS | Panel, Tab/Alt+Enter flow, confirm prompt |
| Search/Replace external file | **FAIL** | BUG-005 / #2112 — no matches for /tmp files |
| Command palette (Ctrl+P) | PASS | Mode switching via BSpace |
| Palette fuzzy file finder | PASS | File mode shows project files |
| Palette input leak | **FAIL** | BUG-006 / #2113 — keystrokes can enter editor |
| Terminal integration (Alt+`) | PASS | Utility dock; Ctrl+Space toggles focus |
| Theme selector | PASS | 8 themes; applied successfully |
| Multi-cursor (Ctrl+D) | PASS | 2+ cursors; simultaneous edit; undo works |
| Diagnostics panel | PASS | Opens in dock; "No results" for plain text |

### Issues Filed
| Issue | Title | Verdict |
|-------|-------|---------|
| #2112 | Search/Replace panel: no matches for external files | **Real bug** — reproduced twice |
| #2113 | Command palette: keystroke leak into editor buffer | **Real bug** — timing-sensitive |

### False Positive Rate: 0% (0 of 2)

---

## Run #3 — 2026-05-26
### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (~3 min build)
- Read docs: `docs/features/editing.md`, `docs/features/file-explorer.md`, `docs/features/terminal.md`, `docs/features/search-replace.md`, `docs/configuration/keyboard.md`, `CHANGELOG.md` (0.3.8) — MANDATORY PRE-TEST checklist followed
- Launched tmux session (220×50), executed 20+ test cases
- Filed 0 new GitHub issues (no confirmed bugs beyond existing known issues)

### Key Technical Discovery
- **Tab switching**: `Ctrl+PgDn` / `Ctrl+PgUp` = Next Buffer / Previous Buffer (not Ctrl+Tab)
- **Save As**: accessible via File menu only (not command palette); pre-fills current path
- **File Explorer focus**: `Ctrl+E` switches focus, then DECCKM arrows navigate. Auto-preview tabs appear as you navigate.
- **Close buffer prompt** format: `(s)ave, (d)iscard, (C)ancel?` — requires typing the letter THEN Enter to confirm
- **Find Previous**: palette shows `Ctrl+Shift+N` binding; `Shift+F3` (documented) is NOT recognized in tmux; both have terminal compatibility problems
- **Mouse Support**: can be toggled via View menu (was off by default, inadvertently enabled during test)
- **BUG-006 NOT reproduced**: Two attempts at reproducing palette input leak; no leaks detected. May be fixed or timing-dependent.

### Test Results Summary
| Test | Status | Notes |
|------|--------|-------|
| TC-025: Save As | PASSED | File menu → Save As; pre-fills path |
| TC-027: Close saved file | PASSED | Alt+W closes without dialog |
| TC-028: Multiple tabs | PASSED | Multiple tabs visible in tab bar |
| TC-029: Tab switching | PASSED | Ctrl+PgDn/PgUp switches buffers |
| TC-034: Cut (Ctrl+X) | PASSED | Cuts selected text; Ctrl+V pastes back |
| TC-036: Block selection | PASSED | Alt+Shift+Down/Right creates column block; typing edits all rows |
| TC-037: Comment toggle (Ctrl+/) | PASSED | Works on JS files; not on .txt (no language context) |
| TC-038: Auto-indent | PASSED | Enter after `{` inserts indented new line |
| TC-043: Shift+F3 (prev match) | PARTIAL | Find Previous works via palette; Shift+F3 NOT recognized in tmux |
| TC-048: Case-sensitive (Alt+C) | PASSED | Toggles case-sensitive search on/off |
| TC-049: Regex (Alt+R) | PASSED | Toggles regex mode; actual regex matching confirmed |
| TC-055: File explorer open file | PASSED | Arrow keys preview, Enter opens permanent tab |
| TC-056: Toggle line numbers | PASSED | Via command palette "Toggle Line Numbers" |
| TC-057: Toggle line wrap | PASSED | Via View menu "☑ Line Wrap"; status bar confirms |
| TC-058: Terminal features | PASSED | Ctrl+Space mode toggle; Ctrl+F scrollback search |
| TC-NEW-001: Revert prompt | PASSED | "(r)evert, (C)ancel?" confirmed |
| TC-NEW-005: ⚠ indicator | RESOLVED | Shows LSP diagnostic count; first launch = Test i18n plugin = benign |
| TC-NEW-006: BUG-006 repro | NOT REPRODUCED | 2 attempts, 0 leaks; possibly fixed or intermittent |
| TC-060-065: Command palette | PARTIALLY PASSED | Fuzzy search, theme search, buffer switch all work |

### Issues Filed
None — no new confirmed bugs beyond existing open issues.

### False Positive Rate: 0% (0 of 0)

---

## LESSONS LEARNED — Run #3

### Lesson 8: Close buffer prompt requires letter + Enter, not just the letter
**What happened:** After `(s)ave, (d)iscard, (C)ancel?` prompt appeared, pressing just "d" typed "d" into the prompt. Had to press "d" then Enter to discard.
**Rule:** Close buffer prompt may be context-dependent. When it shows as a bottom-line prompt, it appears to require letter + Enter. Test if the VS Code-like "just press the key" works in future runs.

### Lesson 9: Tab switching is Ctrl+PgDn / Ctrl+PgUp
**What happened:** We tried Ctrl+Tab (sent Tab character to buffer — bug). The correct shortcuts are:
- `Ctrl+PgDn` = Next Buffer (tmux: `C-NPage`)
- `Ctrl+PgUp` = Previous Buffer (tmux: `C-PPage`)
- Alternatively: `Ctrl+P` → `#` for buffer picker

### Lesson 10: File Explorer focus workflow
1. `Ctrl+B` opens/closes the explorer
2. `Ctrl+E` switches focus to the explorer
3. DECCKM arrows (`$'\033O[A-D]'`) navigate; preview tabs open automatically
4. `Enter` promotes preview tab to permanent
5. `Ctrl+E` again switches focus back to editor

### Lesson 11: Save As is not in command palette
- `File > Save As...` is accessible ONLY via the File menu (`Alt+F` or `F10` → navigate)
- No palette command exists for Save As
- Ctrl+Shift+S in terminals: Shift is stripped from Ctrl+S; use the menu

---

## Run #4 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, ~7 min)
- Launched tmux session with fresh --no-restore
- Executed 30+ test cases across command palette, settings UI, edge cases, and advanced features
- No new bugs confirmed this run

### Test Results Summary
| Category | Passed | Failed | Notes |
|----------|--------|--------|-------|
| Command Palette (TC-060–065) | 6 | 0 | Full coverage |
| Settings & Configuration (TC-070–073) | 4 | 0 | Theme, Keybindings, Settings UI |
| Edge Cases (TC-081–085) | 4 | 0 | Binary, empty, rapid keys, resize |
| Advanced: Git Log | 1 | 0 | 55 commits, live diff preview |
| Advanced: Macro recording | 1 | 0 | F5 stop, F4 play, 12-action macro |
| Advanced: Bookmarks | 1 | 0 | Set via palette, Alt+1/2 jump |
| Advanced: Markdown preview | 1 | 0 | ANSI bold/italic confirmed |
| Advanced: Review Diff | 1 | 0 | Panel opens, 0 hunks (correct) |
| Advanced: Live Grep | 1 | 0 | 77 matches, streaming, live preview |
| Advanced: Diagnostics Panel | 1 | 0 | Opens in dock, 0 items |
| Editing: Smart Home | 1 | 0 | Toggles non-ws ↔ col 1 |
| Editing: Position History | 1 | 0 | Alt+Left back across files |
| Editing: Auto-close | 1 | 0 | ( → () cursor inside |
| Editing: Surround selection | 1 | 0 | Select word, [ → [word] |
| Editing: Duplicate Line | 1 | 0 | Via command palette |
| Editing: Ctrl+L Select Line | 1 | 0 | Selects line, advances |
| Misc: path:line:col opening | 1 | 0 | Palette file mode, exact Ln+Col |
| TC-084: 10+ files open | 1 | 0 | 12 tabs, tab switching works |

### Issues Filed
None — 0 new bugs confirmed.

### False Positive Rate: 0%

---

## LESSONS LEARNED — Run #4

### Lesson 12: Settings UI is comprehensive
- `Ctrl+P → "Open Settings"` opens a full visual settings editor
- Left panel: categories (General, Clipboard, Editor, File Browser, File Explorer, Packages, Plugins, Terminal, Warnings, Plugin: dashboard/flash/vi_mode)
- Right panel: visual controls (dropdowns, checkboxes, text inputs)
- Press `/` to search settings by name within the panel
- Keyboard hints at bottom: `↑↓ Navigate Tab Next Enter Edit / Search Esc Close`

### Lesson 13: Keybinding Editor details
- `Ctrl+P → "Open Keybinding Editor"` or `Edit → Keybinding Editor...`
- Shows all 843 bindings (builtin + plugins) grouped by source
- Press `/` to filter by text; press `r` for key-recording search
- Press `c` to cycle context filter; `s` to cycle source filter
- `Enter` to edit, `a` to add, `d` to delete custom bindings
- Config saved to `/root/.config/fresh/config.json` via `Ctrl+S`

### Lesson 14: Macro recording workflow
- `Ctrl+P → "Record Macro"` → prompt `Record macro (0-9):` → type digit → Enter
- Status bar confirms "Recording macro 'N' (F5 or Ctrl+P → Stop Recording)"
- `F5` stops recording; status bar shows "Macro 'N' saved (X actions)"
- `F4` plays the last macro
- Macros survive session (stored in Fresh state dir)

### Lesson 15: Theme selection
- `Ctrl+P → "Select Theme"` shows theme list; current theme marked "(current)"
- Arrow key navigation in list requires DECCKM sequences ($'\033OA'/OB) 
- Theme change takes effect immediately; status bar: "Theme changed to 'X'"
- Theme persisted in /root/.config/fresh/config.json as "theme": "builtin://X"

### Lesson 16: Bookmark workflow  
- `Ctrl+P → "Set Bookmark"` → prompt `Set bookmark (0-9):` → digit → Enter
- `Alt+N` (where N = 0-9) jumps to bookmark N
- Status bar confirms "Bookmark 'N' set" and "Jumped to bookmark 'N'"
- `Ctrl+Shift+N` shortcut for Set Bookmark may not work in tmux — use palette

### Lesson 17: Binary file handling
- Binary files open with `[BIN]` in the tab label
- Content displayed as hex escapes: `<FF><FE>...`
- File is automatically marked `[RO]` (read-only) in the status bar
- No crash, no corruption — safe and informative

### Lesson 18: Welcome plugin overrides Ctrl+S
- In Keybinding Editor, Ctrl+S shows as "Plugin Demo: Save File" (source: welcome plugin, keymap)
- The action bound is still the built-in `save` action, so Ctrl+S still saves correctly
- This is cosmetic — the description says "Plugin Demo" but functionality is unchanged
- NOT a bug — the welcome plugin uses the built-in save action for demonstration

---

## Run #5 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, ~8 min)
- Executed 15+ test cases across large files, syntax highlighting, code folding, git features, themes, encoding, whitespace indicators, new 0.3.8 features
- Filed 1 new GitHub issue (#2117 — Review Diff discard hunk patch failure)

### Test Results Summary
| Category | Passed | Failed | Partial | Notes |
|----------|--------|--------|---------|-------|
| TC-080: Large file (159MB) | 1 | 0 | 0 | Byte offsets shown, Scan Line Index works, line nav to 1,000,000 |
| Multi-language syntax highlighting | 3 | 0 | 0 | Rust, Python, JS all confirmed with ANSI color codes |
| Code Folding | 1 | 0 | 0 | Toggle Fold works, gutter ▸/▾, nav skips folded regions |
| Git Blame | 1 | 0 | 0 | Opens *blame:tab*, shows commit blocks, q to close |
| Review Diff (basic) | 1 | 0 | 0 | Opens diff view with +/- hunks, n/p navigation confirmed |
| Review Diff (discard) | 0 | 1 | 0 | **BUG #2117**: d→Enter fails "Patch failed" consistently |
| Whitespace Indicators | 1 | 0 | 0 | Trailing spaces show as ··· (dot indicators) |
| File Encoding (Latin-1) | 1 | 0 | 0 | Windows-1252 auto-detected, éàñ decoded correctly |
| Theme Editor | 0 | 0 | 1 | Opens, navigation works, color swatch display confirmed; editing complex via keyboard |
| Move File Explorer to Other Side | 1 | 0 | 0 | NEW 0.3.8 feature works: explorer moves left↔right |
| Live Diff | 1 | 0 | 0 | vs HEAD shows green +lines; status bar confirms mode |
| Vertical Rulers | 1 | 0 | 0 | Add Ruler at col 80 shows [48;5;236m] tinted column |
| Orchestrator (Alt+Q) | 1 | 0 | 0 | Opens session selector; fresh BASE visible in left panel |
| Workspace Trust | 1 | 0 | 0 | NEW 0.3.8: dialog shows ⚠ warning, T/K options, auto-detects .envrc |

### Issues Filed
| Issue | Final Status | Verdict |
|-------|-------------|---------|
| #2117 Review Diff: discard hunk "patch does not apply" | **Open** | Confirmed bug — reproduced 3 times consistently |

### False Positive Rate: 0%

---

## LESSONS LEARNED — Run #5

### Lesson 19: Large file mode byte offsets
- Files larger than a threshold open with byte offsets in the gutter (0, 111, 222...)
- Status bar shows "Byte 0" instead of "Ln N, Col N"
- Run "Scan Line Index" from command palette to build line index first
- Status bar confirms "Line index built successfully"
- After scan: `:1000000` in palette → Enter navigates to line 1,000,000

### Lesson 20: Code folding workflow
- Fold indicators appear in gutter when file has tree-sitter grammar (Rust, Python, JS)
- `▾` = expanded (can fold), `▸` = collapsed (can unfold)
- Folded blocks show `...` preview on fold line
- Navigation (Down arrow) skips folded lines (e.g. line 1 folded→ Down goes to line 7)
- Command: `Ctrl+P → "Toggle Fold"` while cursor is on the block opener

### Lesson 21: Git Blame workflow
- `Ctrl+P → "Git Blame"` opens *blame:filename* tab (read-only)
- Shows commit blocks as separators: `── be6b4d1 (Author, time) "commit msg" ──`
- Status bar: "Git blame: N blocks | b: blame at parent | q: close"
- Close with `q`

### Lesson 22: Review Diff discard requires a picker dialog
- Pressing `d` on a hunk row triggers a picker: "Discard hunk" (Permanently lose) vs "Cancel"
- Default selection is "Discard hunk"; press Enter to confirm
- **BUG**: The discard fails with "Patch failed: error: patch failed: README.md:275error: README.md: patch does not apply" even on valid patches
- The raw `git diff HEAD -- file | git apply --reverse` command works fine from shell

### Lesson 23: Workspace Trust dialog (0.3.8)
- `Ctrl+P → "Workspace Trust…"` opens a security warning overlay in the editor
- Shows project path, detected config files (.envrc, Cargo.toml, package.json)
- Options: `[T]rust folder & Allow Tooling` and `[K]eep Restricted (Default)`
- Close with Escape (does not change trust level)
- The enforcement gate is OFF by default in 0.3.8 (untrusted = treated as trusted)

### Lesson 24: Move File Explorer to Other Side (0.3.8)
- `Ctrl+P → "Move File Explorer to Other Side"` switches explorer between left and right
- Takes effect immediately; persisted to config
- Confirm: Explorer appears on RIGHT side after command

### Lesson 25: Review Diff keyboard hint clarification
- The bottom bar shows: `[n] next hunk  [p] prev hunk ... [d] discard ... [S U D] file-level`
- `[S U D]` uppercase = FILE-level stage/unstage/discard
- `[d]` lowercase = HUNK-level discard → opens confirmation picker
- Both `d` and `D` result in the "Patch failed" bug (BUG #2117)

### Lesson 26: Settings UI Trailing Spaces
- `Ctrl+P → "Open Settings"` → press `/` to search → type "whitespace" to find all whitespace settings
- "Trailing Spaces: [ ]" is the key setting to enable trailing space dots
- After enabling and saving: files show `···` (dim dots) after trailing spaces
- Config key: `editor.whitespace_trailing_spaces` (or whitespace_show as master toggle)

### Lesson 27: Discard confirmation selection via ANSI
- Review Diff's discard confirmation is a 2-item picker overlay at the bottom of screen
- "Discard hunk" is highlighted first by default (`[48;5;25m]` dark blue)
- Pressing Enter confirms the highlighted item
- The picker appears AFTER pressing `d` (lowercase) — looking for `d` + Enter is WRONG, need `d` then check ANSI then Enter

### Lesson 28: Palette search returns ALL related commands
- When searching "Git Blame", results also include sub-commands: "Git Blame: Close", "Git Blame: Go Back"
- The main "Git Blame" command is above the visible area by default
- Navigate UP (multiple times) to reveal and select the main command
- Always check ANSI capture (`[48;5;25m]` = highlighted) before pressing Enter
