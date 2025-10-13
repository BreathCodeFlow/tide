# 🌊 Tide v1.0

**Refresh your system with the update wave**

Ein modernes, hochperformantes macOS System Update Tool mit paralleler Ausführung, Progress-Tracking und umfangreicher Konfiguration.

> 🌊 Wie die Gezeiten kommt Tide regelmäßig, erfrischt dein System und hält alles aktuell - automatisch, zuverlässig und elegant.

## ✨ Features

### Core Features

- **📦 Parallele Task-Ausführung** - Führe unabhängige Tasks gleichzeitig aus
- **📊 Live Progress Bars** - Visuelles Feedback mit `indicatif`
- **🎨 Beautiful CLI** - Farbige Ausgabe mit `colored` und `crossterm`
- **🔐 Keychain Integration** - Sichere Passwort-Speicherung
- **📝 TOML Configuration** - Übersichtliche, erweiterbare Config
- **🏃 Async/Await** - Moderne async Rust mit `tokio`
- **⚡ Smart Preconditions** - Tasks nur ausführen wenn nötig
- **🛡️ Robust Error Handling** - Mit `anyhow` und `thiserror`

### CLI Features

- **Dry-Run Mode** - Teste ohne Änderungen
- **Group Filtering** - Führe nur bestimmte Gruppen aus
- **Task Listing** - Zeige alle konfigurierten Tasks
- **Verbose Mode** - Detaillierte Ausgabe
- **Force Mode** - Überspringe Bestätigungen

## 📦 Dependencies

Das Script nutzt moderne Rust-Libraries für maximale Performance und Wartbarkeit:

```toml
clap = "4.5"          # CLI argument parsing
serde = "1.0"         # Serialization
toml = "0.8"          # Config format
anyhow = "1.0"        # Error handling
thiserror = "1.0"     # Custom errors
dirs = "5.0"          # Platform directories
which = "6.0"         # Command detection
shellexpand = "3.1"   # Path expansion
colored = "2.1"       # Terminal colors
indicatif = "0.17"    # Progress bars
dialoguer = "0.11"    # Interactive prompts
crossterm = "0.28"    # Terminal control
tokio = "1.41"        # Async runtime
chrono = "0.4"        # Date/time
reqwest = "0.12"      # HTTP client
```

## 🚀 Installation

### From Source

```bash
# Clone repository
git clone https://github.com/markussommer/tide
cd tide

# Build release binary
cargo build --release

# Install to system
sudo cp target/release/tide /usr/local/bin/

# Config initialisieren
tide --init

# Config anpassen
nano ~/.config/tide/config.toml
```

### Quick Start

```bash
# Nach der Installation
tide --init          # Erstelle Konfiguration
tide --list          # Zeige verfügbare Tasks
tide                 # Führe Updates aus
```

## 📝 Usage

```bash
# Normale Ausführung
tide

# Dry-run (keine Änderungen)
tide --dry-run

# Nur bestimmte Gruppen
tide --groups "Homebrew,System Updates"

# Gruppen überspringen
tide --skip-groups "Developer Caches"

# Alle Tasks anzeigen
tide --list

# Verbose mit Details
tide --verbose --list

# Parallel mit 8 Workers
tide --parallel 8

# Quiet mode (minimal output)
tide --quiet

# Force ohne Bestätigung
tide --force
```

## ⚙️ Configuration

### Settings Section

```toml
[settings]
show_banner = true                  # ASCII banner anzeigen
show_weather = true                 # Wetter-Info am Ende
show_system_info = true             # System-Stats anzeigen
show_progress = true                # Progress bars anzeigen
parallel_execution = false          # Parallele Ausführung aktivieren
parallel_limit = 4                  # Max parallele Tasks
skip_optional_on_error = false      # Optionale Tasks bei Fehler überspringen
keychain_label = "tide-sudo"        # Keychain Label für sudo
use_colors = true                   # Farbige Ausgabe
verbose = false                     # Detaillierte Ausgabe
log_file = "/path/to/log.txt"       # Optional: Log-Datei
```

### Task Groups

```toml
[[groups]]
name = "Group Name"
icon = "🚀"
description = "Detailed description of this group"
enabled = true
parallel = true  # Tasks in dieser Gruppe parallel ausführen

  [[groups.tasks]]
  name = "Task Name"
  icon = "📦"                        # Optional: Override group icon
  description = "What this task does"
  command = ["cmd", "arg1", "arg2"]
  required = true                    # Fehler stoppt Ausführung
  sudo = true                        # Mit sudo ausführen
  enabled = true                     # Task aktiviert
  check_command = "brew"             # Nur wenn Command existiert
  check_path = "~/.config/file"      # Nur wenn Pfad existiert
  timeout = 300                      # Timeout in Sekunden
  working_dir = "~/projects"         # Working directory

  # Environment variables für diesen Task
  [groups.tasks.env]
  CUSTOM_VAR = "value"
  PATH = "/custom/path:$PATH"
```

## 🎯 Advanced Examples

### Parallel Development Tools Update

```toml
[[groups]]
name = "Development Tools"
icon = "🛠️"
description = "Update all development tools in parallel"
parallel = true  # Alle Tasks dieser Gruppe parallel

  [[groups.tasks]]
  name = "Rust Update"
  command = ["rustup", "update"]
  check_command = "rustup"

  [[groups.tasks]]
  name = "Node Update"
  command = ["fnm", "install", "--lts"]
  check_command = "fnm"

  [[groups.tasks]]
  name = "Python Update"
  command = ["pyenv", "install", "3.13:latest"]
  check_command = "pyenv"
```

### Conditional Cleanup Task

```toml
[[groups.tasks]]
name = "Clean Old Logs"
description = "Remove logs older than 30 days"
command = ["find", "~/logs", "-mtime", "+30", "-delete"]
required = false
check_path = "~/logs"  # Nur wenn logs Ordner existiert
timeout = 60
```

### Task with Custom Environment

```toml
[[groups.tasks]]
name = "Custom Build"
command = ["make", "build"]
working_dir = "~/myproject"

[groups.tasks.env]
CC = "clang"
CFLAGS = "-O3 -march=native"
BUILD_TYPE = "release"
```

## 🔥 Performance Tips

1. **Parallele Ausführung**: Aktiviere `parallel = true` für Gruppen mit unabhängigen Tasks
2. **Timeout setzen**: Verhindere hängende Tasks mit `timeout`
3. **Check Commands**: Nutze `check_command` um unnötige Tasks zu überspringen
4. **Dry Run first**: Teste mit `--dry-run` bevor du produktiv läufst

## 🎨 UI Features

- **Progress Bars**: Live-Updates für jeden Task
- **Color Coding**:
  - 🟢 Grün = Success
  - 🔴 Rot = Failed
  - 🟡 Gelb = Skipped
  - 🔵 Blau = Info
- **System Info Display**: Disk, Battery, macOS Version, Uptime
- **Weather Integration**: Optional weather display
- **Interactive Prompts**: Sichere Bestätigungen mit `dialoguer`

## 🔐 Security

- **Keychain Integration**: Sudo-Passwörter sicher in macOS Keychain
- **No Plain Text Passwords**: Niemals Passwörter in Config
- **Confirmation Prompts**: Bestätigung vor kritischen Operationen

## 🐛 Debugging

```bash
# Verbose output
tide --verbose

# List all tasks with details
tide --list --verbose

# Dry run to see what would happen
tide --dry-run

# Check specific group
tide --groups "Homebrew" --dry-run
```

## 📊 Task Status

- **Success ✓**: Task erfolgreich ausgeführt
- **Failed ✗**: Task fehlgeschlagen (stoppt bei `required = true`)
- **Skipped ○**: Task übersprungen (Bedingung nicht erfüllt oder optional)

## 🚀 Performance

Das neue System ist **deutlich schneller** als die Original-Version:

- **Parallele Ausführung** spart bis zu 70% Zeit
- **Async I/O** mit Tokio für non-blocking Operations
- **Smart Caching** der sudo-Authentifizierung
- **Conditional Checks** verhindern unnötige Ausführungen

## 💡 Pro Tips

1. **Gruppiere verwandte Tasks** für bessere Organisation
2. **Nutze parallele Gruppen** für unabhängige Tasks
3. **Setze Timeouts** für langlaufende Tasks
4. **Nutze check_command** für bedingte Ausführung
5. **Teste mit --dry-run** vor produktivem Einsatz
6. **Aktiviere verbose** für Debugging
7. **Nutze force mode** für Automation

---

**Built with ❤️ and Rust 🦀**
