# Phase 4 — Application: persistence, logging, failures

Goal: the game saves and resumes, and every technical error has exactly one place that handles it.

### P4-01 — SQLite persistence

- **Owner:** Sonnet
- **Depends on:** P3-01
- **Files:** `Cargo.toml`, `Cargo.lock`, `src/{lib.rs,paths.rs}`, `src/storage/**`, `tests/integration/**`, `tests/architecture/**`
- **Scope (R-18, R-19, T-02):**
  - Add `rusqlite` (feature `bundled`) to the root package only.
  - `paths.rs`: directories from `directories::ProjectDirs::from("", "oXyFlow", "oXySol")`;
    `OXYSOL_HOME` points everything at one directory (tests, portable use).
  - `SqliteStorage` implements `oxysol_tui::Storage` over `oxysol.db` with the tables in
    `architecture.md`; schema in `PRAGMA user_version`, migrations as an ordered list of SQL
    strings applied in one transaction; every write in a transaction.
  - `start_game` inserts a `games` row (variant, seed, UTC start time) and returns its id;
    `finish_game` sets outcome, end time and elapsed time; `save_game` keeps one row tied to a game id.
  - Errors: a newer `user_version` → `StorageError::Incompatible`; not a database → `Corrupt` and the
    file is renamed to `oxysol.db.corrupt`; SQLite and I/O errors keep their `source`. Storage logs nothing.
  - Architecture test: `rusqlite` appears only in the root `Cargo.toml`.
- **Acceptance:** temp-dir tests: settings and game round trip; missing rows → `Ok(None)`; history
  rows with seed and outcome; a newer schema version; a corrupt file (error, file renamed, next open
  starts empty); a read-only location (error with `source`); `clear_game`; reopening the database
  keeps everything.
- **Done:** #31

### P4-02 — Logging, panic hook, entry point

- **Owner:** Haiku
- **Depends on:** P3-01, M-18
- **Files:** `src/{main.rs,lib.rs,logging.rs,panic_hook.rs,error.rs}`, `locales/*/cli.toml`, `tests/architecture/**`
- **Scope:**
  - `main`: parse arguments with `cli::parse` (M-18); `--version` and `--help` print localized text
    and exit; `--seed <u64>` starts straight into a new game with that seed (R-18); an argument error prints usage and exits with code 2; `--ascii` and `--colors`
    override detected capabilities.
  - `logging.rs` (T-24): `tracing-subscriber` writing to `oxysol.log` in the data directory; the
    file is truncated at start when above 1 MB; level from `OXYSOL_LOG` (default `info`). If logging cannot
    start: one warning on stderr before the TUI starts, and the app continues.
  - `panic_hook.rs`, in this order: `oxysol_tui::restore_terminal()`, a short stderr message with
    the panic location and the log path, a diagnostic record through a file handle opened earlier.
    No large allocations; the hook itself must not panic. Ignoring a write error inside the hook is
    the one permitted exception to the ignored-result rule, with a `reason`.
  - `error.rs`: `AppError` aggregating layer errors (`thiserror`). `main` returns `ExitCode`:
    0 success, 1 runtime failure, 2 usage error. A critical error is `tracing::error!` with target
    `oxysol::critical`.
  - No `windows_subsystem`, no `AttachConsole`/`AllocConsole` (decision T-10).
  - Architecture test: `src/main.rs` installs the panic hook before starting the TUI.
- **Acceptance:** a child-process test shows the hook leaving a log record and a non-zero exit code;
  `oxysol --version` prints the version from `metadata.toml`; an unknown argument exits with 2.
- **Done:** #37

### P4-03 — Autosave and resume

- **Owner:** Sonnet
- **Depends on:** P3-03, P4-01
- **Files:** `crates/tui/src/app.rs`, `crates/tui/src/screens/board/**`, `tests/integration/**`, `locales/*/common.toml`
- **Scope:**
  - A new game calls `start_game` (variant, seed) and logs the deal at `info`. Save the game
    (`SessionRecord` + elapsed time) after every session change and when leaving the board. A won
    game calls `finish_game(Won)` and removes the save; starting another game over a save calls
    `finish_game(Abandoned)`.
  - Startup: load settings (missing or failing → defaults and an alert). Resume rebuilds the session
    with `Session::restore` and restores the clock.
  - A corrupt, incompatible or unreplayable save: alert, remove the save, return to the menu.
  - A write failure during play: one alert per session, one log record, the game continues.
- **Acceptance:** integration tests with `SqliteStorage` in a temp dir: play → drop `App` → new `App`
  → Resume gives identical `piles()`, history position, seed and time; the history table holds the seed and outcome; a corrupt save leads to an alert
  and the menu; a storage that fails on write neither stops the game nor alerts twice.
- **Done:** #45

### P4-05 — Restarts counted separately in games without undo

- **Owner:** Sonnet
- **Depends on:** P4-04, P3-12
- **Files:** `src/` (storage, schema migration), `crates/tui/src/screens/stats*`,
  `crates/tui/src/screens/board/**` (header counter), locales, tests
- **Scope:** owner decision 2026-11-19: Restart stays allowed in games without undo, but it is counted
  separately. Each game record keeps a restart count (schema migration with a default of 0). The board
  header shows the count in no-undo games once it is above zero. Statistics show no-undo results
  without restarts apart from those with restarts (for example "wins without restart" next to all
  wins), and the history lists the restart count. Games with undo are unchanged.
- **Acceptance:** migration test from the current schema; restart increments the count only in no-undo
  games; statistics and history show it; both locales.

### P4-06 — Automatic moves in statistics: failed attempts and streaks

- **Owner:** Sonnet
- **Depends on:** P4-05
- **Files:** `crates/core/src/session*` (counters only), `src/` (storage, schema migration),
  `crates/tui/src/screens/stats*`, `crates/tui/src/screens/board/**` (game facts), locales, tests
- **Scope:** owner request 2026-11-19: detect a game played mostly by the automatic moves. Count per game
  the automatic moves made (quick move, auto-move to the foundations, auto-finish counted apart),
  the failed automatic attempts (a quick move with no target), the manual moves, and the longest streak
  of automatic moves in a row. Store them with the game (schema migration with defaults). Statistics and
  history show the share of automatic moves, the failed attempts and the longest automatic streak, and
  mark a game as "assisted" when the share is above a threshold (a constant, e.g. 80 %, named in the
  decision log). The counters are information only; they never block a move.
- **Acceptance:** tests for each counter on scripted games, the streak, the migration, the "assisted"
  mark at the threshold; both locales.

### P4-04 — Statistics, game history and seed copying

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-03
- **Files:** `crates/core/src/session{.rs,/tests.rs}`, `crates/tui/src/{clipboard,storage,app/deps,app/effects,app/persist,app/persist_tests}.rs`, `crates/tui/src/screens/{stats/**,main_menu.rs}`, `src/{clipboard.rs,storage/**}`, `locales/*/stats.toml`, `tests/integration/{migration,sqlite_storage}.rs`, `docs/tech/{architecture,decisions}.md`
- **Scope:** Schema v2 with a forward migration: `games.moves` (NULL for older rows) and `games.undo_allowed`; `Storage::start_game(variant, seed, undo_allowed)`, `Storage::finish_game(id, outcome, elapsed_ms, moves)` and `Storage::history()`. `Session::moves_made()` counts executed moves, undo, redo and restart. A Statistics item in the main menu shows a summary per variant and undo mode, and the history newest first, with "Play this seed again" and "Copy seed" (`c`). Copying uses `arboard` with an OSC 52 fallback. Dates are shown in UTC. Decision T-27.
- **Acceptance:** `cargo xtask check --quick` passed; the architecture and e2e suites also passed locally (PR #61).
- **Done:** #61

### O-05 — Keep a double-clicked console open after a startup error

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-02
- **Files:** `src/{console,lib,main}.rs`, `locales/{en-US,pl-PL}/cli.toml`, `docs/tech/decisions.md`
- **Scope:** After a failed run (usage, paths, storage, terminal or output error), a process started by double-click waits for Enter before its console closes. `console::owns_window()` detects a console owned by the process (`GetConsoleProcessList`, declared locally, no new dependency). `console::should_pause(failed, owns_window, stdin_is_terminal)` is a pure function with a unit test. Success, a run from an existing terminal and piped stdin never wait. Localized prompt `cli.press_enter`. T-10 gets one sentence on the exception.
- **Acceptance:** E2E: child processes with piped stdin never pause (4 passed). Manual on Windows: a run with `--bogus` opened its own console and stayed open; a run from an existing PowerShell exited with code 2 without waiting (PR #39).
- **Done:** #39

### O-07 — Portable mode: data next to the executable

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-01
- **Files:** `src/paths.rs`, `docs/tech/{architecture,decisions}.md`, `README.md`
- **Scope:** Decision R-21. A copy is portable when a regular file `oxysol.portable` sits beside the executable; its database and log then live in `data/` beside it, and nothing is written to system folders. Precedence in `AppPaths::detect()`: `OXYSOL_HOME` (when set and not empty), then the portable marker, then the platform data directory. A failing `current_exe()` means not portable.
- **Acceptance:** Unit tests in `src/paths.rs`: marker gives `data/`, no marker gives the platform directory, `OXYSOL_HOME` wins, and a directory named `oxysol.portable` is not a marker; `cargo xtask check --quick` passed (PR #41).
- **Done:** #41

### O-09 — Application icon for the Windows executable

- **Owner:** not recorded (follow-up)
- **Depends on:** P0-01
- **Files:** `build.rs`, `Cargo.toml`, `Cargo.lock`, `assets/icon/**`, `xtask/src/{main,check,error}.rs`, `xtask/src/icon/**`, `xtask/Cargo.toml`, `docs/tech/decisions.md`
- **Scope:** The icon as vector sources (`assets/icon/oxysol.svg`, simplified `oxysol-32.svg` and `oxysol-16.svg`). `cargo xtask icon` renders them with resvg to 16…256 px and writes the committed `assets/icon/generated/oxysol.ico` and `oxysol-256.png`; `cargo xtask icon --check` compares pixels with a tolerance of 2 and runs in `cargo xtask check`. Root `build.rs` embeds the ICO and a version resource with winresource, only for Windows targets. New dependencies: resvg 0.50.0 (xtask) and winresource 0.1.34 (build dependency). Decision T-26.
- **Acceptance:** `cargo xtask check --quick` passes; on Windows the built executable shows the icon and version information; `icon --check` fails after a source SVG is edited without regenerating. Linux and macOS builds were not verified (PR #51).
- **Done:** #51

### O-11 — `--seed` starts the game directly; typed keys stay in text fields

- **Owner:** not recorded (follow-up)
- **Depends on:** P3-10
- **Files:** `src/{main,seed}.rs`, `crates/tui/src/{app,input,test_support}.rs`, `crates/tui/src/app/tests.rs`, `crates/tui/src/screens/main_menu.rs`, `locales/*/cli.toml`, `tests/integration/{audio,autosave}.rs`, `docs/tech/architecture.md`
- **Scope:** `oxysol --seed N` opens the game selection for seed N; picking a variant deals that game. The interim override that made only the first new game use N is removed; later new games get fresh seeds. While the seed field is open, a character key is text: `?` and `i` no longer open overlays there. F1 and Ctrl+C still work everywhere (`MainMenu::captures_text`, `is_text_key`). Not included: Backspace in `KEY_TABLE` (item 3 of O-11), because it needs a new `Action` variant that would change the board while P3-11 edits it; the seed field handles Backspace directly.
- **Acceptance:** Tests: `--seed` opens the games of that seed; Enter deals with the chosen variant; a later new game gets a fresh seed; Esc keeps the seed; `?` and `i` are typed inside the field and open overlays outside it; F1 still opens Help. `cargo xtask check --quick` passed (PR #58).
- **Done:** #58

### O-15 — `--new-window` flag and the command-line options in Help

- **Owner:** not recorded (follow-up)
- **Depends on:** O-05, O-14
- **Files:** `src/{cli,error,lib,main,new_window,new_window/tests}.rs`, `crates/tui/src/app/modal.rs`, `crates/tui/src/widgets/overlay/**`, `locales/*/{cli,help}.toml`, `docs/tech/{architecture,decisions}.md`
- **Scope:** `--new-window` (also `-n`) relaunches oXySol in a new console window with the same arguments minus the flag; the original process exits with code 0 at once. If no window can be started, a localized error names what was tried and the process exits with code 2. Windows uses `CREATE_NEW_CONSOLE`; Linux tries `$TERMINAL -e`, `x-terminal-emulator -e`, `gnome-terminal --`, `konsole -e` and `xterm -e`; macOS runs `osascript`. Command construction is pure and unit-tested; the spawn is not. `--help` is regrouped and fully translated to Polish. Help gets a third tab, "Command line", showing the `cli.help` text; Tab now cycles Keys, Rules and Command line. Decision T-28; the architecture contract changes.
- **Acceptance:** `cargo xtask check --quick` passed. Manual on Windows 10 (release build): a second console opened with the game, the parent exited with code 0, and the O-05 pause works in the relaunched console. Linux and macOS: construction tests only (PR #70).
- **Done:** #70

### O-19 — `--new-window`: alacritty, kitty, wezterm and foot on Linux

- **Owner:** not recorded (follow-up)
- **Depends on:** O-15
- **Files:** `src/new_window.rs`, `src/new_window/tests.rs`
- **Scope:** Adds alacritty (`-e`), kitty (no `-e`), wezterm (`start --`) and foot to the Linux fallback list, between konsole and xterm. The tests check the whole attempt sequence of all eight built-in terminals and the exact program names and arguments.
- **Acceptance:** `cargo xtask check --quick` passed (PR #72).
- **Done:** #72
