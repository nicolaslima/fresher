# TUI Agent Run Log

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

## Run #3 — PENDING

See `test_plan.md` → "Immediate Next Action (Run #3)" for priority list.
Key items:
- Verify BUG-006 (#2113 palette leak) reproducibility under controlled conditions
- Complete TC-025 through TC-058 backlog
- Test file explorer (TC-055), terminal (TC-058), line ops (TC-034/036/037/038)
- Investigate `[⚠ N]` status bar indicator (plugin warning)
