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
name = "catppuccin_mocha"
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
