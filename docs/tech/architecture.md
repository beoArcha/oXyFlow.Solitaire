# oXySol architecture — contract

Status: contract v0.2 (2026-11-15: SQLite, seeds, CI — R-16…R-19; P3-01 refined the TUI section to match the code; P3-11 added board guidance). Later tasks may refine it.
Any contract change lands in this file in the same PR. Rationale: `decisions.md`.

## Packages and dependency direction

| Package | Path | Responsibility | Allowed dependencies |
|---|---|---|---|
| `oxysol-core` | `crates/core` | cards, decks, shuffling, rules of four games, session (undo/redo, replay) | `serde`, `thiserror` |
| `oxysol-tui` | `crates/tui` | rendering, layout, input, themes, localization, screens, accessibility | `oxysol-core`, `ratatui` (Crossterm via `ratatui::crossterm`), `serde`, `toml`, `thiserror`, `tracing` |
| `oxysol` | `src/` (lib + bin) | entry point, arguments, paths, SQLite persistence, audio output, logging, panic hook | both above, `rusqlite` (`bundled`), `serde_json`, `directories`, `midly`, `cpal`, `tracing`, `tracing-subscriber`, `thiserror` |
| `xtask` | `xtask/` | `check`, `affected`, `lint`, `sync-metadata`, `fonts`, `midi`, `version` | `syn`, `proc-macro2`, `toml`, `toml_edit`, `thiserror`, `ra-ap-rustc_lexer` |

`oxysol` → `oxysol-tui` → `oxysol-core`. The engine uses no `std::fs`, `std::io`, `std::env`,
`std::time`, `std::net`, `std::process`, `std::thread` and no printing macros.

## Layout

```text
metadata.toml            canonical metadata (name, version, license, URL)
src/                     main.rs lib.rs cli.rs paths.rs storage.rs logging.rs panic_hook.rs error.rs audio/
crates/core/src/         card.rs deck.rs rng.rs pile.rs moves.rs rules.rs game.rs session.rs
crates/core/src/games/   mod.rs (helpers shared by the games) klondike/ spider/ freecell/ pyramid/
                         (each: mod.rs rules.rs moves.rs tests.rs and tests/)
crates/core/tests/       playout.rs (random playouts over Variant::ALL)
                         flows/ (per-variant Session scenarios, golden deals, pinned save JSON)
crates/tui/src/          lib.rs terminal.rs app.rs action.rs input.rs mouse.rs caps.rs clock.rs glyphs.rs
                         i18n.rs settings.rs storage.rs (+ storage/memory.rs) audio.rs announce.rs
                         theme/ layout/ widgets/ screens/
locales/<lang>/          common.toml menu.toml board.toml settings.toml help.toml cards.toml cli.toml audio.toml
assets/cards/fronts/<style>/   style.toml glyphs/*.svg
assets/cards/backs/<style>/    style.toml glyphs/*.svg
assets/cards/codepoints.toml   glyph → code point registry
assets/audio/<style>/          style.toml *.mid (optional scores/*.txt)
xtask/src/               main.rs check.rs affected.rs version.rs sync_metadata.rs lint/ fonts.rs midi.rs
tests/                   architecture/ integration/ e2e/   (each with main.rs)
docs/                    site/ wiki/ tech/ task_list/
.github/workflows/       pr.yml main.yml release.yml
```

## Engine (`oxysol-core`)

The card model separates identity, rank, suit and order. A card's value is not part of the model —
the game that needs one defines it (Pyramid). Appearance and card backs belong to the TUI.

```rust
pub enum Suit { Clubs, Diamonds, Hearts, Spades }
pub enum Color { Red, Black }                       // Suit::color()
pub enum Rank { Ace, Two, /* … */ King }            // Rank::ALL, index() 0..=12, lower(), higher()
pub struct CardId(pub u16);                         // unique within a deal (Spider has duplicates)
pub struct Card { pub id: CardId, pub suit: Suit, pub rank: Rank }

pub enum PileRef {
    Stock, Waste, Foundation(u8), Tableau(u8), Cell(u8),
    Pyramid { row: u8, index: u8 },
}
pub struct TableCard { pub card: Card, pub face_up: bool }
pub struct PileSnapshot { pub pile: PileRef, pub cards: Vec<TableCard> }   // bottom to top

pub enum Move {
    Stock,                                                    // draw / deal a row / recycle
    Transfer { from: PileRef, to: PileRef, count: u8 },       // top `count` cards of `from`
    Remove { first: PileRef, second: Option<PileRef> },       // Pyramid: a pair, or a lone king
}
impl Move { pub fn source(&self) -> Option<PileRef>; pub fn target(&self) -> Option<PileRef>; }

pub enum MoveError { /* rejection reasons: Unsupported, NoSuchPile, EmptySource, FaceDown,
                        InvalidSequence, TargetRejects, StockEmpty, EmptyColumnBlocksDeal, … */ }

pub enum Variant { Klondike(DrawCount), Spider(SuitCount), FreeCell, Pyramid(PyramidRules) }
impl Variant { pub const ALL: [Variant; 8]; }               // every concrete variant, menu order
pub enum DrawCount { One, Three }
pub enum SuitCount { One, Two, Four }
pub enum PyramidRules { Relaxed, Classic }

pub trait Game {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError>;
    fn legal_moves(&self) -> Vec<Move>;
    fn hint(&self) -> Option<Move>;
    fn is_won(&self) -> bool;
    fn piles(&self) -> Vec<PileSnapshot>;
}
pub fn deal(variant: Variant, seed: u64) -> Box<dyn Game>;    // the only place that knows all games
```

Rules of the `Game` contract:

- `apply` validates, executes and performs automatic effects (flip a card, remove a completed run).
  On `Err` the state is unchanged. No input may panic.
- `legal_moves()` returns exactly the set of moves that `apply` accepts, up to the order of a
  Pyramid pair: `legal_moves()` lists each pair once, `apply` accepts both orders.
- Once `is_won()` is true the game is closed: `legal_moves()` is empty, `hint()` is `None` and
  `apply` returns `MoveError::Unsupported`.
- `MoveError` is an expected validation result, not a system error.
- Game modules are `pub(crate)`; the outside world sees only `Game`, `Variant`, `deal` and `Session`.
- `Game` has no supertraits (no `Send`, no `Debug`): the TUI is single-threaded. A bound can be added later.
- Serialized names are snake_case: `{"klondike":"three"}`, `"free_cell"`, `{"pyramid":"classic"}`.

The session implements history by replaying the move log from the seed (decision T-03):

```rust
pub struct SessionRecord { pub variant: Variant, pub seed: u64, pub moves: Vec<Move>, pub cursor: usize }
pub enum RestoreError {
    CursorBeyondMoves { cursor: usize, len: usize },
    MoveRejected { index: usize, source: MoveError },        // replay of the log failed at `index`
}

impl Session {
    pub fn new(variant: Variant, seed: u64) -> Self;
    pub fn restore(record: SessionRecord) -> Result<Self, RestoreError>;
    pub fn record(&self) -> SessionRecord;
    pub fn apply(&mut self, mv: Move) -> Result<(), MoveError>;   // truncates the redo tail
    pub fn undo(&mut self) -> bool;
    pub fn redo(&mut self) -> bool;
    pub fn can_undo(&self) -> bool;                               // cursor > 0
    pub fn can_redo(&self) -> bool;                               // cursor < moves.len()
    pub fn restart(&mut self);                                    // same deal, empty history
    pub fn game(&self) -> &dyn Game;
    pub fn variant(&self) -> Variant;
    pub fn seed(&self) -> u64;
}
```

`restore` validates the whole log, including the redo tail after `cursor`. A rejected `apply` changes
neither the game nor the history. `undo` replays `moves[..cursor - 1]` from the seed; `redo` applies
the next move to the live game. Tests inject a toy game through crate-private constructors that take a
deal function; the public API only offers `new` and `restore`, which use `deal`.

Serialized types (`SessionRecord`, `Move`, `PileRef`, `Variant`) have stable serde names.

## TUI (`oxysol-tui`)

```rust
pub struct Deps {
    pub storage: Box<dyn Storage>,
    pub audio: Box<dyn Audio>,
    pub caps: TerminalCaps,
    pub seeds: Box<dyn FnMut() -> u64>,   // fresh seed for "New game"; a typed seed bypasses it
    pub start_seed: Option<u64>,          // `--seed`: the main menu opens on the games dealt from it
}
pub enum Input { Key(KeyEvent), Mouse(MouseEvent), Resize(u16, u16), Tick }
pub enum Screen { MainMenu, GameSelect, Board, Settings }
pub enum Modal { Help, HowToPlay(HowToPlay), Pause, Confirm { prompt, on_yes: Box<Effect>, yes_focused }, Alert { message }, Win { menu_focused } }

impl App {
    pub fn new(deps: Deps) -> Self;                         // loads settings and the saved game from `Storage`
    pub fn handle(&mut self, input: Input, now: Instant);
    pub fn draw(&mut self, frame: &mut Frame);
    pub fn should_quit(&self) -> bool;
}
pub fn run(deps: Deps) -> Result<(), TuiError>;   // terminal.rs: raw mode, alternate screen, mouse, event loop
pub fn restore_terminal();                        // best effort; a Drop guard calls it on every exit path of `run`
pub enum TuiError { Io(std::io::Error) }

pub trait Storage {
    fn load_settings(&self) -> Result<Option<Settings>, StorageError>;
    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError>;
    fn load_game(&self) -> Result<Option<SavedGame>, StorageError>;
    fn save_game(&mut self, game: &SavedGame) -> Result<(), StorageError>;
    fn clear_game(&mut self) -> Result<(), StorageError>;
    fn start_game(&mut self, variant: Variant, seed: u64) -> Result<GameId, StorageError>;  // history row
    fn finish_game(&mut self, id: GameId, outcome: Outcome, elapsed_ms: u64) -> Result<(), StorageError>;
}
pub struct GameId(pub i64);
pub enum Outcome { Won, Abandoned }
pub struct SavedGame { pub id: GameId, pub record: SessionRecord, pub elapsed_ms: u64 }
pub enum StorageError {
    Io(std::io::Error),                              // file-level failure
    Corrupt { reason: String },                      // unreadable data (the app renames the file)
    Incompatible { found: u32, supported: u32 },     // schema newer than this build
    Backend { message: String },                     // any other backend failure, e.g. a rusqlite error
}
pub struct MemoryStorage { pub settings, pub game, pub history, pub failing }   // test double and fallback

pub enum SoundCue { CardPick, CardPlace, CardFlip, StockDeal, Illegal, Undo, Hint, Win, MenuMove, MenuSelect }
pub struct AudioSettings { pub style: String, pub volume: u8, pub cues: bool, pub music: bool }
pub trait Audio {
    fn available(&self) -> bool;
    fn configure(&mut self, settings: &AudioSettings);   // style, volume, cues on/off, music on/off
    fn play(&mut self, cue: SoundCue);
}
pub struct NullAudio;                                    // always unavailable and silent
```

- **Flow:** terminal event → `input` maps it to a semantic `Action` → `App` hands key and action to the
  current screen (or to the open modal) → the screen returns an `Effect` (`None`, `Goto`, `Start`, `Resume`,
  `Restart`, `Apply(Settings)`, `Open(Modal)`, `HowToPlay`, `Quit`) → `App` applies it. Screens read a `ViewCtx` (theme, glyphs,
  settings, caps, clock instant) and never change shared state or call `Storage` themselves.
  The key table `KEY_TABLE` in `input.rs` is the single source for mapping, the help overlay and the footer;
  `Action::ForceQuit` (Ctrl+C) is handled by `App` on every screen, a key release maps to nothing.
  A screen that types text reports it through `captures_text` (the main-menu seed field): then a typed
  character is text and not a global shortcut (`?` Help, `i` Rules), while F1 and Ctrl+C still work.
  `App` tells the board the terminal size (`BoardScreen::resize`) so cursor movement uses the drawn geometry.
- **Modals:** while a modal is open it takes all input, and the board clock stops. `overlay::handle`
  returns `None` (stay open) or `Some(effect)` (close, then apply).
- **How to play (P3-10):** `Modal::HowToPlay(HowToPlay { rules, scroll })` is the rules window
  (`widgets/overlay/how_to_play/`): tabs for Klondike, Spider, FreeCell and Pyramid, sections Goal, Layout,
  Moves, Controls and Tips from `locales/*/rules.toml`, a fixed 80 × 26 box that scrolls its text. The
  Controls keys are read from `KEY_TABLE`; texts name keys through placeholders such as `{hint}`. It opens
  through `Effect::HowToPlay` (main-menu item "How to play", and `Action::Rules`, key `i`, handled by `App`
  on every screen like `Action::Help`); `App` picks the game of the board's session, else Klondike.
  Keys: Left/Right/Tab switch games, Up/Down/PgUp/PgDn/Home/End and the wheel scroll, Esc/Enter/Close
  close; arrows, tabs and Close are clickable.
- **Board guidance (P3-11):** the board tells the player what is going on and what is possible.
  *Aura:* `widgets/aura.rs` draws a one-cell halo around the focused card or run (`Token::AuraFocus`,
  `Glyphs::aura_focus`), the picked-up run (`Token::AuraCarry`, `aura_carry`) and, weaker, every drop
  target (`Token::Target`, `aura_target`). It paints only free table cells, never a card, so the layout
  does not change and hit tests are unaffected; the three halos use three different symbols, so they
  read without color. The two aura tokens are in `TEXT_PAIRS` against `Table`. *Action line:* two rows
  under the status line, `Now:` (browsing / carrying with the target count / no legal moves / game won)
  and `Possible:` (the actions of this exact state with their keys from `KEY_TABLE`; parts are dropped,
  least useful first, when the row is wider than the screen). `screens/board/guidance.rs` builds both
  from the controller state and `Session` (`legal_moves`, `hint`, `can_undo`/`can_redo`); no rule is
  decided in the TUI. It replaces the first-steps hint; the announcer line (focus and last event) stays.
  *Guide panel:* `widgets/guide.rs`, a 30-column panel at the right of the board (goal of the game, the
  key groups generated from `KEY_TABLE`, the mouse gestures); the keys usable now carry a marker symbol
  and bold type. `Settings::guide_panel` (default `true`; an older saved value without the field loads
  as `true`) switches it; `Action::Guide`, key `g`, and a click on the panel title toggle it through
  `Effect::Apply`. `layout::chrome_with(area, guide)` returns the panel only when the screen is at
  least `BOARD_MIN_WIDTH + GUIDE_WIDTH` = 130 columns wide, so the board always keeps 100 columns.
- **Storage failures:** a failing load or save is logged once in `App` and shown as an `Alert`; settings
  hold for the session, a game that could not be recorded is still played.
- **Too small:** below 100 × 30 cells (T-11) only a notice with the current and required size is drawn;
  only `q` and Ctrl+C work.
- **Localization:** `i18n::t(key)` and `tf(key, params)`; the locale is a per-thread setting
  (`set_locale`). A key is `<area>.<table path>.<name>`, the area being the file name under
  `locales/<lang>/` (`common.menu.new_game` is `[menu] new_game` in `common.toml`). Fallback is
  `en-US`, then the key. Tests keep the key sets of all locales identical and check that every
  `"<area>.…"` literal in the sources exists.
- **Board interaction is shared by all games:** after a source is picked, the TUI filters
  `legal_moves()` by `Move::source()`, highlights targets and applies the matching move. A move with
  no target (a king in Pyramid) applies immediately. A Pyramid pair is picked from either card.
- **Board controller** (`screens/board/`): the cursor is a pile plus a depth (cards from the top; only a
  column has more than one). Left/Right/Up/Down go to the nearest pile in that direction by
  `BoardGeometry` (a row counts twice a column, off-axis piles are penalized); Up/Down first change the
  depth inside a column (`+`/`-` change it too). Empty Pyramid positions are skipped. Enter/Space on a
  pile picks, on a target drops, on the stock draws; a drop on a pile that is not a target is tried
  against the engine and shows its `MoveError` message. Esc cancels a pick, otherwise leaves for the
  menu. Every `MoveError` has a key `board.error.*`; other messages are `board.status.*`; restart,
  new game and quit ask through `Modal::Confirm` (`Effect::Restart`, `Effect::Start`, `Effect::Quit`).
  The screen collects `SoundCue`s and `App` plays them after each input (`BoardScreen::take_cues`);
  `persist()` raises a flag after every session change; `App` reads it (`BoardScreen::take_dirty`)
  at the end of each `apply` and saves (see Autosave below). A win opens `Modal::win()` (help, pause, win and dialogs are drawn by `widgets/overlay/`); a
  position without legal moves shows a status message. `Modal::Pause` hides the cards
  (`BoardScreen::set_covered`); the clock stops for every modal and after a win.
- **Geometry:** `layout::compute(variant, &piles, body) -> BoardGeometry` returns rectangles for piles
  and cards (`BoardGeometry::hit(col, row)` for the mouse); `layout::chrome(area)` splits the screen into
  header, `body`, status line, the two rows of the action line and footer; `chrome_with(area, guide)` also
  reserves the guide panel at the right (see Board guidance). The same result drives rendering and hit-testing. Two layouts:
  columns (Klondike, Spider, FreeCell) and pyramid. Card size is the largest of 13×11, 11×9, 9×7, 7×5
  that fits the window (a column of a king-to-ace run plus a margin must fit); a column compresses
  face-down steps, then face-up steps, then collapses its face-down cards into one row with a counter.
  Highlights are passed as `widgets::pile::Marks` (focus, selected run, targets, hints).
- **Themes:** `ThemeId { Dark, Light, HighContrast }`; a `Palette` has a `Swatch { rgb, ansi16 }` for every
  `Token`; `Theme::new(id, depth)` resolves it: true color emits the RGB, 256 colors use `rgb_to_ansi256`,
  16 colors use the named ANSI color. Components ask `Theme` for tokens and never use colors directly.
  `TEXT_PAIRS` lists the text/background token pairs drawn together; each must reach
  `ThemeId::contrast_target()` (4.5, High Contrast 7.0).
- **Large card faces (P3-13):** `widgets/card/` is a module (`mod.rs` drawing and labels, `frame.rs`
  borders, `paint.rs` clipped painting, `face.rs` centre layout, `art.rs` suit pictograms, `rank_font.rs`
  pixel font). A face-up card whose interior is big enough draws a three-row rank (a 3x5 pixel font in
  half blocks, `^ _ #` in ASCII, taken from the localized rank label; a character without a shape falls
  back to plain text) above a suit pictogram: 7x4 cells at 13x11, 5x3 at 11x9, 5x2 at 9x7. 7x5 keeps the
  single suit symbol. Unicode art uses only WGL4 block characters, ASCII art only printable ASCII; the
  four shapes differ in both sets, and the art uses the suit color token, so nothing depends on color.
- **Card rendering:** front style and back style are selected independently. Each is drawn in one of
  three glyph modes — `Font` (Private Use Area glyphs from the oXySol card font), `Unicode`, `Ascii` —
  described by the style's `style.toml`. `Font` is opt-in through the graphics-check screen.
- **Terminal capabilities:** `TerminalCaps { color, unicode, remote }` from detection, overridden by
  settings and arguments. `remote` (an SSH session) switches audio off.
- **Accessibility:** every `Action` has a key binding and a mouse path. `announce` produces a
  one-line plain-text description of the focus and the last event ("Tableau 3: seven of hearts
  selected, 2 targets") shown in the status area, followed by the last event. The `Unicode` glyph set
  uses only WGL4 symbols (no `▶ ◀`, rounded, heavy, dashed or dotted lines). Sound cues duplicate,
  never replace, visible feedback.
- **Clock:** `GameClock` computes elapsed time from a passed-in `Instant`; pause and modals stop it.

- **Autosave (P4-03):** `app/persist.rs`. A new game calls `Storage::start_game` and logs the deal at
  `info`. After every session change, when leaving the board and on quit, `App` writes
  `SavedGame { id, record, elapsed_ms }` and keeps a copy in memory (the Resume item reads it). A won
  game calls `finish_game(Won)` and `clear_game`; dealing over a save calls `finish_game(Abandoned)`
  first; quitting keeps the save. A corrupt or incompatible save at startup, or one that
  `Session::restore` rejects, gives one alert, `clear_game` (and `Abandoned` for a known id) and the
  menu. A failed write is logged once and alerts once per session; the game goes on. `oxysol-tui`
  re-exports `ratatui::crossterm` so that callers can build `Input` without their own dependency.

## Application (`oxysol`)

- **Persistence:** `SqliteStorage` implements `Storage` over `oxysol.db` in the data directory
  (`OXYSOL_HOME` overrides; an `oxysol.portable` file beside the executable selects `data/` beside it, R-21). Tables: `settings` (one row, JSON), `games` (id, variant, seed,
  started_at, finished_at, outcome, elapsed_ms) and `saved_game` (one row: game id, `SessionRecord`
  as JSON, elapsed_ms, app_version). Schema version in `PRAGMA user_version`; a version newer than
  the app knows is `StorageError::Incompatible`; a file that is not a readable database is
  `StorageError::Corrupt` and is renamed to `*.corrupt`. A missing row is `Ok(None)`. The TUI decides
  how to react (alert, defaults) and logs there.
- **Seeds (R-18):** the binary supplies fresh seeds through `Deps::seeds` (from
  `std::collections::hash_map::RandomState` mixed with the clock — no new dependency). `--seed <u64>`
  (`Deps::start_seed`) opens the main menu on the game selection for that seed, the stage that
  "New game from seed" reaches after a typed seed; the chosen variant starts the deal. Every other new
  game takes a fresh seed. The board header shows the seed; the menu offers "New game from seed".
- **Audio:** `src/audio/` implements `Audio`: MIDI sequencing and a built-in synthesizer as pure
  code (`render` is testable without a device), a pure mixer of cues and music, plus `cpal` output
  (`output.rs`, feature `audio`). Sound styles are embedded from `assets/audio/` at build time. A
  `NullAudio` is used when the feature is off, no device exists, or the session is remote. `available()`
  is true only while the device runs and the selected style loads. Linux release builds are made with
  `--no-default-features` and are silent (T-18).

## Errors and logging

| Kind | Example | Handling |
|---|---|---|
| Gameplay result | illegal move | `MoveError` → status-bar message and cue, no log |
| Recoverable | corrupt save, read-only directory, no audio device | alert or silent fallback, logged once at the boundary, game continues |
| Operational | terminal refuses raw mode | `TuiError` → `main` logs, message on stderr, exit code 1 |
| Critical / panic | broken invariant | panic hook: restore terminal, write diagnostics, message |

Logs go only to `oxysol.log`; nothing is written to stdout or stderr while the TUI runs.

## Quality gates

| Command | Steps | Where |
|---|---|---|
| `cargo xtask check --quick` | fmt check, Clippy `-D warnings`, `xtask lint`, every test target except `architecture` and `e2e` | locally before a PR |
| `cargo xtask check --pr <base>` | fmt, Clippy, lint, the `architecture` and `e2e` suites, unit and integration tests of the packages that `affected --base <base>` selects | `pr.yml` (Linux) |
| `cargo xtask check` | everything: all test targets and `sync-metadata --check` | `main.yml` and `release.yml` on three systems |

Definitions used by `xtask lint`:

- **LOC:** lines that are neither blank nor comment-only.
- **Nesting depth:** the number of nested `if`, `match`, `for`, `while`, `loop`, block (`{}`,
  `unsafe`, `async`) and block-bodied closure constructs inside a function body. The function body
  is level 0. `else if` stays at the level of its `if`. A `match` arm adds no level.
