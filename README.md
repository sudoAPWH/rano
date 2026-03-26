# rano

A from-scratch rewrite of [GNU nano](https://www.nano-editor.org/) in Rust.

## What is this?

rano is a terminal text editor that aims to replicate the behavior and feel of GNU nano 8.7, built entirely in Rust. It uses **crossterm** for terminal I/O instead of ncurses, making it portable across platforms without C dependencies.

## Features

- **Full text editing** — character insertion, deletion, word/line operations, autoindent
- **Cut, copy, paste** — line and region-based, with mark support
- **Search and replace** — forward/backward, case-sensitive/insensitive, regex support
- **Go to line/column** — jump to any position in the file
- **Undo/redo** — full undo history
- **Multiple buffers** — open and switch between files
- **Mouse scroll** — scroll wheel support out of the box
- **File format detection** — Unix, DOS, and Mac line endings
- **Backup files** — optional backup on save
- **Help viewer** — built-in keybinding reference (`^G`)
- **RC file support** — reads `.nanorc` configuration
- **Syntax highlighting** — framework in place (behind feature flag)
- **File browser** — directory navigation (behind feature flag)
- **Search/position history** — persistent across sessions (behind feature flag)

## Building

Requires Rust 1.70+.

```sh
cargo build --release
```

The binary will be at `target/release/rano`.

## Installation

```sh
cargo build --release
sudo cp target/release/rano /usr/local/bin/
```

Or install directly with Cargo:

```sh
cargo install --path .
```

## Usage

```sh
rano                    # Open an empty buffer
rano file.txt           # Edit a file
rano -l file.txt        # Show line numbers
rano -i file.txt        # Enable autoindent
rano -E -T 4 file.txt   # Use 4 spaces instead of tabs
```

### Key bindings

| Shortcut | Action |
|----------|--------|
| `^O` | Save (Write Out) |
| `^S` | Save (quick) |
| `^X` | Exit |
| `^W` | Search |
| `^\` | Search and replace |
| `^K` | Cut line |
| `^U` | Paste |
| `^6` | Set mark |
| `^Z` | Undo |
| `^Y` | Redo |
| `^G` | Help |
| `^_` | Go to line |
| `^T` | Suspend |

### Command-line options

```
-l, --linenumbers       Show line numbers
-m, --mouse             Enable click-to-position
-c, --constantshow      Always show cursor position
-i, --autoindent        Auto-indent new lines
-S, --softwrap          Soft-wrap long lines
-T, --tabsize <N>       Set tab width (default: 8)
-E, --tabstospaces      Insert spaces instead of tabs
-w, --nowrap            Disable line wrapping
-x, --nohelp            Hide the shortcut bar
-D, --boldtext          Use bold instead of reverse video
-v, --view              Read-only mode
-B, --backup            Create backup files on save
-L, --nonewlines        Don't add trailing newline
-I, --ignorercfiles     Skip .nanorc files
-Y, --syntax <NAME>     Force a syntax definition
```

## Architecture

The codebase is organized into modules that mirror nano's source files:

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, CLI argument parsing (clap) |
| `definitions.rs` | Core types, enums, bitflags, data structures |
| `editor.rs` | Central `Editor` struct, main loop, key dispatch |
| `winio.rs` | Terminal I/O via crossterm, screen drawing |
| `text.rs` | Text editing operations, undo/redo |
| `movement.rs` | Cursor movement |
| `search.rs` | Search, replace, go-to-line |
| `cut.rs` | Cut, copy, paste |
| `prompt.rs` | Prompt bar and yes/no dialogs |
| `files.rs` | File I/O, open, save, exit |
| `global.rs` | Keybinding table, function table |
| `chars.rs` | Character/string utilities, Unicode width |
| `utils.rs` | Path utilities, number parsing |
| `color.rs` | Syntax highlighting (feature-gated) |
| `rcfile.rs` | RC file parsing |
| `help.rs` | Help viewer |
| `history.rs` | Search/position history (feature-gated) |
| `nano.rs` | Resize handling, word chopping, line/word/char counting |
| `browser.rs` | File browser (feature-gated) |

### Key design decisions

- **`Vec<Line>` instead of linked lists** — nano uses doubly-linked lists for lines; rano uses indexed vectors for cache-friendly access and simpler code.
- **`Editor` struct instead of globals** — all mutable state lives in a single `Editor` struct rather than C-style global variables.
- **crossterm instead of ncurses** — no C dependencies, works on macOS/Linux/Windows.
- **`bitflags` for flags** — type-safe flag sets instead of `#define` bit masks.
- **Cargo features mirror `#ifdef`** — nano's compile-time feature flags map to Cargo features.

## License

This project is a clean rewrite inspired by GNU nano. It does not contain any code from the original GNU nano project.
