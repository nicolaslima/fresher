# Orchestrator Session Persistence & Recovery — Hermetic Test Plan

Status: draft / design
Owner: orchestrator
Related code:
- `crates/fresh-editor/src/app/orchestrator_persistence.rs`
- `crates/fresh-editor/plugins/orchestrator.ts` (`resolveCanonicalRepoRoot`, `pathIsInsideGitWorkTree`)
- `crates/fresh-editor/src/main.rs` (`restore_inactive_window_workspaces`)
- `crates/fresh-editor/tests/orchestrator_persistence_paths.rs`
- Existing semantic framework: `tests/common/scenario/persistence_scenario.rs`, `tests/semantic/`

## 1. Goal

Make session persistence/recovery testable the same way the rest of the editor is: as **scenarios-as-values**, executed in a **hermetic** environment, where we can enumerate every combination of factors that influences load-time behavior. The output should be:

- a fixed regression corpus (one file per equivalence class), and
- a generator that explores the full cartesian product, model-checking style,
- a shadow model the runner cross-checks against.

We need to cover **upgrade paths from any historical or malformed on-disk shape**, **any git topology (including no git at all)**, and **any base-path / canonicalization quirk**.

## 2. Factor inventory

Each axis below is an independent input to the system-under-test. The test harness enumerates the cartesian product (with smart pruning — see §5) of a curated finite alphabet per axis.

### A. On-disk envelope (schema / format)

| Code | Variant |
|------|---------|
| `none` | No `windows.json` at any path |
| `v1_single` | One legacy `<data>/orchestrator/<encoded_cwd>/windows.json`, schema v1 (no `project_path`, no `shared_worktree`, no `version` field or `version=1`) |
| `v1_many` | Multiple v1 files for different cwds (multi-project history) |
| `v1_collision` | Two v1 files with overlapping `id`s (mtime determines winner per migration logic) |
| `v2_global` | `<data>/orchestrator/windows.json` v2 with `version=2` and `project_path` populated |
| `v2_partial` | v2 envelope present, but per-window `project_path` field missing on some entries (mid-migration shape) |
| `v1_and_v2` | Both v1 and v2 files coexist (interrupted migration, downgrade-then-upgrade) |
| `migrated_bak_only` | Only `.migrated.bak` files exist; no live v1 and no v2 (post-migration replay) |
| `future_v3` | `version=3` plus unknown fields — forward-compat behavior |
| `malformed_json` | Truncated / invalid JSON |
| `empty_file` | Zero-byte `windows.json` |
| `non_utf8` | Binary garbage |
| `wrong_perms` | File exists but unreadable (EACCES) |
| `crash_tmp` | A stale `windows.json.tmp` left by a previous interrupted atomic write |

Plugin-state file axis is analogous: `none / valid / malformed / wrong-name / huge`.

### B. Active-window / cwd interaction

- `env.active` points at: this-cwd | another-cwd | nonexistent-id | id 0 | u64::MAX
- `next_id`: lower than max(window.id) (collision risk) | normal | u64::MAX
- Number of persisted windows: 0 / 1 / many (≥10)
- Per-window `root` vs `project_path`: both set & equal | both set & different (linked-worktree case) | only `root` (v1) | only `project_path` (synthetic) | both missing

### C. Git topology of launch cwd

| Variant | Meaning |
|---------|---------|
| `no_git_binary` | `git` not on PATH at all |
| `git_present_no_repo` | Plain dir, no `.git` |
| `main_worktree` | Normal repo |
| `linked_worktree` | `git worktree add` checkout; `.git` is a file pointing to common dir |
| `bare_repo` | Bare repo as cwd |
| `submodule_cwd` | Cwd is a submodule working tree |
| `corrupt_git` | `.git` directory exists but `git rev-parse` fails |
| `detached_head` | Repo, but HEAD detached |
| `worktree_removed` | Persisted session referenced a linked worktree that no longer exists on disk |
| `common_dir_moved` | Repo was moved; `--git-common-dir` resolves to a new path vs persisted `project_path` |
| `nested_repo` | Repo inside a repo (cwd is inner; outer is also a repo) |

### D. Filesystem / path shape of cwd vs persisted root

- `exact_match`
- `trailing_slash_differs`
- `symlink_to_persisted`
- `persisted_was_symlink_now_real` (and vice versa)
- `cwd_is_parent_of_persisted`
- `cwd_is_child_of_persisted`
- `cwd_is_sibling`
- `case_differs` (on case-insensitive FS only)
- `path_no_longer_exists`
- `path_replaced_by_file`
- `path_on_different_device` (different `st_dev`)
- `unicode_normalization_differs` (NFC vs NFD)
- `non_utf8_path_component`
- `very_long_path` (>4096)

### E. Process / concurrency state

- No other editor instance | another instance running (lock present, alive PID) | stale lock (PID dead) | lock owned by us (re-entrant)
- `recovery_pending` flag set on some buffers
- Recovery files: none | present-and-consistent | present-but-stale-mtime | present-but-size-mismatch | chunked | full-file
- `hot_exit` config: off / on
- `restore_previous_session` config: off / on
- Modified buffers at last quit: 0 / some / many
- In-flight `.tmp` from interrupted save

### F. Plugin state surface

- Installed plugins: same set as persisted | superset | subset (missing plugin owns persisted state) | renamed plugin
- Per-plugin state size: empty | small | 10 MB (oversize)
- Unsafe plugin name (path-traversal attempt in filename)

## 3. Hermetic environment

Every scenario runs against an isolated sandbox built **per test**, no shared state, no network, no real `$HOME`, no real git interaction by default.

Pieces we need (some already exist, some new):

1. **`OrchestratorScenarioFs`** — wraps `VirtualFs` (already used by `PersistenceScenario`) and lets the scenario *declare* the on-disk layout under `<data_dir>/orchestrator/...` as data. The runner materializes it into a `tempfile::TempDir` keyed to one unique data dir per test, then sets `FRESH_DATA_DIR` (or the in-process injection point used by `orchestrator_persistence::data_paths`).
2. **Mock `git` shim** — a small executable on PATH (resolved via `PATH` override in the test process) that replies to `rev-parse --show-toplevel` / `--git-common-dir` / `--is-inside-work-tree` from a scenario-declared table. Variants: missing binary, exits nonzero, hangs (timeout test), returns malformed output. We must NOT shell out to the real `git`.
3. **Mock clock** — for `mtime`-driven migration tiebreaks and recovery-staleness checks. The persistence module should accept an injected `Clock` (small refactor — see §7).
4. **Path resolver hook** — `canonicalize` calls go through a small trait so a scenario can simulate symlinks, missing paths, `EACCES`, and cross-device behavior without touching the real FS.
5. **No real network / no real `$HOME`** — guarded by env scrub in the test harness entry.

Each scenario therefore declares everything in a struct:

```rust
pub struct SessionPersistenceScenario {
    pub description: &'static str,

    // On-disk envelope (factor A)
    pub on_disk: OnDiskLayout,                // none / v1 / v2 / mixed / malformed / ...
    pub plugin_state_files: Vec<PluginStateFile>,

    // Launch context
    pub launch_cwd: PathSpec,                  // factor D
    pub git_topology: GitTopology,             // factor C
    pub clock: FakeClock,
    pub other_instance: OtherInstance,         // factor E
    pub config: ConfigOverrides,               // hot_exit, restore_previous_session

    // Expectations
    pub expect: ExpectedOutcome,
}

pub struct ExpectedOutcome {
    pub picked_active_window_id: Option<u64>,
    pub final_envelope_on_disk: PersistedWindowsShape, // after load+save cycle
    pub legacy_files_handled: Vec<LegacyFileFate>,     // renamed to .migrated.bak | left alone
    pub warnings: WarningSet,                          // expected log warnings (factor: malformed)
    pub created_paths_in_cwd: BTreeSet<PathBuf>,       // MUST be empty (#1991 invariant)
}
```

Same shape as existing `PersistenceScenario` / `WorkspaceScenario` (see `tests/common/scenario/persistence_scenario.rs:25-226`), so we plug into the same runner conventions.

## 4. Model-checking-style enumeration

Two layers, both fed by the same scenario value:

### 4.1 Regression layer — curated equivalence-class corpus

One scenario JSON per **interesting cell** of the factor matrix. Stored under `tests/semantic/orchestrator_persistence/corpus/*.json`. The schema is locked by a round-trip test (mirroring `scenario_shapes.rs:1-100`).

Curated cells:

- One scenario per row of factor A (envelope shape).
- For each of A∈{v1_single, v2_global}, the seven git-topology variants of factor C that matter for load-time matching.
- For each of `pick_active` fallback branches in `orchestrator_persistence.rs:186-209`, an explicit scenario asserting which branch fired.
- All known issue-regressions (#1991, #2026) re-expressed as scenarios.

### 4.2 Generator layer — bounded model checker

A proptest strategy produces full `SessionPersistenceScenario` values by picking one variant from each axis. Two properties hold for every generated scenario:

1. **Safety invariants** (always):
   - No file is created under `launch_cwd` (the `.fresh`-in-worktree regression). Check the `created_paths_in_cwd` observable.
   - The post-load in-memory `PersistedWindows` is `version == CURRENT_VERSION`, and every `window.id < next_id`, and `windows[i].id` are unique.
   - If `picked_active_window_id == Some(id)`, then `windows` contains a window with that id, and that window's resolved path matches `launch_cwd` under `paths_equal`.
   - No panic, no unbounded I/O (bounded by per-scenario byte budget), no shell-out to real `git`.
   - Legacy v1 files are *either* still present *or* renamed to `.migrated.bak` — never deleted, never overwritten.
2. **Shadow-model agreement**:
   - A pure Rust function `model::pick_active(scenario) -> Outcome` re-implements the spec without any I/O. The runner asserts `model_outcome == actual_outcome` for every generated scenario. This is the "model-checking" piece: the model is the reference, the implementation is the checked artifact, divergence is a counterexample.

Proptest config matches the rest of the suite (`properties.rs:50-107`): 32 cases per file, 256 shrink iters, regression-recording on.

### 4.3 Total enumeration (small / hermetic)

For axes whose alphabet is small enough (A × C × D-subset × {hot_exit,restore} ≈ 14 × 11 × 10 × 4 ≈ 6160 cells), provide an `#[test] fn exhaustive_load_matrix()` that walks the entire product. Pruning rules drop trivially infeasible combos (e.g. `no_git_binary` + `linked_worktree`). The cap is "must complete in <30s on CI".

## 5. Pruning rules (feasibility)

Pure cartesian product is mostly noise. We define a `feasible(scenario) -> bool` predicate:

- `git_topology == no_git_binary` implies any persisted `shared_worktree==false` is "stranded" — covered, but we only generate one such case per envelope variant.
- `git_topology ∈ {linked_worktree, submodule, bare}` requires git binary present.
- `on_disk == none` makes most active-window factors moot; collapse to a single representative case.
- `path_no_longer_exists` + `exact_match` is contradictory.
- `case_differs` only generated on case-insensitive FS marker.

Infeasible combos are statically skipped so the generator's effective space stays in the low thousands.

## 6. Integration with the semantic testing framework

We extend, not fork, the existing framework:

- New scenario type `SessionPersistenceScenario` alongside `PersistenceScenario`, `WorkspaceScenario`, etc., under `tests/common/scenario/`.
- New observable types (`PersistedWindowsShape`, `LegacyFileFate`, `WarningSet`) follow the existing pattern in `tests/common/scenario/observable.rs`.
- New runner `check_session_persistence_scenario(&scenario)` follows `check_persistence_scenario` (persistence_scenario.rs:25-226) — sets up the hermetic env, runs the editor init path (`Editor::with_options` or the lower-level `read_persisted_windows_env` + `pick_active_window_for_cwd`), and diffs observed-vs-expected.
- Schema lockdown: add the new scenario to `scenario_shapes.rs` round-trip tests so on-disk corpus stays compatible.
- Three consumers of one scenario value:
  1. regression test (the curated corpus),
  2. property generator (bounded random),
  3. shadow-model checker (pure Rust spec).

## 7. Minimal code changes required first

These are pre-requisites; each is small and independent.

1. Inject a `Clock` and a `GitProbe` trait into `orchestrator_persistence.rs` (currently `mtime` and git shell-outs are called directly). Keep the existing free-function entrypoints as thin wrappers that pass real impls. (~50 LOC.)
2. Make `migrate_legacy_windows`, `read_persisted_windows_env`, `pick_active_window_for_cwd` take a `&dyn Filesystem` (mirroring `VirtualFs` in tests). Some of this already routes through `filesystem` — finish the threading.
3. Expose `data_paths()` so it accepts an explicit base, not just `$XDG_DATA_HOME` / platform default. Currently a test env var works but is global; switch to per-call injection.
4. Extract the "pick active" decision into a pure function `decide_active(envelope, cwd, canonicalizer) -> Decision` — this is the shadow-model target *and* what the production caller uses.

After (4), the model and the implementation are literally the same function called with different canonicalizers; divergence then collapses to "I/O wrappers differ", which is exactly what we want to test.

## 8. Concrete test files we'll add

```
tests/common/scenario/
    session_persistence_scenario.rs        # the scenario type + runner
    orchestrator_observables.rs            # PersistedWindowsShape etc.
    fake_git.rs                            # GitProbe impl + scenario hooks
    fake_clock.rs

tests/semantic/orchestrator_persistence/
    mod.rs
    corpus/                                # one .json per equivalence class
        v1_single_main_worktree.json
        v1_collision_mtime_tiebreak.json
        v2_global_linked_worktree.json
        v2_partial_missing_project_path.json
        v1_and_v2_coexist.json
        migrated_bak_only.json
        future_v3_unknown_fields.json
        malformed_json.json
        empty_file.json
        crash_tmp_left_over.json
        no_git_binary_with_v2.json
        worktree_removed_active_points_to_it.json
        sibling_cwd_no_match.json
        symlink_resolves_to_persisted.json
        case_differs_case_insensitive.json
        case_differs_case_sensitive.json
        long_path_4097_bytes.json
        non_utf8_component.json
        cross_device_st_dev_differs.json
        stale_lock_dead_pid.json
        recovery_pending_with_hotexit_off.json
        plugin_state_for_uninstalled_plugin.json
        plugin_state_path_traversal_name.json

    properties.rs                          # proptest generator + invariants
    exhaustive_matrix.rs                   # bounded cartesian sweep
    shadow_model.rs                        # pure-Rust reference

tests/semantic/scenario_shapes.rs          # extend with new scenario round-trip
```

## 9. What "pass" looks like

- Curated corpus: all green; new regressions added by writing one JSON file plus an expected-outcome block.
- Property suite: 32 cases × N proptest files green, no shrunk counterexample saved in `proptest-regressions/`.
- Exhaustive sweep: completes in <30s, zero failures.
- Shadow model: no divergence in 10,000-case soak run (nightly).
- Coverage check: `tarpaulin`/`llvm-cov` shows every branch in `pick_active_window_for_cwd`, `window_matches_cwd`, `migrate_legacy_windows`, and `resolveCanonicalRepoRoot` plumbing hit by at least one scenario.

## 10. Open questions / decisions to confirm

1. Do we want the shadow model written in Rust only, or also a TLA+/Alloy spec for the migration state machine? Recommendation: Rust only — same language, same CI, low ceremony.
2. Should `future_v3` cause hard-fail or silent-skip-and-leave-alone? Current code returns `None` (silent). Plan codifies that as the spec; revisit if product wants warnings.
3. Mock-git scope: do we need `--git-dir`, `--show-superproject-working-tree` (submodules), `worktree list --porcelain`? Start with the three already in `resolveCanonicalRepoRoot` and grow as scenarios demand.
4. Concurrency tests with a second live editor instance: in-process or out-of-process? Recommendation: in-process two-Editor harness — out-of-process is flaky and slow.
