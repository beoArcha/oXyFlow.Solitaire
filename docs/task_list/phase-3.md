# Phase 3 — Terminal interface

Goal: a complete, readable interface driven by keyboard and by mouse. Sonnet builds the foundation
and the board (P3-01…P3-03); Haiku adds the screens around it.

Rules for the whole phase:

- Text through `i18n`, colors through theme tokens, symbols through `Glyphs`.
- A new key is added to both locales in the same PR. In `pl-PL` it may be a copy of the English
  text; micro tasks M-17 and M-34 translate.
- Every screen works by keyboard and by mouse. Focus and selection are visible without color
  (border style, marker).
- Rendering is tested on `TestBackend`.

### P3-01 — TUI foundation

- **Owner:** Sonnet
- **Depends on:** P1-02
- **Files:** `crates/tui/**`, `locales/**`
- **Scope:**
  - `lib.rs`: `run`, `restore_terminal`; raw mode, alternate screen, mouse capture, hidden cursor;
    the terminal is restored on every exit path (a `Drop` guard).
  - `app.rs`: `App`, `Screen { MainMenu, GameSelect, Board, Settings }`, modal layer
    `Modal { Help, Pause, Confirm, Alert, Win }`, `handle`, `draw`, `should_quit`.
  - `action.rs`, `input.rs`: `Action` and the key table — the single source for mapping, help and footer.
  - `settings.rs`: `Settings` with locale, theme, front style, back style, glyph mode
    (`Font`/`Unicode`/`Ascii`), color mode, sound style, cues on/off, music on/off, volume.
    Styles are referenced by folder id.
  - `storage.rs`: trait `Storage`, `SavedGame`, `StorageError`. `audio.rs`: trait `Audio`,
    `SoundCue`, `AudioSettings`, `NullAudio`.
  - `i18n.rs`: `t(key)`, `tf(key, params)`, fallback to `en-US`, then to the key. All locale area
    files from `architecture.md` are created and embedded here; `common.toml` is filled. Tests:
    identical key sets in both locales; every key used in the sources exists.
  - `theme/`: token struct, Dark palette, resolution for (`ThemeId`, `ColorDepth`) using `ansi.rs`
    from M-13. Constructor stubs for Light and High Contrast.
  - `glyphs.rs` (Unicode and ASCII sets), `clock.rs` (`GameClock`), `announce.rs` (stub).
    `caps.rs` and `widgets/logo.rs` come from M-15 and M-16.
  - "Terminal too small" screen below 100 × 30 showing current and required size.
  - Stubs with fixed signatures for later tasks: `screens/{main_menu,game_select,settings}.rs`,
    `widgets/overlay/`, `mouse.rs`.
  - Finalize micro cards M-11, M-12 (token list, pairs to test) and mark them `READY`.
- **Acceptance:** `TestBackend` tests: start, stub menu rendered, quit by key, too-small screen,
  resize; `cargo run` starts and leaves the terminal as it found it (checked by hand, or stated as
  not checked); `architecture.md` matches the code.

### P3-02 — Board geometry and rendering

- **Owner:** Sonnet
- **Depends on:** P3-01, P2-01
- **Files:** `crates/tui/src/layout/**`, `crates/tui/src/widgets/{card.rs,pile.rs}`, `crates/tui/src/screens/board/view.rs`
- **Scope:**
  - `layout::compute` → `BoardGeometry` (a rectangle per pile and per card, `hit(col, row)`).
    Column layout: a top row (stock, waste, cells, foundations) and fanned tableau columns.
    Pyramid layout: overlapping rows, stock and waste below.
  - Card size: the largest step that fits the game (for example 7×5, 9×7, 11×9, 13×11); spacing and
    margins grow with the window.
  - A column always fits vertically: fan compression, and in the extreme case face-down cards
    collapse into one row with a counter.
  - Card widget v1: front (rank and suit in the corners) in the International style, back, empty slot,
    focus and selection states. Style-driven rendering arrives in P5-02.
  - Klondike Draw 3: up to three top waste cards fanned horizontally.
  - Board screen: header (game, variant, seed, time), board, status line, footer with key hints.
- **Acceptance:** for every variant at 100 × 30 and 160 × 50: all rectangles inside the area, top
  cards fully visible, ranks of face-up cards never covered; a larger window yields a larger card
  step; a worst-case Spider column test.

### P3-03 — Board controller

- **Owner:** Sonnet
- **Depends on:** P3-02
- **Files:** `crates/tui/src/screens/board/**` (except `view.rs`), `crates/tui/src/app.rs`, `locales/*/board.toml`
- **Scope:**
  - Cursor: movement between piles by geometry, run-depth selection, pick, drop, cancel. After a
    pick, highlight the targets from `legal_moves()`.
  - Commands: stock action, quick move (first legal target, foundation before tableau), undo, redo,
    hint (highlight source and target from `hint()`, message when there is none), restart and new
    game with confirmation, pause (clock stops, board hidden), back to menu.
  - Every `MoveError` variant has a localized status message. No ERROR logs.
  - Win detection → `Modal::Win`. No legal moves → a message.
  - Emit a `SoundCue` at each semantic event through `Audio`.
  - The clock runs from the start of the game; pause and modals stop it.
  - A `persist()` hook called after every session change (filled by P4-03).
  - Finalize micro card M-17 and mark it `READY`.
- **Acceptance:** key-sequence tests on fixed seeds: legal move, illegal move with message, undo/redo,
  hint, pause, quick move, a pair and a king in Pyramid, a deal in Spider; a recording `Audio` double
  proves the cues fire.

### P3-04 — Menus and game selection

- **Owner:** Haiku
- **Depends on:** P3-01
- **Files:** `crates/tui/src/screens/{main_menu.rs,game_select.rs}`, `locales/*/menu.toml`
- **Scope:** main menu: Resume (only when a save exists), New game, New game from seed (a numeric
  input field, then the game selection), Settings, Help, Quit. Game and
  variant selection: Klondike (Draw 1, Draw 3), Spider (1, 2, 4 suits), FreeCell, Pyramid (Relaxed,
  Classic). The logo
  from `widgets/logo.rs`. Arrows and `j`/`k`, Enter, Esc; click selects, wheel scrolls. Centered
  layout that scales with the window.
- **Acceptance:** tests: visiting every item by keyboard and by mouse, Resume hidden without a
  save, every variant creates a session with the right `Variant`, a typed seed reaches `Session`,
  a non-numeric seed is rejected with a message, rendering in both locales and in ASCII.

### P3-05 — Settings screen

- **Owner:** Haiku
- **Depends on:** P3-01
- **Files:** `crates/tui/src/screens/settings.rs`, `locales/*/settings.toml`
- **Scope:** options: language, theme, front style, back style, glyph mode, color mode, sound style,
  cues on/off, music on/off, volume. A change applies immediately and is saved through `Storage`.
  When `Audio::available()` is false the sound options are shown as unavailable with the reason.
  Storage failure: an alert; the setting holds for the session.
- **Acceptance:** tests: changing each option changes app state and reaches a test storage, by
  keyboard and by mouse; language switches text at once; a failing storage shows an alert and does
  not interrupt.

### P3-06 — Mouse on the board

- **Owner:** Haiku
- **Depends on:** P3-03
- **Files:** `crates/tui/src/mouse.rs`
- **Scope:** translate mouse events into the same `Action`s the keyboard produces, through
  `BoardGeometry::hit`. Click a card — pick; click a target — drop; click the stock — stock action;
  double click (same card within 400 ms) — quick move; right button — cancel. Drag: press on a
  card, move, release over the target; where the terminal reports no drag events, click–click works.
- **Acceptance:** tests with synthetic events on known geometry: each gesture above, a click on
  empty space, a release outside the board, a click on a face-down card.

### P3-07 — Overlays: help, pause, win, dialogs

- **Owner:** Haiku
- **Depends on:** P3-03
- **Files:** `crates/tui/src/widgets/overlay/**`, `locales/*/help.toml`
- **Scope:** help generated from the key table, grouped by topic, including the mouse gestures;
  a pause overlay that covers the board; a win screen with the elapsed time and "new game" / "menu";
  confirmation and alert dialogs with a clearly focused button, operable by keyboard and by click.
  Centered and scaled with the window.
- **Acceptance:** render tests for every overlay in both locales, Unicode and ASCII; help lists
  every binding in the table; pause shows no card.

### P3-08 — Accessibility: announcer and tests

- **Owner:** Haiku
- **Depends on:** P3-03
- **Files:** `crates/tui/src/announce.rs`, `crates/tui/tests/accessibility.rs`, `locales/*/board.toml`
- **Scope:**
  - `announce`: a one-line plain-text description of the focus and of the last event, built from
    localized card and pile names ("Tableau 3: seven of hearts selected, 2 targets"; "Illegal move:
    needs a black eight"). Shown in the status line of the board.
  - Tests that keep accessibility from regressing:
    1. every `Action` has a key binding and appears in help;
    2. with all colors removed, the rendered board still distinguishes focus, selection, legal
       targets, face-down cards and the four suits;
    3. every interactive element of every screen is reachable by keyboard alone;
    4. every theme meets its contrast target for the token pairs used together.
- **Acceptance:** the four tests pass; announcements exist for every `MoveError` and every action
  that changes the board, in both locales.

### P3-09 — Gate: TUI

- **Owner:** Sonnet
- **Depends on:** P3-04…P3-08, P4-03
- **Files:** fixes in `crates/tui/**`, `locales/**`
- **Scope:** review the merged diff of phases 3 and 4 against `AGENTS.md`: hard-coded text or
  colors, consistency of keys, mouse gestures and focus across screens, completeness of both
  locales, storage error handling. Findings name the rule and location; fix small ones, open cards
  for larger ones. Finalize micro cards M-30…M-34. Update the Status columns.
- **Acceptance:** review report; the full path menu → game → settings → quit exercised for every
  game, by test or by hand (say which); `cargo xtask check` passes.

### P3-11 — Board guidance: focus aura, action line and guide panel

- **Owner:** Sonnet
- **Depends on:** P3-06, P3-08, P3-10
- **Files:** `crates/tui/src/{widgets,screens/board,screens/settings,layout,theme}/**`,
  `crates/tui/src/{action,input,settings,glyphs,announce}.rs`, `locales/*/{board,help,settings}.toml`,
  `tests/integration/`, `docs/tech/architecture.md`
- **Scope:**
  - Focus aura: a one-cell halo (`aura_focus`, `aura_carry` tokens in every palette, covered by the
    contrast tests; a distinct symbol per halo) around the focused card or run and the carried run,
    a weaker halo around valid drop targets; drawn on free cells only, so the layout does not move.
  - Action line under the board: `Now:` (what is in progress) and `Possible:` (the actions of this
    exact state with their keys), built from the controller, `Session` and `KEY_TABLE`. It absorbs
    the first-steps hint of P3-10; the announcer message stays.
  - Guide panel at the right (30 columns): goal, key groups from `KEY_TABLE` with the usable keys
    marked, mouse gestures. `Settings::guide_panel` (default on), a row in Settings, key `g` and a
    click on the title toggle it; hidden when the board would drop below 100 columns.
  - New texts in `en-US` and `pl-PL`.
- **Acceptance:** tests for the aura of every variant (present at the focus, absent elsewhere), the
  aura without color, the action line (browsing, carrying, no moves, won), the panel (shown at
  140 × 40, hidden at 100 × 30 and when off), the setting through storage and an old value without
  it, WGL4/ASCII glyphs only; `cargo xtask check --quick` passes.

### P3-12 — Board counters, the no-undo mode and copying the seed

- **Owner:** Sonnet
- **Depends on:** P3-11, P4-04, P2-06
- **Files:** `crates/tui/src/{action,input,settings,storage}.rs`, `crates/tui/src/app/**`,
  `crates/tui/src/screens/{board,game_select,stats,mod}.rs`, `crates/tui/src/screens/{board,game_select,stats}/**`,
  `crates/tui/src/widgets/{guide.rs,overlay/{mod,win}.rs}`, `locales/*/{board,common,help,stats}.toml`,
  `src/storage/sqlite.rs`, `tests/integration/`, `docs/tech/architecture.md`
- **Scope:**
  - Header `Moves: N` (from `Session::moves_made()`; also in the win dialog next to the time).
  - Stock counters from `Game::stock_status`: Klondike "Turned over: N", Pyramid "Pass N of M" / "Last
    pass" / "No passes left", Spider "Deals left: N", nothing for FreeCell. Beside the stock when a free
    row exists, else in the `Now:` line; always in the guide panel.
  - No-undo mode: `undo_allowed` in `Effect::Start`, `Settings` (last choice, serde default `true`),
    `SavedGame` (serde default `true`) and `Storage::start_game`; a toggle (`u` or a click) in the game
    selection; replay from the statistics uses the row's mode; undo and redo are rejected with a message,
    hidden from the action line, footer and guide panel, and the header shows a "No undo" badge.
  - Key `c` (`Action::CopySeed`) and a click on the seed in the header copy the seed through
    `Effect::CopySeed`; the board announces "Seed N copied" or the failure with the seed shown.
  - The statistics summary is no longer capped at 8 rows; it grows to the 16 groups (8 games × 2 modes).
- **Acceptance:** tests for the counters after moves, undo, redo and restart; the stock counter of every
  game; a no-undo game (rejection, hidden hints, save and resume, `undo_allowed = false` in storage); the
  toggle by key and mouse and the remembered setting; the copy key and the header click against the
  clipboard double; 100 × 30 in both locales with WGL4/ASCII glyphs only; `cargo xtask check --quick` passes.

### U-04 — Interface review: usability and readability

- **Owner:** Fable with Opus (owner request, 2026-11-16)
- **Depends on:** P3-12, P3-13, O-14
- **Files:** `docs/tech/ui-review.md` (new)
- **Scope:** review the whole interface for maximum usability and readability on an old laptop with
  a small screen, at low and at high resolution (from 80 × 24 / 100 × 30 up to 200 × 60 and more cells,
  small and large fonts, the classic Windows console, Windows Terminal, a Linux and a macOS terminal).
  Cover the board (card faces, focus and aura, counters, header, action line, footer, guide panel),
  menus, game selection, settings, statistics, help and dialogs, both locales and both glyph modes.
  Name each problem with a screenshot or text render, its cause and a concrete fix, ranked by impact.
  The owner's findings of 2026-11-16 are inputs: the large block-art rank reads worse than the small
  face did (D looks like 0, W like H); the bottom area mixes game information with controls; the guide
  panel has no frame; zoom is not documented or reachable by keyboard; Tab does not move between
  buttons. The review refines P3-14…P3-17 and O-16 before they start and may add cards.
  Owner priority (2026-11-16): readability first, then ease and intuitiveness of use — a newcomer
  must see at once what is on the board, what is selected and what to do next, without reading help.
  Also covers P3-18…P3-21 (guide panel on smaller screens, auto-finish, winnable deals, aura options).
- **Acceptance:** `docs/tech/ui-review.md` with ranked findings and a recommended layout per screen
  size class; the cards it refines are updated; owner sign-off.

### P3-14 — Readable card faces

- **Owner:** Sonnet
- **Depends on:** U-04
- **Files:** `crates/tui/src/widgets/card/**`, `crates/tui/src/layout/size.rs`, tests
- **Scope:** owner feedback: the block-art rank of P3-13 is less readable than the small face (D and 0,
  W and H look alike; the shapes are blurry). Replace it with what U-04 recommends, for example: corner
  indices that are larger and always visible (rank and suit on their own cells, bold, on every size),
  the centre showing the rank as plain text next to the suit (doubled or with spacing on large cards)
  and the suit pictogram only where it stays crisp. Rank letters must never be confusable (D/0, W/H,
  10/1O, 8/B); check every rank in both locales. Keep WGL4 and an ASCII equivalent.
- **Acceptance:** renders of every rank and suit at every size in both locales and glyph modes,
  reviewed by the owner; a test that no two ranks render the same at any size; existing render tests
  pass.

### P3-15 — Screen layout: game information apart from controls

- **Owner:** Sonnet
- **Depends on:** U-04
- **Files:** `crates/tui/src/layout/**`, `crates/tui/src/screens/board/{view,header,status,guidance}.rs`,
  `crates/tui/src/widgets/{guide,footer}*`, locales, tests
- **Scope:** owner feedback: the bottom area is overloaded. Put game information in one place (game and
  variant, seed, time, moves, stock counters, undo mode, the focused card, short statistics such as the
  best time for this game) and the interface in another (the `Now:` / `Possible:` guidance and the key
  footer), with no fact shown twice (the announcer line repeats the focus that `Now:` already shows).
  The guide panel gets a frame or clear separators from the header, the board and the bottom area. Use
  the layout per screen size class from U-04, so a small screen keeps the board first and a large one
  uses the room.
- **Acceptance:** renders at 100 × 30, 130 × 40 and 200 × 60 in both locales; no duplicated fact; the
  guide panel framed; existing guidance, footer and accessibility tests updated.

### P3-16 — Zoom: card size by keyboard and console font

- **Owner:** Sonnet
- **Depends on:** U-04
- **Files:** `crates/tui/src/{input,action,settings}.rs`, `crates/tui/src/layout/**`, `src/` (console
  font on Windows), help locales, tests
- **Scope:** owner decision 2026-11-16: both. (1) In-game zoom: keys (e.g. Ctrl+Plus / Ctrl+Minus and
  a plain-key alternative for terminals that swallow Ctrl+Plus) step the card size up and down within
  what fits, with "auto" (largest that fits) as the default; the choice is a setting. (2) In the classic
  Windows console, the same keys may also change the real console font size (`GetCurrentConsoleFontEx`
  / `SetCurrentConsoleFontEx`; needs an approved dependency such as `windows-sys` and a justified
  `unsafe`; record a decision). Windows Terminal, Linux and macOS terminals cannot be resized by the
  program: Help explains their own zoom (Ctrl+mouse wheel, Ctrl+Plus/Minus/0). Zoom keys appear in the
  key table, the help window and the guide panel.
- **Acceptance:** tests for stepping, the "auto" default, the limit when nothing larger fits and the
  remembered setting; the console font change verified by hand on Windows (say where); help text in
  both locales.

### P3-17 — Tab moves the focus

- **Owner:** Haiku
- **Depends on:** U-04
- **Files:** `crates/tui/src/screens/**`, `crates/tui/src/widgets/overlay/**`, `crates/tui/src/input.rs`, tests
- **Scope:** Tab and Shift+Tab move the focus through every focusable element of the current screen or
  dialog in reading order (menu items, toggles, buttons such as Back, Copy seed, Close), wrapping at the
  ends; Enter or Space activates the focused element. Where Tab already has a meaning (the help tabs,
  how-to-play), keep it consistent with U-04's recommendation. The focus is visible without color.
- **Acceptance:** tests per screen and dialog for the Tab order, wrap-around, Shift+Tab and activation;
  help lists Tab; accessibility tests updated.

### O-16 — Full-screen option

- **Owner:** Sonnet
- **Depends on:** U-04
- **Files:** `src/` (a new module), `src/cli.rs`, `crates/tui/src/settings.rs` and the settings screen,
  locales, docs
- **Scope:** owner decision 2026-11-16: "borderless" means the game window fills the whole screen with no
  frame or title bar. A setting "Full screen" and a flag `--fullscreen`. Classic Windows console: switch
  the console to full screen (`SetConsoleDisplayMode` or the Alt+Enter equivalent; record the approved
  dependency and `unsafe` in a decision). Windows Terminal: relaunch through `wt.exe --fullscreen` when
  the game was started by double-click, or tell the player to press F11 / Alt+Enter. Linux and macOS:
  the xterm window-manipulation sequence (`CSI 10 ; 1 t`) where the terminal supports it, otherwise a
  hint with the terminal's own shortcut. Never fail the start when full screen is unavailable; say so
  once in the status line.
- **Acceptance:** the setting and flag parsed and remembered; the per-terminal decision unit-tested;
  verified by hand in the classic console and Windows Terminal (say which); help text in both locales.

### P3-18 — Guide panel on screens slightly too small

- **Owner:** Sonnet
- **Depends on:** U-04, P3-15
- **Files:** `crates/tui/src/layout/**`, `crates/tui/src/widgets/guide.rs`, `crates/tui/src/screens/board/**`, tests
- **Scope:** owner feedback: below 130 columns the guide panel silently disappears even when it is on.
  Offer what U-04 recommends, for example: a compact panel (narrower, abbreviated) between 100 and 130
  columns, the panel as a collapsible strip at the bottom, or switchable views (board / guide) with a key
  and a visible hint; when the panel cannot be shown, say so once and how to show it. Never hide a
  setting's effect without telling the player.
- **Acceptance:** renders at 100 × 30, 115 × 35 and 130 × 40 with the panel on; the hint when it does not
  fit; tests for the chosen fallback.

### P3-19 — Auto-finish a won game

- **Owner:** Haiku
- **Depends on:** P2-07
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/settings.rs` and the settings screen, locales, tests
- **Scope:** when the engine reports that the rest of the game is a forced win (P2-07), the board says so
  ("Game solved — finishing") and plays the remaining moves to the foundations one by one at a readable
  pace (each move animated or stepped by the clock, interruptible by any key), then shows the win dialog.
  A setting "Auto-finish" (on by default) and a key to finish at once when it is off. Statistics count
  the game as won with the moves actually played plus the finishing moves.
- **Acceptance:** tests on fixed deals that reach a forced win (Klondike with every card face up and the
  stock empty, FreeCell, Spider where P2-07 supports it); interruption; the setting off; the win recorded.

### P3-20 — Winnable deals only

- **Owner:** Sonnet
- **Depends on:** P2-08
- **Files:** `crates/tui/src/screens/game_select*`, `crates/tui/src/settings.rs`, `crates/tui/src/app/**`,
  `src/` (seed search off the UI thread), locales, tests
- **Scope:** an option in the game selection (remembered) "Winnable deals only": a new game draws seeds
  until the solver (P2-08) proves one winnable within its budget, showing progress and allowing cancel;
  the deal is marked as winnable in the header and in the history. Typed seeds and replays are never
  replaced; the header says whether a typed seed is known winnable, unknown or unwinnable. The search
  must not freeze the interface (a worker thread or chunked steps).
- **Acceptance:** tests with a stub solver (accepts the n-th seed); cancel; the marker in the header and
  history; a typed seed kept as typed.

### P3-21 — Aura options and reshuffle

- **Owner:** Haiku
- **Depends on:** U-04
- **Files:** `crates/tui/src/widgets/aura.rs`, `crates/tui/src/screens/board/**`, `crates/tui/src/settings.rs`
  and the settings screen, `crates/tui/src/input.rs`, locales, tests
- **Scope:** (1) a setting to show only the aura of the focused card and its position, hiding the carry
  and drop-target auras (targets keep their non-color marker, so the information is not lost), or the
  finer choice U-04 recommends. (2) "Reshuffle": deal the current game again with a new seed in one step
  (one key and a menu entry, with confirmation when moves were made, offered in the action line). Ask the
  owner first whether "przetasuj ponownie" also means reshuffling the remaining stock inside a running game
  (a house rule some players use); implement that only if confirmed, as an option off by default.
- **Acceptance:** tests for the aura setting (focus aura present, others absent, markers kept) and its
  persistence; reshuffle starts the same variant and undo mode with a new seed and records the abandoned
  game; help lists the key.
