# Phase 6 (0.6) — Themes as data, four-colour French, TriPeaks and game options

Goal: themes and the interface size scale move to data files; the four-colour French card style and the TriPeaks game are added; the owner's game options are added to the games they belong to. Version 0.6.x.

Exit criteria:

- P3-36, P6-04 and P6-05 are DONE;
- P6-06 is DONE for the options the owner chose (its DRAFT is finalized by Opus first);
- the developer bridge P6-07, P6-08 and P6-09 is DONE (decision T-42).

Order of work: the theme data first (P3-36), then the card style and the game, then the options.

### P3-36 — Themes as data and the interface size scale

- **Owner:** Medium
- **Depends on:** P3-29, P3-30
- **Files:** `crates/tui/src/theme/**`, `assets/themes/` (new), `crates/tui/src/layout/**`,
  `crates/tui/src/settings.rs`, `src/` (loading a user theme), `docs/tech/ui-inventory.md`, tests
- **Scope:** owner request 2026-11-19: every item of `docs/tech/ui-inventory.md` is changed in one of
  three ways, recorded as a column in the inventory: by the theme (colors and text styles), by the
  interface size (one scale S / M / L / XL that sets the card size cap, fan steps, gaps, side panel and
  modal widths together, building on P3-16), or by its own setting (table, card face, card back, aura,
  glyphs, color vision). Themes move from Rust tables to embedded TOML files under `assets/themes/`
  (Dark, Light, High Contrast, Monokai, Dracula), validated by the contrast tests at build or test
  time. A user theme file in the data folder is loaded at start, checked with the same contrast rules
  and rejected with a readable message when it fails.
- **Acceptance:** the built-in themes render exactly as before; a valid and an invalid user theme; the
  size scale renders at 100 × 30, 130 × 40 and 200 × 60 per step; the inventory column filled; both
  locales.

### P6-04 — Card style: four-colour French

- **Owner:** Small
- **Depends on:** P5-03, P3-35
- **Files:** `assets/cards/fronts/french-four-colour/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The third new card style: the French suits with the four-colour deck of P3-35 (spades black, hearts red, diamonds
    blue, clubs green), with the court labels Valet, Dame and Roi and the ASCII letters S, H, D, C.
  - Each suit is told apart by its symbol first; the colour is an extra signal. In monochrome the four suits still differ.
  - Colours come from the theme tokens of P3-35; no colour outside the tokens.
- **Acceptance:** the four suits differ in monochrome and in ASCII; the four-colour option passes the contrast tests of
  P3-35 at every card size; snapshot tests; both locales.

### P6-05 — Game: TriPeaks

- **Owner:** Medium
- **Depends on:** P5-04
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/tripeaks/**`, `crates/core/src/solver/tripeaks*` (new,
  if the solver is in this card), the variant list of `oxysol-core` where the games are matched,
  `crates/tui/src/screens/main_menu/**` (the entry only), `locales/*/rules.toml`, `locales/*/menu.toml`,
  `docs/wiki/rules/tripeaks.md` (new), `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:**
  - TriPeaks (traditional, public domain): three peaks of cards dealt face up on a stock; the waste starts with one
    card; a peak card may be played onto the waste when its rank is adjacent to the waste top, as the rules tab text
    states (including the King and Ace rule).
  - A solver if the search stays within the budget of T-29; if the verdict is `Unknown` for most seeds, the winnable
    option stays off for TriPeaks and the PR says so.
  - Rules tab, menu entry and the rules page as in P4-10.
- **Acceptance:** engine tests for the peaks, the stock and the waste; the same seed gives the same deal; the menu entry
  and the rules tab in both locales; the solver result for seeds 0 to 49 is in the PR, if a solver is added.

### P6-06 — More game options

- **Owner:** Large (finalizes the scope), then Small
- **Status:** DRAFT (the owner chooses the options; Opus writes the final scope before dispatch)
- **Depends on:** P4-09, P3-27
- **Files:** `crates/core/src/games/**` (only the options of the games named), `crates/tui/src/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/*/settings.toml`, `locales/*/menu.toml`, tests
- **Shared files:** `crates/tui/src/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/en-US/settings.toml`, `locales/pl-PL/settings.toml`
- **Scope:** owner request: more options for the existing games. Candidates to choose from, not yet decided:
  - Klondike: a limit on the number of passes through the stock (today the passes are unlimited, see the assumed defaults
    in `decisions.md`);
  - FreeCell: the number of free cells;
  - any other option the owner names.
  An option that changes the rules is off by default, is shown in the game information, and is part of the seed's
  record in the history.
- **Acceptance:** the options chosen by the owner, each with engine tests for both values, the setting remembered, the
  history records the option, both locales; the winnable-deals option (P3-20) is either correct for the option or turned
  off for it, and the PR says which.

## Developer testing infrastructure: headless backend and JSON-lines dev bridge

Owner request 2026-11-21. An agent or a script drives the real application without a terminal: it sends one JSON command
per line on `stdin` and reads one JSON response per line on `stdout`, and every step is deterministic (decision T-42).
These cards run one after another and go to `main` as one PR when P6-09 is done. The screenshot cards P7-06 to P7-09 (phase 7) and the layout linter and MCP wrapper (phase 8) build on them.

Most of the base already exists:
- `App::handle(input, now)` never reads the system time (T-14).
- `App::draw` renders into any Ratatui backend.
- The tests already run the full pipeline on `TestBackend` (T-12, `test_support::Harness`).

The cards reuse that pipeline; they do not copy input handling, state updates or rendering.

### P6-07 — Dev bridge: headless driver, virtual clock and UI state snapshot

- **Owner:** Medium
- **Depends on:** T-12, T-14
- **Files:** `crates/tui/Cargo.toml` (feature `dev`), `crates/tui/src/clock.rs`, `crates/tui/src/terminal.rs`,
  `crates/tui/src/headless/**` (new, `#[cfg(feature = "dev")]`), `crates/tui/src/app/snapshot.rs` (new,
  `#[cfg(feature = "dev")]`), `crates/tui/src/lib.rs`, `crates/tui/src/solver.rs` (a synchronous solver for the
  headless run, if it is not already there), `docs/tech/architecture.md` (TUI contract), tests
- **Shared files:** `crates/tui/src/app.rs` (only the `mod` line of the snapshot module)
- **Scope:**
  - **Clock:** a `Clock` trait with `now() -> Instant`, and two implementations:
    - `SystemClock`, in every build, used by `terminal::run`;
    - `VirtualClock`, a fixed base `Instant` plus an offset that moves only when stepped.

    `terminal::drive` takes the clock instead of calling `Instant::now()` itself. Nothing else in the interactive loop
    changes.
  - **One step function** is shared by the terminal loop and the headless driver: handle one `Input`, then draw. The
    headless driver must not reimplement anything the interactive loop does.
  - **`Headless`** (feature `dev`) owns `App`, a `Terminal<TestBackend>` of `cols × rows` and a `VirtualClock`:
    - `step(input)` runs handle, then draw, and increments `frame_id`;
    - `resize(cols, rows)` resizes the backend and sends `Input::Resize`;
    - `tick(ms)` advances the clock and sends `Input::Tick`;
    - the cell buffer is Ratatui's `Buffer` of the backend: symbol, foreground, background, and the bold, underline,
      reversed and dim modifiers;
    - `text_frame()` returns the `cols × rows` plain text grid.
  - **Dependencies of a headless run.** Nothing runs on another thread, so the same commands always give the same
    frames:
    - `MemoryStorage`, seeded with the start settings;
    - no audio;
    - a clipboard that records;
    - seeds from a counter that starts at the given seed;
    - a synchronous solver.
  - **`App::snapshot() -> UiSnapshot`** (feature `dev`, `Serialize`). It is read-only and never changes state. It holds:
    - `screen`, `mode`, `active_overlay`, `overlay_rect`, `dock_mode`;
    - `focus`, `hover`, `tooltip_visible`;
    - `status_line`, `last_error`;
    - `game_stats`: variant, seed, moves, elapsed time;
    - on the board, a summary of the legal actions from the existing move list.
- **Acceptance:**
  - A default build (`cargo build`, `cargo build --no-default-features`) contains no headless code, and the interactive
    loop behaves as before.
  - Tests show:
    - `VirtualClock` moves only when stepped;
    - the same inputs on two `Headless` runs give identical buffers and snapshots;
    - a headless frame equals the `Harness` frame for the same inputs;
    - the snapshot names the screen, the overlay with its rectangle, the focus and the game stats.

### P6-08 — Dev bridge: `--dev-bridge` JSON-lines protocol

- **Owner:** Medium
- **Depends on:** P6-07
- **Files:** `Cargo.toml` (feature `dev-bridge = ["oxysol-tui/dev"]`, off by default), `src/cli.rs`, `src/main.rs`,
  `src/dev_bridge/**` (new, `#[cfg(feature = "dev-bridge")]`), `src/lib.rs`, `docs/tech/architecture.md` (application
  contract), `tests/integration/dev_bridge.rs` (new), tests
- **Scope:**
  - `--dev-bridge` exists only in a build with the feature. A default build rejects it as an unknown argument, and
    `--help` does not list it.
  - The loop reads one JSON command per line from `stdin` and writes one JSON response per line to `stdout`, flushing
    after every response. Every command is one synchronous step: handle, update, render, respond. There is no sleep and
    no thread.
  - Commands (missing optional fields take the defaults shown in the PR):
    - `{"cmd":"start","cols":120,"rows":40,"game":"klondike","seed":12345,"ascii":false,"colors":"truecolor","lang":"en-US","theme":"dark"}`;
      `"game":"menu"`, or no game, starts on the main menu.
    - `{"cmd":"key","key":"Enter","modifiers":["shift","ctrl"]}`
    - `{"cmd":"mouse","action":"move|down|up|click|double_click|right_click","x":10,"y":5}`
    - `{"cmd":"resize","cols":80,"rows":24}`
    - `{"cmd":"tick","ms":1000}`
    - `{"cmd":"snapshot","include_text":true,"include_state":true}`
    - `{"cmd":"quit"}`
  - Input mapping:
    - key names map to the keys of `crates/tui/src/input.rs`;
    - a click is a down and an up;
    - a double click uses the application's double-click rule on the virtual clock.
  - A response holds:
    - `frame_id` and `ok`;
    - `error`: a protocol or command error, such as malformed JSON, an unknown command, or a command before `start`;
    - `state`: the `UiSnapshot`;
    - `text_frame`, on request;
    - `events`: what happened in this step, such as screen, overlay, focus and hover changes, and move results. An
      illegal move is an event, not an ERROR log.
  - A protocol error never ends the loop. End of input ends it like `quit`.
- **Acceptance:**
  - Integration tests drive the protocol through these steps:
    - start on the main menu;
    - open Help, Settings and New game;
    - start a seeded Klondike game;
    - make moves by keyboard and by mouse;
    - resize;
    - quit cleanly.
  - A malformed line and an unknown command each give `ok: false`, and the loop continues.
  - `cargo build` without the feature still rejects `--dev-bridge`.

### P6-09 — Dev bridge: `--dev-log` structured log and determinism tests

- **Owner:** Medium
- **Depends on:** P6-08
- **Files:** `src/dev_bridge/**`, `src/cli.rs`, `tests/integration/dev_bridge.rs`, `xtask/src/**` (one step in
  `cargo xtask check` that builds and tests with `--features dev-bridge`), `docs/tech/workflow.md` (the new check step),
  `docs/wiki/` or `docs/tech/` (a short page on driving the bridge), tests
- **Scope:**
  - `--dev-log <path>` works only in dev builds, together with `--dev-bridge`. It appends one JSON line, with the
    `frame_id`, for every command, state transition, focus change, move result and engine error, for post-mortem
    inspection. A log file that cannot be opened is a readable start-up error.
  - Determinism: two runs with the same `--seed` and the same command script give byte-identical `text_frame` and
    `state` for every `frame_id`.
  - `cargo xtask check` also builds and tests the dev feature, so the bridge cannot rot. A default build stays free of
    it.
- **Acceptance:**
  - A test runs one script twice and compares every response byte for byte.
  - A test reads the log back and finds the command, transition, focus, move and error entries.
  - `cargo build` and `cargo build --no-default-features` pass without the feature.
  - `cargo xtask check --quick` passes.

## Done before the 2026-11 replan

### P6-01 — MIDI sequencer and synthesizer

- **Owner:** Sonnet
- **Depends on:** P3-01
- **Files:** `src/audio/{sequence.rs,synth.rs,style.rs}`, `assets/audio/README.md`, `Cargo.toml` (adds `midly`)
- **Scope:**
  - `style.rs`: the `style.toml` schema — `[cues]` mapping each `SoundCue` to a `.mid` file,
    optional `[music]` with one looping file, `[programs.<n>]` defining the timbre of a MIDI
    program (waveform, attack, decay, sustain, release, gain). Document the schema in
    `assets/audio/README.md`.
  - `sequence.rs`: load a Standard MIDI File with `midly` into a list of note events with sample
    timestamps, honoring tempo changes.
  - `synth.rs`: a polyphonic synthesizer (oscillators plus envelopes, a voice limit, a soft limiter)
    with `fill(&mut [f32])` that never allocates, and `render(...) -> Vec<f32>` for tests.
  - Pure code: no audio device, no files beyond the bytes passed in.
- **Acceptance:** tests: deterministic output for a fixed input; silence for an empty sequence; note
  onsets at the expected sample positions, including after a tempo change; the voice limit holds;
  output stays within [-1, 1] and contains no NaN; a malformed file is an error, not a panic.
- **Done:** #49

### P6-02 — Audio output and settings

- **Owner:** Haiku
- **Depends on:** P6-01, P3-05
- **Files:** `src/audio/{mod.rs,output.rs}`, `build.rs`, `src/main.rs`, `Cargo.toml` (adds `cpal`, feature `audio`), `.github/workflows/{pr.yml,main.yml}`, `locales/*/audio.toml`
- **Scope:**
  - A build script embeds every folder under `assets/audio/`.
  - `output.rs`: a `cpal` output stream that mixes active cues and the music loop from `synth`.
  - `mod.rs`: the `oxysol_tui::Audio` implementation: `configure` (style, volume, cues, music),
    `play`, `available`.
  - `NullAudio` is used when the feature is off, no output device opens, or `TerminalCaps::remote`
    is true. Failure to open a device is logged once and reported through `available()`.
  - Feature `audio` is on by default; the crate builds and passes tests without it. Workflows
    install the Linux audio development package.
- **Acceptance:** tests with the feature off and with a failing device both yield a working app
  with sound options shown as unavailable; `cargo build --no-default-features` succeeds; cues heard
  by hand on at least one system — name it in the report.
- **Done:** #57

### P6-03 — Score-to-MIDI tool

- **Owner:** Haiku
- **Depends on:** P0-01
- **Files:** `xtask/src/midi.rs`, `xtask/Cargo.toml` (adds `midly`), `assets/audio/README.md`
- **Scope:** `cargo xtask midi <style>` compiles `assets/audio/<style>/scores/*.txt` into `.mid`
  files next to `style.toml`. Notation: header lines `tempo <bpm>` and `program <n>`, then tokens
  `<note>:<length>` such as `C4:8 E4:8 G4:4`, rests `R:4`, chords `C4+E4+G4:2`. Hand-made or
  externally composed `.mid` files remain valid — the tool is an authoring aid. Document the
  notation; finalize micro cards M-25…M-27 and mark them `READY`.
- **Acceptance:** tests: a known score compiles to a file that parses back to the expected notes,
  lengths, tempo and program; a syntax error names the file and line.
- **Done:** #43

### O-12 — Selectable sound styles and clearer sound status in Settings

- **Owner:** not recorded (follow-up)
- **Depends on:** P6-02, P3-11
- **Files:** `crates/tui/src/{app,app/deps,audio,settings,lib,test_support}.rs`, `crates/tui/src/screens/settings/**`, `src/audio/**`, `src/main.rs`, `tests/integration/{audio,autosave}.rs`, `locales/*/settings.toml`, `docs/tech/architecture.md`
- **Scope:** Settings offer every embedded sound style by its `style.toml` name (`default`, `retro`), passed to the TUI through `Deps::sound_styles`; a saved style that no longer exists falls back to `default`. `Audio::status()` returns Ready, RemoteSession, NotBuilt, NoDevice or StyleFailed, each with its own text in both locales. Builds without the audio feature say "Sound is not included in this build." Fallback order: remote session, not built, no working device, failed style, ready. The mixer is swapped with `Output::replace_mixer` after the new one is built.
- **Acceptance:** Tests for the style list, the status order, the fallback and the settings rows; `cargo xtask check --quick` and `cargo build --no-default-features` passed. Not verified: real sound output on a device (PR #64).
- **Done:** #64
