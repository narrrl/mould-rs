# mould

mould is a modern Terminal User Interface (TUI) tool designed for interactively generating and editing configuration files from templates. Whether you are setting up a `.env` file from an example, creating a `docker-compose.override.yml`, or editing nested `JSON`, `YAML`, or `TOML` configurations, mould provides a fast, Vim-inspired workflow to get your environment ready.

## Features

- **Universal Format Support**: Handle `.env`, `JSON`, `YAML`, and `TOML` seamlessly.
- **Tree View Navigation**: Edit nested data structures (JSON, YAML, TOML) in a beautiful, depth-colored tree view.
- **Smart Template Comparison**: Automatically discovers `.env.example` vs `.env` relationships and highlights missing or modified keys.
- **Add Missing Keys**: Instantly pull missing keys and their default values from your template into your active configuration with a single keystroke.
- **Neovim Integration**: Comes with a built-in Neovim plugin for seamless in-editor configuration management.
- **Docker Compose Integration**: Automatically generate `docker-compose.override.yml` from `docker-compose.yml`.
- **Vim-inspired Workflow**: Navigate with `j`/`k`, `gg`/`G`, edit with `i`, search with `/`, and save with `:w`.
- **Modern UI**: A polished, rounded interface featuring a semantic Catppuccin Mocha palette.
- **Highly Configurable**: Customize keybindings and semantic themes via a simple TOML configuration.

## Installation

### CLI Application
Ensure you have Rust and Cargo installed, then run:

```sh
cargo install --path .
```

Alternatively, you can build from source:

```sh
git clone <repository_url>
cd mould
cargo build --release
```

### Neovim Plugin
If you use a plugin manager like `lazy.nvim`, you can add the local repository (or remote once published) directly:

```lua
{
  "username/mould", -- replace with actual repo path or github url
  config = function()
    -- Provides the :Mould command
  end
}
```

## Usage

Provide an input template file to start editing. `mould` is smart enough to figure out if it's looking at a template or an active file, and will search for its counterpart to provide diffing.

```sh
mould .env.example
mould docker-compose.yml
mould config.template.json -o config.json
```

### Keybindings (Default)

- **Normal Mode**
  - `j` / `Down`: Move selection down
  - `k` / `Up`: Move selection up
  - `gg`: Jump to the top
  - `G`: Jump to the bottom
  - `i`: Edit the value of the currently selected key (Enter Insert Mode)
  - `o`: Append a new item to the current array
  - `O`: Prepend a new item to the current array
  - `a`: Add missing value from template to active config
  - `/`: Search for configuration keys (Jump to matches)
  - `n`: Jump to the next search match
  - `N`: Jump to the previous search match
  - `:w` or `Enter`: Save the current configuration to the output file
  - `:q` or `q`: Quit the application
  - `:wq`: Save and quit
  - `Esc`: Clear current command prompt

- **Insert Mode**
  - Type your value for the selected key.
  - Arrow keys: Navigate within the input field
  - `Enter` / `Esc`: Commit the value and return to Normal Mode

## Configuration

mould can be configured using a `config.toml` file located in your user configuration directory (e.g., `~/.config/mould/config.toml` on Linux/macOS).

Example configuration:

```toml
[keybinds]
down = "j"
up = "k"
edit = "i"
save = ":w"
quit = ":q"
normal_mode = "Esc"
search = "/"
next_match = "n"
previous_match = "N"
jump_top = "gg"
jump_bottom = "G"

[theme]
# Enable transparency to let your terminal background show through
transparent = false

# Custom color palette (Semantic Catppuccin Mocha defaults)
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
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.