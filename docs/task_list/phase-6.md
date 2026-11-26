# Phase 6 (0.6) — Themes as data, four-colour French, TriPeaks and game options

Goal: themes and the interface size scale move to data files; the four-colour French card style and the TriPeaks game are added; the owner's game options are added to the games they belong to. Version 0.6.x.

Exit criteria:

- P3-36, P6-18, P6-04 and P6-05 are DONE;
- P6-06 is DONE for the options the owner chose (its DRAFT is finalized by Opus first);
- the developer bridge P6-07, P6-08 and P6-09 is DONE (decision T-42).

Order of work: the theme data first (P3-36, then P6-18), then the game, then the options. P6-04 is covered by P3-35 (see
its card).

Phase 6 review (2026-11-26): the exit criteria are met. Before the gate: O-125, O-126 and O-124, which close the gaps
P6-06 and O-123 left, and O-127 (owner request 2026-11-27). The other open cards of this phase (the solver and hint cards O-115, P6-10, P6-11, P6-13 to P6-17,
and O-116, O-117, O-119, O-120, O-121) moved to phase 7 with their IDs.

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
- **Split (orchestrator, 2026-11-26):** this card is the themes as data, the user theme and the inventory column
  (decision T-51). The interface size scale moved to **P6-18**, because it reaches far outside `layout/**` (the board,
  the overlays and the Settings screen). Approved files added for the user theme: `crates/tui/src/settings/**`,
  `crates/tui/src/app.rs`, `crates/tui/src/app/{deps,modal}.rs`, `crates/tui/src/widgets/overlay/mod.rs`,
  `crates/tui/src/screens/settings/{values,change,options}.rs`, `crates/tui/src/test_support.rs`,
  `tests/integration/{audio,autosave}.rs`, `locales/*/settings.toml`, `docs/tech/architecture.md`,
  `docs/tech/decisions.md`. The user theme is `theme.toml` in the data folder, checked with the Dark contrast rules,
  offered as a **Custom** theme in Settings when valid; a failure shows an alert with a static key and logs the details.
- **Acceptance:** the built-in themes render exactly as before; a valid and an invalid user theme; the inventory column
  filled (items the scale will set say "interface size (P6-18)"); every complete locale.

### P6-18 — The interface size scale S / M / L / XL

- **Owner:** Medium
- **Priority:** P2. Split from P3-36 on 2026-11-26.
- **Depends on:** P3-36
- **Files:** `crates/tui/src/layout/**`, `crates/tui/src/settings.rs`, `crates/tui/src/settings/{interface,aids}.rs`,
  `crates/tui/src/screens/settings/{options,tabs,values,change}.rs`,
  `crates/tui/src/screens/board/{mod,view,commands,zoom,guide_view,moves_panel}.rs`,
  `crates/tui/src/widgets/overlay/{mod,dialog,help,guide_window,moves_window,quick_options,command_line}.rs`,
  `crates/tui/src/widgets/overlay/how_to_play/{content,view}.rs`, `locales/*/settings.toml`,
  `docs/tech/ui-inventory.md`, `docs/tech/architecture.md` (layout contract), tests
- **Problem:** the card size cap, fan steps, gaps, side panel and modal widths are separate constants spread over
  `layout/**`, the board and the overlays; the player cannot make the whole interface larger or smaller in one step.
- **Fix:**
  - One setting **Interface size** with the steps S, M, L and XL (Settings, Display tab), building on P3-16's size
    logic. Each step sets together: the card size cap, the fan steps, the gaps, the side panel width and the modal
    widths, from one table in `layout/**`; the board and every overlay read their sizes from it instead of their own
    constants.
  - The step that renders exactly as today is the default; an old settings file loads as that step.
  - A step that does not fit the terminal falls back to the largest that fits, as the card size does today; the
    "needs N columns" text of the aids (`settings/aids.rs`) follows the side panel width of the step.
  - The inventory column of P3-36 says "interface size" for every item the scale sets.
- **Acceptance:** the default step renders exactly as before (the existing snapshot tests pass unchanged); every step
  renders at 100 × 30, 130 × 40 and 200 × 60 without overflow; an old settings file loads as the default; every
  complete locale fits.

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
- **Status:** DONE, covered by P3-35 (owner decision 2026-11-26): the French style (`assets/cards/fronts/french`, court
  labels V, D, R) together with the **Four-color deck** setting of P3-35, which applies to every style, already gives
  this deck with the theme tokens and the contrast tests of P3-35. No separate style; the ASCII letters stay lowercase
  as in every other style.

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

- **Owner:** Medium
- **Status:** scope finalized by Opus 2026-11-26 from the owner's choice (FreeCell cells moved to P7-10)
- **Depends on:** P4-09, P3-27
- **Decision:** T-53 (the implementer completes the row with `GameOptions`, the storage column and the solver rule)
- **Files:** `crates/core/src/game.rs` (`GameOptions`, `deal`), `crates/core/src/games/{klondike,spider,golf}/**` (only
  these options), `crates/core/src/lib.rs` (the export), `crates/tui/src/settings.rs`, `crates/tui/src/settings/tests.rs`,
  `crates/tui/src/screens/settings/**`, `crates/tui/src/screens/board/header.rs` (the game information),
  `crates/tui/src/app/{persist,deals}.rs`, `crates/tui/src/storage.rs`, `crates/tui/src/storage/memory.rs`,
  `src/storage/{schema,sqlite,history,convert}.rs`, `tests/integration/migration.rs`, `locales/*/settings.toml`,
  `locales/*/board.toml`, `locales/*/rules.toml` (one line per option in the rules tab), `docs/tech/architecture.md`,
  `docs/tech/decisions.md` (T-53), tests
- **Shared files:** `crates/tui/src/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/en-US/settings.toml`,
  `locales/pl-PL/settings.toml`
- **Options (owner's choice, 2026-11-26).** Every option keeps today's rule as its default:
  - **Klondike, passes through the stock:** unlimited (default), 3, 1. When the passes are used up the stock cannot be
    turned again (the bottom bar and O-122 follow `legal_moves()` as they are).
  - **Klondike, empty column:** King only (default), or any card.
  - **Spider, deal with an empty column:** not allowed (default), or allowed.
  - **Golf, King to Ace:** a King and an Ace are not adjacent (default), or they are (wrap).
  - **Golf, play on a King:** a Queen may be played on a King (default, the rule before P6-06), or a King ends the
    waste (harder). Corrected by the orchestrator during P6-06: the engine already let a Queen follow a King.
- **Design:**
  - One `GameOptions` value in `oxysol-core` (`Serialize`, `Deserialize`, `Default` = every rule as today, missing
    fields default), passed to `deal(variant, seed, options)`; the games read only their own fields. No per-option
    branches outside the game that owns the option.
  - **Settings**, Gameplay tab: one row per option, grouped under its game's name; the choice applies to the **next**
    game, as the undo mode does.
  - **Game information** in the board header shows every option that differs from the default, as words, not only a
    mark; with all defaults nothing is added.
  - **History and saved game:** the options are part of the game's record. One column
    `options TEXT NOT NULL DEFAULT '{}'` (JSON of `GameOptions`) appended to `MIGRATIONS` by a single
    `ALTER TABLE ... ADD COLUMN`, no rewrite of rows; an old row reads as the defaults. The saved game in progress
    stores and restores its options; an old save loads with the defaults. Restart and "same seed" keep the options of the
    game.
  - **Solver (owner rule: new solver features go to the next phase):** the solver, the winnable-deals search, the
    curated seeds and the hint search stay for the default options only. A game started with any non-default option
    deals without the winnable search, shows no curated-seed or winnable mark, and the game choice says that the
    winnable search is off for the chosen options. Solver support for the options is a card for phase 7, which this PR
    names.
- **Acceptance:** engine tests for both values of every option (passes counted and enforced; any card on an empty
  column; a Spider deal with an empty column accepted or refused; Golf wrap and play on a King); the default options
  deal exactly as today (existing tests unchanged); settings remembered and an old settings file loads the defaults;
  the migration test opens an old database and reads every row as the defaults; a game with options is recorded and a
  saved game resumes with them; the header shows non-default options; the winnable search is off for non-default
  options; every offered locale fits at 100 × 30.
- **Done:** #298. The rules tab lines went to O-125; the seed screen under options to O-126.

### O-118 — Esc on the main menu asks whether to quit

- **Owner:** Small
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** —
- **Files:** `crates/tui/src/screens/main_menu.rs`, `crates/tui/src/screens/main_menu/tests.rs`, `locales/*/menu.toml`
  (only if the text of the board quit question cannot be reused), tests
- **Problem:** on the start screen (the item list of the main menu) Esc does nothing (`main_menu.rs:73`), so a player
  who presses Esc to leave gets no answer.
- **Fix:** Esc on the item list opens the same confirmation modal as `q` on the board (`board.confirm.quit`, two
  buttons, keyboard and mouse); confirm quits, cancel or a second Esc returns to the list with the focus where it was.
  Esc keeps its current meaning in the seed entry, the game picker and the game search (one step back there). The key
  line of the list shows Esc with the quit label; `q` keeps quitting at once as today.
- **Acceptance:** tests: Esc on the list opens the modal; confirm quits; cancel and Esc close it and keep the focus;
  Esc in the seed entry, the picker and the search still goes back; the key line fits at 100 × 30 in every offered
  locale.

### O-122 — No moves left: a dialog that the game cannot be won

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-26 (seen in TriPeaks; applies to every game).
- **Depends on:** —
- **Files:** `crates/tui/src/app/modal.rs`, `crates/tui/src/widgets/overlay/{mod,no_moves}.rs` (`no_moves.rs` new, on
  the pattern of `win.rs`), `crates/tui/src/screens/board/moves.rs` (where the win screen opens), `locales/*/board.toml`,
  `docs/tech/ui-inventory.md`, tests
- **Problem:** when no legal move is left and the game is not won (TriPeaks and Golf with an empty stock and no card
  that fits, Pyramid, FreeCell or Yukon with a blocked board, …), only the bottom bar says "No legal moves". The player
  has to find out alone that the game is over.
- **Fix:**
  - The moment a move leaves the game **not won and without a legal move** (`legal_moves()` is empty), a dialog
    **No moves left** opens, once per such position, in the same way the win screen opens after a winning move. Its text
    says that no move is possible and that this game can no longer be won.
  - Buttons, left to right: **Undo** (only when undo is allowed and possible; it undoes the last move and closes the
    dialog), **Restart** (same deal), **New game** (the game choice, as on the win screen) and **Menu**. Esc closes the
    dialog and leaves the board as it is; the bottom bar keeps its "No legal moves" part with Undo and New game, so the
    dialog can be dismissed without losing a way on. Keyboard and mouse, focus visible without colour.
  - A game whose rules never run out of moves (for example Klondike with an unlimited redeal) never shows it; the
    dialog follows `legal_moves()` only and adds no rule of its own. Records and statistics are not changed: the game
    is recorded as today when the player leaves it.
- **Acceptance:** tests for TriPeaks, Golf and one tableau game (a constructed blocked position): the dialog opens after
  the last move, not before and not on a won game; Undo appears only when it can act and closes the dialog; Restart,
  New game and Menu do what they say; Esc keeps the board and the bar part; it does not open again for the same
  position after Esc; every offered locale fits at 100 × 30.
- **Done:** #294

### O-123 — Game menu: scrolling by wheel and page keys, families that fold

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** —
- **Files:** `crates/tui/src/screens/game_select.rs`, `crates/tui/src/screens/game_select/**`, `crates/tui/src/settings.rs`
  (the folded families), `crates/tui/src/settings/tests.rs`, `locales/*/menu.toml`, `docs/tech/ui-inventory.md`, tests
- **Problem:** the game list scrolls only by following the focus with the arrow keys; the mouse wheel, Page Up / Page
  Down and Home / End do nothing, the `↑` / `↓` marks at the edges cannot be clicked, and with fifteen games in seven
  families the list is long while some families hold a single game.
- **Fix:**
  - **Scrolling:** the mouse wheel over the list scrolls it by three rows without moving the focus off a visible row
    (when the focused row leaves the view, the focus moves to the nearest visible variant); Page Up / Page Down move the
    focus by one page of rows, Home / End to the first / last variant; a click on the `↑` / `↓` mark scrolls by one
    page. The scroll stays clamped as today.
  - **Folding:** each family heading carries a sign `▾` (open) or `▸` (folded) (`v` / `>` without Unicode). Enter, Space
    or a click on a heading, and Left / Right on a heading or on a variant of it, fold and unfold the family; a folded
    family shows its heading only, with the number of its games. The focus can rest on a heading; Enter on a variant
    still starts it. The folded families are remembered in the settings (`folded_families`, empty by default; an old
    settings file loads with every family open).
  - While a search is active every family with a match is shown open, without changing what is remembered; the
    favourites group folds like a family.
  - The footer names the keys that act in the current focus (fold on a heading, start on a variant) and nothing else.
- **Acceptance:** tests: the wheel scrolls and the focus stays visible; Page Up / Page Down, Home / End; a click on the
  edge marks; folding by key and by mouse, the sign and the game count; the folded set survives a restart; a search
  shows matches in folded families and restores the folding when cleared; an old settings file opens every family;
  every offered locale fits at 100 × 30.
- **Done:** #296

### O-124 — Help, Keys tab: the keys of the game menu

- **Owner:** Small
- **Priority:** P3. Found in the review of O-123 (2026-11-26).
- **Depends on:** O-123
- **Files:** `crates/tui/src/widgets/overlay/help.rs` (`sections()`), `locales/*/help.toml`, tests
- **Problem:** the Keys tab of Help does not list the keys that act only in the game menu: Page Up / Page Down,
  Home / End, `/` (search), `w` (winnable deals) and Left / Right (fold a family).
- **Revised 2026-11-27 (the first attempt stopped correctly):** these keys cannot go into `KEY_TABLE`: a row there needs
  an `Action`, Page Up / Page Down / Home / End / `/` / `w` have none (the game menu reads them itself), and Left / Right
  are already bound to `Action::Left` / `Action::Right`, so a second row breaks `no_key_is_bound_twice`. No change to
  `action.rs`, `input/table.rs` or key handling.
- **Fix:** a **Game menu** section in the Keys tab, built like the Mouse section (`GESTURES`): a constant list of
  (keys text key, description key) pairs in `help.rs`, pushed by `sections()` before the Mouse section. Rows: Page Up /
  Page Down (a page of rows), Home / End (first / last game), `/` (search), `w` (winnable deals on or off), Left / Right
  and Enter / Space on a family heading (fold or unfold the family). The descriptions say what the game menu really does
  (read `crates/tui/src/screens/game_select.rs` and its submodules); new `help.*` keys in every locale under `locales/`.
- **Acceptance:** a test that each key the game menu handles itself (the page keys, `/`, `w`, and the fold keys) has a
  row in the Game menu section; the two-column Keys tab still fits at 100 × 30 in every offered locale.

### O-125 — How to play: the rules tab follows the game's options

- **Owner:** Medium
- **Priority:** P2. Found in the phase 6 review (2026-11-26): a gap left by P6-06, whose file list named
  `locales/*/rules.toml` for "one line per option in the rules tab".
- **Depends on:** P6-06
- **Decision:** T-53
- **Files:** `crates/tui/src/widgets/overlay/how_to_play/{content,pages,state,mod}.rs`, `crates/tui/src/app/effects.rs`
  (`open_how_to_play` only), `locales/*/rules.toml`, tests (`how_to_play/tests.rs`, `how_to_play/render_tests.rs`)
- **Shared files:** `locales/en-US/rules.toml`, `locales/pl-PL/rules.toml`
- **Problem:** the rules tab is built from the variant only (`effects.rs:155`, `HowToPlay::for_variant`). With a
  non-default option it states rules that are false for the game on the board:
  - Klondike `moves_2`: "Only a king ... can go to an empty column", false with any card on an empty column;
  - Klondike `moves_5`: "This can be repeated without limit", false with 3 or 1 passes;
  - Spider `moves_5`: "It cannot be used while a column is empty", false when the deal with an empty column is allowed;
  - Golf: the lines on what goes onto a King and on a King and an Ace not being adjacent, false with either Golf option.
- **Fix:**
  - `HowToPlay` takes a `GameOptions` with the variant: the running game's options when How to play opens from a game,
    otherwise the settings' options (`GameOptions::for_variant`) for every variant the overlay shows.
  - Each rule line an option changes has one key per value in `rules.toml` (for example `moves_5_passes_3`,
    `moves_5_passes_1`); the page shows the line for the options in force **in place of** the default line, so the page
    never contradicts itself. A tip that only holds for the default rule (Klondike `tips_3`) is left out when the option
    differs.
  - With the default options the page is exactly as today (existing tests unchanged). No other game changes.
  - All 9 locales.
- **Acceptance:** for each option and each value, a test that the page shows the matching line and not the default one;
  default options give today's rows; the page opened from a running game follows that game's options even when the
  settings differ; every offered locale fits at 100 × 30.

### O-126 — Seed screen: difficulty levels and played seeds under game options

- **Owner:** Small
- **Priority:** P2. Found in the phase 6 review (2026-11-26).
- **Depends on:** P6-06
- **Decision:** T-53
- **Files:** `crates/tui/src/screens/main_menu/seed_screen.rs`, `crates/tui/src/screens/main_menu/seed_levels.rs`,
  `crates/tui/src/screens/main_menu.rs` (passes the options to the seed screen; the file is near the size limit, add no
  more than the call), `crates/tui/src/screens/mod.rs` (only if `Outcome::Start` needs the options),
  `locales/*/menu.toml`, tests (`main_menu/seed_played_tests.rs` and the seed screen tests)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Problem:**
  - The difficulty levels list curated seeds rated for the default rules (T-53: curated seeds stay for the default
    options). With 1 stock pass a "Very easy" Klondike seed is no longer very easy, and the board then shows no mark, so
    the player sees a level name that is not true.
  - A played seed starts through `Outcome::Start` with the settings' options, not the options the game was played with
    (the statistics replay already uses `Effect::Replay` with the row's options).
- **Fix:**
  - When the settings' options are not the default for the variant (`!options.is_default_for(variant)`), the
    **Difficulty levels** entry is shown dimmed and not selectable, with one line saying that the levels are rated for
    the standard rules; with the default options nothing changes.
  - Replaying a played seed (the list and the random pick) deals with that game's stored options, the same way as the
    statistics replay (`Effect::Replay`).
  - All 9 locales.
- **Acceptance:** tests: levels available with default options, dimmed with the reason line with any non-default option
  of the variant, and an option of another game does not dim them; a played seed recorded with options replays with
  those options while the settings hold others; every offered locale fits at 100 × 30.

### O-127 — A scroll bar on the game menu and the Settings list

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-27 (screenshots of the game menu and of Settings, Audio & Visual tab).
- **Depends on:** O-123
- **Files:** `crates/tui/src/screens/game_select/{list,draw}.rs`, `crates/tui/src/screens/game_select.rs` (mouse
  routing only), `crates/tui/src/screens/settings/{view,plan,list}.rs`, `crates/tui/src/screens/settings.rs` (mouse
  routing only), `crates/tui/src/widgets/scrollbar/mod.rs` (only if a small helper is missing), `docs/tech/ui-inventory.md`,
  tests (`game_select/fold_scroll_tests.rs`, `settings/tests.rs` or a new test file beside them)
- **Problem:**
  - The game menu shows only a muted `↑` / `↓` (`^` / `v`) mark at the edges of the list (O-123). It is easy to miss
    and does not say how long the list is or where the view is.
  - The Settings list scrolls with no mark at all; only the title says "row 21 of 27", so the hidden rows below the
    last visible one are not visible as such.
  - Statistics, Help and How to play already have the shared scroll bar `widgets/scrollbar` (a one-column track with a
    thumb, WGL4 blocks or `#` / `:` in ASCII, click on the track pages, drag the thumb).
- **Fix:**
  - The game menu list and the Settings list draw the shared scroll bar at the right edge of the list whenever the list
    is longer than its view (`Metrics::scrolls`); no bar when everything fits.
  - Game menu: the bar replaces the `↑` / `↓` edge marks (their click becomes a click on the track, which pages); the
    wheel, Page Up / Page Down and Home / End keep working as in O-123, and dragging the thumb scrolls the list; the
    focus stays on a visible row as O-123 requires.
  - Settings: the bar follows the scroll of the rows of the active tab; a click on the track pages and dragging the
    thumb scrolls, with the focus moved to the nearest visible row; the title "row N of M" stays.
  - The bar must not cover a label, value or the `◄ ►` arrows of a row, nor the preview pane: the list gives up one
    column for it, and the layout still fits at 100 × 30 at every interface size and in every locale.
  - No new colour: the bar reads without colour, as the shared widget already does.
- **Acceptance:** tests: the bar appears in the game menu and in Settings only when the list scrolls, at 100 × 30 and a
  tall window; its thumb moves with the keys and the wheel; a click on the track pages and a drag scrolls, in Unicode
  and ASCII; the old edge-mark tests are replaced by bar tests; no row text is cut by the bar in any offered locale;
  `docs/tech/ui-inventory.md` describes the bar instead of the edge marks.

### P6-12 — Solver: no heap allocation per node

- **Owner:** Medium
- **Priority:** P2. Solver review 2026-11-25 (item 3).
- **Depends on:** —
- **Files:** `crates/core/src/solver/hash.rs` (119), `crates/core/src/solver/search/{depth_first,best_first}.rs`
  (71, 91), the `key()` of `solver/{klondike,yukon,freecell}/position.rs` (231, 264, 148: each collects a `Vec<u64>`
  before `feed_unordered`; `spider/position.rs` already uses a stack array), tests
- **Problem:** every key builds a `Vec<u64>` (`klondike/position.rs:224`, `yukon/position.rs:257`,
  `freecell/position.rs:139`), every DFS frame owns a `Vec<Step>`, and `Seen` grows from empty.
- **Fix:** `feed_unordered(&mut PositionHash, &mut [u64])` sorting a stack array; one step arena for the DFS stack
  (frames keep a start index and length); `Seen` pre-sized from the budget (capped). No change of search order, no
  undo-instead-of-clone, no Zobrist hashing.
- **Acceptance:** `solver/tests/api.rs` node counts unchanged (same order); every model test passes; the T-29, T-41
  and T-45 tables re-measured (seeds 0-49, release): same verdicts, average time at least 20 % lower in Klondike, Yukon
  and FreeCell, numbers recorded in T-29.

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
- **Done:** #291 (P6-07, P6-08 and P6-09 in one PR)

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
