# mould

mould is a modern Terminal User Interface (TUI) tool designed for interactively generating and editing configuration files from templates. Whether you are setting up a `.env` file from an example, creating a `docker-compose.override.yml`, or editing nested `JSON`, `YAML`, or `TOML` configurations, mould provides a fast, Vim-inspired workflow to get your environment ready.

## Features

- **Universal Format Support**: Handle `.env`, `JSON`, `YAML`, and `TOML` seamlessly.
- **Hierarchical Flattening**: Edit nested data structures (JSON, YAML, TOML) in a flat, searchable list.
- **Docker Compose Integration**: Automatically generate `docker-compose.override.yml` from `docker-compose.yml`.
- **Vim-inspired Workflow**: Navigate with `j`/`k`, edit with `i`, and save with `:w`.
- **Modern UI**: A polished, rounded interface featuring the Catppuccin Mocha palette.
- **Highly Configurable**: Customize keybindings and themes via a simple TOML configuration.
- **Dynamic Alignment**: Automatically aligns keys and values for perfect vertical readability.
- **Default Value Visibility**: Keep track of original template values while editing.
- **Incremental Merging**: Load existing values from an output file to continue where you left off.

## Installation

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

The binary will be installed as `mould`.

## Usage

Provide an input template file to start editing:

```sh
mould .env.example
mould docker-compose.yml
mould config.template.json -o config.json
```

### Keybindings (Default)

- **Normal Mode**
  - `j` / `Down`: Move selection down
  - `k` / `Up`: Move selection up
  - `i`: Edit the value of the currently selected key (Enter Insert Mode)
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

[theme]
# Enable transparency to let your terminal background show through
transparent = false

# Custom color palette (Catppuccin Mocha defaults)
crust = "#11111b"
surface0 = "#313244"
surface1 = "#45475a"
text = "#cdd6f4"
blue = "#89b4fa"
green = "#a6e3a1"
lavender = "#b4befe"
mauve = "#cba6f7"
peach = "#fab387"
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
