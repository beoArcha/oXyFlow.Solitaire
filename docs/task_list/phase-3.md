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
- **Done:** #30

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
- **Done:** #36

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
- **Done:** #42

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
- **Done:** #35

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
- **Done:** #34

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
- **Done:** #50

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
- **Done:** #46

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
- **Done:** #48

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
- **Done:** #59

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
- **Done:** #66

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
- **Done:** #73

### P3-14 — Readable card faces

- **Owner:** Sonnet
- **Depends on:** U-04
- **Files:** `crates/tui/src/widgets/card/**` (delete `rank_font.rs`), `crates/tui/src/glyphs.rs`,
  `crates/tui/src/layout/{fan,columns}.rs`, locales (`board.toml` placeholders), tests
- **Scope:** decisions of `docs/tech/ui-review.md` §4 (D-U04-1, D-U04-6). (1) Remove the pixel font:
  the centre of a 9 × 7, 11 × 9 and 13 × 11 card is one row of bold text `K ♣` (rank label, a space,
  suit symbol; two spaces at 13 × 11) above the existing suit pictogram (Small 5 × 2, Medium 5 × 3,
  Large 7 × 4, unchanged); 7 × 5 keeps the single suit symbol. Corner indices stay in both corners.
  No multi-row rank glyph in any mode. (2) ASCII suits become lowercase `c d h s`, so the Polish
  queen of diamonds is `Dd`, not `DD`. (3) A face-down card whose visible part is one row draws
  the pattern in that row (`┌▒▒▒▒▒┐`, ASCII `+#####+`); when the down step is 1 and at least two
  face-down cards lead a column, collapse them to the counted row (`draw_collapsed`) always.
  (4) Placeholders in empty slots (Slot token): `A` in a foundation; `↑` (U+2191, ASCII `^`) in an
  empty stock that can be recycled, `x` when no pass is left; nothing in cells and columns. The stock
  badge reads `x24`. (5) Marks: the hint shows `?→` on the source and `→?` on the target (U+2192;
  ASCII `?>` / `>?`) and uses a hint border set that differs from the normal one in Unicode too; a
  target shows `+` in both top corners and a centred `+` on an empty slot; focus and selection keep
  `►`/`●` and their border sets. (6) The waste fan at 7 × 5 shows two covered cards, each by its
  label width plus one, and no bottom-row fragments of covered cards.
- **Acceptance:** a test renders every rank of every suit at every size in both locales and glyph
  modes and asserts that the corner label and the centre row contain the rank label and the suit
  symbol as text, that no two cards render the same, and that every mark differs by symbol or border
  in both sets; a Spider deal at 100 × 30 shows face-down columns that differ from empty slots;
  renders of all four sizes in both locales reviewed by the owner; `cargo xtask check --quick` passes.
- **Done:** #75

### P3-15 — Screen layout: game information apart from controls

- **Owner:** Sonnet
- **Depends on:** U-04, P3-14
- **Files:** `crates/tui/src/layout/mod.rs`, `crates/tui/src/screens/board/{view,header,status,guidance,counters}.rs`,
  `crates/tui/src/widgets/{guide,footer}.rs`, `crates/tui/src/announce.rs`, locales, tests,
  `docs/tech/architecture.md` (Board guidance and counters paragraphs)
- **Scope:** decisions of `docs/tech/ui-review.md` §3 (D-U04-2, D-U04-3). Size classes from the
  terminal size at every draw: Compact (100–129 columns or fewer than 36 rows), Standard (130–179
  columns and at least 36 rows), Wide (180 columns and more). (1) Game information in one place:
  Compact uses the header row (`variant · seed · time · moves · stock counter · No undo · best time`,
  dropping the best time and then the counter when the row is too narrow); Standard and Wide put the
  same facts in a framed `Game` box at the top of the side panel and remove the header row. The seed
  stays a click target in both. (2) Controls in one place: two rows under the board, as wide as the
  board: `Now:` in the colon form (`Tableau 3: ace of clubs`, `Carrying: ace of clubs (from Tableau 3),
  4 targets`, `Hint: ace of clubs to Foundation 1`, `Game won`, `No legal moves`) with the last event
  right-aligned on the same row in the Success or Error token and cleared by the next action; `Do:`
  with the keys of this exact state, dropping parts from the right, always ending with `? keys` and
  `Esc menu`. The announcer row and the footer leave the board (other screens keep their footer); the
  texts `Cards picked up…` and the stock counter leave `Now:`. (3) The side panel is framed with its
  title in the top border (WGL4 light box, ASCII `+-`), spans the full height, is 30 columns at
  Standard and 40 at Wide, and holds `Game`, `Goal`, `Keys` (usable ones marked as today), `Mouse` and
  a `Legend` row (`►` focus, `●` picked, `+` target, `?` hint); it never repeats a fact of the controls
  strip. (4) The pile name `Pyramid row 7, position 1` becomes `Row 7, card 1`. (5) The fan may grow
  its step by one when the body has spare rows. A screen reader reads `Now:` as the announcer did.
- **Acceptance:** renders at 100 × 30, 130 × 40, 180 × 50 and 200 × 60 in both locales; no fact
  shown twice; Compact has three chrome rows and Standard two; the panel framed; the Polish `Now:`
  rows grammatical (colon form); guidance, counters, header, footer and accessibility tests updated;
  `cargo xtask check --quick` passes.
- **Deviation (accepted):** the variant's best time is not shown in the `Game` box (the board has no history access).
- **Deviation (accepted):** the size class is decided by width only (P3-18 removed the "fewer than 36 rows" rule); below 130 columns the guide opens as a window instead of a panel (D-U04-3).
- **Done:** #80

### P3-16 — Zoom: card size by keyboard and console font

- **Owner:** Sonnet
- **Depends on:** U-04, P3-15
- **Files:** `crates/tui/src/{input,action,settings}.rs`, `crates/tui/src/layout/**`,
  `crates/tui/src/screens/board/**`, `crates/tui/src/screens/settings/**`, `src/` (console font on
  Windows), `Cargo.toml` (`windows-sys`, after the decision), help and settings locales,
  `docs/tech/decisions.md`, tests
- **Scope:** decision D-U04-4 of `docs/tech/ui-review.md` §5.2. (1) Keys: `+` and `=` zoom in,
  `-` zooms out, `0` returns to Auto (`Action::ZoomIn`, `ZoomOut`, `ZoomAuto`, help group Display).
  `RunMore`/`RunLess` move to Shift+Up / Shift+Down (Up/Down already change the depth inside a
  column). Ctrl+mouse wheel zooms when the terminal delivers it. (2) Setting `card_size: Auto | Small
  | Medium | Large | Huge` (serde default Auto; Small = 7 × 5 … Huge = 13 × 11), row `Card size` in
  Settings, changed by the keys and remembered; a size that does not fit falls back to the largest
  that fits and the game information shows `(auto)`; at the limit the status says so. (3) Classic
  Windows console only: Ctrl+Plus / Ctrl+Minus / Ctrl+0 also change the console font by ±2 pixels /
  back to the start size through `GetCurrentConsoleFontEx` / `SetCurrentConsoleFontEx`
  (`windows-sys`, feature `Win32_System_Console`, one justified `unsafe` each, behind a `Console`
  trait in `oxysol` so the TUI stays free of it); record the dependency in `decisions.md`. In every
  other terminal those keys never arrive and are not bound. (4) Help: a `Display` section on the Keys
  tab with the card-size keys and the terminal's own zoom per terminal (Windows Terminal and Linux
  terminals: Ctrl+Plus, Ctrl+Minus, Ctrl+0, Ctrl+wheel; macOS: Cmd+Plus, Cmd+Minus, Cmd+0; classic
  console: the keys above); the guide panel lists `+/-/0 card size`; the too-small notice names the
  terminal's zoom-out key.
- **Acceptance:** tests for stepping up and down, the Auto default, the fallback when the chosen size
  does not fit, the remembered setting, Shift+Up/Down as run size and the help section in both
  locales; the console font change verified by hand in the classic console (say where); `cargo xtask
  check --quick` passes.

### P3-17 — Tab moves the focus

- **Owner:** Haiku
- **Depends on:** U-04
- **Files:** `crates/tui/src/screens/**`, `crates/tui/src/widgets/{menu_list,overlay/**}.rs`,
  `crates/tui/src/{input,action}.rs`, help locales, tests
- **Scope:** decision D-U04-5 of `docs/tech/ui-review.md` §5.1. `Action::FocusNext` (Tab) and
  `Action::FocusPrevious` (Shift+Tab, `KeyCode::BackTab`) in the key table, group Navigation. Every
  screen and dialog owns a focus ring in reading order, wrapping at both ends; Enter and Space activate
  the focused element; Up/Down keep their meaning inside a list, Left/Right inside a value or a button
  row. Rings: main menu = the items; game selection = the list, then `[ Undo: on ]`; seed entry = the
  field, Continue, Back; settings = the rows; statistics = the list, Play again, Copy seed, Back;
  board = the piles in order (Stock, Waste, Foundations, Cells, Tableau 1…n); confirm, win, alert and
  pause = the buttons; the help window keeps Tab for its top tabs (they are its ring). The focused
  element shows `►` directly before its label and reverse video; `MenuList` draws the marker next to
  the label, not at the width of the longest one; the undo toggle is drawn as a button (Accent on
  Background, reversed when focused), not in the focus style.
- **Acceptance:** tests per screen and dialog for the Tab order, wrap-around, Shift+Tab and
  activation; the board cycles the piles; help lists Tab and Shift+Tab; accessibility tests updated;
  `cargo xtask check --quick` passes.
- **Done:** #76

### O-16 — Full-screen option

- **Owner:** Sonnet
- **Depends on:** U-04, P3-16
- **Files:** `src/` (a new module), `src/cli.rs`, `crates/tui/src/settings.rs` and the settings screen,
  locales, `docs/tech/decisions.md`, docs
- **Scope:** owner decision 2026-11-16 and D-U04-8 of `docs/tech/ui-review.md`: "borderless" means
  the game window fills the whole screen with no frame or title bar. A setting `Full screen` (group
  Display) and a flag `--fullscreen`. The game never relaunches itself for this. Classic Windows
  console: `SetConsoleDisplayMode(CONSOLE_FULLSCREEN_MODE)` through the same `Console` trait and
  `windows-sys` feature as P3-16, verified by hand on Windows 10 and 11 (it is the Alt+Enter mode of
  conhost; if the call is refused, the hint names Alt+Enter). Windows Terminal: no API; the hint names
  F11 and Alt+Enter. Linux and macOS: the xterm window-manipulation sequence `CSI 10 ; 1 t` where the
  terminal answers `CSI 8 ; … t` to a size query within 100 ms, otherwise a hint with the terminal's own
  key (F11 in GNOME Terminal and Konsole, Ctrl+Cmd+F in Terminal.app and iTerm2). Never fail the start;
  say once in the `Now:` event slot that full screen is unavailable and which key the terminal uses.
  The help `Display` section (P3-16) lists the setting, the flag and the keys.
- **Acceptance:** the setting and flag parsed and remembered; the per-terminal decision unit-tested as
  a pure function; verified by hand in the classic console and Windows Terminal (say which); help text
  in both locales; `cargo xtask check --quick` passes.

### P3-18 — Guide panel on screens slightly too small

- **Owner:** Sonnet
- **Depends on:** U-04, P3-15
- **Files:** `crates/tui/src/layout/mod.rs`, `crates/tui/src/widgets/guide.rs`,
  `crates/tui/src/widgets/overlay/**`, `crates/tui/src/screens/board/**`,
  `crates/tui/src/screens/settings/options.rs`, locales, tests
- **Scope:** decision D-U04-3 of `docs/tech/ui-review.md` §3.1. Below 130 columns the setting stays
  on and its effect is visible: `g` opens the panel's content (Game, Goal, Keys, Mouse, Legend) as a
  window over the board (`dialog::boxed`, closed by `g`, Esc or Enter, scrollable like the Rules
  tab), the `Do:` row shows `g: guide`, and the settings row reads `Guide panel: On (needs 130
  columns, now 100)`. At 130 columns and more `g` hides and shows the side panel as today. No compact
  side panel and no bottom strip: the board keeps its 100 columns.
- **Acceptance:** renders at 100 × 30, 115 × 35 and 130 × 40 with the setting on (window at the two
  narrow sizes, panel at the wide one); the settings row text at 100 columns; tests for the window's
  keys and for the `Do:` part; `cargo xtask check --quick` passes.
- **Deviation (accepted):** the size class is decided by width only (P3-18 removed the "fewer than 36 rows" rule); below 130 columns the guide opens as a window instead of a panel (D-U04-3).
- **Done:** #83

### P3-19 — Auto-finish a won game

- **Owner:** Haiku
- **Depends on:** P2-07, P3-15
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/settings.rs` and the settings screen, locales, tests
- **Scope:** when `Session::forced_finish` (P2-07) returns the finishing moves, the board says so in
  `Now:` ("Game solved: finishing") and plays them one by one at a readable pace (stepped by the
  clock, interruptible by any key), then shows the win dialog. A setting "Auto-finish" (group Board,
  on by default); when it is off, the `Do:` row offers `f: finish` and the key plays the moves at
  once. Statistics count the game as won with the moves actually played plus the finishing moves.
- **Acceptance:** tests on fixed deals that reach a forced win (Klondike with every card face up and the
  stock empty, FreeCell, Spider, Pyramid as P2-07 supports them); interruption; the setting off and the
  key; the win recorded; `cargo xtask check --quick` passes.
- **Deviation (accepted):** the recorded time includes the paced finishing time; Finish became clickable in P3-25.
- **Done:** #81

### P3-20 — Winnable deals only

- **Owner:** Sonnet
- **Depends on:** P2-08
- **Files:** `crates/tui/src/screens/game_select*`, `crates/tui/src/settings.rs`, `crates/tui/src/app/**`,
  `src/` (seed search off the UI thread), locales, tests
- **Scope:** an option in the game selection (remembered) "Winnable deals only": a new game draws seeds
  until the solver (P2-08) proves one winnable within its budget, showing progress and allowing cancel;
  the deal is marked as winnable in the game information place (P3-15) and in the history. Typed seeds
  and replays are never replaced; the game information says whether a typed seed is known winnable,
  unknown or unwinnable. The search must not freeze the interface (a worker thread or chunked steps).
- **Acceptance:** tests with a stub solver (accepts the n-th seed); cancel; the marker in the game
  information and the history; a typed seed kept as typed.
- **Deviation (accepted):** no hotkey for the "Winnable deals only" toggle; search capped at 40 seeds with "Play as unknown" / "Keep searching".
- **Done:** #82

### P3-21 — Aura options and the stock recycle button

- **Owner:** Haiku
- **Depends on:** U-04, P3-14
- **Files:** `crates/tui/src/widgets/aura.rs`, `crates/tui/src/screens/board/**`, `crates/tui/src/settings.rs`
  and the settings screen, `crates/tui/src/{input,action}.rs`, locales, tests
- **Scope:** decision D-U04-7 of `docs/tech/ui-review.md`. (1) Setting `aura: Full | FocusOnly`
  (serde default Full), row `Aura` in the Board group: `Focus only` draws the focus halo and hides the
  carry and target halos; the `●` and `+` card marks of P3-14 stay, so nothing is lost. (2) Stock
  recycle button (owner clarification 2026-11-19: "przetasuj ponownie" is not a reshuffle): when the
  stock is empty and the waste can go back (passes left, or unlimited), the empty stock slot shows a
  clear recycle sign (WGL4 and ASCII forms, for example `↺` / `@`) and a short label, so the player sees
  that the cards can be turned back. Clicking it, or Enter on it, turns the waste over, the same as the
  draw key. When no passes are left, the slot shows a distinct "no more passes" sign, not clickable.
  Applies to Klondike and to Pyramid's stock/waste where recycling is allowed.
- **Acceptance:** tests for the aura setting (focus halo present, others absent, card marks kept) and
  its persistence; the recycle sign shown with passes left and the "no more passes" sign without;
  click and Enter on the empty stock recycle the waste; both glyph modes; `cargo xtask check --quick`
  passes.

### P3-22 — Localization templates and localized key names

- **Owner:** Haiku
- **Depends on:** P3-15
- **Files:** `locales/en-US/*.toml`, `locales/pl-PL/*.toml`, `crates/tui/src/input.rs`
  (`KeySpec::display`), `crates/tui/src/announce.rs`, `crates/tui/src/i18n/tests.rs`, tests
- **Scope:** `docs/tech/ui-review.md` §2.5. (1) Rule for every locale: a pile name is never the
  object of a verb; it stands after a colon or in parentheses. Rewrite the remaining templates of
  `board.toml` (`announce.*`, `status.*`, `confirm.*`, `error.*`) that break the rule, and add a test
  that no template puts `{pile}` after a word. (2) Localize the key names shown in Help, the footer,
  the guide panel and the `Do:` row through `help.key.*` (`Up`, `Down`, `Left`, `Right`, `Space`,
  `Enter`, `Esc`, `Tab`, `Shift`, `Ctrl`; Polish `Góra`, `Dół`, `Lewo`, `Prawo`, `Spacja`, …).
  (3) Help shows `Shift+R` for restart, not `R`. (4) Board-specific labels for Select and Back where
  a screen-independent label is wrong (`Enter Wybierz` on the board).
- **Input (2026-11-19):** an external Polish review found that `board: action.browsing`,
  `action.browsing_run`, `action.carrying`, `action.cards` put nominative `{pile}` / `{what}` after
  verbs and prepositions that need the accusative or genitive, and that `settings: value.unavailable`
  does not agree with every label's gender. Proposed workaround: label syntax (`Kursor: {pile}, {what}`,
  `W ręce: {what} ({pile}), {targets}`, `kart: {count}, od: {what}`). Use it or a better template.
- **Acceptance:** the key sets of both locales identical (existing test); the template test; renders
  of the board, Help and the guide panel in Polish reviewed by the owner; `cargo xtask check --quick`
  passes.

### P3-23 — Colors: 16-color swatches, mark contrast, High Contrast board edge

- **Owner:** Haiku
- **Depends on:** U-04
- **Files:** `crates/tui/src/theme/{dark,light,high_contrast,palette,tests}.rs`,
  `crates/tui/src/screens/board/view.rs`, tests
- **Scope:** `docs/tech/ui-review.md` §6. (1) Dark theme ANSI column: `border` → `Gray`;
  `border_focus`, `focus_bg`, `target`, `aura_focus` → `LightYellow` (`Yellow` is dark olive in the
  classic console and xterm). (2) Light theme: `slot` → 200, 230, 210 and `White`. (3) High Contrast:
  `surface` → 16, 16, 16; the board body gets a one-cell `Border` line in this theme so the table has
  an edge. (4) `TEXT_PAIRS` gains `Error` and `Success` on `Background`; a new `MARK_PAIRS` list
  (`Target`, `Selection`, `Hint`, `BorderFocus`, `Slot` on `Table`; `Border` on `Surface`) is tested
  at a 3.0 ratio in every theme.
- **Acceptance:** the contrast tests pass with the new pairs; a render of the board at `--colors 16`
  in each theme reviewed by the owner in the classic console; `cargo xtask check --quick` passes.
- **Must close the GRAPHIC_GAPS exception:** draw the High Contrast board edge (the field edge that
  O-26 left to this card), then remove the High Contrast Table/Background entry from `GRAPHIC_GAPS`
  in `crates/tui/src/theme/tests.rs`, so the 3:1 graphics test covers that pair without exception.

### P3-24 — Menus, settings and statistics polish

- **Owner:** Haiku
- **Depends on:** P3-17
- **Files:** `crates/tui/src/screens/{main_menu,game_select,settings,stats}*`,
  `crates/tui/src/screens/settings/options.rs`, `crates/tui/src/screens/stats/format.rs`, locales, tests
- **Scope:** `docs/tech/ui-review.md` §2.6–2.8. (1) Main menu: the tagline under the logo, the
  version in the bottom-right corner, `Resume` names the saved game (`Resume: Klondike, Draw 3,
  12:34, Moves 42`). (2) Game selection: a one-line description and a difficulty word for the
  focused variant under the list. (3) Settings: two aligned columns (`Language ……… ◄ English ►`),
  groups `Display`, `Sound`, `Board`, a description row for the focused option, style names from the
  manifests instead of folder ids, the `Font` value hidden until P5-02. (4) Statistics: shorter
  Polish headings, `Best streak` hidden below 110 columns, `UTC` only in the heading. (5) The win
  dialog says when the time or the move count is a personal best and offers `Same seed`.
- **Acceptance:** renders at 100 × 30 and 130 × 40 in both locales reviewed by the owner; the
  settings values aligned and grouped; tests for the description row and the win dialog's best
  marker; `cargo xtask check --quick` passes.

### P3-25 — Footer keys as mouse buttons

- **Owner:** Haiku
- **Depends on:** P3-15
- **Files:** `crates/tui/src/widgets/footer*`, `crates/tui/src/screens/**` (mouse routing),
  `crates/tui/src/input.rs`, locales, tests
- **Scope:** owner feedback 2026-11-19: the footer actions (Enter, h, s, p, ?, Esc, …) work only by
  keyboard. Each footer entry becomes a button: a click runs the same action as the key, the hovered or
  focused entry is visible without color, and Tab (P3-17) reaches the footer. The guide panel key `g`
  is missing from the footer: add it (and any other board key the footer omits that U-04 lists).
- **Acceptance:** tests that a click on each footer entry produces the same action as its key on every
  screen with a footer; `g` shown and clickable; both locales; accessibility tests updated.
- **Deviation (accepted):** Tab does not reach footer buttons; `Arrows` and `+/-` are display-only.
- **Done:** #86

### P3-26 — Possible moves panel

- **Owner:** Sonnet
- **Depends on:** P3-15, P3-18
- **Files:** `crates/tui/src/widgets/` (a new panel), `crates/tui/src/screens/board/**`,
  `crates/tui/src/layout/**`, `crates/tui/src/settings.rs`, locales, tests
- **Scope:** owner feedback 2026-11-19: the `Possible:` line becomes a side panel that opens like the
  guide panel (its own key, shown in the footer and help). It lists every board action (draw, recycle,
  move to foundation, move to a column, auto-move, undo, redo, hint, restart, …) as a button. A possible
  action is active: a click or Enter runs it. An impossible one stays listed, inactive and marked
  without color (dimmed plus a sign), with a short reason when space allows. The list follows the
  current focus and selection, the same source as the action line. It works with the guide panel on or
  off and follows the size classes of P3-15/P3-18. The panel's visibility is a setting (see P3-27).
- **Acceptance:** tests that the list matches the legal moves on fixed deals; inactive entries render
  and do nothing on click; a click on an active entry runs the move; the toggle key; renders at
  100 × 30, 130 × 40 and 200 × 60; both locales.

### P3-27 — Quick options during the game and defaults in Settings

- **Owner:** Sonnet
- **Depends on:** P3-19, P3-21, P3-26
- **Files:** `crates/tui/src/settings.rs` and the settings screen, `crates/tui/src/widgets/overlay/**`
  (a new quick options dialog), `crates/tui/src/screens/board/**`, `crates/tui/src/input.rs`, locales,
  tests
- **Scope:** owner request 2026-11-19: visual hints and auto-finish are configurable both before and
  during a game. Settings holds the defaults (group "Hints and help": aura mode, target marks, guide
  panel, possible moves panel, auto-finish). A quick options dialog on the board (a key and a footer
  button) toggles the same items for the running game, takes effect at once, and offers "Save as
  default". Changing the defaults in Settings never changes a running game's quick choices unless
  saved there.
- **Acceptance:** tests for each toggle from both places, the immediate effect on the board, "Save as
  default" persisted, defaults applied to a new game, keyboard and mouse; help lists the key; both
  locales.

### O-23 — Modal fixes: scroll bar, Close shortcut, focus between selector rows

- **Owner:** Sonnet
- **Depends on:** P3-18, P3-25
- **Files:** `crates/tui/src/widgets/overlay/**`, `crates/tui/src/widgets/` (a shared scroll bar),
  `crates/tui/src/app/**` (mouse routing), locales, tests
- **Scope:** owner bug report 2026-11-19 (help window and guide window):
  (1) A scrolling modal has no scroll bar, so nothing shows that more text follows and the mouse can only
  use the wheel. Add a vertical scroll bar on the right edge inside the frame of every scrolling modal
  (help tabs, guide window, how-to-play): a thumb sized and placed by the visible share, WGL4 and ASCII
  forms, visible without color. Mouse: click on the track pages up or down, dragging the thumb scrolls,
  the wheel keeps working. Keep the `1-20 / 37` position text.
  (2) The Close button shows its shortcut: `Close [Esc]` / `Zamknij [Esc]`, clickable as before.
  (3) Help, Rules tab: two selector rows (tabs and games) look the same, so the player cannot tell which
  one Left/Right changes, and Up/Down does not move between them. Make the focus explicit: the focused
  row shows the `◄ ►` arrows and the bracketed selection in the focus style; the other row shows its
  selection without arrows and in the plain style (a marker that is not color only). Up/Down moves the
  focus tabs row → games row → text; in the text Up/Down scroll, and Up at the top of the text returns
  the focus to the games row. Left/Right acts on the focused row. Tab and Shift+Tab follow the same
  order (P3-17). The footer names what Left/Right does for the focused row.
- **Acceptance:** tests for the scroll bar (thumb size and position, click on the track, drag, ASCII
  form), the Close label in both locales, the focus order and the Left/Right effect per row, the focus
  marker without color; renders at 100 × 30 in both locales.
- **Deviation (accepted):** Left/Right inside the Rules text do nothing; Keys and Command line show the ◄ ► arrows.
- **Done:** #91

### P3-28 — Docking modals to a side of the screen

- **Owner:** Sonnet
- **Depends on:** O-23
- **Files:** `crates/tui/src/widgets/overlay/**`, `crates/tui/src/layout/**`, `crates/tui/src/settings.rs`
  and the settings screen, `crates/tui/src/input.rs`, locales, tests
- **Scope:** owner request 2026-11-19: a modal such as help, the guide window or rules can be docked to
  the left or right side of the screen at full height instead of covering the board in the middle, so
  the player reads while seeing the cards. A key (and a button in the modal's title or footer) cycles
  centre → right → left; the choice is remembered per modal kind in the settings. A docked modal takes
  the room the guide panel would take and the board keeps its minimum width; when the screen is too
  narrow, docking falls back to centre with a note. Whether the board stays interactive while a docked
  modal is open is decided with U-04's keyboard model (proposed: the docked modal is read-only and the
  board takes the keys, with a key to focus the modal).
- **Acceptance:** renders docked left and right at 130 × 40 and 200 × 60, the fallback at 100 × 30;
  the choice remembered; keyboard and mouse; both locales.

### P3-29 — Settings: an Interface section (colors, aura, size, font)

- **Owner:** Sonnet
- **Depends on:** P3-16, P3-21, P3-24, P3-27
- **Files:** `crates/tui/src/settings.rs` and the settings screen, `crates/tui/src/widgets/aura.rs`,
  `crates/tui/src/layout/**`, `src/` (console font on Windows), locales, tests
- **Scope:** owner request 2026-11-19: Settings gets an "Interface" (UI/UX) section that gathers
  everything about look and readability in one place: theme (interface colors, P3-30), card back
  (P3-31), aura color, aura style (halo, frame, marks only) and aura size (thin or thick), interface
  size (card size and spacing, the setting of P3-16), font size and font choice. Font size and font face
  can only be changed by the program in the classic Windows console (`SetCurrentConsoleFontEx`, shared
  with P3-16; offer the fonts installed that cover WGL4, e.g. Consolas, Lucida Console, Cascadia Mono).
  In Windows Terminal, Linux and macOS terminals the rows show the terminal's own shortcut instead of a
  value and say why. Every change is previewed at once and kept only after confirmation.
- **Acceptance:** tests for each row (value, persistence, immediate preview, revert on cancel); the
  unsupported-terminal note; the console font change verified by hand in the classic console (say
  where); both locales.

### P3-30 — Themes Monokai and Dracula

- **Owner:** Haiku
- **Depends on:** P3-23
- **Files:** `crates/tui/src/theme/**` (two new palettes), `crates/tui/src/screens/settings/options.rs`,
  locales, tests
- **Scope:** owner request 2026-11-19: two more themes based on the public Monokai and Dracula color
  schemes (Dracula is MIT-licensed; credit both in `docs/tech/decisions.md`). Map every theme token,
  including the card faces (red and black suits must stay distinguishable), aura, success and error,
  and the 16-color fallback swatches of P3-23. They must pass the existing contrast tests (WCAG AA for
  text, the mark contrast of P3-23).
- **Acceptance:** both themes selectable and remembered; the contrast tests pass for them; renders of the
  board in both; both locales.

### P3-31 — Card back selection

- **Owner:** Haiku
- **Depends on:** P3-30
- **Files:** `crates/tui/src/widgets/card/**` (the back), `crates/tui/src/theme/**`, `crates/tui/src/settings.rs`
  and the settings screen, locales, tests
- **Scope:** owner request 2026-11-19: the card back is chosen apart from the theme. Styles: "Theme"
  (today's back, the default), "Monokai" and "Dracula" (their colors and a distinct pattern each, WGL4
  shade or box glyphs with an ASCII form). The face-down count mark (`x21`) stays readable on every back.
  A row in Settings (Interface section once P3-29 exists) with a small preview.
- **Acceptance:** every back at every card size in both glyph modes; the count mark contrast test; the
  choice remembered; both locales.

### O-26 — Contrast fixes found by the UI inventory

- **Owner:** Haiku
- **Depends on:** O-25
- **Files:** `crates/tui/src/theme/**`, `crates/tui/src/widgets/card/**` (only if a frame must be drawn
  on another background), tests
- **Scope:** bugs found by `docs/tech/ui-inventory.md` §1.4 (non-text graphics need 3:1, WCAG 1.4.11):
  (1) Light: the focused card's frame (`BorderFocus` on `Table`) is 1.05:1, so the focus is almost
  invisible; (2) face-down card edges (`CardBack` on `Table`) are 1.31–1.60:1 in all themes; (3) High
  Contrast: `Table` and `Background` are the same black, so the field has no edge; (4) Light: `Slot`
  on `Table` 2.75:1, `Hint` 3.00:1; (5) Dark: the aura pairs are 6.51 and 5.64 while `dark.rs:7` claims
  7.0 (fix the claim or the colors). Adjust the palettes (or the frame's background) so every graphic
  pair that carries meaning reaches 3:1, and add these pairs to the contrast tests as a graphics class
  with a 3:1 target so they cannot regress. Keep the themes' character.
- **Acceptance:** the new graphics pairs tested in every theme; the inventory's §1.4 table updated with
  the new ratios; renders of the board in Light and High Contrast.
- **Deviation (accepted):** High Contrast keeps the black field (Table on Background 1:1, listed in GRAPHIC_GAPS); the field edge becomes a drawn frame in P3-23. The Light focus frame is near-black, and the Dark card backs are light blue with a dark mark (forced by the 3:1 bar).
- **Done:** #92

### P3-32 — Table color and feel

- **Owner:** Haiku
- **Depends on:** O-26, P3-29
- **Files:** `crates/tui/src/theme/**`, `crates/tui/src/screens/board/view.rs`, `crates/tui/src/settings.rs`
  and the Interface section, locales, tests
- **Scope:** owner request 2026-11-19: the table is set apart from the theme. Color presets (green felt,
  blue, red, grey, brown, black, "theme default") and a feel: plain, felt (a sparse, low-contrast
  texture with a shade or dot glyph and an ASCII form) or lined. The texture never sits under text or
  card frames. A combination that fails the contrast tests for the cards, slots and auras on that table
  is shown with a warning and the nearest passing shade is offered.
- **Acceptance:** every preset and feel rendered in every theme; the contrast tests run on each
  combination; the choice remembered; both locales.

### P3-33 — Card face tint

- **Owner:** Haiku
- **Depends on:** O-26, P3-29
- **Files:** `crates/tui/src/theme/**`, `crates/tui/src/widgets/card/**`, `crates/tui/src/settings.rs` and the
  Interface section, locales, tests
- **Scope:** owner request 2026-11-19: the card face color is set apart from the theme: white, ivory,
  light grey and a dark face (light text) for dark themes, plus "theme default". Suit colors adapt so red
  and black suits keep their contrast on the chosen face (tested).
- **Acceptance:** every tint at every card size in every theme; suit contrast tests per tint; the choice
  remembered; both locales.

### P3-34 — Low-vision mode

- **Owner:** Sonnet
- **Depends on:** O-26, P3-16, P3-29
- **Files:** `crates/tui/src/settings.rs` and the Interface section, `crates/tui/src/theme/**`,
  `crates/tui/src/widgets/{aura,card}/**`, `crates/tui/src/layout/**`, locales, tests, `docs/wiki/`
- **Scope:** owner request 2026-11-19: one switch "Low vision" that sets a readable bundle and leaves each
  part adjustable afterwards: High Contrast colors (or a 7:1 variant of the chosen theme), the largest
  card size that fits, thick aura, bold rank and suit, no dim text, larger focus and target marks, wider
  fan steps, and a hint to enlarge the terminal font (classic console: the font change of P3-16/P3-29).
  The mode is offered on the first start and in Settings.
- **Acceptance:** the bundle applied and reverted; every text pair at 7:1 in the mode; renders at
  100 × 30 and 130 × 40; both locales; a wiki page section.

### P3-35 — Color-blind modes

- **Owner:** Sonnet
- **Depends on:** O-26, P3-29
- **Files:** `crates/tui/src/theme/**` (palettes and a simulation used by tests),
  `crates/tui/src/widgets/{card,aura}/**`, `crates/tui/src/screens/board/{controls,now}.rs`,
  `crates/tui/src/settings.rs` and the Interface section, locales, tests
- **Scope:** owner request 2026-11-19: a setting "Color vision": normal, protanopia, deuteranopia,
  tritanopia, monochrome. Each mode remaps the meaning tokens to a palette safe for that type (based on
  Okabe–Ito), and every mode removes color-only meaning found by the inventory (§7): focus, target and
  hint get distinct shapes as well as colors; success and error events get a sign (`✓` / `✗` with ASCII
  forms); a four-color deck option (spades black, hearts red, diamonds blue, clubs green) for every
  mode. Tests simulate each color vision type (Machado or Brettel matrices) and assert a minimum color
  difference between tokens that must be told apart.
- **Acceptance:** the simulation tests per mode; the four-color deck at every card size; the signs on
  events; renders per mode; both locales.

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

### P3-10 — How-to-play overlay and first-steps hint

- **Owner:** not recorded (follow-up)
- **Depends on:** P3-07
- **Files:** `crates/tui/src/widgets/overlay/how_to_play/**`, `crates/tui/src/screens/board/**`, `crates/tui/src/{app,input,action}.rs`, `crates/tui/src/screens/main_menu.rs`, `locales/*/{board,help,menu,rules}.toml` (`rules.toml` new), `docs/tech/architecture.md`
- **Scope:** A "How to play" window (`Modal::HowToPlay`) with tabs for Klondike, Spider, FreeCell and Pyramid, each with Goal, Layout, Moves, Controls and Tips. Controls are generated from `KEY_TABLE`; tips use placeholders such as `{hint}`. Entry points: a main-menu item and `i` (`Action::Rules`, handled on every screen). Until the first action of a game, the board status line shows the basic keys. The rules text comes from `docs/wiki/rules`.
- **Acceptance:** `cargo xtask check --quick` passed with 312 tests in `oxysol-tui` (PR #55).
- **Done:** #55

### P3-13 — Large card faces: big suit and rank on medium and large cards

- **Owner:** not recorded (follow-up)
- **Depends on:** P3-11
- **Files:** `crates/tui/src/widgets/card/{mod,art,face,frame,paint,rank_font,tests}.rs` (split from `widgets/card.rs`), `docs/tech/architecture.md`
- **Scope:** Face-up cards on medium and large sizes draw a large centre: the rank in a three-row pixel font above a multi-row suit pictogram (7×4 at 13×11, 5×3 at 11×9, 5×2 at 9×7). The compact 7×5 card keeps its single suit symbol. Unicode art uses WGL4 block characters; ASCII has its own pictograms, so suits differ by shape without color. The rank is built from the localized label, so Polish `W` and `D` work. The corner indices are unchanged.
- **Acceptance:** Tests: art sizes match their scale; Unicode art is WGL4 and ASCII art is ASCII; nothing is painted outside the card; the four suits and 13 ranks differ at every size in both modes; `10` fits. `cargo xtask check --quick` passed (PR #63).
- **Done:** #63

### O-06 — Unicode glyphs within WGL4 for the Windows console

- **Owner:** not recorded (follow-up)
- **Depends on:** P3-02
- **Files:** `crates/tui/src/glyphs.rs`, `crates/tui/src/glyphs/wgl4.rs`, `crates/tui/src/accessibility_tests/board.rs`
- **Scope:** Every Unicode glyph and border symbol is restricted to WGL4, so conhost (Consolas, Lucida Console) on Windows 10 shows no boxes or question marks. Focus marker ▶ ◀ becomes ► ◄; the normal card keeps plain corners; the selected card uses double and single borders (╒═╕│╘═╛); the target card uses ╓─╖║╙─╜; the hint card uses a light-shade border (░). Dialogs use `BorderType::Plain`. Each highlight stays distinct without color: normal (single border), focus (double border and ►), selected (╒ and ●), target (╓ and +), hint (░ and ?).
- **Acceptance:** `wgl4::is_wgl4` tests: every `Glyphs` field is WGL4 and non-WGL4 symbols are rejected; buffers of the menus, settings, game selection, dialogs and every board variant contain only WGL4; ASCII tests unchanged (PR #47).
- **Done:** #47

### O-10 — Remove unused common locale keys

- **Owner:** not recorded (follow-up)
- **Depends on:** M-17
- **Files:** `locales/en-US/common.toml`, `locales/pl-PL/common.toml`
- **Scope:** Removes `[board] header`, `[board] placeholder` (with its empty table header) and `[dialog] help_body`. No Rust code under `crates/` or `src/` references them.
- **Acceptance:** `cargo xtask check --quick` passed (PR #56).
- **Done:** #56

### O-13 — Sync task list statuses with `main`

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/task_list/README.md`, `docs/task_list/micro.md`
- **Scope:** Phase tasks merged to `main` (P3-01…P3-08, P4-01…P4-03, P6-01…P6-03) are marked DONE; the follow-up table lists O-01…O-13, P2-06, P3-10…P3-13 and P4-04 with their status. M-11, M-12, M-15, M-16, M-17, M-25, M-26 and M-27 are DONE, M-34 IN PROGRESS. Statuses come from the task IDs in commit subjects on `main`.
- **Acceptance:** `cargo xtask check --quick` passed (PR #65).
- **Done:** #65

### O-14 — Game rules as a tab of the help window

- **Owner:** not recorded (follow-up)
- **Depends on:** O-13, P3-10
- **Files:** `crates/tui/src/app.rs`, `crates/tui/src/widgets/overlay/{help,mod,tabs,tests,help_rules_tests}.rs`, `crates/tui/src/widgets/overlay/how_to_play/**`, `locales/*/help.toml`, `docs/tech/architecture.md`
- **Scope:** The help window (F1 / ?) gets two top tabs: Keys (the existing table) and Rules (the how-to-play pages, reused without duplication). Tab, Shift+Tab, Left/Right and mouse clicks switch between them. Rules opens on the variant of the current board, Klondike outside boards. `i` and the main-menu item open the help on Rules. Tabs are drawn as `[Keys]  Rules` with bold and reverse, so focus does not depend on color.
- **Acceptance:** Tests cover keyboard and mouse switching, variant persistence, the 100×30 bounds, WGL4 and ASCII glyphs, and locale parity; `cargo xtask check --quick` passed (PR #67).
- **Done:** #67

### O-17 — Task cards for the owner's interface feedback of 2026-11-16

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/task_list/{README,phase-3}.md`
- **Scope:** Cards U-04 (interface review), P3-14 (readable card faces), P3-15 (game information apart from controls, framed guide panel), P3-16 (zoom by keyboard and console font, terminal zoom in Help), P3-17 (Tab and Shift+Tab move the focus) and O-16 (full-screen option). README rows added; O-12, O-13, P3-12, P3-13 and P4-04 marked DONE.
- **Acceptance:** Cards are in `phase-3.md` and the README rows match them (PR #68).
- **Done:** #68

### O-18 — Task cards for the owner's feedback of 2026-11-16 (second round)

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/task_list/{README,phase-2,phase-3}.md`
- **Scope:** Backlog cards, no implementation: P3-18 (the guide panel is missing below 130 columns), P2-07 with P3-19 (forced-win detection and an auto-finish option), P2-08 with P3-20 (a bounded solver and a winnable-deals option), P3-21 (aura options and a reshuffle). Owner priority recorded for U-04: readability first, then ease and intuitiveness. README rows added; O-17 marked DONE.
- **Acceptance:** Cards are in the phase files and the README rows match them (PR #69).
- **Done:** #69

### O-20 — Sync follow-up task statuses

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/task_list/README.md`
- **Scope:** Status sync: O-15, O-19 and P2-07 marked DONE; the saved unfinished start of P3-19 on branch `feat/P3-19-auto-finish-b` is noted.
- **Acceptance:** README statuses match `main` (PR #74).
- **Done:** #74

### O-21 — Task cards for the owner's feedback of 2026-11-19

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/task_list/{README,phase-3,phase-4}.md`
- **Scope:** P4-05: Restart stays allowed in games without undo and is counted separately. P3-21 revised: a visible, clickable recycle sign on the empty stock replaces the reshuffle idea. P3-25: footer keys become mouse buttons and `g` is added to the footer. P3-26: a possible-moves side panel with active and inactive action buttons. P3-27: quick options during the game for hints and auto-finish, with defaults in Settings.
- **Acceptance:** Cards are in the phase files and the README rows match them (PR #77).
- **Done:** #77

### O-25 — UI inventory of colors, sizes and fonts

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `docs/tech/ui-inventory.md` (new)
- **Scope:** A reference of every visual property: color tokens with their values in every theme, text modifiers, card and screen sizes, fan steps, gaps, modal sizes, glyph sets, the aura, the settings that exist, and accessibility findings. Each entry cites `path:line`. Contrast ratios are computed from the palette RGB values with the formula in `theme/contrast.rs`; no test produces them. Three documentation mismatches are recorded, not fixed.
- **Acceptance:** Facts only, no code changed; `cargo xtask check --quick` passed (PR #89).
- **Done:** #89

### O-27 — Sync the task list with `main`

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `docs/task_list/**` only
- **Scope:** Syncs the task list with `main`: status fixes for the rows that were wrong (P2-08, P3-14, P3-15, P3-17…P3-20, P3-25, O-23, M-34, M-35), missing rows (O-20, O-22, O-27), a card for every follow-up ID that had none, `Done:` lines with PR numbers for every DONE card, accepted deviations recorded on their cards, and the status date in the README.
- **Acceptance:** Every ID in a README or micro.md status table has exactly one card, and every merged PR with `[ID]` in its title has a DONE row; any mismatch is listed in the PR report.
- **Done:** #93
