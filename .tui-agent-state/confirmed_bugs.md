# Confirmed Bugs Registry

---

## Open Bugs

### BUG-002 - Ctrl+H Does Not Open Find & Replace in Terminals
- **Date:** 2026-05-26
- **Severity:** Medium (UX friction for VS Code/Sublime users)
- **Root Cause:** Terminal compatibility — `Ctrl+H` is transmitted as ASCII `0x08` (Backspace) in most terminals including tmux. Fresh receives a backspace, not Ctrl+H.
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2109 (updated)
- **Workaround:** Use `Ctrl+R` for Replace (reliable in all terminals).
- **Notes:** Fresh's "Calibrate Keyboard" wizard exists for exactly this class of issue. The `Ctrl+H` → Backspace collision is particularly harmful because it silently deletes text.

---

### BUG-004 - F3 Does Not Navigate While Search Bar Is Open
- **Date:** 2026-05-26
- **Severity:** Medium (UX friction — contradicts VS Code/Sublime/browser behavior)
- **Description:** Pressing F3 while the search bar is open has no effect. The current workflow requires pressing Enter first (which closes the bar), then F3 to navigate. In VS Code, Sublime Text, and all major browsers, F3 navigates to the next match while the search bar remains visible. This is the dominant paradigm Fresh users will expect.
- **Expected:** F3 advances to next match without closing the search bar
- **Actual:** F3 is silently ignored while search bar is open; must press Enter to close first
- **Workaround:** Enter → closes bar, then F3 works (but this is non-obvious and nothing in the UI explains it)
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2111 (updated with clear expected vs actual)
- **Status:** Confirmed usability bug; no further verification needed

---

## Closed / Retracted

### ~~BUG-001~~ — FALSE POSITIVE (closed #2108)
- We accidentally triggered "Reload with Encoding..." instead of "Revert" via imprecise menu navigation.
- `File > Revert` works correctly: it shows a `(r)evert / (c)ancel` confirmation prompt when the buffer is modified.
- **Lesson:** Always verify menu selection with ANSI capture before asserting behavior.

### ~~BUG-003~~ — BY DESIGN (closed #2110)
- File opening as modified is the **hot exit** feature: Fresh preserves all unsaved buffer state on quit and restores it on the next startup (`hot_exit` config, default: on).
- **Lesson:** Read the docs and CHANGELOG before filing issues. Hot exit is documented in `docs/features/session-persistence.md` and announced in the 0.2.18 release.

---

## Resolved Bugs

*(None yet)*
