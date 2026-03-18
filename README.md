# mould

mould is a modern Terminal User Interface (TUI) tool designed for interactively generating and editing configuration files from templates. Whether you are setting up a `.env` file from an example, creating a `docker-compose.override.yml`, or editing nested `JSON`, `YAML`, `TOML`, `XML`, `INI`, or `Properties` configurations, mould provides a fast, Vim-inspired workflow to get your environment ready.

## Features

- **Universal Format Support**: Read, edit, and write `.env`, `JSON`, `YAML`, `TOML`, `XML`, `INI`, and `Properties` files seamlessly.
- **Tree View Navigation**: Edit nested data structures in a beautiful, depth-colored tree view.
- **Smart Template Discovery**: Rule-based resolver automatically discovers relationships (e.g., `.env.example` vs `.env`, `config.template.properties` vs `config.properties`) and highlights missing keys.
- **Strict Type Preservation**: Intelligently preserves data types (integers, booleans, strings) during edit-save cycles, ensuring your configuration stays valid.
- **Add Missing Keys**: Instantly pull missing keys and their default values from your template into your active configuration with a single keystroke.
- **Advanced Undo/Redo Engine**: Features a tree-based undo/redo history that ensures you never lose changes during complex branching edits.
- **Vim-inspired Workflow**: Navigate with `j`/`k`, `gg`/`G`, edit with `i`, search with `/`, and save with `:w`.
- **Modern UI**: A polished, rounded interface featuring a semantic Catppuccin Mocha palette with support for terminal transparency.
- **Highly Configurable**: Customize keybindings and semantic themes via a simple TOML user configuration.
- **Neovim Integration**: Comes with a built-in Neovim plugin for seamless in-editor configuration management.

---

## Installation

### 1. CLI Application

Ensure you have Rust and Cargo installed, then run:

```sh
# Install directly from the local path
cargo install --path .
```

Alternatively, you can build from source:

```sh
git clone https://git.narl.io/nvrl/mould-rs
cd mould-rs
cargo build --release
# The executable will be located in target/release/mould
```

### 2. Neovim Plugin

If you want to use `mould` directly inside Neovim, the repository includes a built-in Lua plugin that opens `mould` in a floating terminal buffer and synchronizes the file upon exit.

**Using `mini.deps`:**
```lua
add({
	source = 'https://git.narl.io/nvrl/mould-rs',
	checkout = 'main',
})
```

---

## Usage

Provide an input template file to start editing. `mould` is smart enough to figure out if it's looking at a template or an active file, and will search for its counterpart to provide live diffing.

```sh
mould .env.example
mould docker-compose.yml
mould config.template.json -o config.json
```

If you just run `mould` in a directory, it will automatically look for common template patterns like `.env.example` or `docker-compose.yml`.

### Inside Neovim

Open any configuration file in Neovim and run `:Mould`. It will launch a floating window targeting your active buffer.

### Keybindings (Default)

- **Normal Mode**
  - `j` / `Down`: Move selection down
  - `k` / `Up`: Move selection up
  - `gg`: Jump to the top
  - `G`: Jump to the bottom
  - `i`: Edit value (cursor at start)
  - `a`: Edit value (cursor at end)
  - `s`: Substitute value (clear and edit)
  - `r`: Rename the current key
  - `o`: Append a new item (as a sibling or array element)
  - `O`: Prepend a new item
  - `alt+o` / `alt+O`: Append/Prepend a new group/object
  - `t`: Toggle between group/object and standard value
  - `dd`: Delete the currently selected variable or group (removes all nested children)
  - `u`: Undo the last change
  - `U`: Redo the reverted change
  - `a`: Add missing value from template to active config
  - `/`: Search for configuration keys
  - `n` / `N`: Jump to next / previous search match
  - `:w` or `Enter`: Save the current configuration
  - `:q` or `q`: Quit the application
  - `:wq`: Save and quit
  - `Esc`: Clear current command prompt

- **Insert / Rename Modes**
  - Type your value/key string.
  - Arrow keys: Navigate within the input field
  - `Enter`: Commit the value and return to Normal Mode
  - `Esc`: Cancel edits and return to Normal Mode

---

## Configuration

`mould` can be configured using a `config.toml` file located in your user configuration directory:
- **Linux/macOS**: `~/.config/mould/config.toml`
- **Windows**: `%AppData%\mould\config.toml`

Example configuration:

```toml
[theme]
transparent = false
bg_normal = "#1e1e2e"
bg_highlight = "#89b4fa"
bg_active = "#a6e3a1"
bg_search = "#cba6f7"
fg_normal = "#cdd6f4"
fg_dimmed = "#6c7086"
fg_highlight = "#1e1e2e"
fg_warning = "#f38ba8"
fg_modified = "#fab387"
fg_accent = "#b4befe"
border_normal = "#45475a"
border_active = "#a6e3a1"
tree_depth_1 = "#b4befe"
tree_depth_2 = "#cba6f7"
tree_depth_3 = "#89b4fa"
tree_depth_4 = "#fab387"

[keybinds]
down = "j"
up = "k"
edit = "i"
edit_append = "a"
edit_substitute = "s"
save = ":w"
quit = ":q"
normal_mode = "Esc"
search = "/"
next_match = "n"
previous_match = "N"
jump_top = "gg"
jump_bottom = "G"
append_item = "o"
prepend_item = "O"
delete_item = "dd"
undo = "u"
redo = "U"
rename = "r"
append_group = "alt+o"
prepend_group = "alt+O"
toggle_group = "t"
```

---

## Development & Architecture

`mould` is written in Rust and architected to decouple the file format parsing from the UI representation. This allows the TUI to render complex, nested configuration files in a unified tree-view.

### Core Architecture

1. **State Management & Undo Tree (`src/app.rs` & `src/undo.rs`)**
   - The central state is maintained in the `App` struct, which tracks the currently loaded configuration variables, application modes (`Normal`, `Insert`, `InsertKey`, `Search`), and user input buffer.
   - It integrates an **UndoTree**, providing non-linear, branching history tracking for complex edits (additions, deletions, nested renaming). 

2. **Unified Data Model (`src/format/mod.rs`)**
   - Regardless of the underlying format, all data is translated into a flattened `Vec<ConfigItem>`.
   - A `ConfigItem` contains its structural path (`Vec<PathSegment>` handling nested Object Keys and Array Indices), values, type metadata (`ValueType`), and template comparison statuses (e.g., `MissingFromActive`).

3. **Format Handlers (`src/format/*`)**
   - **`env.rs` & `properties.rs`**: Handlers for flat key-value files.
   - **`hierarchical.rs`**: A generalized processor leveraging `serde_json::Value` as an intermediary layer to parse and write nested `JSON`, `YAML`, `TOML`, and even `XML` (via `quick-xml` transposition).
   - **`ini.rs`**: Handles traditional `[Section]` based INI configurations.

4. **Template Resolver (`src/resolver.rs`)**
   - Automatically determines structural pairings without explicit instruction. 
   - Uses hardcoded exact rules (e.g., `compose.yml` -> `compose.override.yml`) and generic fallback rules to strip `.example` or `.template` suffixes to find target output files dynamically.

5. **Terminal UI & Event Loop (`src/ui.rs` & `src/runner.rs`)**
   - **UI Rendering**: Powered by `ratatui`. Renders a conditional side-by-side or vertical layout consisting of a styled hierarchical List, an active Input field, and a status bar indicating keybind availability.
   - **Event Runner**: Powered by `crossterm`. Intercepts keystrokes, resolves sequences (like `dd` or `gg`), delegates to the `tui-input` backend during active editing, and interacts with the internal API to mutate the configuration tree.

6. **Neovim Plugin (`lua/mould/init.lua`)**
   - Implements a Lua wrapper that calculates terminal geometries and launches the CLI `mould` binary inside an ephemeral, floating terminal buffer, triggering automatic Neovim `checktime` syncs on exit.
