//! E2E tests reproducing issue #1537:
//! "go to matching bracket messes up find next/previous"
//!
//! <https://github.com/sinelaw/fresh/issues/1537>
//!
//! Reporter's flow:
//!   1. Ctrl+F, search for a term with multiple matches. Find next/previous
//!      behaves correctly.
//!   2. Place the cursor on a bracket and invoke Go to Matching Bracket
//!      (Ctrl+]).
//!   3. Invoke find next / find previous. The cursor lands on the bracket
//!      instead of on the next / previous search match:
//!
//!       > once you do a go to matching bracket, it seems to insert the
//!       > matching bracket into the find next /. previous.
//!
//! Each test exercises the plain `find_next` / `find_previous` actions
//! (F3 / Shift+F3 — NOT the `find_selection_*` "quick find" actions bound
//! to Ctrl+F3 / Alt+N / Alt+P, which are a different feature).

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::TempDir;

/// Test file with three NEEDLE matches and a pair of matching braces.
///
///   line 0: "line 0 has a NEEDLE here\n"
///   line 1: "line 1 filler text for padding\n"
///   line 2: "line 2 brace { start\n"              ← '{'
///   line 3: "line 3 content inside\n"
///   line 4: "line 4 NEEDLE inside the braces\n"
///   line 5: "line 5 continues here\n"
///   line 6: "line 6 brace } end\n"                ← '}'
///   line 7: "line 7 NEEDLE after the braces\n"
///   line 8: "line 8 more filler text\n"
///   line 9: "line 9 last line\n"
const CONTENT: &str = "\
line 0 has a NEEDLE here
line 1 filler text for padding
line 2 brace { start
line 3 content inside
line 4 NEEDLE inside the braces
line 5 continues here
line 6 brace } end
line 7 NEEDLE after the braces
line 8 more filler text
line 9 last line
";

fn setup() -> (EditorTestHarness, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, CONTENT).unwrap();

    let mut harness = EditorTestHarness::new(140, 30).unwrap();
    harness.open_file(&file_path).unwrap();
    harness.render().unwrap();
    (harness, temp_dir)
}

/// Byte offsets of the three NEEDLE matches.
fn match_positions() -> (usize, usize, usize) {
    let m1 = CONTENT.find("NEEDLE").unwrap();
    let m2 = CONTENT[m1 + 1..].find("NEEDLE").unwrap() + m1 + 1;
    let m3 = CONTENT[m2 + 1..].find("NEEDLE").unwrap() + m2 + 1;
    (m1, m2, m3)
}

fn open_bracket_pos() -> usize {
    CONTENT.find('{').unwrap()
}

fn close_bracket_pos() -> usize {
    CONTENT.find('}').unwrap()
}

/// Perform Ctrl+F "NEEDLE" Enter and verify the cursor landed on the first
/// match.
fn start_search(harness: &mut EditorTestHarness, m1: usize) {
    harness
        .send_key(KeyCode::Char('f'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("NEEDLE").unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.process_async_and_render().unwrap();

    assert_eq!(
        harness.cursor_position(),
        m1,
        "Initial search should land on the first NEEDLE match"
    );
    harness.assert_screen_contains("Found 3 matches");
}

/// Move the cursor to an absolute byte offset.
fn move_cursor_to(harness: &mut EditorTestHarness, pos: usize) {
    harness
        .editor_mut()
        .active_cursors_mut()
        .primary_mut()
        .position = pos;
    harness.render().unwrap();
    assert_eq!(harness.cursor_position(), pos);
}

fn assert_not_on_bracket(harness: &EditorTestHarness, action: &str) {
    let pos = harness.cursor_position();
    let byte = CONTENT.as_bytes()[pos] as char;
    assert!(
        !"(){}[]<>".contains(byte),
        "Cursor after {} must not be on a bracket character. Got {:?} at pos {}",
        action,
        byte,
        pos
    );
}

/// Issue #1537 — F3 (find_next) after Go to Matching Bracket must land on
/// the next NEEDLE match, not on the bracket.
#[test]
fn test_find_next_after_goto_matching_bracket_lands_on_needle() {
    let (mut harness, _tmp) = setup();
    let (m1, _m2, m3) = match_positions();
    let open_pos = open_bracket_pos();
    let close_pos = close_bracket_pos();

    start_search(&mut harness, m1);

    // Place cursor on '{' and jump to its matching '}'.
    move_cursor_to(&mut harness, open_pos);
    harness
        .send_key(KeyCode::Char(']'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert_eq!(
        harness.cursor_position(),
        close_pos,
        "Go to Matching Bracket should move cursor to the matching '}}'"
    );

    // F3 — find_next (see default keymap).
    harness.send_key(KeyCode::F(3), KeyModifiers::NONE).unwrap();
    harness.process_async_and_render().unwrap();

    assert_not_on_bracket(&harness, "F3 after Go to Matching Bracket");
    assert_eq!(
        harness.cursor_position(),
        m3,
        "After F3 following Go to Matching Bracket, cursor should be on \
         the next NEEDLE match (line 7 at pos {}), but was at pos {}",
        m3,
        harness.cursor_position()
    );
}

/// Issue #1537 — Shift+F3 (find_previous) after Go to Matching Bracket
/// must land on the previous NEEDLE match, not on the bracket.
#[test]
fn test_find_previous_after_goto_matching_bracket_lands_on_needle() {
    let (mut harness, _tmp) = setup();
    let (m1, _m2, _m3) = match_positions();
    let open_pos = open_bracket_pos();
    let close_pos = close_bracket_pos();

    start_search(&mut harness, m1);

    // Place cursor on '}' and jump to its matching '{'.
    move_cursor_to(&mut harness, close_pos);
    harness
        .send_key(KeyCode::Char(']'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert_eq!(
        harness.cursor_position(),
        open_pos,
        "Go to Matching Bracket should move cursor to the matching '{{'"
    );

    // Shift+F3 — find_previous (see default keymap).
    harness
        .send_key(KeyCode::F(3), KeyModifiers::SHIFT)
        .unwrap();
    harness.process_async_and_render().unwrap();

    assert_not_on_bracket(&harness, "Shift+F3 after Go to Matching Bracket");
    assert_eq!(
        harness.cursor_position(),
        m1,
        "After Shift+F3 following Go to Matching Bracket, cursor should be \
         on the previous NEEDLE match (line 0 at pos {}), but was at pos {}",
        m1,
        harness.cursor_position()
    );
}
