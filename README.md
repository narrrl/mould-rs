# cenv-rs

cenv-rs is a Rust-based Terminal User Interface (TUI) tool designed to help developers interactively generate `.env` files from `.env.example` templates. With a focus on speed and usability, it features Vim-like keybindings and out-of-the-box support for theming, defaulting to the Catppuccin Mocha palette.

## Features

- Parse `.env.example` files to extract keys, default values, and comments.
- Vim-like keybindings for quick navigation and editing.
- Built-in theming support with Catppuccin Mocha as the default.
- Configurable through a standard TOML file.

## Installation

Ensure you have Rust and Cargo installed, then run:

```sh
cargo install --path .
```

Alternatively, you can build from source:

```sh
git clone <repository_url>
cd cenv-rs
cargo build --release
```

## Usage

Navigate to a directory containing a `.env.example` file and run:

```sh
cenv-rs
```

### Keybindings

- **Normal Mode**
  - `j` / `Down`: Move selection down
  - `k` / `Up`: Move selection up
  - `i`: Edit the value of the currently selected key (Enter Insert Mode)
  - `:w` or `Enter`: Save the current configuration to `.env`
  - `q` or `:q`: Quit the application without saving
  - `Esc`: Clear current prompt or return from actions

- **Insert Mode**
  - Type your value for the selected key.
  - `Esc`: Return to Normal Mode

## Configuration

cenv-rs can be configured using a `config.toml` file located in your user configuration directory (e.g., `~/.config/cenv-rs/config.toml` on Linux/macOS).

Example configuration:

```toml
[theme]
# Default theme is "catppuccin_mocha"
name = "catppuccin_mocha"
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
