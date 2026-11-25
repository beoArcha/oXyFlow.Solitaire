<h1 align="center">oXySol</h1>

<p align="center"><strong>Solitaire for your terminal.</strong><br>
Klondike, Spider, FreeCell, Pyramid and the Yukon family in one small program.</p>

<p align="center">
  <a href="https://github.com/beoArcha/oXyFlow.Solitaire/actions/workflows/main.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/beoArcha/oXyFlow.Solitaire/main.yml?branch=main&label=CI"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/beoArcha/oXyFlow.Solitaire"></a>
  <a href="https://github.com/beoArcha/oXyFlow.Solitaire/releases"><img alt="Latest pre-release" src="https://img.shields.io/github/v/release/beoArcha/oXyFlow.Solitaire?include_prereleases&label=latest%20build"></a>
  <a href="https://www.rust-lang.org/"><img alt="Rust 1.99" src="https://img.shields.io/badge/Rust-1.99-orange?logo=rust"></a>
  <a href="https://ratatui.rs/"><img alt="TUI: Ratatui" src="https://img.shields.io/badge/TUI-Ratatui%200.30-blueviolet"></a>
  <img alt="Languages" src="https://img.shields.io/badge/languages-9-green">
  <a href="https://beoarcha.github.io/oXyFlow.Solitaire/"><img alt="Website" src="https://img.shields.io/badge/website-9%20languages-blue"></a>
</p>

<p align="center">
  <img alt="Animation: the cursor picks up the ace of clubs and drops it on a foundation" src="docs/site/src/img/en/anim-move.svg" width="860">
</p>

## What it is

oXySol is a terminal solitaire with **Klondike, Spider, FreeCell, Pyramid and the Yukon family** (Yukon, Russian
Solitaire, Alaska, Moosehide and Australian Patience), shipped as the single binary `oxysol`. It is a pre-release
(version 0.5; the plan ends at 0.9): it plays, and it is still in development.

- **Game menu.** The games are listed under family headings; popular ones carry a star, favourites have their own
  list, and a search box finds a game by name.
- **Keyboard and mouse.** Every action works with both: click, double-click, drag, right-click to put a card back.
- **Help while you play.** A guide panel, a possible moves panel, hints, undo and redo, and auto-finish when the game
  is certainly won.
- **Seeds and winnable deals.** Every deal has a seed you can copy and replay, a game can start from any seed,
  and a new game can search for a deal that is proven winnable (a solver covers every game, the Yukon family included).
- **Statistics and history** of every game you play: wins without a restart, personal bests, a scroll bar, and a
  mark for each deal that was checked as winnable or not.
- **Restart from the beginning** in the pause menu (`Esc`) or with `Shift+R`, always confirmed; in a game without undo
  restarts are counted.
- **Card size you choose.** `]` and `[` make the cards bigger or smaller, `0` returns to the automatic size that fits
  the window; Ctrl+wheel and the Card size setting do the same.
- **Fits the Windows console.** Started by a double-click, the game opens full screen (Alt+Enter mode), or widens its
  window to 130 columns so the side panels fit. Started from an open console, it leaves the console alone. In the
  classic Windows console, Ctrl+Plus, Ctrl+Minus and Ctrl+0 change the console font size.
- **Card styles.** Settings has five front styles (International, French, German, Italian and Swiss) and two back styles
  (Lattice and Ornament).
- **Five themes:** Dark, Light, High contrast (for low vision), Monokai and Dracula. Color is never the only signal.
  Color-vision modes are planned, not built yet.
- **Nine languages.** English, Polish, German, French, Italian, Brazilian Portuguese, Ukrainian, Dutch and Spanish.
  Switch the language with the flag selector in the top-left corner of the main menu, or in Settings. On first start
  the game follows the system language.

## Screenshots

Every image is generated from the real game by `cargo xtask-art screenshots`.

<table>
  <tr>
    <td><img alt="Klondike" src="docs/site/src/img/en/klondike.svg"></td>
    <td><img alt="Spider" src="docs/site/src/img/en/spider.svg"></td>
  </tr>
  <tr>
    <td><img alt="FreeCell" src="docs/site/src/img/en/freecell.svg"></td>
    <td><img alt="Pyramid" src="docs/site/src/img/en/pyramid.svg"></td>
  </tr>
  <tr>
    <td><img alt="The High contrast theme" src="docs/site/src/img/en/theme-high-contrast.svg"></td>
    <td><img alt="The Light theme" src="docs/site/src/img/en/theme-light.svg"></td>
  </tr>
</table>

More screens, the animations and all five themes are on the [website](https://beoarcha.github.io/oXyFlow.Solitaire/gallery.html).

## Install

You need a terminal. Pick one way.

### Download a build

Get the newest build from the [releases page](https://github.com/beoArcha/oXyFlow.Solitaire/releases). Every build so
far is a pre-release, so look at the top of the list and not at "latest". Archives exist for Windows (x86-64), Linux
(x86-64 and ARM64) and macOS (Apple silicon). Unpack one and run `oxysol` (`oxysol.exe` on Windows). The Linux builds
have no sound. `SHA256SUMS` on the release lets you check a download.

### Build from source

You build the `release` branch, the last commit that passed the full check on Linux, Windows and macOS.

**Rust is required.** Install it with [rustup](https://rustup.rs). The toolchain version is pinned in
[`rust-toolchain.toml`](rust-toolchain.toml); rustup picks it up by itself on the first build. The build also compiles
a little C code (SQLite), so you need a C toolchain: on Windows the Visual Studio Build Tools with the "Desktop
development with C++" workload, on macOS the Xcode command line tools (`xcode-select --install`), on Linux `gcc` and,
for sound, the ALSA headers (`sudo apt install build-essential libasound2-dev`; without them add
`--no-default-features` to the build command).

**Windows (PowerShell).** Verified on Windows 10 on 2026-11-19.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

**Linux and macOS.** Not verified yet: written, not run.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

The program ends up in `target/release/` (`oxysol.exe` on Windows). Run it from there or copy it to a folder on your
`PATH`. The first build takes a few minutes.

**With `cargo install`.** One step instead of the download. Verified on Windows 10; not verified on Linux and macOS.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

This puts `oxysol` in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows), which rustup adds to your `PATH`.

## Usage

Start the game in a terminal that is at least 100 columns by 30 rows (it says so when it is too small):

```sh
oxysol
```

The panels beside the board need 130 columns; on narrower terminals `g` and `v` open them in a window.
The full key list is in the game: press `?`.

| Key | What it does |
|---|---|
| Arrows (or `k` and `j` for up and down) | Move the cursor |
| `Tab` and `Shift+Tab` | Move the focus to the next or previous area |
| `Enter` or `Space` | Pick up a card, then drop it on a target |
| `Esc` | Put the card back, or go back |
| `s` | Turn or deal from the stock |
| `a` | Quick move to a foundation |
| `+` and `-` | Pick more or fewer cards of a run |
| `]`, `[` and `0` | Bigger cards, smaller cards, automatic card size |
| `u` (`Ctrl+Z`) and `r` (`Ctrl+Y`) | Undo and redo |
| `h` | Hint |
| `f` | Finish the game: play out the remaining moves once the win is certain |
| `g` | Guide panel |
| `v` | Possible moves panel |
| `o` | Quick options for this game (hints, aura, panels, auto-finish) |
| `Shift+R` | Restart this deal (asks first) |
| `n` | New game |
| `p` | Pause |
| `m` | Main menu |
| `c` | Copy the seed of this deal |
| `i` | How to play |
| `?` or `F1` | Help: every key |
| `q` | Quit (`Ctrl+C` quits at once) |

In the classic Windows console, `Ctrl+Plus`, `Ctrl+Minus` and `Ctrl+0` change the console font size; other terminals
zoom with their own keys (Cmd on a Mac).

Command-line options (`oxysol --help` lists them):

| Option | What it does |
|---|---|
| `--seed <number>` | Choose a game for this seed at once |
| `--ascii` | Draw cards with ASCII characters only |
| `--colors <16\|256\|truecolor>` | Set the color depth of the terminal |
| `-n`, `--new-window` | Open the game in a new console window and return to the prompt |
| `-fs`, `--fullscreen`, `--no-fullscreen` | Force full screen on or off for this run (the setting is `Auto` by default; the last one given wins) |
| `--priority <level>` | Set the process priority for this run: `highest`, `high`, `normal`, `low` or `lowest` (beats the setting) |
| `--version`, `-h`, `--help` | Print the version, or the help, and exit |

**Where the files are.** Settings, the saved game and the history are in one SQLite file, `oxysol.db`, next to the log
`oxysol.log`. The folder is:

- Windows: `%APPDATA%\oXyFlow\oXySol\data`
- Linux: `~/.local/share/oxysol` (or under `$XDG_DATA_HOME`). Not verified.
- macOS: `~/Library/Application Support/oXyFlow.oXySol`. Not verified.

The environment variable `OXYSOL_HOME` points the game at another folder, and `OXYSOL_LOG` sets the log level
(`error`, `warn`, `info`, `debug` or `trace`).

**Portable copy.** Unpack a `-portable` archive anywhere: the game keeps its files in a `data` folder next to the
program and writes nothing to system folders. Delete the `oxysol.portable` file next to the program to use the system
folder again.

## Accessibility

Accessibility is a requirement here, not polish.

- Every action works by keyboard and by mouse.
- The focus is visible without color: a `►` marks the focused item (`>` in ASCII mode), and the line under the
  board says in words where the cursor is and what it can do.
- Color is never the only signal: suits are drawn as signs (♠ ♥ ♦ ♣) next to the rank, and picked cards and drop
  targets have their own marks.
- The **High contrast** theme is for low vision: white text on black and a frame around the playing field. Text contrast
  is tested against a ratio of 4.5:1 in every theme and 7:1 in High contrast.
- `--ascii` and `--colors 16` serve limited terminals.
- Color-vision modes for color-blind players are planned. They are not in the game yet.

## Development

The project is a Rust workspace:

| Part | What it is |
|---|---|
| `crates/core` | `oxysol-core`: the engine and the games. No terminal, no files, no clock |
| `crates/tui` | `oxysol-tui`: rendering, input, themes and localization. Changes the game only through the engine's session |
| `src/` | `oxysol`: the binary that wires it together: storage, audio, logging, arguments |
| `xtask/` | `cargo xtask`: quality gates, lint, release tooling and the screenshot generator |

The toolchain is pinned in `rust-toolchain.toml`. From the repository root:

```sh
cargo run                    # play from source
cargo xtask check --quick    # fmt, Clippy, lint and the fast tests: run before a pull request
cargo xtask check            # everything, as CI does on main
cargo xtask-art screenshots      # regenerate the SVG screenshots and animations of the website
cargo xtask fonts            # build the oXySol Mono card font (needs Python 3.11+, see below)
```

`cargo xtask fonts` builds `assets/fonts/oXySolMono-Regular.ttf` from the Cascadia Mono base and the card glyph SVGs,
and writes a preview sheet to `target/fonts/preview.html`. It is the only command that uses Python: install the pinned
packages with `pip install -r xtask/fonts/requirements.txt` (fontTools and skia-pathops). Players do not need it.

Read [`AGENTS.md`](AGENTS.md) for the rules of the code, then [`docs/tech/`](docs/tech): the
[architecture](docs/tech/architecture.md), the [workflow](docs/tech/workflow.md) (branches, checks, releases) and the
[decisions](docs/tech/decisions.md). The rules of the games are in [`docs/wiki/rules/`](docs/wiki/rules).
The website is built from [`docs/site`](docs/site) with mdBook and published with each release.

## Links

- Website: <https://beoarcha.github.io/oXyFlow.Solitaire/>, in nine languages: English, Polski, Deutsch, Español,
  Français, Italiano, Nederlands, Português (Brasil) and Українська. The game interface is available in the same nine
  languages; the screenshots show the English version.
- Po polsku: pasjans w terminalu z Klondike, Pająkiem, FreeCellem, Piramidą i rodziną Yukon. Polska wersja strony:
  <https://beoarcha.github.io/oXyFlow.Solitaire/pl/index.html>

## License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 Marcin "beoArcha" Stefan, under the oXyFlow brand.
