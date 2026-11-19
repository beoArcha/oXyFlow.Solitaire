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
