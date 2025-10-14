# 🌊 Tide

**An opinionated macOS maintenance orchestrator with an `iocraft`-powered interface.**

Tide coordinates macOS software updates, Homebrew cleanups, and any custom shell tasks you describe in TOML. The new UI layer is rendered with [`iocraft`](https://crates.io/crates/iocraft), so every run delivers consistent colors, typography, and layout without hand-rolled ANSI escape codes.

## Contents

- [Highlights](#highlights)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Configuration](#configuration)
- [Examples](#examples)
- [UI Tour](#ui-tour)
- [Development](#development)
- [License](#license)

## Highlights

### Automation

- **Concurrent or sequential execution** – Flag a group as parallel and Tide fans out workers while respecting global limits.
- **Smart preconditions** – Skip tasks when binaries or paths are missing instead of failing your whole run.
- **Keychain-aware sudo** – Refresh authentication automatically and optionally store credentials in the macOS Keychain.
- **Async core** – Built on Tokio to keep prompts responsive while commands execute.
- **Declarative config** – TOML groups capture commands, timeouts, environment overrides, and conditional checks.

### Interface (powered by `iocraft`)

- **Consistent theming** – All banners, headings, and summaries are rendered by `iocraft`, keeping colors and typography uniform.
- **Modern progress spinners** – Unicode dot spinners decorate every task with group context and live status updates.
- **Detailed summary** – Color-coded output highlights successes, skips, failures, and the longest-running task.
- **Context cards** – Optional system stats and weather reports render in matching `iocraft` layouts without blocking completion.

### Safety

- Dry-run mode to preview commands with zero side effects.
- Optional fail-fast behaviour that halts optional work after a required task fails.
- Verbose logging for debugging plus quiet mode for automation owners.

## Requirements

- macOS (tested on Apple Silicon; Intel should work as long as the commands you call are available).
- Rust 1.76+ to build from source.
- Any tooling you invoke in your configuration (Homebrew, `mas`, `rustup`, etc.).

## Installation

```bash
git clone https://github.com/markussommer/tide
cd tide

cargo build --release
sudo install -m755 target/release/tide /usr/local/bin/tide
```

Remove the installed binary to uninstall.

## Quick Start

```bash
tide --init          # Scaffold ~/.config/tide/config.toml
tide --list          # Inspect groups and tasks with styled output
tide                 # Run interactively with confirmations
tide --dry-run       # Preview without executing commands
tide --force         # Skip prompts for unattended automation
```

## Usage

Core CLI options:

- `--groups <A,B>` – Only run the listed groups.
- `--skip-groups <A,B>` – Exclude specific groups.
- `--parallel <N>` – Override the global worker limit (default 4).
- `--quiet` – Suppress banner, system info, and weather.
- `--verbose` – Print task descriptions and full command lines.
- `--dry-run` – Simulate all tasks without side effects.
- `--force` – Skip the interactive confirmation step.

Example workflow:

```bash
tide --groups "System Updates,Homebrew" --parallel 6 --force
```

## Configuration

Tide reads `~/.config/tide/config.toml` by default (override with `--config`). Generate a starter file with `tide --init`, then tailor it. At a high level:

```toml
[settings]
show_banner = true
show_weather = true
show_system_info = true
show_progress = true
parallel_execution = false
parallel_limit = 4
skip_optional_on_error = false
keychain_label = "tide-sudo"
verbose = false
log_file = ""                  # Optional: capture command output

[[groups]]
name = "System Updates"
icon = "🍎"
description = "Core macOS updates"
enabled = true
parallel = false

  [[groups.tasks]]
  name = "macOS Updates"
  icon = "🍎"
  command = ["softwareupdate", "--install", "--all"]
  description = "Install all available macOS updates"
  required = true
  sudo = true
  check_command = "softwareupdate"
  timeout = 3600

  [[groups.tasks]]
  name = "App Store"
  icon = "🏬"
  command = ["mas", "upgrade"]
  required = true
  check_command = "mas"
  timeout = 600
```

### Task Fields

- `command` – Array form prevents shell quoting issues.
- `required` – When true, Tide marks the run as failed if the task fails.
- `sudo` – Tide handles authentication and optional Keychain storage.
- `enabled` – Toggle tasks on/off without deleting them.
- `check_command` / `check_path` – Skip tasks automatically when prerequisites are missing.
- `timeout` – Abort long-running commands (seconds).
- `env` – Command-specific environment overrides.
- `working_dir` – Set the working directory (supports `~`).

## Examples

Parallel developer tooling refresh:

```toml
[[groups]]
name = "Development Tools"
icon = "🛠️"
description = "Update core developer toolchains"
parallel = true

  [[groups.tasks]]
  name = "Rust Toolchain"
  icon = "🦀"
  command = ["rustup", "update"]
  check_command = "rustup"

  [[groups.tasks]]
  name = "Node.js"
  icon = "🟢"
  command = ["fnm", "install", "--lts"]
  check_command = "fnm"

  [[groups.tasks]]
  name = "Python"
  icon = "🐍"
  command = ["pyenv", "install", "3.13:latest"]
  check_command = "pyenv"
```

Conditional cleanup:

```toml
[[groups.tasks]]
name = "Clean Old Logs"
icon = "🧹"
command = ["find", "~/logs", "-mtime", "+30", "-delete"]
required = false
check_path = "~/logs"
timeout = 60
```

## UI Tour

1. **Banner** – Rendered by `iocraft`, showing the compiled version and consistent cyan theming.
2. **Progress** – Dot spinners display `[Group ▸ Task]` with colored status icons and elapsed time.
3. **Summary Table** – Styled rows outline successes, skips, failures, and highlight the longest task.
4. **Context Cards** – Optional system and weather sections reuse the same `iocraft` primitives for cohesive output.

Because the UI is declarative, future layout tweaks stay isolated inside the `ui` module—no more scattered ANSI formatting.

## Development

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

The spinner UI relies on `iocraft` for formatting; changes to output should go through the helpers in `src/ui.rs`.

## License

Tide is released under the MIT License. See [LICENSE](LICENSE) for details.
