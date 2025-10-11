# diffscape

[![Rust](https://img.shields.io/badge/rust-1.85+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A fast, beautiful Terminal User Interface (TUI) for viewing git diffs with syntax highlighting and intuitive navigation.

## Overview

`diffscape` provides a modern, efficient way to review git changes directly in your terminal. With support for syntax highlighting, side-by-side comparisons, and responsive layouts, it makes code review a pleasant experience.

## Features

- **File List**: Shows all changed files with their status (M=Modified, A=Added, D=Deleted)
- **Diff Display**: Shows diff content with syntax highlighting for added/removed lines
- **Syntax Highlighting**: Full syntax highlighting support for all programming languages using syntect
- **Responsive Layout**: Automatically switches between unified and side-by-side diff views based on terminal width
- **Keyboard Navigation**: Navigate between files and scroll through diff content

## Usage

```bash
# Show diff of current working directory
diffscape

# Show diff between commits
diffscape HEAD~1

# Show diff between branches
diffscape main..feature-branch

# Show staged changes
diffscape -- --cached
```

## Keyboard Shortcuts

- `q` - Quit the application
- `j`/`↓` - Move to next file
- `k`/`↑` - Move to previous file
- `d`/`Page Down` - Scroll down in diff (10 lines)
- `u`/`Page Up` - Scroll up in diff (10 lines)
- `g` - Go to top of current file
- `G` - Go to bottom of current file
- `s` - Toggle between side-by-side and unified diff view (when terminal is wide enough)
- `h` - Hide/show keyboard shortcuts footer

## Layout

- **Left Panel**: File list with status indicators
- **Right Panel**: Diff content
  - **Unified View**: Traditional diff format (default for narrow terminals)
  - **Side-by-Side View**: Old and new versions side by side (available for terminals ≥120 columns)
- **Bottom Footer**: Toggleable keyboard shortcuts reference (press `h` to hide/show)

### From Source

```bash
git clone https://github.com/dbradf/diffscape.git
cd diffscape
cargo build --release
./target/release/diffscape
```

### Prerequisites

- Rust 1.85 or higher
- Git installed and available in PATH
