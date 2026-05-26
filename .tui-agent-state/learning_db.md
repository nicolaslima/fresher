# Fresh TUI Editor — Knowledge Base

## Application Overview
- **Name:** Fresh — a modern terminal text editor
- **Binary:** `/opt/node22/bin/fresh` (installed via npm @fresh-editor/fresh-editor)
- **Version:** 0.3.9
- **Launch:** `fresh [file]` — opens with optional file argument
- **Session Persistence:** Fresh restores all previously open buffers on next launch (built-in session memory)

## tmux Session Setup
- Session: `fresh-tui-test`
- Size: 220 columns × 50 rows
- Create: `tmux new-session -d -s fresh-tui-test -x 220 -y 50`
- Kill: `tmux kill-session -t fresh-tui-test`
- Send keys: `tmux send-keys -t fresh-tui-test 'text' Enter`
- Read output: `tmux capture-pane -t fresh-tui-test -p`
- Read with colors: `tmux capture-pane -t fresh-tui-test -p -e`

## Critical tmux send-keys Notes
- Spaces in `send-keys` arguments are interpreted as KEY SEPARATORS
  - BAD: `tmux send-keys -t s "Split H"` → sends "Split" (invalid key) and "H" (capital H)
  - GOOD: `tmux send-keys -t s 'S' 'p' 'l' 'i' 't' ' ' 'H'` → sends each char individually
- Special key names: Enter, Escape, BSpace (backspace), Space, Up/Down/Left/Right, Home/End
- Ctrl keys: `C-a`, `C-p`, `C-z`, etc.
- Alt keys: `M-w`, `M-a`, etc.  
- Shift+arrow: `S-Down`, `S-Up`, `S-Left`, `S-Right`
- F-keys: `F1`, `F3`, `F10` — but F10 reliability is timing-dependent (can insert `[21~]` escape sequence)

## Menu Bar Navigation
- **F10** — opens File menu (first menu)
- **Right/Left** — navigate between menus while one is open
- **Down/Up** — navigate within a menu (WARNING: Down from File menu switches to Edit; can confuse)
- **Escape** — close menu
- **Enter** — execute menu item
- Menu order: File | Edit | View | Selection | Go | LSP | Help
- Menu navigation with 3+ separate Right presses correctly navigates: File→Edit→View→Selection

## Key Bindings (Confirmed)
### File Operations
- **Ctrl+N** — New File (new empty buffer tab)
- **Ctrl+O** — Open File (file picker with browser + sizes/timestamps)
- **Ctrl+S** — Save (prompts Save As for new buffers)
- **Ctrl+Q** — Quit (no prompt for unsaved new buffers)
- **Alt+W** — Close Tab (but ALSO toggles whole-word search depending on context!)
- **Close Buffer** (command palette) — prompts save/discard for modified buffers

### Save/Discard Dialog
- Prompt: `'[filename]' modified. (s)ave, (d)iscard, (C)ancel? `
- Input: Type letter + Enter (NOT single keypress)
- 's' + Enter → save
- 'd' + Enter → discard
- 'C' + Enter or Escape → cancel

### Editing
- **Ctrl+Z** — Undo (single character/action granularity)
- **Ctrl+Y** — Redo
- **Ctrl+X** — Cut
- **Ctrl+C** — Copy (shows "Copied" status)
- **Ctrl+V** — Paste
- **Ctrl+A** — Select All
- **Ctrl+W** — Select Word (at cursor)
- **Ctrl+L** — Select Line
- **Ctrl+D** — Add cursor at next match (multi-cursor)
- **Ctrl+/** — Toggle Comment (works on code files; no effect on plain text)
- **Ctrl+Alt+↑/↓** — Add cursor above/below
- **Alt+Shift+I** — Add cursors to line ends

### Navigation
- **Ctrl+G** — Go to Line (prompt at bottom: type number + Enter)
- **Ctrl+Right/Left** — Word movement
- **Shift+Down/Up/Left/Right** — Selection extend
- **Ctrl+PgDn / Ctrl+NPage** — Next Buffer/Tab
- **Ctrl+PgUp / Ctrl+PPage** — Previous Buffer/Tab
- **Home/End** — Beginning/End of line
- **F3** — Find Next (uses last search)
- **Shift+F3** — Find Previous

### Search & Replace
- **Ctrl+F** — Find (bar at bottom with options)
  - **Alt+C** — Toggle Case Sensitive
  - **Alt+W** — Toggle Whole Word  
  - **Alt+R** — Toggle Regex
  - **Enter** — Jump to current match and CLOSE search bar
  - **F3** — Next match WITHOUT closing search bar
- **Ctrl+Alt+R** — Query Replace
  - First field: search term (Enter to advance)
  - Second field: replacement (Enter to start)
  - Per-match: y=yes, n=skip, a=all remaining, c=cancel
- **Alt+A** — Search and Replace in Project (cross-file)

### Views
- **Ctrl+B** — Toggle File Explorer (sidebar with directory tree)
- **F10** — Access menu bar
- **Ctrl+`** / **Alt+`** — Open Terminal in Utility Dock

### Command Palette
- **Ctrl+P** — Open Command Palette
  - Default mode: `>command` (command search)
  - Delete `>` → `file` mode (fuzzy file search)
  - `:` → line mode
  - `#` → buffer mode
  - Type characters with spaces: use single-char `send-keys` format

### Terminal
- **Alt+`** — Open Terminal in Utility Dock
- **Ctrl+Space** — Toggle terminal input mode (enter/exit interactive terminal)
- When terminal mode active: editor keyboard shortcuts are captured by terminal
- Close terminal split via "Close Split" command

## UI Layout
- Menu bar: top row (File/Edit/View/Selection/Go/LSP/[Explorer]/Help)
- Tab bar: second row (shows all open buffers, * = modified)
- Editor area: main area with line numbers + content
- Status bar: bottom-1 (filename, cursor pos, status message, encoding, filetype, hint)
- Dialogs/prompts: appear at very bottom (BELOW status bar) — easy to miss!

## Status Bar Format
`Local | [filename] [+] | Ln N, Col N | [message] | encoding | filetype | hint`
- `[+]` = unsaved changes (same as `*` in tab)
- Encoding: UTF-8, ASCII, LF
- Filetype: Text, TOML, Rust, etc.
- Hint: "Palette: Ctrl+P" = normal mode; "Terminal mode..." = in terminal

## Important Behaviors

### Session Persistence
Fresh remembers all open buffers across sessions. Each launch restores the previous state. This means test buffers accumulate. Explicitly close/discard them before each run.

### Multi-cursor
1. Position cursor at first word
2. Ctrl+W to select word
3. Ctrl+D to add cursor at NEXT occurrence
4. Repeat Ctrl+D for more cursors
5. Type to edit all cursors simultaneously

### Save/Discard Dialog
The save prompt appears at the VERY BOTTOM of the screen, below the status bar. Typing after the dialog opens inputs into the dialog's text field (not the editor). Always check for this dialog before typing.

### Alt+W Quirk
Alt+W behaves differently depending on context:
- In search bar: toggles Whole Word search option
- On a tab (when not in search): closes the tab ("Close Tab")
- Outside search, while in editing: may toggle whole-word setting (BUG CANDIDATE)
Safer alternative: Use "Close Buffer" or "Close Tab" commands via Ctrl+P.

### F10 Reliability
F10 can sometimes insert escape sequence `[21~]` into the active buffer instead of opening the menu. This is timing-dependent with tmux. If this happens, undo with Ctrl+Z. Prefer using command palette (Ctrl+P) for menu actions.

## Run History
- Run 1 (2026-05-26): First run, built binary, tested ~35 test cases across all sprints
  - Sprints 1-9 largely completed
  - One bug candidate identified (Alt+W inconsistency)
  - Session cleanup: fresh exited cleanly via Ctrl+Q
