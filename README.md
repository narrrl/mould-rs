# mould

`mould` is a modern Terminal User Interface (TUI) tool designed for interactively generating and editing configuration files from templates. Whether you are setting up a `.env` file from an example, creating a `docker-compose.override.yml`, or editing nested `JSON`, `YAML`, `TOML`, `XML`, `INI`, or `Properties` configurations, `mould` provides a fast, Vim-inspired workflow to get your environment ready.

## Features

- **Universal Format Support**: Read, edit, and write `.env`, `JSON`, `YAML`, `TOML`, `XML`, `INI`, and `Properties` files seamlessly.
- **Tree View Navigation**: Edit nested data structures in a beautiful, depth-colored tree view.
- **Smart Template Discovery**: Rule-based resolver automatically discovers relationships (e.g., `.env.example` vs `.env`, `config.template.properties` vs `config.properties`) and highlights missing keys.
- **Strict Type Preservation**: Intelligently preserves data types (integers, booleans, strings) during edit-save cycles, ensuring your configuration stays valid.
- **Add Missing Keys**: Instantly pull missing keys and their default values from your template into your active configuration with a single keystroke.
- **Advanced Undo/Redo Engine**: Features a tree-based undo/redo history that ensures you never lose changes during complex branching edits. It properly tracks all modifications, including nested renames and item additions/deletions.
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
  - `i`: Edit value (cursor at start). If selected is a group, enters rename mode.
  - `a`: Edit value (cursor at end)
  - `s` / `S`: Substitute value (clear and edit)
  - `r`: Rename the current key or group. (Cannot rename array indices).
  - `o`: Append a new item (as a sibling or array element). Enters rename mode for non-array items.
  - `O`: Prepend a new item (as a sibling or array element). Enters rename mode for non-array items.
  - `alt+o`: Append a new group/object as a child. Enters rename mode for the new group.
  - `alt+O`: Prepend a new group/object as a child. Enters rename mode for the new group.
  - `t`: Toggle the selected item between a group/object and a standard variable.
  - `dd`: Delete the currently selected variable or group (removes all nested children).
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
  - `Enter`: Commit the value/key and return to Normal Mode. If renaming, checks for key collisions.
  - `Esc`: Cancel edits and return to Normal Mode. Reverts changes to the current field.

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

### Core Architectural Principles:

-   **Separation of Concerns**: Clear boundaries between UI rendering, application state, input handling, and file format logic.
-   **Unified Data Model**: All parsed configuration data is normalized into a single `Vec<ConfigItem>` structure, simplifying application logic across different file types.
-   **Vim-inspired Modality**: The application operates in distinct modes (`Normal`, `Insert`, `InsertKey`, `Search`), each with specific keybinding behaviors, enabling efficient interaction.
-   **Non-linear Undo/Redo**: A robust undo tree allows users to revert and re-apply changes across complex branching edit histories.

### Key Components

1.  **State Management & Undo Tree (`src/app.rs` & `src/undo.rs`)**
    *   The central state is maintained in the `App` struct, which tracks the currently loaded configuration variables, application modes, and user input buffer.
    *   It integrates an **UndoTree**, providing non-linear, branching history tracking for complex edits (additions, deletions, nested renaming). Each significant state change (`save_undo_state`) pushes a snapshot to this tree.

2.  **Unified Data Model (`src/format/mod.rs`)**
    *   Regardless of the underlying file format (JSON, YAML, .env, etc.), all data is translated into a flattened `Vec<ConfigItem>`.
    *   A `ConfigItem` represents a single configuration entry. It contains:
        *   `key`: The display key (e.g., `port` or `[0]`).
        *   `path`: A `Vec<PathSegment>` (composed of `PathSegment::Key(String)` for object keys and `PathSegment::Index(usize)` for array indices) that defines its full hierarchical location.
        *   `value`: `Option<String>` holding the actual value.
        *   `is_group`: A boolean indicating if this item is a structural node (object or array).
        *   `status`: (`ItemStatus::Present`, `MissingFromActive`, `Modified`) reflecting comparison with a template.
        *   `value_type`: (`ValueType::String`, `Number`, `Bool`, `Null`) to ensure type preservation during writes.

3.  **Format Handlers (`src/format/*`)**
    *   Each file format has a dedicated handler (`EnvHandler`, `IniHandler`, `HierarchicalHandler`, `PropertiesHandler`) implementing the `FormatHandler` trait.
    *   **`HierarchicalHandler`**: A generalized processor leveraging `serde_json::Value` as an intermediary layer to parse and write nested `JSON`, `YAML`, `TOML`, and even `XML` (via `quick-xml` transposition). This allows complex structures to be flattened for editing and then re-nested accurately.
    *   These handlers are responsible for translating between the file's native format and the `Vec<ConfigItem>` internal representation.

4.  **Template Resolver (`src/resolver.rs`)**
    *   Automatically determines template-active file pairings without explicit user instruction.
    *   Uses hardcoded exact rules (e.g., `compose.yml` -> `compose.override.yml`) and generic fallback rules to strip `.example` or `.template` suffixes to find target output files dynamically.

5.  **Terminal UI & Event Loop (`src/ui.rs` & `src/runner.rs`)**
    *   **UI Rendering (`src/ui.rs`)**: Powered by the `ratatui` library. Renders a flexible layout consisting of a styled hierarchical list, an active input field for editing, and a dynamic status bar.
    *   **Event Runner (`src/runner.rs`)**: Powered by `crossterm`. It intercepts raw keyboard events, resolves multi-key sequences (like `dd` or `gg`), delegates character input to the `tui-input` backend during active editing, and dispatches actions to mutate the `App` state. It includes logic to prevent "one-key-behind" issues and manage complex keybindings like `alt+o`.

6.  **Neovim Plugin (`lua/mould/init.lua`)**
    *   Implements a Lua wrapper that calculates terminal geometries and launches the CLI `mould` binary inside an ephemeral, floating terminal buffer, ensuring automatic Neovim `checktime` synchronization upon `mould`'s exit.

### Development Process Highlights:

-   **Iterative Refinement**: Features like key renaming, group creation, and advanced undo/redo were developed iteratively, responding to user feedback and progressively enhancing the core data model and interaction logic.
-   **Robust Error Handling**: Key functions (`commit_input`, `enter_insert_key`) include collision detection and validation to ensure data integrity during user modifications.
-   **Modality-driven Design**: The introduction of `InsertKey` mode and careful handling of `InsertVariant` demonstrates a commitment to a precise, context-aware user experience, minimizing ambiguity during editing.
-   **Debugging and Performance**: Issues like UI "hangs" were traced to subtle interactions in key event processing and input buffer management, leading to refactored key sequence logic for improved responsiveness.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
