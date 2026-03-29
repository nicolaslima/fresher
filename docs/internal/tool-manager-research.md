# Tool Manager: Prior Art Research

Research findings for an integrated tool manager that allows users to discover, install, update, and manage external development tools (LSP servers, formatters, linters, DAP servers) directly from within Fresh.

## Table of Contents

- [1. Mason.nvim (Neovim)](#1-masonnvim-neovim)
- [2. VS Code Extension Ecosystem](#2-vs-code-extension-ecosystem)
- [3. VS Code Extension Case Studies](#3-vs-code-extension-case-studies)
- [4. Helix Editor](#4-helix-editor)
- [5. Zed Editor](#5-zed-editor)
- [6. Fresh Current State](#6-fresh-current-state)
- [7. Cross-Cutting Patterns](#7-cross-cutting-patterns)

---

## 1. Mason.nvim (Neovim)

**License:** Apache-2.0 (safe). Source reviewed directly from cloned repo.

### Architecture

Mason is a portable package manager for Neovim that installs LSP servers, DAP servers, linters, and formatters into Neovim's data directory (`vim.fn.stdpath("data")/mason/`).

- **Isolated install directory**: Each package gets its own subdirectory under `mason/packages/<name>/`.
- **Single `bin/` directory**: All executables are linked into `mason/bin/`, prepended to Neovim's `PATH` during `setup()`. Tools are only visible to Neovim, not polluting the system.
- **Registry-driven**: Mason itself contains no package definitions. All package metadata lives in a separate registry repository (`mason-org/mason-registry`). Downloaded/updated at runtime. Users can also point to custom registries via `file:` or `github:` protocols.
- **Compiler architecture**: Each purl type (`pkg:npm`, `pkg:cargo`, `pkg:github`, etc.) has a corresponding "compiler" module that parses the source spec and executes installation.

### Registry Package Format

Package definitions are YAML files at `packages/<name>/package.yaml`, validated against a JSON schema. Core structure:

```yaml
name: rust-analyzer
description: Language Server Protocol for Rust
homepage: https://github.com/rust-lang/rust-analyzer
licenses: [Apache-2.0, MIT]
languages: [Rust]
categories: [LSP]  # LSP | DAP | Formatter | Linter | Compiler | Runtime

source:
  id: pkg:github/rust-lang/rust-analyzer@2026-03-23  # purl with version
  asset:  # per-platform download specs
    - target: linux_x64_gnu
      file: rust-analyzer-x86_64-unknown-linux-gnu.gz
      bin: rust-analyzer-x86_64-unknown-linux-gnu
    - target: win_x64
      file: rust-analyzer-x86_64-pc-windows-msvc.zip
      bin: rust-analyzer.exe

bin:
  rust-analyzer: "{{source.asset.bin}}"

neovim:
  lspconfig: rust_analyzer  # maps to lspconfig server name
```

Key features:
- **Purl identifiers** determine installation strategy (`pkg:npm/...`, `pkg:cargo/...`, `pkg:github/...`, `pkg:pypi/...`, `pkg:golang/...`)
- **Target selectors**: `darwin_arm64`, `linux_x64_gnu`, `linux_x64_musl`, `win_x64`, etc. — up-to-3-part tuples: `OS_arch_environment`
- **Expressions**: `{{version}}`, `{{source.asset.bin}}`, `{{version | strip_prefix "v"}}` for dynamic interpolation
- **`version_overrides`**: Pin older versions for specific semver constraints

### Installation Strategies

| Purl type | Strategy | Example |
|---|---|---|
| `pkg:npm/...` | `npm init` + `npm install` into package dir | pyright, typescript-language-server |
| `pkg:pypi/...` | Python venv + pip install | python-lsp-server |
| `pkg:cargo/...` | `cargo install` into package dir | rnix-lsp |
| `pkg:github/.../release` | Download GitHub release asset, extract | rust-analyzer, lua-language-server |
| `pkg:golang/...` | `go install` with `GOBIN` set | gopls |
| `pkg:gem/...` | `gem install` | solargraph |
| `pkg:openvsx/...` | Download from Open VSX marketplace | DAP adapters |

### Platform Detection

Located in `lua/mason-core/platform.lua`:
- Uses `vim.loop.os_uname().machine` mapped through `arch_aliases` (x86_64 → x64, aarch64 → arm64)
- Libc detection: tries `getconf GNU_LIBC_VERSION`, falls back to `ldd --version`, looks for "musl" vs "glibc"
- Platform matching via metatable: `M.is.linux_x64_musl` parses the string into OS/arch/libc components
- Additional: `os_distribution()` for Linux distro detection, `get_homebrew_prefix()` for macOS

### Binary Linking

Located in `lua/mason-core/installer/linker.lua`:

- **Unix**: Creates relative symlinks from `mason/bin/<name>` to the actual binary
- **Windows**: Creates `.cmd` batch wrapper scripts:
  ```batch
  @ECHO off
  GOTO start
  :find_dp0
  SET dp0=%%~dp0
  EXIT /b
  :start
  SETLOCAL
  CALL :find_dp0
  endLocal & goto #_undefined_# 2>NUL || title %%COMSPEC%% & "%%dp0%%\relative\path\to\executable.exe" %%*
  ```
  Same approach npm uses on Windows. Uses `%~dp0` to locate script directory and construct relative path.

### mason-lspconfig Bridge

Reads `neovim.lspconfig` field from package specs to map Mason package names to lspconfig server names. `automatic_enable` (default: true) calls `vim.lsp.enable()` for all installed servers after registry update. Listens to `package:install:success` to enable newly installed servers.

Filetype mapping generated from registry: `["go"] = { "gopls" }`, `["rust"] = { "rust_analyzer" }`, etc.

### Pros
1. Truly cross-platform with clean platform abstraction
2. Declarative YAML registry — schema-validated, auto-updated by Renovate
3. Isolated per-package installations, no system pollution
4. Many installation strategies (npm, pip, cargo, go, GitHub releases, etc.)
5. Expression system keeps package definitions DRY
6. Large ecosystem (500+ packages), actively maintained
7. Custom registries for organizations

### Cons
1. Depends on external package managers (npm, pip, cargo, go) already being installed
2. No lockfile or reproducibility mechanism
3. Version lag (depends on Renovate PRs being merged)
4. Build-from-source packages are fragile on unusual systems
5. No dependency resolution between packages
6. Registry is a single point of failure (mitigated by local caching)

---

## 2. VS Code Extension Ecosystem

**License:** MIT (safe). Source reviewed directly from cloned repo.

### Platform Targeting

VS Code uses a `TargetPlatform` enum with 11 concrete values:

```
win32-x64, win32-arm64, linux-x64, linux-arm64, linux-armhf,
alpine-x64, alpine-arm64, darwin-x64, darwin-arm64, web, universal
```

Extensions are published as separate VSIX packages per platform using `vsce package --target <platform>`. The marketplace stores multiple versions, each tagged with a `targetPlatform`. At install time, `getTargetPlatform(platform, arch)` resolves the user's system.

Platform detection (`extensionManagementUtil.ts`):
- Reads `/etc/os-release` to detect Alpine Linux (musl)
- Maps OS + architecture pairs to TargetPlatform enum values
- `UNIVERSAL` fallback for pure-JS extensions

### Extension Recommendations

Two layers of recommendations:

**File-based** (built into VS Code's `product.json`):
- Triggers when users open specific file types
- Conditions: language matching, path glob matching, content regex matching
- Recommendations cached for 7 days, ranked by recency
- Important ones shown as notifications

**Workspace-based** (`.vscode/extensions.json`):
- `recommendations` and `unwantedRecommendations` arrays
- Watches for file changes, fires `onDidChangeRecommendations`

### Extension Install Lifecycle

`AbstractExtensionManagementService`:
1. Compatibility checks (allowed list + platform compatibility)
2. Task management with `installingExtensions` Map, queuing, cancellation
3. Post-uninstall hooks via `manifest.scripts['vscode:uninstall']` (Node.js script, 5s timeout)
4. Storage cleanup after uninstall

Events: `onInstallExtension`, `onDidInstallExtensions`, `onUninstallExtension`, `onDidUninstallExtension`

### Pros
1. Clean platform separation — each VSIX only contains binaries for that platform
2. Marketplace handles resolution; clients just declare their platform
3. Multi-signal recommendation system (workspace, file-based, exe-based)
4. `UNIVERSAL` fallback for platform-agnostic extensions

### Cons
1. Publishers must build and publish N separate VSIXs per platform
2. File-based recommendations are hardcoded in product.json — third parties can't register new triggers
3. Each language extension reinvents tool management (Go uses `go install`, Rust downloads binaries, etc.)
4. Alpine/musl must be handled as a separate platform

---

## 3. VS Code Extension Case Studies

All MIT licensed (except vscode-java, EPL-2.0 — skipped). Source reviewed directly.

### Strategy A: Bundled Pre-compiled Binary (ruff-vscode)

Ruff bundles the Rust-based binary directly in platform-specific VSIX packages.

**Binary resolution priority:**
1. User-specified `path` setting
2. Bundled executable (if `importStrategy === "useBundled"`)
3. Python environment search (via discovery script)
4. Global PATH lookup via `which`
5. Fallback to bundled

**Platform matrix** (from CI): 9 targets including `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-unknown-linux-musl` (alpine), etc. Each maps to a VS Code `code-target`.

**Takeaway:** Binary bundling eliminates runtime installation but requires per-platform CI builds. Good for Rust-based tools where compilation is the bottleneck.

### Strategy B: GitHub Release Download (rust-analyzer, vscode-clangd)

**rust-analyzer** pre-builds binaries during release CI and includes them in the VSIX. Resolution priority:
1. `__RA_LSP_SERVER_DEBUG` env var
2. `rust-toolchain.toml` file → `rustup which rust-analyzer`
3. Bundled binary
4. Special NixOS handling (patches ELF interpreter)

**vscode-clangd** wraps the `@clangd/install` npm package. Checks system PATH first, falls back to downloading from GitHub Releases.

**Takeaway:** GitHub Releases are the most common distribution channel for pre-compiled binaries. Need to handle per-platform asset selection.

### Strategy C: Dynamic Download with Signature Verification (vscode-zig)

Most sophisticated approach. Manages both Zig compiler and ZLS downloads.

**Platform detection:**
```typescript
// Architecture: process.arch → canonical name
"ia32" → "x86", "x64" → "x86_64", "arm64" → "aarch64"
// OS: process.platform → canonical name
"darwin" → "macos", "win32" → "windows"
```

**Version selection:** Queries `releases.zigtools.org/v1/zls/select-version?zig_version=X&compatibility=only-runtime` with response caching.

**Installation flow:**
1. Determine target name: `${arch}-${os}` (e.g., `x86_64-linux`)
2. Download tarball from canonical URL or randomized mirrors
3. Download minisign signature, verify with Ed25519 (libsodium)
4. Extract with `tar -xf` (on Windows, uses `%SYSTEMROOT%\system32\tar.exe` to get bsdtar instead of GNU tar from Git Bash)
5. Validate extracted binary version matches expected
6. `chmod 755` on Unix
7. LRU cleanup: keep max 5 versions

**Storage layout:**
```
globalStorageUri/
├── zls/
│   ├── aarch64-linux-0.13.0/
│   │   └── zls
│   └── x86_64-windows-0.13.0/
│       └── zls.exe
└── zig/
    └── ...
```

**Takeaway:** Best reference for a comprehensive download manager. Handles mirrors, signatures, progress, concurrency deduplication, and cleanup. The `tar` Windows quirk is a real-world cross-platform lesson.

### Strategy D: Language Runtime Installation (vscode-go, vscode-ruby-lsp)

**vscode-go** uses `go install pkg@version`:
- Hardcoded version compatibility matrix (Go < 1.21 → gopls v0.15.3, etc.)
- Tracks declined installs/updates to avoid re-prompting
- Requires Go toolchain on the system (minimum Go 1.21)

**vscode-ruby-lsp** uses `gem install ruby-lsp`:
- Supports 6+ Ruby version managers (asdf, rbenv, rvm, chruby, shadowenv, custom)
- Shell activation: wraps commands in `$SHELL -i -c '...'` on Unix, runs directly on Windows
- Daily update check with rate limiting
- Passes activated Ruby environment's `env` to all gem commands

**Takeaway:** Runtime-based installation is the simplest when the runtime exists, but requires detecting and activating the right version manager. Cross-platform shell semantics (interactive login shell on Unix vs direct exec on Windows) are a key challenge.

### Strategy E: Bundled Node Module (pyright)

Pyright is pure TypeScript, bundled via webpack into the extension as `dist/server.js`. Server is spawned as a Node.js module using the editor's own Node.js runtime via IPC transport.

Uses file-based cancellation (creates/deletes files in `os.tmpdir()`) instead of process signals for cross-platform reliability.

**Takeaway:** Simplest strategy — no installation needed. Only works for tools that can run in Node.js. File-based IPC patterns are more portable than signals.

---

## 4. Helix Editor

**License:** MPL-2.0 (source not reviewed — copyleft). Information from public documentation only.

### Current Approach

Helix relies entirely on the system PATH for language servers. Configuration is in `languages.toml`:

```toml
[[language]]
name = "go"
language-servers = ["gopls"]

[language-server.gopls]
command = "gopls"
```

No integrated tool management. Users must install tools manually via system package managers, language-specific tools, or Mason (which some users run alongside Helix despite it being Neovim-focused).

### Community Stance

There are active GitHub discussions requesting Mason-like functionality. The Helix team has historically preferred a minimal approach, suggesting external tool management. The friction of manual setup is a frequently cited pain point.

### Takeaway

Helix represents the "do nothing" baseline. The community demand for integrated tool management validates the need for this feature in Fresh. The pain points users report (different install methods per tool, PATH configuration, version management) are exactly what we're solving.

---

## 5. Zed Editor

**License:** GPL-3.0 / AGPL-3.0 (source not reviewed — strong copyleft, competing editor). Information from public documentation and blog posts only.

### Known Approach

From public docs: Zed manages language servers natively, downloading and sandboxing them automatically. Extensions define language server support including download URLs for different platforms. Zed handles architecture detection (Apple Silicon vs x86_64) and downloads appropriate binaries.

### Takeaway

Zed validates the integrated approach — automatic download and management without user intervention is the UX target. Their extension-based language server definitions are conceptually similar to Mason's registry.

---

## 6. Fresh Current State

### Existing Package System

Located in `src/services/packages.rs`. Uses `package.json` with a `fresh` block:

```json
{
  "name": "package-name",
  "fresh": {
    "entry": "plugin.ts",
    "lsp": { "command": "lsp-server" },
    "grammar": { "file": "syntax.tmLanguage" },
    "language": { "tab_size": 2 },
    "themes": [...],
    "config_schema": {}
  }
}
```

Package types: plugin, theme, language, bundle. Scanned from `~/.config/fresh/languages/packages/` and `~/.config/fresh/bundles/packages/` at startup.

### LSP Configuration

In `config.json` → `lsp` section. Per-language config supporting single or multiple servers:

```json
{ "lsp": { "rust": { "command": "rust-analyzer", "args": [] } } }
```

Key fields: `command`, `args`, `enabled`, `auto_start`, `root_markers`, `env`, `language_id_overrides`, `feature_filter`, `process_limits`.

### LSP Spawning

`LspManager` in `src/services/lsp/` uses `tokio::process::Command`. State machine: Initial → Starting → Initializing → Running → Stopping → Stopped. Restart throttling: max 5 restarts per 180 seconds with exponential backoff. Process limits (cgroups v2 on Linux).

### Plugin System

QuickJS-based TypeScript runtime (`crates/fresh-plugin-runtime/`). Type-safe API generated via `ts-rs`. ~100 methods exposed. Async via callback IDs. Hot-reload support.

### Configuration System

4-layer hierarchy (lowest to highest precedence):
1. System (hardcoded defaults)
2. User (`~/.config/fresh/config.json`)
3. Project (`.fresh/config.json`)
4. Session (`.fresh/session.json`)

Plus platform overrides: `config_linux.json`, `config_macos.json`, `config_windows.json`.

### TUI Patterns

Ratatui 0.30 + crossterm 0.29. Popup system with kinds: Completion, Hover, Action, List, Text. Controls: TextInput, NumberInput, Dropdown, Toggle, Button, KeybindingList, TextList, MapInput. Each control follows state/input/render pattern.

### Cross-Platform

- Windows: `fresh-winterm` crate, `.cmd` keybinding map
- macOS: native menu bar via `muda`, macOS keybinding map
- Linux: GPM mouse support, cgroups v2 for process limits
- Platform detection: `cfg!(target_os = "...")` at compile time

---

## 7. Cross-Cutting Patterns

### Platform Detection Approaches

| System | OS Detection | Arch Detection | Libc Detection |
|---|---|---|---|
| Mason | `vim.fn.has("win32/mac/linux")` | `vim.loop.os_uname().machine` | `getconf` / `ldd` |
| VS Code | `process.platform` | `process.arch` | `/etc/os-release` for Alpine |
| vscode-zig | `process.platform` | `process.arch` with mapping table | N/A |
| Fresh (Rust) | `cfg!(target_os)` | `cfg!(target_arch)` | Compile-time |

### Target Triple Mapping (Node.js → Canonical)

```
process.arch    → Canonical
"ia32"          → "x86"
"x64"           → "x86_64"
"arm"           → "armv7a" or "arm"
"arm64"         → "aarch64"

process.platform → Canonical
"win32"          → "windows"
"darwin"         → "macos" or "darwin"
"linux"          → "linux"
```

### Binary Execution Patterns

| Platform | Executable | Permissions | Shell Semantics |
|---|---|---|---|
| Unix | No extension | `chmod +x` required | `$SHELL -i -c '...'` for env activation |
| Windows | `.exe` extension | Not needed | Direct execution or `.cmd` wrapper |
| macOS | No extension | `chmod +x` required | Same as Unix but may need notarization |

### Installation Strategy Selection

From the case studies, there are 5 strategies, applicable based on how the upstream tool is distributed:

1. **Pre-compiled binary download** (GitHub Releases, custom CDN) — most common for Rust/C/C++/Go tools
2. **Language package manager** (`npm install`, `pip install`, `cargo install`, `go install`, `gem install`) — requires runtime
3. **Bundled in package** (binary shipped with the extension/package itself) — zero network, but large packages
4. **Build from source** (`git clone` + build) — most fragile, last resort
5. **System PATH lookup** — fallback, no management

### Signature Verification

- vscode-zig: Minisign (Ed25519 via libsodium) — lightweight, modern
- Mason: Relies on HTTPS + GitHub's integrity
- VS Code marketplace: VSIX signing

### Version Management Patterns

- **Semantic versioning** with compatibility constraints (vscode-go's Go version matrix)
- **LRU cleanup** of old versions (vscode-zig keeps max 5)
- **Daily update checks** with rate limiting (vscode-ruby-lsp)
- **Version pinning** from project files (`.zigversion`, `rust-toolchain.toml`)

### Error Handling Patterns

- **Declined installs tracking** — don't re-prompt for tools the user explicitly declined (vscode-go)
- **Graceful degradation** — continue without the tool if installation fails (vscode-ruby-lsp)
- **Mirror failover** — try random mirrors before canonical URL (vscode-zig)
- **Cached responses** — use cached version data on network failure (vscode-zig)
- **NixOS special-casing** — patch ELF interpreter for pre-compiled binaries (rust-analyzer)
