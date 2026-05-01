//! E2E tests for the Number control's click and key handling in settings.
//!
//! Regression test for #1825:
//! - Clicking on the value area `[ N ]` of a Number control should enter
//!   number-editing mode (currently it only selects the item, leaving the
//!   user with no obvious way to type a value).
//! - In number-editing mode, pressing Tab should exit edit mode (commit the
//!   value), matching the behavior of Enter. Today Tab is unhandled and the
//!   user appears stuck in the input box.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

/// Locate the on-screen 0-indexed column of the first occurrence of `target`
/// on the row that contains `marker_text`. Returns (col, row).
fn find_char_in_row(
    harness: &EditorTestHarness,
    marker_text: &str,
    target: char,
) -> Option<(u16, u16)> {
    let buf = harness.buffer();
    for row in 0..buf.area.height {
        let line = harness.screen_row_text(row);
        if !line.contains(marker_text) {
            continue;
        }
        let after_marker = line.find(marker_text)?;
        let chars: Vec<char> = line.chars().collect();
        let start = after_marker + marker_text.chars().count();
        for (i, c) in chars.iter().enumerate().skip(start) {
            if *c == target {
                return Some((i as u16, row));
            }
        }
    }
    None
}

/// Open settings and bring the "Tab Size" Number control into view by
/// search+Enter then mouse-scrolling until the row appears.
fn navigate_to_tab_size(harness: &mut EditorTestHarness) {
    harness.open_settings().unwrap();
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    harness.type_text("Tab Size").unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    for _ in 0..30 {
        if harness
            .screen_to_string()
            .lines()
            .any(|l| l.contains("Tab Size") && l.contains('['))
        {
            return;
        }
        harness.mouse_scroll_down(60, 15).unwrap();
        harness.render().unwrap();
    }
    panic!(
        "could not bring Tab Size into view. Screen:\n{}",
        harness.screen_to_string()
    );
}

/// Read the Tab Size value from the "Tab Size ... [ N ] [-] [+]" row.
fn extract_tab_size_value(screen: &str) -> Option<i64> {
    for line in screen.lines() {
        if !line.contains("Tab Size") {
            continue;
        }
        // First "[ N ]" after "Tab Size" — careful, the [-] / [+] buttons
        // also use brackets, so look for a bracket whose interior parses as
        // an integer.
        let after = line.split("Tab Size").nth(1)?;
        for chunk in after.split('[') {
            let inside = chunk.split(']').next()?;
            let trimmed = inside.trim();
            if let Ok(n) = trimmed.parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

/// Clicking the value `[ N ]` of a Number control should put the control
/// into number-editing mode. The cursor styling on the digit shows up as a
/// reversed-bg cell on the focused digit (we don't depend on its exact form;
/// rather we observe that subsequent typing replaces the value).
#[test]
fn test_clicking_number_value_enters_edit_mode() {
    let mut harness = EditorTestHarness::new(110, 30).unwrap();
    navigate_to_tab_size(&mut harness);

    // Find a digit inside `[ N ]` on the Tab Size row. The default value
    // contains '4', and there are no other digits between "Tab Size" and the
    // value brackets.
    let (digit_col, digit_row) = find_char_in_row(&harness, "Tab Size", '4')
        .expect("Tab Size row should contain the default value digit '4'");

    // Click the value cell.
    harness.mouse_click(digit_col, digit_row).unwrap();
    harness.render().unwrap();

    // Type "12" — if we're in number-editing mode the click selects the value
    // and the digits replace it. If we're NOT in edit mode, the digits do
    // nothing and the value remains 4.
    harness.type_text("12").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let after = harness.screen_to_string();
    let value_after =
        extract_tab_size_value(&after).expect("could not read Tab Size value after edit");
    assert_eq!(
        value_after, 12,
        "Clicking the Number value area should enter edit mode so typing '12' \
         replaces the value. Got {} instead.\nScreen:\n{}",
        value_after, after
    );
}

/// While editing a Number control, pressing Tab should commit the value and
/// exit edit mode (just like Enter). Today Tab is unhandled and the user
/// stays in edit mode indefinitely.
#[test]
fn test_tab_exits_number_editing_mode() {
    let mut harness = EditorTestHarness::new(110, 30).unwrap();
    navigate_to_tab_size(&mut harness);

    // Enter edit mode via Enter on the focused Tab Size row, then type.
    // navigate_to_tab_size already left the focus on Tab Size after the
    // search jump.
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("7").unwrap();
    harness.render().unwrap();

    // Now press Tab and then type a printable digit. If Tab exited edit
    // mode, the digit shouldn't be appended to the value (it would either
    // be ignored or interpreted as a navigation key).
    harness.send_key(KeyCode::Tab, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness.type_text("9").unwrap();
    harness.render().unwrap();

    let after = harness.screen_to_string();
    let value_after = extract_tab_size_value(&after).expect("could not read Tab Size after Tab");

    assert_ne!(
        value_after, 79,
        "Pressing Tab should exit number-editing mode. The subsequent '9' \
         keypress should NOT have been appended to the value (got 79).\nScreen:\n{}",
        after
    );
    // Value should be the pre-Tab value (7) since Tab commits.
    assert_eq!(
        value_after, 7,
        "After typing '7' and pressing Tab, the value should be committed as 7. \
         Got {} instead.\nScreen:\n{}",
        value_after, after
    );
}
