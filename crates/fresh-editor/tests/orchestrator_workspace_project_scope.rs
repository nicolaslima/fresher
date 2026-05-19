//! Regression for the user-reported "tabs from yesterday's other
//! projects appear in today's editor window" symptom.
//!
//! Invariant under test (the target the fix enforces):
//!
//!   > A window must recover with exactly the same tabs, splits,
//!   > buffers, and files as were open in that window the last time it
//!   > quit. Each project is a separate window — they never mix into
//!   > a single window.
//!
//! Concretely: a workspace file (`<data>/workspaces/<encoded_cwd>.json`)
//! is permitted today to record absolute file paths outside its
//! `working_dir` — `external_files: Vec<PathBuf>` plus absolute paths
//! in `split_states[…].open_tabs`. On restore those entries are
//! mounted as live tabs in the active window, mixing foreign-project
//! files into the current project's tab bar.
//!
//! After the fix, foreign-project entries are dropped at restore (so
//! a stale on-disk file can't re-introduce the bleed) and never
//! written at save (so the persisted state stays project-scoped).
//!
//! Reproducer matches what was demonstrated manually in tmux, scoped
//! to the workspace layer.

mod common;

use fresh::app::Editor;
use fresh::config::Config;
use fresh::config_io::DirectoryContext;
use fresh::model::filesystem::StdFileSystem;
use fresh::view::color_support::ColorCapability;
use serde_json::json;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

struct HermeticEnv {
    prev_xdg: Option<OsString>,
    prev_home: Option<OsString>,
}

impl HermeticEnv {
    fn new(root: &Path) -> Self {
        let prev_xdg = std::env::var_os("XDG_DATA_HOME");
        let prev_home = std::env::var_os("HOME");
        std::env::set_var("XDG_DATA_HOME", root);
        std::env::set_var("HOME", root);
        Self {
            prev_xdg,
            prev_home,
        }
    }
}

impl Drop for HermeticEnv {
    fn drop(&mut self) {
        match &self.prev_xdg {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match &self.prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}

fn resolved_data_dir() -> PathBuf {
    dirs::data_dir().expect("data dir resolvable").join("fresh")
}

/// Filename `workspace.rs::encode_path_for_filename` produces for an
/// absolute path: `/` → `_`, literal `_` → `%5F`, alphanumerics +
/// `-`/`.` pass through, everything else percent-encoded. We can't
/// call the function directly from an integration test crate without
/// pulling in the whole world, and the format is documented, so we
/// inline the subset we need.
fn encode_path_for_filename(path: &Path) -> String {
    let s = path.to_string_lossy();
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '/' | '\\' => out.push('_'),
            c if c.is_ascii_alphanumeric() => out.push(c),
            '-' | '.' => out.push(c),
            '_' => out.push_str("%5F"),
            c => {
                for b in c.to_string().as_bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

/// Seed a workspace file at `<data>/workspaces/<encoded_cwd>.json`
/// whose `open_tabs` list mixes files from `project_a` (the window's
/// own cwd) and `project_b` (a foreign project). This is the on-disk
/// artifact a buggy persistence path leaves behind.
fn seed_mixed_workspace(data_dir: &Path, project_a: &Path, project_b: &Path) -> PathBuf {
    let ws_dir = data_dir.join("workspaces");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let canon_a = project_a
        .canonicalize()
        .unwrap_or_else(|_| project_a.to_path_buf());
    let encoded = encode_path_for_filename(&canon_a);
    // The production encoder drops the leading underscore from the
    // root path component on Linux when working_dir starts with `/`:
    // observed empirically by saving a real workspace and inspecting
    // the resulting filename. The integration test we're regressing
    // here looks at *both* candidate names; the editor itself will
    // write whichever one matches.
    let path_underscore = ws_dir.join(format!("{encoded}.json"));
    let path_no_underscore = ws_dir.join(format!("{}.json", encoded.trim_start_matches('_')));

    let a_cfg = project_a.join("config_a.toml");
    let a_readme = project_a.join("README_A.md");
    let b_leaked = project_b.join("leaked_yesterday.txt");
    let b_only = project_b.join("only_in_b.md");
    for p in [&a_cfg, &a_readme, &b_leaked, &b_only] {
        if !p.exists() {
            std::fs::write(p, b"x").unwrap();
        }
    }

    let ws = json!({
        "version": 1,
        "working_dir": canon_a,
        "split_layout": { "Leaf": { "file_path": "config_a.toml", "split_id": 0 } },
        "active_split_id": 0,
        "split_states": {
            "0": {
                "open_tabs": [
                    { "File": "config_a.toml" },
                    { "File": "README_A.md" },
                    { "File": b_leaked },
                    { "File": b_only }
                ],
                "active_tab_index": 0,
                "open_files": ["config_a.toml", "README_A.md"],
                "active_file_index": 0,
                "file_states": {},
                "tab_scroll_offset": 0,
                "view_mode": "Source",
                "compose_width": null
            }
        },
        "file_explorer": {
            "visible": false, "width": "30%", "side": "left",
            "expanded_dirs": [], "scroll_offset": 0,
            "show_hidden": false, "show_gitignored": false
        },
        "histories": {},
        "search_options": { "case_sensitive": true, "whole_word": false, "use_regex": false, "confirm_each": false },
        "bookmarks": {},
        "terminals": [],
        "external_files": [b_leaked, b_only],
        "saved_at": 0
    });
    // Write both candidate filenames; the editor reads only the one
    // it expects, the other is harmless. (Linux production wrote the
    // no-underscore form when we observed it empirically.)
    std::fs::write(&path_underscore, serde_json::to_vec_pretty(&ws).unwrap()).unwrap();
    std::fs::write(&path_no_underscore, serde_json::to_vec_pretty(&ws).unwrap()).unwrap();
    path_underscore
}

fn make_editor(cwd: &Path, data_dir: &Path) -> Editor {
    let dir_context = DirectoryContext {
        data_dir: data_dir.to_path_buf(),
        config_dir: data_dir.join("config"),
        home_dir: Some(data_dir.parent().unwrap().to_path_buf()),
        documents_dir: None,
        downloads_dir: None,
    };
    let filesystem: Arc<dyn fresh::model::filesystem::FileSystem + Send + Sync> =
        Arc::new(StdFileSystem);
    let config = Config {
        check_for_updates: false,
        ..Config::default()
    };
    Editor::for_test(
        config,
        80,
        24,
        Some(cwd.to_path_buf()),
        dir_context,
        ColorCapability::TrueColor,
        filesystem,
        None,
        None,
        false,
        false,
    )
    .expect("editor builds")
}

#[test]
fn workspace_restore_drops_foreign_project_tabs() {
    let root = TempDir::new().unwrap();
    let _env = HermeticEnv::new(root.path());

    let project_a = root.path().join("proj_a");
    let project_b = root.path().join("proj_b");
    std::fs::create_dir_all(&project_a).unwrap();
    std::fs::create_dir_all(&project_b).unwrap();
    let data_dir = resolved_data_dir();
    std::fs::create_dir_all(&data_dir).unwrap();

    seed_mixed_workspace(&data_dir, &project_a, &project_b);

    let mut editor = make_editor(&project_a, &data_dir);
    let _ = editor.try_restore_workspace();

    let active_paths: Vec<String> = editor
        .all_buffer_ids_for_tests()
        .into_iter()
        .map(|id| editor.get_buffer_display_name(id))
        .collect();

    let canon_b = project_b.canonicalize().unwrap_or(project_b.clone());
    let b_prefix = canon_b.to_string_lossy().to_string();
    let leaks: Vec<&String> = active_paths
        .iter()
        .filter(|p| {
            p.contains(&b_prefix) || p.contains("leaked_yesterday") || p.contains("only_in_b")
        })
        .collect();

    assert!(
        leaks.is_empty(),
        "project A launch restored foreign-project tabs from the workspace file.\n\
         active-window buffer_paths = {active_paths:?}\n\
         offending entries = {leaks:?}\n\
         Invariant: a window's restored state contains only files inside its working_dir."
    );
}

#[test]
fn workspace_save_does_not_record_foreign_project_tabs() {
    let root = TempDir::new().unwrap();
    let _env = HermeticEnv::new(root.path());

    let project_a = root.path().join("proj_a");
    let project_b = root.path().join("proj_b");
    std::fs::create_dir_all(&project_a).unwrap();
    std::fs::create_dir_all(&project_b).unwrap();
    let a_cfg = project_a.join("config_a.toml");
    let b_leaked = project_b.join("leaked_yesterday.txt");
    std::fs::write(&a_cfg, b"x").unwrap();
    std::fs::write(&b_leaked, b"y").unwrap();

    let data_dir = resolved_data_dir();
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut editor = make_editor(&project_a, &data_dir);
    // Open the in-project file (legit), then a foreign-project file.
    // The foreign file currently lands in this window's tab bar; the
    // invariant says it should not be captured into this window's
    // saved state.
    editor.open_file(&a_cfg).expect("open A file");
    editor.open_file(&b_leaked).expect("open B file");

    // Capture the workspace as save would.
    let ws = editor.capture_workspace();

    let foreign_external: Vec<_> = ws
        .external_files
        .iter()
        .filter(|p| {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            let canon_a = project_a
                .canonicalize()
                .unwrap_or_else(|_| project_a.clone());
            !canon.starts_with(&canon_a)
        })
        .collect();
    assert!(
        foreign_external.is_empty(),
        "captured workspace records foreign-project files in external_files: {foreign_external:?}.\n\
         Invariant: a window's captured state contains only files inside its working_dir."
    );

    // Also: no open_tabs entry in any split should be an absolute
    // path outside working_dir.
    let canon_a = project_a.canonicalize().unwrap_or(project_a.clone());
    for (split_id, ss) in &ws.split_states {
        for tab in &ss.open_tabs {
            if let fresh::workspace::SerializedTabRef::File(p) = tab {
                if p.is_absolute() {
                    let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
                    assert!(
                        canon.starts_with(&canon_a),
                        "split {split_id} open_tabs has absolute foreign-project entry {p:?}.\n\
                         Invariant: only in-project paths may be captured."
                    );
                }
            }
        }
    }
}
