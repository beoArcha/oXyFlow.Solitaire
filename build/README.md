# Build and install scripts

The scripts in this directory check build dependencies before compiling `oxysol` from the `release` branch into `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows).

- [`install.sh`](install.sh) — Linux (`apt`, `dnf`, `pacman`, `zypper`) and macOS (Xcode Command Line Tools, Homebrew if already present). POSIX `sh`.
- [`install.ps1`](install.ps1) — Windows (PowerShell 5.1+ and `winget`).

## Modes

- `--check` — stops at the first missing dependency with a non-zero exit code.
- `--list` — lists every missing dependency and the exact install command for this system.
- `--install` — asks before installing missing dependencies via the system package manager, re-checks, and builds and installs only if the second check passes.
- `--no-audio` — skips the ALSA/`pkg-config` check on Linux and builds with `--no-default-features`.

Without a mode flag, an interactive terminal asks which mode to use; a non-interactive run behaves like `--list`.

## One-line usage

Linux / macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

Windows (PowerShell):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Replace `--list` with `--install` to install dependencies and build.
