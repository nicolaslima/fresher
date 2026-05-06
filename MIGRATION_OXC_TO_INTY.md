# Migration: oxc → inty

Phase 0 proposal. **Not yet a plan to execute.** Stops at "what to
build and in what order" so the maintainer can poke holes before any
code is touched.

## Goal

Drop `fresh-parser-js` and every `oxc_*` workspace dependency. Inty
replaces it as the type checker, bundler, and `.d.*` emitter. The
plugin API gets a v2 surface that lives within inty's deliberately
limited type system; v1 backward compatibility is not preserved.

## Inventory of the v1 surface that has to change

### Where oxc / `fresh-parser-js` is called from

| Caller | Function used | Job |
|---|---|---|
| `fresh-plugin-runtime/src/thread.rs:1288` (`prepare_plugin`) | `extract_plugin_dependencies` | Pull `import "fresh:plugin/NAME"` lines for topological sort |
| `…thread.rs:1297` | `emit_isolated_declarations` | Per-plugin `.d.ts` for `<config_dir>/types/plugins.d.ts` |
| `…thread.rs:1313–1336` | `has_es_imports`, `bundle_module`, `has_es_module_syntax`, `strip_imports_and_exports`, `transpile_typescript` | Read TS plugin source → JS for QuickJS |
| `…thread.rs:1674` | `topological_sort_plugins` | Order plugins by declared deps |
| `fresh-plugin-runtime/src/backend/quickjs_backend.rs:5215` (`load_module_with_source`) | same as above | File-loaded plugin pipeline |
| `…quickjs_backend.rs:5321` (`execute_source`) | strip + transpile | Buffer-loaded plugin pipeline |
| `fresh-plugin-runtime/src/ts_export.rs` | `oxc_parser::Parser`, `oxc_codegen::Codegen`, `SourceType::d_ts()` | Validate + pretty-print the `fresh.d.ts` editor-API surface that the proc macro emits |
| `fresh-editor/src/main.rs:1725` (`check_plugin_bundle`) | `bundle_module`, `transpile_typescript`, `has_es_module_syntax` | `fresh --cmd check-plugin <path>` debug command |
| `fresh-editor/src/init_script.rs:573` (`check`) | `oxc_parser::Parser` | `fresh --cmd init check` syntax-checks `~/.config/fresh/init.ts` |

Two things stand out:

1. **`extract_plugin_dependencies` is dead code in practice.** Searching
   the entire `crates/fresh-editor/plugins/` tree finds zero
   `fresh:plugin/` import specifiers. Cross-plugin dependencies are
   currently expressed exclusively through the runtime registry
   (`editor.exportPluginApi(...)` / `editor.getPluginApi(...)`). The
   topo-sort therefore always degrades to alphabetical. v2 can drop
   the import-scheme entirely without losing functionality.
2. **`ts_export.rs` is special.** It does not transform plugin source —
   it's where `fresh-plugin-runtime` validates and pretty-prints the
   ts-rs-derived editor-API surface before writing it to
   `crates/fresh-editor/plugins/lib/fresh.d.ts`. Inty's pretty
   printer (the TS-flavor one shipped under P8) replaces this — but
   the *output target* changes from `.d.ts` to `.d.js`, which has
   knock-on effects on every plugin file that uses
   `/// <reference path="./lib/fresh.d.ts" />`.

### Plugin-side patterns that have to change

(Counts are over `crates/fresh-editor/plugins/**/*.ts`.)

| inty restriction | v1 plugin usage | Resolution path |
|---|---|---|
| no `keyof` (5 uses) | only in `fresh.d.ts`: `getPluginApi<K extends keyof FreshPluginRegistry>`, `on<K extends keyof HookEventMap>`, etc. | API redesign — see "v2 API". Drop `keyof` cleverness. |
| no indexed access `T[K]` | same place — `FreshPluginRegistry[K]`, `HookEventMap[K]` | Same. |
| no method overloads | `getPluginApi`, `on`, `off` are each declared 2–3× via interface merging | Pick one signature per method. |
| no declaration merging | `interface EditorAPI { ... }` augmented 3× in `fresh.d.ts`; `declare global { interface FreshPluginRegistry { ... } }` augmented in `live_grep.ts`, `dashboard.ts`, `vi_mode.ts` | Single emitted `EditorAPI` row; replace per-plugin global augmentation with an explicitly-annotated `getPluginApi` consumer (caller writes the type). |
| no intersection `A & B` | `grepProjectStreaming(...): PromiseLike<GrepMatch[]> & { searchId: number }`; a few small uses | Inline merged shape: `{ ...PromiseLike fields, searchId: Number }`. |
| no `any` (106), `unknown` (134), `Function` (1) | API and plugin code | Each gets a concrete replacement: closed string-literal union, structural row, named alias, or `Promise<T>`. `setSetting(value: unknown)` → discrete alias `JsonValue` (recursive, expressible since P6). |
| no bounded type parameters | `FinderConfig<T>`, `NavigationController<T>` already use unbounded `<T>` | OK as-is. |
| no discriminated-union narrowing on `.kind` (P5 deferred) | `code-tour.ts`, `dashboard.ts`, `pkg.ts`, `theme_editor.ts`, `live_diff.ts`, `devcontainer.ts`, `audit_mode.ts` — about 7 plugins | **The biggest blocker.** Each call site has to lift the discriminator out: branch on a bare `state` (string) instead of `state.kind`, and carry payload fields directly, OR rely on row-polymorphic refactor where the variants share a flat row. |
| no `static` / `extends` on classes | none in plugin code; lib classes don't use either | OK. |
| `Partial<T>` (~13 uses incl. `Partial<OverlayOptions>`) | inty has no mapped types | Replace with a hand-expanded alias `OverlayOptionsPartial` whose every field is `T \| Undefined`. Generate from the same Rust source as `OverlayOptions`. |
| **tuples `[number, number, number]`** (~50 uses for RGB; also `[T, U]` shapes) | `OverlayColorSpec`, `style.fg/bg`, mouse coordinate pairs, env arrays | inty rejects tuples. Two options below in "Open questions". |
| `Record<string, T>` (88) | very common, including `Record<string, unknown>` | inty has `Record<K, V>`. Replace `unknown` value type with a recursive `JsonValue` alias. |
| `as const`, `as [number, number, number]` casts (~10) | tuple-narrowing helpers in `live_diff.ts`, `devcontainer.ts`, `pkg.ts` | inty rejects type assertions. Will fall out naturally once the tuple replacement lands. |
| `interface Foo { ... }` | many | P8 maps cleanly to row aliases — no work needed once P8's `.d.ts` loader ships, otherwise rewrite to `/** type Foo = ... */`. |
| `enum X` | none in plugin code; mentioned only in comments | n/a. |

### Plugin sources that import lib helpers

`code-tour.ts`, `clangd_support.ts`, `diagnostics_panel.ts`,
`find_references.ts`, `git_grep.ts`, `git_find_file.ts`,
`live_grep.ts`, `pkg.ts`, `audit_mode.ts` import from
`./lib/finder.ts`, `./lib/index.ts`,
`./lib/virtual-buffer-factory.ts`. Inty-bundle resolves only `.js`
and `.d.js`. Either:

1. Rename `.ts` → `.js` and rewrite the annotations to JSDoc comments
   (the inty default), or
2. Wait on / push for full `.ts` source loading in inty (P8 has TS
   *type* parsing; `.ts` *source* loading was deferred), or
3. Add a pre-pass on the fresh side that maps `.ts` → `.js`
   in-memory before handing to inty.

Recommended: (1). It's grindy but transparent and removes a layer of
"is this file in TS flavor or inty flavor?" confusion.

### Patterns that genuinely don't fit and need an inty change

I expected to surface several. After the audit I found **one** that
needs flagging, plus one I'd want to discuss:

- **Discriminated-union narrowing (P5).** Already deferred on the inty
  side but unavoidable on the fresh side. ~7 plugins use it as the
  primary state-machine pattern. If P5 stays deferred, those plugins
  need rewrites to a flat-row form, which is genuine churn. Worth
  reconsidering before committing to the migration.
- **Tuples.** P8's rejection table says "use a record `{ first, second }`".
  RGB triples appear ~50× in plugin code and in every overlay-style call.
  The Rust side currently exposes `[u8; 3]` via ts-rs (`AnimationRect`,
  `OverlayColorSpec`, theme color triples, etc.). The decision affects
  both the generated `.d.js` and every plugin call site. **Not an
  inty change** — but big enough to call out.

Everything else (no `any`, no `keyof`, no merging, no overloads, no
intersection) is absorbed by the v2 API redesign without an inty
change.

## v2 API proposal

Targeted at what inty actually supports. Every entry below is
expressible as a row alias plus standard generics — no `keyof`, no
overloads, no merging, no narrowing.

### Editor surface

A single non-generic `EditorAPI` row, emitted by inty's TS / inty
pretty printer from the existing ts-rs-derived schema. No augmentation,
no overloads, no `<K extends keyof …>`. Fields fall into four groups:

1. **Pure queries** (`getActiveBufferId`, `listBuffers`, `getCwd`, …) —
   all return concrete row aliases or primitive types.
2. **Async operations** that may be cancelled — return `Cancellable<T>`:
   ```js
   /** type Cancellable<T> = {
     *   result: Promise<T>,
     *   kill: () => Promise<Boolean>
     * }
     */
   ```
   Replaces `ProcessHandle<T> extends PromiseLike<T>`. Caller writes
   `(await handle.result)` rather than `await handle`. Trivial
   wrapper change in callers, removes the `extends PromiseLike`
   inheritance which inty can't model.
3. **Per-event subscription methods**, one per event, generated from the
   same `HookEventMap` Rust source today drives `fresh.d.ts`:
   ```js
   editor.onBufferActivated((args /*: BufferActivatedArgs */) => { ... });
   editor.onAfterFileSave((args /*: AfterFileSaveArgs */) => { ... });
   editor.offBufferActivated(handler);
   ```
   Each handler is fully typed without `keyof` or overloads. Verbose,
   but every payload is concrete and the proc macro can generate the
   wrapper. The untyped fallback `editor.on(name, handlerName)` for
   string-named handlers (used by `registerHandler`) stays as a single
   non-generic method — no overload — for cases where the event is
   determined at runtime.
4. **Plugin-API discovery**. Same `exportPluginApi` / `getPluginApi`
   as today, but the latter loses its `<K extends keyof …>` overload:
   ```js
   /** const api: DashboardApi | Null */
   const api = editor.getPluginApi("dashboard");
   ```
   Caller annotates the return type. The annotation is verified at use
   sites (field accesses against the row); if the plugin authoring
   `dashboard` ships a `dashboard.d.js` exporting
   `DashboardApi`, the consumer can `import` it and re-use the alias.
   Replaces declaration merging with explicit imports — same
   information, expressible in inty.

### Plugin discovery and dependencies

Drop the `import "fresh:plugin/NAME"` scheme. It's unused (zero
references in the current plugin tree), and dependency ordering
already falls back to alphabetical anyway. Plugins coordinate
exclusively via `exportPluginApi` / `getPluginApi`. If a plugin
needs another's API, it ships a `.d.js` and the consumer
`import`s the type. Order of plugin loading remains deterministic
(alphabetical) — that's already the de-facto behaviour.

### Handler registration

`registerHandler(name, fn)`: today `fn: Function`. inty doesn't have
a top function type. Replace with the broadest concrete shape:
```js
/** function registerHandler(
  *   name: String,
  *   fn: (args: Record<String, JsonValue>) => Undefined | Promise<Undefined>
  * ): Undefined
  */
```
Plugins that need typed args use the per-event registration methods
above instead, where the payload type is concrete.

### Cancellable async

`Cancellable<T>` is a user-defined generic alias (P6 ships this) —
no inty change needed. Plugins that wrap `editor.spawnProcess` etc.
can read the same alias.

### What v2 removes outright

- `editor.on<K extends keyof HookEventMap>(...)` overloads — replaced by
  per-event methods.
- `getPluginApi<K extends keyof FreshPluginRegistry>(...)` — replaced
  by caller annotation.
- `interface FreshPluginRegistry {}` augmentation — replaced by
  `import { DashboardApi } from "./dashboard.d.js"`.
- `Partial<OverlayOptions>` — replaced by a generated
  `OverlayOptionsPartial` row.
- `unknown` values — replaced by a recursive `JsonValue` alias for
  payloads that are honestly opaque, or concrete rows everywhere else.

## Phasing plan

Each phase ends with a green build, a commit, and a stop. Order is
chosen so each cutover has a clear rollback target.

**A. Audit, no code.** This document. Resolves the open questions
(see below) before anything else.

**B. Decide tuples and `Partial`.** Two design decisions land in
specs/CHANGELOGs without code: how RGB and similar tuples are
encoded post-migration, and how `Partial<…>` is generated. Locks
the shape of every later code change.

**C. Inty as a parallel checker.** Add inty (and `inty-bundle`) as
workspace deps. Run inty over the existing plugin tree without
trusting it: produce per-plugin `.d.js` and compare against the
oxc-emitted `.d.ts`. Surface every plugin file that fails to
type-check under inty + the new v2 surface. **Do not** wire it into
the load path. Validation only. Output is a list of plugins that
need rewrites.

**D. v2 editor surface in `fresh-plugin-runtime`.** Add a second
emit path in `ts_export.rs` (or a sibling module) that produces the
v2 `fresh.d.js` — per-event registration, no overloads, no keyof.
Keep the legacy `.d.ts` emit alongside. Both files written; plugins
opt in by switching their `/// <reference path>` comment.

**E. Plugin migration, smallest first.** Convert each plugin to:
   - `.js` with JSDoc annotations (or `.ts` if the `.d.ts` loader
     ships first).
   - v2 event registration.
   - Caller-annotated `getPluginApi` consumers.
   - Discriminated-union rewrites where P5 isn't supported.
   Order: simplest LSP shims (`zig-lsp.ts`, `java-lsp.ts`, …) →
   focused features (`flash.ts`, `code-tour.ts`) → mid-size
   (`git_log.ts`, `dashboard.ts`) → the heavy hitters
   (`audit_mode.ts`, `vi_mode.ts`, `theme_editor.ts`,
   `merge_conflict.ts`). One commit per plugin; each plugin's
   commit re-runs the inty check from phase C.

**F. Cut over the runtime.** Replace `fresh_parser_js::bundle_module`
with `inty_bundle::bundle` in `prepare_plugin`,
`load_module_with_source`, `execute_source`. Replace
`emit_isolated_declarations` with
`inty::declarations::emit_declarations`. Drop
`extract_plugin_dependencies` and `topological_sort_plugins`
(alphabetical fallback is enough — confirmed by the audit).

**G. Cut over `init.ts`.** Replace
`init_script::check`'s oxc parse with `inty::parser::parse`. Init.ts
goes through the same v2 pipeline as a regular plugin.

**H. Cut over `ts_export.rs`.** Switch the editor-API surface from
`.d.ts` to `.d.js`. Drop `oxc_parser`, `oxc_codegen`, `oxc_span` from
`fresh-plugin-runtime/Cargo.toml`. The `validate_typescript` /
`format_typescript` helpers either delete or move to a thin inty
wrapper.

**I. Cut over the debug subcommand.** `fresh --cmd check-plugin`
calls `inty_bundle::bundle` and prints the result.

**J. Delete `crates/fresh-parser-js`.** Drop from workspace
`Cargo.toml`. Drop every `oxc_*` dep from `fresh-editor/Cargo.toml`
and `fresh-plugin-runtime/Cargo.toml`. Final commit.

Risky cutovers: **F** (live runtime path; mitigated by phase C
having already validated every plugin) and **E** (long; bounded by
plugin count, but each plugin is independent — easy to bisect a
regression).

## Open questions

These should be settled before phase B begins.

1. **Discriminated-union narrowing (P5).** Stay deferred? If yes,
   ~7 plugins need a rewrite to flat rows. If reconsidered, P5
   lands inty-side first and the migration is dramatically simpler.
   I'd argue for landing P5 — it's the single most common modelling
   pattern in the existing plugins, the deferral note describes a
   contained scope (`===` / `!==` on string-literal tags, switch
   statements, early-return form), and the alternative is mass
   plugin rewrites that aren't really about the migration.
2. **Tuples.** Two viable shapes:
   - `{ r: Number, g: Number, b: Number }` records — natural
     in inty, but rewrites every overlay call site and every
     ts-rs-derived RGB type in `fresh-core/api`.
   - `OverlayColorSpec` becomes string-only (theme keys) plus a
     small set of named constants for inline RGBs.
   First is more invasive but preserves direct RGB; second steers
   all colour usage through the theme registry, which is already
   the recommended idiom. Either way, the choice affects ~50 call
   sites and the Rust API surface.
3. **`.ts` source loading in inty.** Migrate plugins to `.js` with
   JSDoc, or push for inty's deferred `.ts` source loader to land
   first? `.js`-with-JSDoc is honest and removes a flavor toggle,
   but is a lot of mechanical churn.
4. **`.d.js` vs `.d.ts` for the editor-API surface.** P8 ships
   `inty declarations --format=ts`. So `fresh.d.js` (inty flavor)
   *and* `fresh.d.ts` (TS flavor) are both available — should we
   ship both for editor tooling (LSP completions in init.ts) while
   inty itself only consumes the `.d.js`?
5. **`Partial<T>` generation.** Hand-write each `XPartial` alias,
   or generate them from the proc macro alongside the base type?
   Generated form is mechanical and doesn't drift; manual form is
   only needed for a few types (`OverlayOptions`,
   `LanguagePackConfig`, `LspServerPackConfig`).
6. **Per-event method generation.** Confirms-or-denies: is generating
   ~40 `editor.onXxx` methods (one per `HookEventMap` entry) acceptable
   surface bloat? Alternative is keeping the single
   `editor.on(name: String, handlerName: String): Undefined` form and
   asking authors to type-annotate the handler. Less typed but
   simpler API.
7. **`fresh:plugin/NAME` import scheme.** Confirms-or-denies: is the
   audit accurate that no plugin currently uses it? If yes, drop in
   phase F. If something not in `crates/fresh-editor/plugins/` uses
   it, that subtree needs migrating first.
8. **`tsconfig.json` for `init.ts`.** Today
   `crates/fresh-editor/src/init_script.rs` writes a starter
   `tsconfig.json` so VS Code / TS LSP work in the user's
   `~/.config/fresh`. Post-migration: keep emitting a tsconfig?
   Switch to whatever inty's LSP equivalent is? (inty-lsp exists in
   the sibling repo.) Or both?

## Working notes (carried from the task brief)

- Build with `cargo build` (debug). Tests via `cargo nextest run`,
  30s/run.
- One feature per commit; finish (impl + tests + fmt + clippy)
  before starting the next.
- Filesystem access goes through `self.authority.filesystem`, not
  `std::fs` or `Path::exists`.
- No status-bar messages as primary error channel.

The proposal is the deliverable. Stopping here.
