# Fresher

<p align="center">
  <img src="docs/fresher-logo.png" alt="Fresher Logo" width="200">
</p>

<p align="center">
  <strong>A fork of <a href="https://github.com/sinelaw/fresh">Fresh</a> — a modern, full-featured terminal text editor with zero configuration.</strong>
</p>

<p align="center">
  Familiar keybindings, mouse support, and IDE-level features — no learning curve required.
</p>

---

## About

Fresher is a community fork of [Fresh](https://github.com/sinelaw/fresh), keeping the core editor synchronized with upstream while adding small quality-of-life improvements.

### What's different from upstream

| Customization | Description |
|---|---|
| **Gruvbox folder colors** | File explorer folders and breadcrumbs use `editor_fg` instead of `syntax_keyword` for better visibility with the Gruvbox theme |
| **Update checker** | Points to [nicolaslima/fresher](https://github.com/nicolaslima/fresher) releases instead of upstream |
| **`fresher-update` script** | Simple install/update script with platform detection and a `fresher` alias symlink |
| **Simplified CI** | Stripped-down GitHub Actions — just build, release, and upstream sync. No publishing to package managers |

Everything else is identical to upstream. Syncing with Fresh is automated via [sync-upstream.yml](.github/workflows/sync-upstream.yml).

---

## Quick Install (Fresher)

```bash
curl -sL https://raw.githubusercontent.com/nicolaslima/fresher/master/scripts/fresher-update | bash
```

This installs the `fresh` binary to `~/.local/bin/` and creates a `fresher` alias symlink, so you can run either `fresh` or `fresher` from your terminal.

**Requires:** `gh` CLI ([GitHub CLI](https://cli.github.com/)) installed and authenticated.

---

## Features

Fresh brings the intuitive UX of VS Code and Sublime Text to the terminal. Standard keybindings, full mouse support, menus, and a command palette — everything works the way you'd expect, right out of the box.

| Category | Features |
|----------|----------|
| **File Management** | open/save/new/close, file explorer, tabs, auto-revert, git file finder |
| **Editing** | undo/redo, multi-cursor, block selection, smart indent, comments, clipboard |
| **Search & Replace** | incremental search, find in selection, query replace, git grep |
| **Navigation** | go to line/bracket, word movement, position history, bookmarks, error navigation |
| **Views & Layout** | split panes, line numbers, line wrap, backgrounds, markdown preview |
| **Language Server (LSP)** | go to definition, references, hover, code actions, rename, diagnostics, autocompletion |
| **Productivity** | command palette, menu bar, keyboard macros, git log, diagnostics panel |
| **Extensibility** | TypeScript plugins (sandboxed QuickJS), color highlighter, TODO highlighter, merge conflicts, path complete, keymaps |
| **Internationalization** | Multiple language support, plugin translation system |

### Command Palette & Fuzzy Finder

![Command Palette](docs/blog/productivity/command-palette/showcase.gif)

### Multi-Cursor Editing

![Multi-Cursor](docs/blog/editing/multi-cursor/showcase.gif)

### Themes & Customization

![Select Theme](docs/blog/themes/select-theme/showcase.gif)

---

## Installation (from Fresh upstream)

If you prefer to install from the original Fresh project:

| Platform | Method |
|----------|--------|
| macOS | `brew install fresh-editor` |
| Windows | `winget install fresh-editor` |
| Arch Linux | [AUR](https://aur.archlinux.org/packages/fresh-editor-bin) |
| Debian/Ubuntu | [.deb from releases](https://github.com/sinelaw/fresh/releases) |
| Fedora/RHEL | [.rpm from releases](https://github.com/sinelaw/fresh/releases) |
| npm | `npm install -g @fresh-editor/fresh-editor` |
| Nix | `nix run github:sinelaw/fresh` |
| From source | `cargo install --locked fresh-editor` |

See the [Fresh README](https://github.com/sinelaw/fresh) for full installation instructions.

---

## Documentation

- [User Guide](https://getfresh.dev/docs)
- [macOS Tips](https://getfresh.dev/docs/configuration/keyboard#macos-terminal-tips)
- [Plugin Development](https://getfresh.dev/docs/plugins/development)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Privacy

Fresh checks for new versions daily to notify you of available upgrades. Alongside this, it sends basic anonymous telemetry (version, OS/architecture, terminal type) to help understand usage patterns. No personal data or file contents are collected.

To disable both upgrade checks and telemetry, use `--no-upgrade-check` or set `check_for_updates: false` in your config.

## License

Copyright (c) Noam Lewis

This project is licensed under the GNU General Public License v2.0 (GPL-2.0).

---

<p align="center">
  <img src="art/ink-and-glitch.png" alt="Fresher — ink-and-glitch" width="600">
</p>
