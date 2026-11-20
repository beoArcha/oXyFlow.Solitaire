# Phase 6 (0.6) — Themes as data, four-colour French, TriPeaks and game options

Goal: themes and the interface size scale move to data files; the four-colour French card style and the TriPeaks game are added; the owner's game options are added to the games they belong to. Version 0.6.x.

Exit criteria:

- P3-36, P6-04 and P6-05 are DONE;
- P6-06 is DONE for the options the owner chose (its DRAFT is finalized by Opus first).

Order of work: the theme data first (P3-36), then the card style and the game, then the options.

### P3-36 — Themes as data and the interface size scale

- **Owner:** Sonnet
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

- **Owner:** Haiku
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

- **Owner:** Sonnet
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

- **Owner:** Opus (finalizes the scope), then Haiku
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
