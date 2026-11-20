# oXySol architecture — contract

Status: contract v0.6. History: `git log --oneline -- docs/tech/architecture.md` (each squash commit names its task `[ID]`); this line is not a changelog.
Any contract change lands in this file in the same PR. Rationale: `decisions.md`.

## Packages and dependency direction

| Package | Path | Responsibility | Allowed dependencies |
|---|---|---|---|
| `oxysol-core` | `crates/core` | cards, decks, shuffling, rules of four games, session (undo/redo, replay) | `serde`, `thiserror` |
| `oxysol-tui` | `crates/tui` | rendering, layout, input, themes, localization, screens, accessibility, the search for winnable deals (state only; the solving runs behind the `Solver` trait) | `oxysol-core`, `ratatui` (Crossterm via `ratatui::crossterm`), `serde`, `toml`, `thiserror`, `tracing` |
| `oxysol` | `src/` (lib + bin) | entry point, arguments, paths, SQLite persistence, audio output, clipboard, logging, panic hook | both above, `rusqlite` (`bundled`), `serde_json`, `directories`, `midly`, `cpal`, `arboard` (default features off), `tracing`, `tracing-subscriber`, `thiserror` |
| `xtask` | `xtask/` | `check`, `affected`, `lint`, `sync-metadata`, `fonts`, `midi`, `version` | `syn`, `proc-macro2`, `toml`, `toml_edit`, `thiserror`, `ra-ap-rustc_lexer` |

`oxysol` → `oxysol-tui` → `oxysol-core`. The engine uses no `std::fs`, `std::io`, `std::env`,
`std::time`, `std::net`, `std::process`, `std::thread` and no printing macros.

## Layout

```text
metadata.toml            canonical metadata (name, version, license, URL)
about.toml about.hbs     cargo-about configuration and template for THIRD-PARTY-LICENSES.md (release time only, T-31)
src/                     main.rs lib.rs about.rs cli.rs new_window.rs console.rs console/display.rs paths.rs storage/ logging.rs panic_hook.rs error.rs clipboard.rs solver.rs audio/
crates/core/src/         card.rs deck.rs rng.rs pile.rs moves.rs rules.rs game.rs session.rs
crates/core/src/games/   mod.rs (helpers shared by the games) klondike/ spider/ freecell/ pyramid/
                         (each: mod.rs rules.rs moves.rs tests.rs and tests/)
crates/core/src/solver/  mod.rs (API) face.rs hash.rs search/ (depth-first, best-first) klondike/ spider/
                         freecell/ pyramid/ (each: mod.rs position.rs steps.rs tests.rs) tests/
crates/core/tests/       playout.rs (random playouts over Variant::ALL)
                         solver_perf.rs (ignored benchmark: 50 seeds per variant)
                         flows/ (per-variant Session scenarios, golden deals, pinned save JSON)
crates/tui/build.rs      embeds assets/cards/{fronts,backs}/*/style.toml and codepoints.toml (reruns only on assets/cards)
crates/tui/src/          lib.rs terminal.rs app.rs action.rs input.rs mouse.rs caps.rs clock.rs glyphs.rs card_style.rs (+ card_style/manifest.rs)
                         i18n.rs (+ i18n/locale.rs i18n/plural.rs i18n/flags.rs) settings.rs solver.rs storage.rs (+ storage/memory.rs) audio.rs clipboard.rs announce.rs
                         theme/ layout/ widgets/ screens/
locales/<lang>/          meta.toml common.toml menu.toml board.toml settings.toml help.toml cards.toml cli.toml
                         audio.toml rules.toml stats.toml
assets/cards/fronts/<style>/   style.toml glyphs/*.svg
assets/cards/backs/<style>/    style.toml glyphs/*.svg
assets/cards/codepoints.toml   glyph → code point registry
assets/audio/<style>/          style.toml *.mid (optional scores/*.txt)
xtask/src/               main.rs check.rs affected.rs version.rs version_scheme.rs sync_metadata.rs lint/ fonts.rs midi.rs
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

/// Stock counters (P2-06). `recycles` counts waste turned back into the stock (Klondike, Pyramid;
/// Spider is always 0). The first traversal is pass 1, so the pass in progress is `recycles + 1`.
/// `recycles_left`: `None` unlimited (Klondike), `Some(0)` no recycle possible (Spider) or the last
/// one made (Pyramid; Relaxed starts at 2, Classic at 0). `deals_left`: Spider row deals in the stock.
pub struct StockStatus { pub recycles: u32, pub recycles_left: Option<u32>, pub deals_left: Option<u32> }

pub trait Game {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError>;
    fn legal_moves(&self) -> Vec<Move>;
    fn hint(&self) -> Option<Move>;
    fn is_won(&self) -> bool;
    fn piles(&self) -> Vec<PileSnapshot>;
    fn stock_status(&self) -> Option<StockStatus>;       // None for FreeCell (no stock)
    fn forced_finish(&self) -> Option<Vec<Move>>;        // moves that finish a certain win, else None
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
- `stock_status` is read-only and derived from the game state, so undo, redo, restart and restore
  report the counters of the rebuilt game. Its shape is not serialized: no save format changes.
- `forced_finish` (P2-07) is pure, deterministic and cheap: it returns the moves that finish the game
  when the rest is a certain win and `None` otherwise (also for a won game). Applied in order through
  `apply` they are all legal and end in `is_won()`; each sequence is replayed on a clone before it is
  returned (`games/finish.rs`), so a bug can only give `None`. The rules are conservative, a winnable
  position outside them gives `None`: Klondike — stock and waste empty and no face-down card, then
  cards go to the foundations; FreeCell — the closure of foundation moves from cells and column tops
  empties the table; Spider — stock empty and every column one same-suit run, then whole runs are joined
  onto the run they continue; Pyramid — a depth-first search over removals only (no draw, no recycle)
  with a budget of 5000 positions. `Session::forced_finish` delegates; nothing is applied until the
  caller plays the moves through `Session::apply`.
- The solver (P2-08, `crate::solver`, decision T-29) answers for one deal whether it can be won:

  ```rust
  pub enum Verdict { Winnable { moves: Vec<Move> }, Unwinnable, Unknown }
  pub struct SolveBudget { pub nodes: u64 }                 // a count of generated positions, never time
  impl SolveBudget { pub const fn nodes(n: u64) -> Self; pub const fn default_for(v: Variant) -> Self }
  pub fn solve(variant: Variant, seed: u64, budget: SolveBudget) -> Verdict;
  pub fn solve_report(variant: Variant, seed: u64, budget: SolveBudget) -> Report;   // + nodes used
  ```

  Pure and deterministic: the same input gives the same verdict, moves and node count. `Winnable`
  moves are replayed on a fresh `deal` and must end in `is_won()`, otherwise the answer is `Unknown`.
  `Unwinnable` only when the search space was exhausted within the budget (no pruning drops a line a
  win could need); otherwise `Unknown`. The solver sees the whole deal (face-down cards and stock
  order), so "winnable" means thoroughly solvable. Each game has a compact model of its rules built
  from `piles()`; the search never calls the engine except for the replay and for `forced_finish`,
  which ends the search early when a model predicts a certain win. Klondike, Spider and Pyramid use a
  depth-first search, FreeCell a greedy best-first search; every search keeps a visited set of
  128-bit canonical keys (column and cell order ignored). Spider with 2 and 4 suits mostly answers
  `Unknown`. `SolveBudget::default_for` is tuned to about half a second per deal in a release build.
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
    pub fn moves_made(&self) -> usize;                            // applied moves = the cursor; undo takes one back
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
    pub clipboard: Box<dyn Clipboard>,
    pub caps: TerminalCaps,
    pub seeds: Box<dyn FnMut() -> u64>,   // fresh seed for "New game"; a typed seed bypasses it
    pub start_seed: Option<u64>,          // `--seed`: the main menu opens on the games dealt from it
    pub version: &'static str,            // effective version (T-21), computed by the root `build.rs`
    pub sound_styles: Vec<StyleInfo>,     // every embedded sound style, in order (id + name from style.toml)
    pub solver: Box<dyn Solver>,          // checks deals off the UI thread (P3-20)
    pub display: DisplayDeps,             // console (O-16, P3-16): `Box<dyn ConsoleDisplay>`, `DisplayFacts { owns_window, kind, flag }`, `Box<dyn ConsoleFont>`
}
pub enum Input { Key(KeyEvent), Mouse(MouseEvent), Resize(u16, u16), Tick }
pub enum Screen { MainMenu, GameSelect, Board, Settings, Stats }
pub enum Modal { Help, HowToPlay(HowToPlay), Quick(QuickOptions), Pause, Confirm { prompt, on_yes: Box<Effect>, yes_focused }, Alert { message }, Win { focus, best: PersonalBest { time, moves } } }

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
    fn start_game(&mut self, variant: Variant, seed: u64, undo_allowed: bool) -> Result<GameId, StorageError>;  // history row
    fn finish_game(&mut self, id: GameId, outcome: Outcome, elapsed_ms: u64, moves: u32) -> Result<(), StorageError>;
    fn record_restart(&mut self, id: GameId) -> Result<(), StorageError>;   // +1 restart of a game without undo (P4-05)
    fn record_assist(&mut self, id: GameId, assist: AssistCounters) -> Result<(), StorageError>;   // replaces the assist counters (P4-06)
    fn record_proof(&mut self, id: GameId, proof: Proof) -> Result<(), StorageError>;   // winnability of a history row (P3-20)
    fn history(&self) -> Result<Vec<GameRecord>, StorageError>;   // every row, oldest first
}
pub struct GameId(pub i64);
pub enum Outcome { Won, Abandoned }
pub struct GameRecord {                   // one history row
    pub id: GameId, pub variant: Variant, pub seed: u64,
    pub started_at_ms: i64, pub finished_at_ms: Option<i64>,        // UTC, Unix milliseconds
    pub outcome: Option<Outcome>, pub elapsed_ms: Option<u64>,      // None while the game is open
    pub moves: Option<u32>,               // None for rows written before schema v2
    pub undo_allowed: bool,               // the undo mode of the game (P3-12)
    pub proof: Option<Proof>,             // what the solver said about the deal; None while unchecked (P3-20)
    pub counters: GameCounters,           // per-game counters (P4-05); later counters are new fields, the trait stays
}
pub struct GameCounters { pub restarts: u32, pub assist: AssistCounters }   // restarts of a game without undo; 0 for older rows and for games with undo
pub struct SavedGame { pub id: GameId, pub record: SessionRecord, pub elapsed_ms: u64, pub undo_allowed: bool }  // serde default true
pub enum StorageError {
    Io(std::io::Error),                              // file-level failure
    Corrupt { reason: String },                      // unreadable data (the app renames the file)
    Incompatible { found: u32, supported: u32 },     // schema newer than this build
    Backend { message: String },                     // any other backend failure, e.g. a rusqlite error
}
pub struct MemoryStorage { pub settings, pub game, pub history, pub failing }   // test double and fallback

pub enum SoundCue { CardPick, CardPlace, CardFlip, StockDeal, Illegal, Undo, Hint, Win, MenuMove, MenuSelect }
pub struct AudioSettings { pub style: String, pub volume: u8, pub cues: bool, pub music: bool }
pub enum AudioStatus { Ready, RemoteSession, NotBuilt, NoDevice, StyleFailed }   // why sound is or is not available
pub struct StyleInfo { pub id: String, pub name: String }                        // a sound style the settings offer
pub trait Audio {
    fn status(&self) -> AudioStatus;                     // the only status that plays is `Ready`
    fn available(&self) -> bool;                         // provided: `status() == Ready`
    fn configure(&mut self, settings: &AudioSettings);   // style, volume, cues on/off, music on/off
    fn play(&mut self, cue: SoundCue);
}
pub struct NullAudio { status: AudioStatus }             // silent; reports the status it was built with (`NullAudio::new`)

pub trait Clipboard {
    fn copy(&mut self, text: &str) -> Result<(), ClipboardError>;
}
pub enum ClipboardError { Unavailable, Io(std::io::Error) }
pub struct NullClipboard;                                // every copy fails with Unavailable
```

- **Flow:** terminal event → `input` maps it to a semantic `Action` → `App` hands key and action to the
  current screen (or to the open modal) → the screen returns an `Effect` (`None`, `Goto`, `Start`, `Resume`,
  `Restart`, `SameSeed`, `Apply(Settings)`, `Preview(Settings)`, `LeaveSettings { keep, to }`, `Quick(Aids)`, `SaveAids`, `Open(Modal)`, `Board(Action)`, `HowToPlay`, `CopySeed(u64)`, `Quit`; `Start` is
  `{ variant, seed: Option<u64>, undo_allowed: bool }`) → `App` applies it. Screens read a `ViewCtx` (theme, glyphs,
  settings, caps, `audio_status`, `sound_styles`, `saved: Option<SavedSummary>` (variant, elapsed, moves of the saved game, for the Resume row), clock instant) and never change shared state or call `Storage` themselves.
  The key table `KEY_TABLE` in `input.rs` is the single source for mapping, the help overlay and the footer;
  each screen with a key line owns a `FOOTER` table of `(Action, label key)` pairs (O-29): it lists only the keys that work on
  that screen, and each is labelled in that screen's words (main menu: `Enter` Open, no `Esc`; settings: `Enter` Change).
  `Action::ForceQuit` (Ctrl+C) is handled by `App` on every screen, a key release maps to nothing.
  A screen that types text reports it through `captures_text` (the main-menu seed field): then a typed
  character is text and not a global shortcut (`?` Help, `i` Rules), while F1 and Ctrl+C still work.
  `App` tells the board the terminal size (`BoardScreen::resize`) so cursor movement uses the drawn geometry.
- **Modals:** while a modal is open it takes all input, and the board clock stops. `overlay::handle`
  returns `None` (stay open) or `Some(effect)` (close, then apply).
- **Focus (P3-17):** `Action::FocusNext` (Tab) and `Action::FocusPrevious` (Shift+Tab, `KeyCode::BackTab`) are
  rows of `KEY_TABLE` (group Navigation), so the help window lists them. Every screen and dialog owns a focus
  ring in reading order that wraps at both ends; `screens/focus.rs` holds the shared step, marker and button style.
  Rings: main menu language selector, then its items (initial focus stays on the first item); seed entry field, Continue, Back; game selection list, then the undo toggle; settings
  rows; statistics list, Play this seed again, Copy seed, Back; the board's piles (Stock, Waste, Foundations,
  Cells, tableau columns, then pyramid rows; `cursor::cycle`); the buttons of confirm, win, alert and pause
  (Tab toggles the two-button dialogs). The help window walks its parts with Tab and Shift+Tab (see How to play). Enter and Space
  press the focused element; Up/Down (lists) and Left/Right (value or button row) keep their meaning. Typed
  digits reach the seed field only while it has the focus. The focused element carries the marker `►` (`>` in
  ASCII) directly before its label and, for buttons, reverse video; `MenuList` draws the marker next to the label.
- **Docked help window (P3-28):** only the help window (`Modal::Help`, `HowToPlay`, `CommandLine`, with its
  tabs) docks; the Guide window and the confirm, pause, win and alert dialogs stay centred. `d` cycles the
  placement centre → right → left, also by a click on the placement button on the top border (`Dock: <side>`);
  the choice is `Settings::help_dock` (serde default Centre), saved at once. A docked window is a column at full
  height, `overlay::dock::split`: its width is the room beyond the board's minimum width (`BOARD_MIN_WIDTH`),
  between 30 and 60 columns, and the rest of the screen (the board) draws beside it. Below 130 columns it stays
  centred and a note says so. Its text is word-wrapped and scrolls by wrapped lines (`dock/dock_view.rs`);
  Keys is one column. Keys: while it is open the board keeps the keys (its clock runs) until F6 gives the
  window the focus (F6 again returns it); the window then takes Tab, Left/Right, Up/Down, PgUp/PgDn, Home/End,
  Esc and Enter (close); `?` closes it from the board. The wheel and clicks over the window go to it; the arrows
  of its tab and game rows and its Close button are clickable. Focus shows without colour: the title carries
  a marker and reversed text.
- **Language selector (P3-37):** the top-left corner of the main menu (`widgets/language_picker/`, state `LanguagePicker`,
  drawn by `MainMenu::draw_items` last, over the logo area). It is a ring element before the items (Tab from the last item and
  Shift+Tab from the first reach it; Tab and Shift+Tab leave it) and carries the marker and the focus style like any other.
  Layout in Unicode with colors: `◄ [flag] ► ▼` over two rows (the flag is 4 × 2 cells of `▀` with foreground and background
  from the 4 × 4 pixel grid of `i18n::flags::flag(locale)`, the language code under the right arrow). In ASCII glyph mode or at
  16 colors the code replaces the flag: `< EN > v`. Keys while it has the focus: Left/Right choose the previous/next
  `Locale::offered()` entry (complete locales only, T-36), Down, Enter and Space open the list; in the open list Up/Down move, Enter chooses, Esc closes, and every
  other key is swallowed. Mouse: a click on an arrow chooses or opens, a click on the flag or code gives it the focus, in the
  open list a row chooses (hover and wheel move the highlight) and a click outside closes it without reaching what lies below.
  The picker never changes the settings: a choice becomes `Effect::Apply(Settings { locale, .. })`, the same effect as the Language
  row of the settings screen, so the language applies and is saved at once. The tooltip is a framed box under the corner with the
  language's own name (`Locale::own_name()`, from `meta.toml`, independent of the current language); it shows
  while the pointer (`ViewCtx::pointer`) is over the corner or the selector has the focus. A flag is data in `i18n/flags.rs`
  (`Flag { code, pixels }`); a new locale adds its flag there and names it in the locale list. Flag colors are
  content, not theme tokens (T-34).
- **How to play (P3-10, O-14, O-15):** `Modal::Help` (the keys table), `Modal::HowToPlay(HowToPlay { rules, scroll })`
  (the rules pages) and `Modal::CommandLine(CommandLine { scroll })` (the options of `oxysol --help`) form the help
  window (`widgets/overlay/`): three top tabs ("Keys", "Rules" and "Command line") switch between them by
  Tab / Shift+Tab (all tabs, wrapping), Left-Right (Keys and Command line), `?` and `i` and mouse clicks.
  The Command line tab (`overlay/command_line.rs`) shows the `cli.help` text, the same key `--help` prints, so
  the two cannot differ; it scrolls by Up/Down/PgUp/PgDn/Home/End and the wheel inside the box of the Keys tab. The rules tab (`widgets/overlay/how_to_play/`)
  provides tabs for Klondike, Spider, FreeCell and Pyramid, sections Goal, Layout, Moves, Controls and Tips from
  `locales/*/rules.toml`, scrolling its text inside the dialog box. The Controls keys are read from `KEY_TABLE`;
  texts name keys through placeholders such as `{hint}`. It opens through `Effect::HowToPlay` (main-menu item
  "How to play", and `Action::Rules`, key `i`, handled by `App` on every screen like `Action::Help`); `App`
  picks the game of the board's session, else Klondike. Keys: Tab cycles the tabs, Left/Right switches
  games in Rules, Up/Down/PgUp/PgDn/Home/End and the wheel scroll, Esc/Enter/Close close; arrows, tabs and
  Close are clickable.
  *Selector focus (O-23):* the Rules tab has three parts in reading order, the tabs row, the games row and the text
  (`how_to_play::Focus`; the window opens on the games). Up and Down move the focus down and up through them; inside
  the text they scroll, and Up at the top returns the focus to the games. Left and Right act on the focused part only:
  the top tab on the tabs row, the game on the games row, nothing in the text. Tab and Shift+Tab take the same order and
  continue to the next or the previous top tab at either end; Keys and Command line, which have only a tabs row, keep
  Tab and Left/Right for the top tabs. The focused row shows the arrows `◄ ►` (`<` `>` in ASCII) and its selection in
  bold reverse video; the other row shows its selection as plain `[Name]` without arrows; the focused text shows `►` in
  its left margin. The footer names what Left/Right does for the focused part (`help.rules.footer_*`).
  *Close and scroll bar (O-23):* the Close button of every tab and of the guide window reads `Close [Esc]`
  (`help.rules.close` with the key). Every scrolling window (Rules, Command line, guide window) draws a vertical
  scroll bar (`widgets/scrollbar.rs`) in the last column inside the frame, from `Glyphs::scroll_track` and
  `scroll_thumb` (`░ █`, ASCII `: #`), only while the text is longer than the view; the thumb's length is the visible
  share and its place the position. A click on the track above or below the thumb pages up or down, dragging the thumb
  scrolls (the grab row lives in the window state until the release), the wheel and the position text stay.
- **Board guidance (P3-11):** the board tells the player what is going on and what is possible.
  *Aura:* `widgets/aura.rs` draws a one-cell halo around the focused card or run (`Token::AuraFocus`,
  `Glyphs::aura_focus`), the picked-up run (`Token::AuraCarry`, `aura_carry`) and, weaker, every drop
  target (`Token::Target`, `aura_target`). It paints only free table cells, never a card, so the layout
  does not change and hit tests are unaffected; the three halos use three different symbols, so they
  read without color. The two aura tokens are in `TEXT_PAIRS` against `Table`. `Settings::aura` (P3-21;
  `AuraMode::Full` by default, `Focus only` = `focus_only`; an older value without the field loads as `Full`)
  decides the halos: `FocusOnly` draws the focus halo only and drops the carry and target halos, while the
  `●` and `+` card marks stay. *Controls strip (P3-15; it replaces the P3-11 action line, the announcer row and the footer on the board):*
  two rows under the board, as wide as the board only (`screens/board/controls.rs`). `Now:` says what is
  focused, carried or hinted in the colon form `{pile}: {what}` (`Tableau 3: ace of clubs`, `Carrying: ace of
  clubs (from Tableau 3), 4 targets`, `Hint: ace of clubs to Foundation 1`, `Game won`, `No legal moves`; a pile
  name is never the object of a verb, so Polish needs no declension; the texts are built in
  `screens/board/now.rs`) and carries the last event right-aligned on the same row in the Success or Error token
  (`board.status.*` and `board.error.*`, at most 40 columns, tested; cleared by the next action). `Do:` lists the
  actions of this exact state with their keys from `KEY_TABLE`; parts are dropped, least useful first, when the row
  is wider than the board, but a fixed tail always stays: `?: keys` and `Esc: menu` (while cards are carried Esc
  puts them back, so the tail is `?: keys` only). `screens/board/guidance.rs` builds both rows from the
  controller state and `Session` (`legal_moves`, `hint`, `can_undo`/`can_redo`); no rule is decided in the TUI.
  The `Do:` row carries `p: pause` and `v: moves` in its tail (O-37, drop order 4); when it has no room for them the `Now:` row
  ends with the same buttons (`controls::Buttons`), so both stay clickable at every width.
  The strip is drawn on `Surface`, so `Text`, `Success` and `Error` use pairs that `TEXT_PAIRS` tests. The `Now:`
  text is what a screen reader reads; `announce.rs` keeps only the names (`pile_name`, `card_name`, `look_text`,
  `targets_text`), the separate announcer line and its `focus`/`announce` functions are gone. The texts `Cards
  picked up…` and the stock counter are not in `Now:`.
  *Guide panel (P3-11, framed by P3-15):* `widgets/guide/` (`mod.rs` frame and layout, `keys.rs` goal, keys and
  mouse sections, `legend.rs`). A framed panel (WGL4 light box `┌─┐│└─┘`, ASCII `+-+|`) with its title in the
  top border (`Guide (g: hide)`; a click on the top border toggles it), full height at the right of the board:
  30 columns at Standard, 40 at Wide. From the top: a `Game` compartment (heading plus the game facts, closed by a
  rule `├──┤`), then `Goal`, `Keys` (usable ones carry a marker symbol and bold type), `Mouse` and `Legend` (`►`
  focus, `●` picked, `+` target, `?→` hint, in the symbols of the glyph set). Blank rows between the sections
  appear only when every section still fits. It never repeats a fact of the controls strip. `Settings::guide_panel`
  (default `true`; an older saved value without the field loads as `true`) switches it; `Action::Guide`, key `g`,
  and the title click toggle it through `Effect::Apply`.
  *Size classes:* `layout::Class::of(area)` is decided at every draw from the width alone (P3-18 dropped the
  height limit): Compact below 130 columns, Standard from 130 columns, Wide from 180 columns. The panel exists in
  Standard and Wide only, and only when the setting is on, whatever the height (a short wide screen, 130 × 30,
  has it; the terminal is never smaller than 100 × 30, so the board keeps its 100 columns and at least 28 rows
  beside it). Chrome rows: Compact has the header row plus the two strip rows; Standard and Wide have the two
  strip rows only, the facts moving into the `Game` compartment. When the panel is off (setting) the header row
  comes back, so the game information has exactly one place at all times. When the panel is shorter than its
  sections, `widgets/guide` drops the `Mouse` section first and the `Legend` next (`fitted`), so a heading never
  stands without its lines.
  *Guide window (P3-18):* below 130 columns the setting stays on and its effect stays visible. `g`
  (`BoardScreen::panel_unavailable`, the class of the last drawn size) opens `Modal::Guide(GuideWindow)`
  (`widgets/overlay/guide_window.rs`, a `dialog::boxed` of 62 columns and 17 text rows) with the same content as
  the panel, laid out by `guide::window_lines` (`Game`, `Goal`, `Keys` with the usable ones marked, `Mouse`,
  `Legend`, a blank row between them). It scrolls with Up, Down, PgUp, PgDn, Home, End and the wheel, shows its
  position in the footer, and closes with `g`, Esc, Enter or Close; the clock stops while it is open and no key
  reaches the board. The `Do:` row of a Compact screen carries `g: guide` in its fixed tail (before `?: keys`),
  and the settings row reads `Guide panel: On (needs 130 columns, now 100)` (`settings.value.needs_columns`;
  `options::line` takes the screen width). From 130 columns `g` hides and shows the side panel as before.
- **Possible moves panel (P3-26):** `widgets/moves.rs` (the framed list, `MoveEntry`, `PanelMap`),
  `screens/board/moves_panel.rs` (entries, focus, clicks) and `widgets/overlay/moves_window.rs`. Every board
  action (stock draw, deal or recycle, pick up or drop, quick move or send to foundation, put back, hint, undo,
  redo, finish, restart, new game) is one button; `BoardScreen::move_entries` takes "active" from
  `Guidance::usable`, the same source as the `Do:` row, so the panel never decides a rule. An inactive entry stays
  listed, dimmed and marked `×` (ASCII `x`; active `•`, ASCII `o`: `Glyphs::entry_on/entry_off`), and carries a
  short reason (`board.moves.reason.*`) that is drawn under it when every reason fits in the panel. Key `v`
  (`Action::Moves`, help group Moves) and a click on the title toggle `Settings::moves_panel` (default `true`; an
  older value without the field loads as `true`) through `Effect::Apply`. *Layout:* `layout::chrome_panels(area,
  guide, moves, frame)` (`chrome_with(area, guide, frame)` is the guide-only form) puts the panels in the one side column of the size class
  (30 or 40 columns): alone either takes the full height (without the guide the header row is back); together the
  guide is above and the moves panel below it with 12 rows (22 from a column height of 56, where the reasons fit),
  at most half the column. `Chrome::moves` is the new rectangle; the board width never changes. Below 130
  columns `v` opens `Modal::Moves(MovesWindow)` with the same entries; Up, Down and Tab choose one, Enter or a
  click runs an active one through `Effect::Board(Action)` (the application runs the action as its key would,
  after the window closed), an inactive one does nothing. *Focus:* while the panel is drawn it is the last stop of
  the board's Tab ring (piles, then the entries, then round); Up and Down choose an entry, Enter and Space run it
  (an inactive one only says `board.status.not_possible`), Esc, Left and Right return to the piles.
- **Settings values and arrows (P3-24 part 2):** the focused row frames its value as `◄ value ►` (`<` `>` in ASCII, the
  `focus_right` and `focus_left` glyphs); every other row keeps the same two columns blank, so the values line up.
  `settings/arrows.rs` records the arrows' cells when the screen draws: a click on `◄` steps the value back, a click on
  `►` forward, a click elsewhere on the row forward as before (`settings/values.rs::line`).
- **Aids and quick options (P3-27):** the board aids are `Settings::aura`, `target_marks` (default `true`; an older
  value without the field loads as `true`), `guide_panel`, `moves_panel` and `auto_finish`; `settings/aids.rs` holds them as
  `Aids` (a `Copy` value; `Settings::aids()` and `with_aids()`) and `Aid` (one row: label, description and value keys,
  `toggled`, `value_for`), shared by the Settings screen and the quick options dialog. Settings groups them under
  "Hints and help" (`Group::Hints`, replacing the old Board group), after Display and Sound; the moves panel and the
  target marks got rows there. They are the *defaults for a new game*. The running game has its own copy:
  `App::quick` (`App::aids()`), set from the defaults whenever a game is dealt or a saved game other than the one
  on the board is resumed (a restart and resuming the same game keep it, the choices are not saved with the game).
  The board and the modal layer draw with `App::running` (the settings with `quick` laid over their aids) while the
  screen is the board; every other screen sees the settings. So `ViewCtx::settings` on the board *is* the running game's
  view, and a default changed in Settings never reaches a game in progress. `target_marks: false` leaves `Marks::targets`
  empty (no `+` and no target halo); the focus and the carried run stay marked. *Quick options:* key `o`
  (`Action::Options`, help group Moves, `board.action.options`) and the footer button `o: options` in the fixed tail
  of the `Do:` row open `Modal::Quick(QuickOptions { aids, focus })` (`widgets/overlay/quick_options.rs`). Its focus ring is
  the five rows, "Save as default" and Close (Tab, Shift+Tab, Up, Down wrap; Left/Right change a row or move between the
  two buttons; Enter and Space press; a click presses; Esc, `o` and Close close). Every change reaches the game at
  once: after each key or click `App::sync_quick` copies the dialog's `aids` into `quick` while the dialog is open.
  "Save as default" returns `Effect::SaveAids`: `quick` becomes the settings' defaults, they are saved, the dialog
  closes and the `Now:` row says `board.status.defaults_saved`. `g` and `v` on the board (and the title clicks)
  are quick choices too (`Effect::Quick`): they do not save; "Save as default" is the only way to persist them.
- **Interface section (P3-29):** Settings has the groups Display (language, colors, full screen), Interface, Sound and Hints. Interface
  holds every row about look: theme, card back, front style, back style, symbols, console font, card size and the aura look
  (`Settings::aura_colour` `AuraColour { Theme, Text, Selection, Hint }`, `aura_style` `AuraStyle { Halo, Frame, MarksOnly }`,
  `aura_size` `AuraSize { Thin, Thick }`, in `settings/interface.rs`; older values without the fields load as Theme, Halo, Thin). The aura
  colour is one theme token for all three halos (still told apart by symbol); `Frame` draws a line frame (`┌─┐` for the focus, `╔═╗`
  for the carried run, plain ASCII `+-|` and `#=H` in ASCII mode; a target keeps its dot), `Thick` is two rings of free cells instead of
  one. `settings/sample.rs` draws a sample of the three halos under the list.
  *Console font face and size:* two more Interface rows follow Console font: `Settings::console_face` (`ConsoleFace { Default,
  Consolas, LucidaConsole, CascadiaMono }`, `settings/console_face.rs`; `Default` leaves the face the console had) and
  `console_font_size` (pixels from `CONSOLE_FONT_SIZES`, `0` leaves the size; older values load as the default). Only the classic Windows
  console lets the game change them: `ConsoleFont` gets `use_face(Option<&str>) -> CardFontState` and `use_size(Option<u16>) -> bool`
  (default: unavailable). In Windows Terminal, Linux and macOS terminals both rows show the terminal's own zoom shortcut instead of a
  value (`TerminalKind::font_keys_key`: `Ctrl+= / Ctrl+-`, `Ctrl+Shift+= / Ctrl+-`, `Cmd+= / Cmd+-`) and their description says the game
  cannot change that font; a step on them does nothing. The oXySol font option and the face row share one owner: `FontZoom` in
  `src/console/font.rs` keeps the font the console had before the game first changed its face (`saved`) and `start`, the font at launch.
  The card font owns the face while it is in use (`use_face` then changes nothing and the row says "oXySol font"); turning it off puts
  the player's face back through `App::sync_console_look` (card font, then face, then size). Putting a face back keeps the size chosen
  meanwhile; at exit `FontZoom` sets `start` back whenever the game changed the face, the size or the card font. A face is checked by
  reading it back, as for the card font: `Off` / `Active` / `NotRegistered` (shown "not installed") / `Refused`. The Settings
  description of the focused row wraps over two lines.
  *Preview and confirmation:* a change of an Interface row returns `Effect::Preview(settings)`: `App` shows it at once
  (`show_settings`, which also drives the console font and the theme) without saving and keeps the settings as they were in
  `interface_baseline` (`app/interface.rs`). While it is set, `persist_settings` writes the baseline's interface fields
  (`Settings::with_interface_of`), so a row outside the section saves without the preview and a closed program never keeps one. A
  preview that returns to its start clears the baseline. Leaving the settings screen with a preview open (`leave_or_ask`) opens a
  `Modal::Confirm` built with `Modal::confirm_or(prompt, on_yes, on_no)`: Yes is `Effect::LeaveSettings { keep: true, to }`, No and Esc are
  `keep: false`, which restores the interface fields and saves. A note under the list says a preview is open. The list scrolls when
  the screen is too low for all rows (`settings/list.rs::first_shown`); the title then shows the position of the focused row.
- **Zoom (P3-16, R-24):** `Settings::card_size` is `CardSizeSetting { Auto, Small, Medium, Large, Huge }` (`settings/card_size.rs`;
  default `Auto`, an older value without the field loads as `Auto`; Small 7 × 5 … Huge 13 × 11) and the Settings row `Card size` in
  the Interface group. `Auto` is the largest size that fits, a named size is the largest the player wants: a window too small for it falls
  back to the largest that fits (`CardSize::largest_fit_within`) and the game information gets the fact `Cards: <size>` (dropped first
  when the header row is narrow), `Cards: <size> (auto)` when it fell back; with `Auto` there is no such fact. `Action::ZoomIn` (`]`),
  `ZoomOut` (`[`) and `ZoomAuto` (`0`) are rows of `KEY_TABLE` in the help group Display, and Ctrl+mouse wheel up or down is the same
  as `]` or `[` where the terminal delivers it; `+` and `-` stay `RunMore` / `RunLess`. The step is the pure
  `settings::zoom_step(shown, largest, Zoom) -> Zoomed::{To(setting), AtLimit}`: `shown` is the size drawn now and `largest` the size
  of `layout::compute` for the same `chrome().body`, so the field inset of a theme with a frame (`FRAME_INSET`, only when
  `Theme::field_edge`) counts. Zooming in stops at the largest that fits and zooming out at 7 × 5, and the event slot says so
  (`board.status.zoom_largest` / `zoom_smallest`, both refusals). The board returns `Effect::CardSize(setting)`, which `App` saves
  with the settings without taking the quick choices of the running game along; `0` says `board.status.zoom_auto`. The help window's
  Display topic adds the row `Ctrl +/-/0` for the terminal's own zoom (`help.display.*`); to fit a 100 × 30 terminal the topics of the
  Keys tab have no blank rows between them. The too-small notice names `Ctrl+Minus`. *Console font (T-35):* Ctrl+Plus, Ctrl+Minus and
  Ctrl+0 reach `Deps::display.font` (`ConsoleFont::step(FontStep)`) before any other handling; `oxysol` wires `console::font::FontZoom`
  only in the classic Windows console, every other terminal gets `NullConsoleFont`.
- **Board counters, no-undo mode and seed copying (P3-12):** the game information (`screens/board/header.rs`, `Facts`) reads
  `variant | seed N | Time m:ss | Moves: N | stock counter | No undo` in one header row at Compact size or as the
  lines of the `Game` compartment at Standard and Wide size (the counter is the first fact dropped when the header
  row is too narrow; the best time of the variant is not shown yet, the board has no access to the history) (`Session::moves_made()`; the win dialog shows the time and the moves too). The stock counter
  comes from `Game::stock_status` (`screens/board/counters.rs`): Klondike "Turned over: N", Pyramid
  "Pass N of M" (`recycles + 1` of `recycles + recycles_left + 1`), "Last pass (N of N)" when no recycle
  is left, "No passes left" once the last pass has emptied the stock, Spider "Deals left: N", FreeCell
  none. It is game information and is shown only there, never in `Now:` and never beside the stock. *No-undo mode:* every game is dealt with `undo_allowed` (`Effect::Start`, `Storage::start_game`,
  `SavedGame`, `BoardScreen::begin`); the engine keeps the history for replay and restore, the board
  refuses `Action::Undo` and `Action::Redo` (status `board.status.undo_off`, an illegal-move cue), leaves
  them out of the `Do:` row and the guide panel and adds a "No undo" badge to the game information.
  `Settings::undo_allowed` (default `true`, an older value without the field loads as `true`) is the last
  choice; the game selection (also behind "New game from seed" and `--seed`) shows `[ Undo: on|off ]`,
  switched by `Action::Undo` (`u`) or a click, through `Effect::Apply`. A new game from the board keeps
  the mode of the running game; "Play this seed again" in the statistics replays the row's own mode.
  *Game menu (P4-09):* the game selection lists the variants under family headings in the fixed order of
  `screens/game_select/catalog.rs` (`Family::ALL`, `Family::of`), the favourites group first; a favourite appears only
  there. A star (`Glyphs::star`, `*` in ASCII) marks the popular games (`is_popular`). `Settings::favourites` (a list of
  variants, an older value without the field loads as empty) is toggled by `Action::Finish` (`f`) or a click on the
  marks, through `Effect::Apply`. `/` or a click on the field opens a search that filters by variant name while typing
  (`GameSelect::captures_text`, so letters are text and not shortcuts); Esc closes it first and a second Esc goes back.
  The list scrolls to keep the focused variant and its heading in view. A new game adds a `Family` arm or a variant
  to an existing family; nothing else in the menu changes.
  *Restarts (P4-05):* Restart stays allowed without undo but is counted. `BoardScreen::restart` returns whether it
  counted (only when `undo_allowed` is false) and `App::restart_game` then calls `Storage::record_restart` at once, so
  a quit or a crash cannot lose it; a game with undo never counts one. The header shows `Restarts: N` (`Od nowa: N`)
  after the `No undo` badge once N is above 0 (`BoardScreen::restarts`); a resume reads N back from the history row
  (`App::stored_restarts`). When the header row is too narrow the stock counter goes first and the winnable mark next, so
  the badge and the count stay at 100 columns. The
  saved game keeps the mode: `SavedGame::undo_allowed` has a serde default of `true`, and `SqliteStorage`
  reads it from the `games` row of the saved game's id (a left join), so the database schema is unchanged.
  *Copy the seed:* `Action::CopySeed`, key `c` (help group Game; the statistics screen's own `c` is the same
  key and the same effect), and a click on the seed (underlined, in the header row or in the `Game` compartment), return
  `Effect::CopySeed`; `App::copy_seed` announces the result on the statistics screen and as the last event of
  the board ("Seed N copied", or "Could not copy the seed"; the seed is always visible as text). The statistics summary shows every group (up to 8 games × 2 modes = 16
  rows) and leaves the history list at least 5 rows at the minimum size.
- **Winnable deals (P3-20):** `solver.rs` holds the service the TUI asks: `Proof { Winnable, Unwinnable, Unknown }`,
  `Query { id, variant, seed }`, `Answer { id, proof }` and `trait Solver { submit, cancel, poll }`. `submit` queues a
  query and never blocks, `poll` returns the answers that arrived, oldest first, `cancel` drops the queries not yet
  started (an answer in flight is ignored by the application). The binary wires `oxysol::solver::ThreadSolver`: one
  worker thread that runs `oxysol_core::solve` with `SolveBudget::default_for(variant)`, a generation counter for
  cancelling, and an `Unknown` answer for every query when the thread has stopped, so a search never waits for ever;
  when the thread cannot start `main` uses `NullSolver` (answers `Unknown` at once). `App` polls on every input, and
  the event loop's 200 ms tick advances a search. `Settings::winnable_only` (default `false`, an older value without
  the field loads as `false`) is the option `[ Winnable deals only: on|off ]` of the game selection (focus ring: list,
  undo toggle, this toggle; a click also switches it), remembered through `Effect::Apply`. *Search:* a new game whose
  seed is `None` (`Effect::Start`, from the selection or from the board's "new game") with the option on opens
  `Modal::Search(SearchProgress)` instead of dealing: `app/search.rs` (state) and `app/deals.rs` (flow) queue
  `SEED_CAP` = 40 fresh seeds from `Deps::seeds`, the first `Winnable` answer deals that seed and marks it, `Esc` or
  the Cancel button (`Effect::SearchStep(Cancel)`) drops the search and leaves the previous screen and game as they
  were, and the dialog shows "Seeds tried: N of 40". When all 40 answers are in and none is winnable the dialog offers
  "Play as unknown" (the first seed the solver could not decide; absent when every seed was proven unwinnable),
  "Keep searching" (another 40) and Cancel. In the game selection the variants that the solver rarely proves
  (`solver::slow_to_prove`: Spider with 2 and 4 suits, Pyramid Classic, decision T-29) carry "(the search can take
  long)" while the option is on, and the dialog repeats it. *Seeds the player chose are never replaced:* a typed seed
  (`Effect::Start` with `Some(seed)`: "New game from seed", `--seed`) and a replay from the statistics are dealt as
  they are and queued as one background query; a resumed game is checked again. The answer sets
  `BoardScreen::set_deal(DealCheck)`; `DealCheck { Unchecked, Checking, Known(Proof) }` adds one fact to the game
  information (`Facts`, so the Compact header row and the `Game` compartment both show it): `Winnable: checking...`,
  `Winnable: yes`, `Winnable: unknown`, `Winnable: no`; a random game dealt without the option says nothing. Every
  proof of the running deal is written to its history row with `Storage::record_proof` (a failure is logged, not
  alerted: the mark is advisory). The statistics history has a `Winnable` column (`yes`, `no`, `unknown`, `-` when
  unchecked). Schema v3 adds `games.proof TEXT` (NULL, `winnable`, `unwinnable`, `unknown`) by forward migration.
- **Assist counters (P4-06):** `AssistCounters` (`storage/counters.rs`) counts per game the moves made by hand (`manual`), by
  the quick move key or a double click (`quick`), by the auto-finish (`finish`, counted apart), the quick moves that found no
  target (`failed`) and the longest run of quick moves (`longest_streak`; a manual move ends the run, the finish and a failed
  attempt do not). `BoardScreen` tallies them in `play_from` (`board/assist.rs`); `App::save_board` writes them with the saved
  game through `Storage::record_assist`, a resume reads them back and starts a new run, a restart or a new game starts from
  zero. The share is quick moves over manual plus quick moves, the finish excluded; a game is assisted when the share is above
  `ASSISTED_ABOVE_PERCENT` (80) with at least `ASSISTED_MIN_MOVES` (10) such moves. The counters are information only and never
  block a move. The game has no separate auto-move to the foundations, so only the quick move and the finish are counted.
- **Statistics (P4-04):** `screens/stats.rs` (+ `stats/`: `summary.rs` pure statistics, `format.rs`, `list.rs`
  selection and scrolling, `view.rs`). The main-menu item "Statistics" opens it; `App` reads `Storage::history`
  when the screen opens (a failure is logged and the screen says so) and hands the rows to `StatsScreen::show`.
  A summary per variant and undo mode (games played, won, win rate, best winning time, fewest moves in a win,
  current and longest streak; once a game without undo has finished, a "No restart" column beside the wins counts the wins
without a restart, `-` for the undo rows; only finished games count, an abandoned game ends a streak, wins without a move
  count are skipped for "fewest moves") sits above the history list, newest first: UTC date and time (the
  standard library cannot convert to local time and no dependency was added), game, seed, result, playing
  time, moves, restarts (P4-05: the count for a game without undo, `-` with undo). The status line under the buttons shows the
  assist line of the selected game (P4-06; `stats/assist.rs`): the share of automatic moves, the failed quick moves, the longest
  run of quick moves, the auto-finish moves and the tag "assisted game"; the check keys show instead for a row without counted
  moves. The summary has an "Assisted" column (finished games above the threshold) from 110 columns. Enter replays the selected row (`Effect::Start` with its variant and seed), `c` copies the seed
  (`Effect::CopySeed`); both are buttons too, as is Back; Up/Down/PgUp/PgDn/Home/End and the wheel move the
  selection; the selected row is marked with the focus glyphs. `App::copy_seed` calls `Clipboard::copy` and tells
  the screen, which announces "Seed N copied" or "Could not copy the seed — it is shown here: N" on its status
  line; the seed is always visible as text. Move count rule: a game's `moves` is `Session::moves_made()` when it
  ends — each executed move counts once, an undo takes one back, a redo puts it again, a restart resets it.
- **Storage failures:** a failing load or save is logged once in `App` and shown as an `Alert`; settings
  hold for the session, a game that could not be recorded is still played.
- **Too small:** below 100 × 30 cells (T-11) only a notice with the current and required size is drawn;
  only `q` and Ctrl+C work.
- **Localization:** `i18n::t(key)` and `tf(key, params)`; the locale is a per-thread setting
  (`set_locale`). A key is `<area>.<table path>.<name>`, the area being the file name under
  `locales/<lang>/` (`common.menu.new_game` is `[menu] new_game` in `common.toml`). Fallback is per
  key (T-38): a key the locale lacks reads the `en-US` text, then the key itself; a counted text
  (`tn`) the locale lacks in the needed form reads the `en-US` text of the same form, then the
  `_other` text (the locale's, then `en-US`'s). Parity is checked by `i18n::diff::diff`, which
  compares a locale with `en-US` (plural groups by the forms of the locale's rule) and reports
  missing keys, extra keys and `{placeholder}` mismatches. A complete locale must have none of the
  three; an incomplete locale may miss keys but has no extra key and no placeholder mismatch
  (`Diff::passes`). So a new English key goes into `locales/en-US/` and every complete locale, or a
  locale that cannot get it yet is set back to `complete = false` and shows English for it.
  `cargo xtask-art locale-diff [<lang>] [--ref <git-ref>]` prints the same report for one
  locale (or a one-line summary per locale) and exits non-zero when anything differs. Tests also
  check that every `"<area>.…"` literal in the sources exists.
- **Locales (O-41, T-36):** `i18n/locale.rs` holds the one list of locales: each line names the
  `Locale` variant, its folder (also its serde name), its plural rule and its flag constant; the
  macro builds `Locale::ALL`, `id()`, `plural_rule()`, `flag()` and the embedded sources. Each folder
  has the ten area files and `meta.toml` (`own_name`, the language's name in itself, shown in the
  selector and the Settings Language row; `complete`). Only complete locales are offered:
  `Locale::offered()` feeds the main-menu selector and the Settings Language row; a saved setting
  (on load in `App`) or an environment language (`LC_ALL`, `LC_MESSAGES`, `LANG`, matched on the
  language part by `Locale::from_language_tag`, so any `pt` is `pt-BR`) naming an incomplete locale
  falls back to `en-US` through `Locale::or_source()`. UI tests that check fit or text iterate
  `Locale::offered()`, so a translation is checked as soon as it is marked complete.
- **Plurals:** a counted text is a key group `<key>_one`, `_few`, `_many`, `_other`, read with
  `i18n::tn(key, count, params)`, which picks the form by the locale's CLDR rule for whole numbers
  (`PluralRule`: `OneOther` for en, de, nl, it, es; `ZeroOneOther` for fr, pt-BR; `Polish`;
  `EastSlavic` for uk). A locale has exactly the forms of its rule (`PluralRule::forms`: one/other
  or one/few/many); the parity test enforces it for every locale (a form of the rule may be missing
  only in an incomplete locale, which then reads the English text). A plural group is recognized by en-US having
  both `<key>_one` and `<key>_other`.
- **Adding or translating a locale:** a new language adds one line to the list in
  `i18n/locale.rs` (variant, folder, plural rule, flag), its flag in `i18n/flags.rs`, and
  `locales/<lang>/` as a copy of `en-US` with `meta.toml` saying `complete = false` (plural groups
  given the forms of its rule). It ships hidden. A translation PR then only replaces the ten area
  files of that folder (same keys, same `{placeholders}`, the plural forms of the rule) and sets
  `complete = true` in its `meta.toml`; `cargo xtask check --quick` then runs the key-parity,
  placeholder, fit and layout tests for it as well. No code changes.
- **Board interaction is shared by all games:** after a source is picked, the TUI filters
  `legal_moves()` by `Move::source()`, highlights targets and applies the matching move. A move with
  no target (a king in Pyramid) applies immediately. A Pyramid pair is picked from either card.
- **Board controller** (`screens/board/`): the cursor is a pile plus a depth (cards from the top; only a
  column has more than one). Left/Right/Up/Down go to the nearest pile in that direction by
  `BoardGeometry` (a row counts twice a column, off-axis piles are penalized). Up deepens the focus
  inside a column and leaves it upwards at the deepest card; Down and `-` lift it, and from the top card
  jump to the deepest one; `+` deepens and, at the deepest card, returns to the top one. Only a pile with
  more than one card wraps; a one-card pile lets Down leave it. Empty Pyramid positions are skipped. Enter/Space on a
  pile picks, on a target drops, on the stock draws; a drop on a pile that is not a target is tried
  against the engine and shows its `MoveError` message. Esc cancels a pick, otherwise leaves for the
  menu. Every `MoveError` has a key `board.error.*`; other messages are `board.status.*`; restart,
  new game and quit ask through `Modal::Confirm` (`Effect::Restart`, `Effect::Start`, `Effect::Quit`).
  The screen collects `SoundCue`s and `App` plays them after each input (`BoardScreen::take_cues`);
  `persist()` raises a flag after every session change; `App` reads it (`BoardScreen::take_dirty`)
  at the end of each `apply` and saves (see Autosave below). A win opens `Modal::win()` (help, pause, win and dialogs are drawn by `widgets/overlay/`); a
  position without legal moves shows a status message. *Win dialog (P3-24 part 2):* `App::open_modal` fills `best`
  (`app/records.rs`, `personal_best`): the time is a personal best when it beats the shortest earlier win of the same
  variant and undo mode in `Storage::history` (the game itself and open or lost rows do not count), the moves when they beat
  the fewest; a first win is no best. Three buttons, `New game`, `Same seed` (`Effect::SameSeed`, which deals the same
  seed again through `start_game`) and `Menu`; Tab and the arrows move the focus, a click presses. `Modal::Pause` hides the cards
  (`BoardScreen::set_covered`); the clock stops for every modal and after a win.
- **Auto-finish (P3-19):** `board/finish.rs`. After every change `settle` asks `Session::forced_finish`
  and keeps the moves (`forced`). With the setting `auto_finish` (default on) `BoardScreen::advance(now, auto)`,
  called by `App` after each event while the board shows with no modal, plays them one per `STEP` (400 ms)
  through `play`, so the history and statistics see every move; the status line shows
  `board.status.finishing`. Any key or mouse press calls `interrupt_finish`, which leaves the position to the
  player until it changes; the `f` key (`Action::Finish`) restarts the paced finish. With the setting off, `f`
  plays all moves at once; the action line offers `f: finish game` when a forced win is not played out by itself.
- **Geometry:** `layout::compute(variant, &piles, body) -> BoardGeometry` returns rectangles for piles
  and cards (`BoardGeometry::hit(col, row)` for the mouse); `layout::chrome(area)` splits the screen into
  an optional header row, the `field` (the playing area), its `body` (the area to pass to `compute`) and the two
  rows of the controls strip; `chrome_with(area, guide, frame)` also reserves the full-height guide panel at the
  right and drops the header row when the panel is there (see Board guidance), and insets `body` by `FRAME_INSET`
  (one cell) when `frame` is set, which is `Theme::field_edge()`: only a theme whose felt has no edge of its own
  (High Contrast, `Table` = `Background` = black) draws the one-cell `Border` field frame (`widgets/edge.rs`,
  `┌─┐│└┘`, ASCII `+-|`) on the ring of `field`; in every other theme `body == field` and no frame is drawn.
  `BoardScreen` takes the flag with the settings at every draw, key and mouse event (`follow`), so drawing,
  mouse hit-testing and `commands.rs` share one `chrome()` (`chrome_panels` with the guide, moves and frame flags).
  `Chrome { class, header: Option<Rect>, field, body, guide: Option<Rect>, moves: Option<Rect>, now, possible }`. The same result drives rendering and hit-testing. Two layouts:
  columns (Klondike, Spider, FreeCell) and pyramid. Card size is the largest of 13×11, 11×9, 9×7, 7×5
  that fits the window and is not above `Settings::card_size` (`layout::compute_within`; `compute` has no limit; a column of a king-to-ace run plus a margin must fit); a column first tries a face-up step one larger than natural when it has spare rows (P3-15), then compresses
  face-down steps, then face-up steps, then collapses its face-down cards into one row with a counter.
  Highlights are passed as `widgets::pile::Marks` (focus, selected run, targets, hints).
- **Themes:** `ThemeId { Dark, Light, HighContrast, Monokai, Dracula }`; a `Palette` has a `Swatch { rgb, ansi16 }` for every
  `Token`; `Theme::new(id, depth)` resolves it: true color emits the RGB, 256 colors use `rgb_to_ansi256`,
  16 colors use the named ANSI color. Components ask `Theme` for tokens and never use colors directly.
  `TEXT_PAIRS` lists the text/background token pairs drawn together; each must reach
  `ThemeId::contrast_target()` (4.5, High Contrast 7.0).
- **Readable card faces (P3-14, replaces the P3-13 pixel font):** `widgets/card/` is a module (`mod.rs` drawing,
  labels and the three entry points `draw`, `draw_back_row`, `draw_fanned`, `draw_collapsed`; `frame.rs` borders;
  `mark.rs` `Mark` and `Hint`; `rows.rs` corner labels, marker symbols and the stock badge; `slot.rs` placeholders;
  `paint.rs` clipped painting; `face.rs` centre layout; `art.rs` suit pictograms). The rank is text, never a
  picture: a face-up card of 9x7, 11x9 or 13x11 draws one bold row `K ♣` (the localized corner label, one space,
  the suit symbol; two spaces at 13x11) in the suit color at the top of its interior, with the suit pictogram
  below it (5x2 at 9x7, 5x3 at 11x9, 7x4 at 13x11); 7x5 keeps the single suit symbol. The corner labels stay in
  the top-left and bottom-right corners. Unicode art uses only WGL4 characters, ASCII art only printable ASCII;
  the ASCII suits are lowercase `c d h s`, so the Polish queen of diamonds is `Dd`, not `DD`. No two ranks render
  the same at any size, in both locales and both glyph modes (tested).
  *Face-down cards:* a covered face-down card whose visible part is one row draws the back pattern in that row
  (`draw_back_row`, `┌▒▒▒▒▒┐`, ASCII `+#####+`); when the face-down step of the size is 1 (7x5, 9x7) and at least
  two face-down cards lead a column, `layout::fan` always collapses them to the counted row (`draw_collapsed`).
  The stock badge reads `x24`. *Placeholders (`Face::Slot(Placeholder)`):* `A` (`board.placeholder.foundation`) in
  a foundation, `↑` (ASCII `^`, `Glyphs::recycle`) in an empty stock that can be turned over again, `x`
  (`Glyphs::spent`) when no pass is left, nothing in cells and columns; `pile::draw` takes `recycle`, which
  `screens/board/counters.rs::can_recycle` takes from the engine: an empty stock is recyclable when
  `Game::legal_moves` lists `Move::Stock` (P3-21). A click or Enter on the empty stock sends `Move::Stock`
  through `Session`, which turns the waste back; the sign is never a reshuffle.
  *Marks:* `Mark { focus, selected, target, hint: Option<Hint> }`. The hint marks the source `?→` and the target
  `→?` (ASCII `?>`, `>?`; `Glyphs::hint_source/hint_target`) and has its own border (Unicode `╓─╖ ║ ╙─╜`, ASCII
  `.` and `:`); `Marks::hints` lists the moved cards first and the destination last, which is how `pile::draw`
  tells source from target. A target draws `+` (`Glyphs::target`, both sets) in both top corners, over any border,
  and a centred `+` in an empty slot; focus and selection keep `►`/`●` and their border sets. *Waste fan:* a
  covered waste card shows its left border and corner label only (label width plus one column, 3 or 4), the rows
  under its top row and its bottom border stay blank in that strip (`draw_fanned`); only the playable card shows a
  bottom index.
- **Card rendering (P5-02):** front style and back style are selected independently (`Settings::front_style`,
  `back_style`; defaults `international`, `lattice`). Each is drawn in one of three glyph modes — `Font` (Private
  Use Area glyphs from the oXySol card font), `Unicode`, `Ascii` — described by the style's `style.toml`.
  `crates/tui/build.rs` embeds every style folder and `codepoints.toml`, so a new folder needs no code change;
  `card_style::CardStyles::embedded()` reads them once, resolves the font keys through the registry (a code point
  outside U+E6BC–U+E6FF or an unknown key rejects the style), checks the symbols (Unicode: WGL4; ASCII: one
  lowercase letter per suit) and the color tokens (a front: `card_red` or `card_black`; a back: any token), and
  leaves a style that fails out with a log line. A front gives per suit a name key, an optional font glyph, the
  Unicode symbol, the ASCII letter and a color token; `[ranks]` replaces the localized labels of ace and court
  ranks; `[courts]` names the court emblem glyphs; a front without font keys is valid and draws Unicode in `Font`
  mode. A back gives the `tile` that fills it, a 2 × 2 `mosaic` and the tokens `fg` and `bg`. `Glyphs::styled(mode,
  front, back)` puts the styles into the glyph set (`suits`, `face_down`, `front`, `back`, `font`); an unknown id
  uses the default style and `App` rewrites a saved unknown id to the default. The mosaic is drawn only on a back of
  at least 11 × 9 (Large, Huge); smaller backs use the tile. *Interaction with `card_back` (P3-31):* the back style
  chooses the pattern, `card_back` (`Theme`, `Monokai`, `Dracula`) chooses the palette only: `Theme` resolves the
  tokens of the back style in the current theme, the others in the theme of that name. `Font` is opt-in: it is
  drawn only when `Settings::font_confirmed` is set (the graphics check), `Settings::drawn_glyph_mode()` gives
  `Unicode` otherwise, so Private Use Area code points never show as squares. In `Font` mode a face-up card
  of 9 × 7 or more shows one glyph in the centre: the court emblem of a court card, else the suit.
  *The oXySol font (P5-02):* the Symbols row offers `Font` as a third value. Choosing it without a confirmation
  opens `Modal::GraphicsCheck` (`widgets/overlay/graphics_check.rs`): samples in the card font (the suits with
  their names, the court emblems, the tile and the mosaic of the chosen styles) and a Yes/No question. Yes applies
  `glyph_mode = Font` and `font_confirmed = true`; No (or Esc) keeps Unicode and opens a notice that points to the
  wiki. The Console font row (`Settings::console_font`, `Field::ConsoleFont`) asks `ConsoleFont::use_card_font(bool)
  -> CardFontState { Off, Active, NotRegistered, Refused, Unavailable }`; `src/console/card_font.rs` implements it
  for the classic console: it keeps the current `FontInfo`, sets the face `oXySol Mono` with the same size, reads the
  font back and compares the face (an unregistered face is accepted by the console and silently not used, so only
  the read-back tells), puts the previous font back when it is not in use, and restores it when the option is turned
  off and when `FontZoom` is dropped at exit. `Active` also switches the symbols to `Font` (the font is known to be
  there). The drawn mode is `Settings::drawn_glyph_mode(console_shows_font)`: `Font` only with `font_confirmed` or an
  active console font, else Unicode. `ViewCtx::font: FontFacts { terminal, card_font, cell_px }` carries the state
  (the terminal kind gives `WT_SESSION` through `TerminalKind::WindowsTerminal`); the Windows Terminal hint on the
  font face is in the graphics check and in the description of the Console font row, nowhere else. `cell_px` (the
  console's cell size in pixels, from `ConsoleFont::cell_size`) keeps the Font mosaic off a cell below 10 × 24.
- **Terminal capabilities:** `TerminalCaps { color, unicode, remote }` from detection, overridden by
  settings and arguments. `remote` (an SSH session) switches audio off.
- **Accessibility:** every `Action` has a key binding and a mouse path. The `Now:` row
  of the controls strip is the one-line plain-text description of the focus ("Tableau 3: seven of hearts",
  "Carrying: seven of hearts (from Tableau 3), 2 targets") and carries the last event at its right end;
  `announce` supplies the localized names. The `Unicode` glyph set
  uses only WGL4 symbols (no `▶ ◀`, rounded, heavy, dashed or dotted lines). Sound cues duplicate,
  never replace, visible feedback.
- **Clock:** `GameClock` computes elapsed time from a passed-in `Instant`; pause and modals stop it.

- **Autosave (P4-03):** `app/persist.rs`. A new game calls `Storage::start_game` and logs the deal at
  `info`. After every session change, when leaving the board and on quit, `App` writes
  `SavedGame { id, record, elapsed_ms }` and keeps a copy in memory (the Resume item reads it; its row names the variant, the time and the moves, `main_menu/resume.rs`, and shortens itself when the row is narrow). A won
  game calls `finish_game(Won)` and `clear_game`; dealing over a save calls `finish_game(Abandoned)`
  first; quitting keeps the save. A corrupt or incompatible save at startup, or one that
  `Session::restore` rejects, gives one alert, `clear_game` (and `Abandoned` for a known id) and the
  menu. A failed write is logged once and alerts once per session; the game goes on. `oxysol-tui`
  re-exports `ratatui::crossterm` so that callers can build `Input` without their own dependency.

## Application (`oxysol`)

- **Persistence:** `SqliteStorage` implements `Storage` over `oxysol.db` in the data directory
  (`OXYSOL_HOME` overrides; an `oxysol.portable` file beside the executable selects `data/` beside it, R-21). Tables: `settings` (one row, JSON), `games` (id, variant, seed,
  started_at, finished_at, outcome, elapsed_ms, moves, undo_allowed, proof, restarts) and `saved_game` (one row: game id, `SessionRecord`
  as JSON, elapsed_ms, app_version). Schema version in `PRAGMA user_version` (5: v5 added `games.manual_moves`, `quick_moves`, `finish_moves`, `failed_quick` and `quick_streak`, each `INTEGER NOT NULL DEFAULT 0`, so older rows read 0 and show no assist line; 4: v4 added `games.restarts INTEGER NOT NULL DEFAULT 0`, so older rows read 0; 3: v3 added `games.proof TEXT`, NULL until the solver has answered; 2: v2 added `games.moves` — NULL for older rows — and `games.undo_allowed INTEGER NOT NULL DEFAULT 1`, by forward migration); a version newer than
  the app knows is `StorageError::Incompatible`; a file that is not a readable database is
  `StorageError::Corrupt` and is renamed to `*.corrupt`. A missing row is `Ok(None)`. The TUI decides
  how to react (alert, defaults) and logs there.
- **Command line (T-10, T-28):** `src/cli.rs` parses `--version`, `--help`, `--ascii`, `--colors`, `--seed`,
  `--fullscreen`, `--no-fullscreen`, `--priority <level>` and `--new-window` / `-n` by hand (`Cli::new_window`). `--version` prints two lines from `src/about.rs`: the effective version (`OXYSOL_VERSION`, T-21, R-23: the metadata version with the patch derived from git, `<version>+src` without git history; computed only by the root `build.rs` with `xtask/src/version_scheme.rs`; the binary gives it to the TUI as `Deps::version` for the main menu label, so `oxysol-tui` has no build script and does not rebuild after a commit; it is also the Windows file version) and the
  copyright with the license, built from `CARGO_PKG_AUTHORS` and `CARGO_PKG_LICENSE` through `cli.version_legal`. With `--new-window`, `main` starts only the log and calls
  `new_window::open`, which tries the plans of `new_window::plans(&Context)` in order and returns; the parent exits
  with 0, or with 2 (`AppError::NewWindow`) when no console could be started. `Context` holds the OS
  (`HostOs`), executable, working directory, forwarded arguments (`forwarded` removes the flag), `OXYSOL_HOME` /
  `OXYSOL_LOG` and `$TERMINAL`; the plans are pure and unit-tested, only the spawn is not.
- **Seeds (R-18):** the binary supplies fresh seeds through `Deps::seeds` (from
  `std::collections::hash_map::RandomState` mixed with the clock — no new dependency). `--seed <u64>`
  (`Deps::start_seed`) opens the main menu on the game selection for that seed, the stage that
  "New game from seed" reaches after a typed seed; the chosen variant starts the deal. Every other new
  game takes a fresh seed. The board header shows the seed; the menu offers "New game from seed".
- **Full screen (R-22, T-33):** `Settings::full_screen` is `Auto` (default), `On` or `Off`; `--fullscreen` and
  `--no-fullscreen` (`Cli::full_screen`) override it until the player changes the setting. `App` asks the pure
  `settings::decide(setting, flag, owns_window, kind)` at start and on every change of the setting and calls the
  `ConsoleDisplay` trait of `Deps::display`; `src/console/display.rs` implements it (`ConsoleWindow`) with
  `SetConsoleDisplayMode`, falling back to a posted Alt+Enter checked by `GetConsoleDisplayMode`, and leaves full screen
  when dropped. Terminals the game cannot switch get one hint in the `Now:` slot (`board.status.full_screen_*`).
- **Console font (P3-16, T-35):** `src/console/font.rs`. `trait Console { fn font(&self) -> io::Result<FontInfo>; fn set_font(&mut self,
  font: &FontInfo) -> io::Result<()> }` is the only code that talks to the system; `ClassicConsole` implements it with
  `GetCurrentConsoleFontEx` / `SetCurrentConsoleFontEx` (kernel32, `extern "system"`, one justified `unsafe` each, no `windows-sys`) and
  refuses with `Unsupported` off Windows. `FontZoom` (`ConsoleFont`) reads the start font once, then each step reads the font again and
  asks the pure `target_height(start, current, FontStep)` (±2 px, 8 to 72, Reset to the start height) and sets the same font with only
  the size changed (the face, family, weight and index are kept; a step passes width 0, Reset restores the start width). A refusal is
  logged at `info` and the step reports no change. `main` builds it only for `TerminalKind::ClassicConsole` and when the font can be read.
- **Process priority (P4-13):** `Settings::process_priority` (`ProcessPriority`: `Highest`, `High`, `Normal` default, `Low`,
  `Lowest`; never real time) is a row of the Interface group, so a change is previewed and confirmed like the other
  Interface rows. `--priority <level>` (`Cli::priority`, `DisplayDeps::priority.flag`) wins until the player changes the
  setting. `App` asks the `PriorityControl` trait (`DisplayDeps::priority`) at start and on every change of the level, once
  per level; the answer is a `PriorityOutcome` (`Applied`, `NeedsPrivileges`, `Failed`) that the row shows beside the value
  (`FontFacts::priority`), never an error log per frame. `src/priority.rs` holds the pure `native_value(Flavour, level)` and
  `ProcessControl`; `trait Native` (`set_process`, `lower_current_thread`) is the only code that talks to the system, in
  `src/priority/system.rs` (`SystemPriority`, hand-declared FFI, one justified `unsafe` each, no new dependency). Windows:
  `SetPriorityClass` with `HIGH`, `ABOVE_NORMAL`, `NORMAL`, `BELOW_NORMAL`, `IDLE`. Linux and macOS: `setpriority` with nice
  -10, -5, 0, 5, 10; a negative nice, and any raise back after a lower one, needs privileges (`EPERM` / `EACCES` is
  `NeedsPrivileges` and the priority stays). On Linux the nice value is per thread: the UI thread is set and the sound thread
  started before keeps 0. The solver worker (`ThreadSolver::spawn_with`) calls `Native::lower_current_thread` first:
  `SetThreadPriority(THREAD_PRIORITY_BELOW_NORMAL)` on Windows, nice 15 (`SOLVER_NICE`, above every level) on Linux, nothing
  on macOS (reported at `info`). Tests use `priority::fake::FakeNative`.
- **Clipboard (T-27):** `src/clipboard.rs` implements `Clipboard` with `SystemClipboard`: the native clipboard
  through `arboard` (kept alive between copies), else, or when the session is remote, the OSC 52 escape
  sequence written to standard output. A terminal does not answer OSC 52, so that path reports "sent".
- **Audio:** `src/audio/` implements `Audio`: MIDI sequencing and a built-in synthesizer as pure
  code (`render` is testable without a device), a pure mixer of cues and music, plus `cpal` output
  (`output.rs`, feature `audio`). Sound styles are embedded from `assets/audio/` at build time.
  `bank::style_list()` gives the binary's `Deps::sound_styles`: every embedded style in folder order,
  named by the `name` of its `style.toml` (the id when the name is missing). The settings offer that
  list, and `App` maps a saved style that is not in it back to `default` (`fit_sound_style`).
  The reason sound is unavailable is decided once, in this order: remote session (`RemoteSession`),
  feature off (`NotBuilt`), no working device (`NoDevice`), selected style does not load
  (`StyleFailed`), otherwise `Ready`. A `NullAudio` carries the reason. The settings show the reason
  text from `settings.audio.*`. Changing the style builds the new `Mixer` before the output lock is
  taken and swaps it in under the lock (`Output::replace_mixer`). Linux release builds are made with
  `--no-default-features` and report `NotBuilt` (T-18).

## Errors and logging

| Kind | Example | Handling |
|---|---|---|
| Gameplay result | illegal move | `MoveError` → status-bar message and cue, no log |
| Recoverable | corrupt save, read-only directory, no audio device | alert or silent fallback, logged once at the boundary, game continues |
| Operational | terminal refuses raw mode | `TuiError` → `main` logs, message on stderr, exit code 1 |
| Critical / panic | broken invariant | panic hook: restore terminal, write diagnostics, message |

Logs go only to `oxysol.log`; nothing is written to stdout or stderr while the TUI runs.

## Card font build (`cargo xtask fonts`)

`cargo xtask fonts` writes `assets/fonts/oXySolMono-Regular.ttf` (family `oXySol Mono`, D-06, T-16): the base
`assets/fonts/base/CascadiaMono.ttf` (Cascadia Mono 2407.24, OFL 1.1, see `assets/fonts/NOTICE.md`) plus every
card glyph SVG under `assets/cards/{fronts,backs}/<style>/glyphs/`. The registry `assets/cards/codepoints.toml`
maps each key `<fronts|backs>/<style>/<name>` to one code point in U+E6BC–U+E6FF; Rust checks it (unique, in
range, every SVG has an entry, an entry without an SVG is skipped with a note) and runs `xtask/fonts/build_font.py`
with Python 3.11 or newer and the versions pinned in `xtask/fonts/requirements.txt` (`fontTools`, `skia-pathops`).
Python is used by this command only, never by `cargo build`, `cargo test` or `cargo xtask check`. The same command
writes the contact sheet `target/fonts/preview.html`. Two runs give identical bytes.

## Quality gates

| Command | Steps | Where |
|---|---|---|
| `cargo xtask check --quick` | fmt check, Clippy `-D warnings`, `xtask lint` (including conflict markers in every tracked file and decision-ID uniqueness), every test target in one `cargo test --workspace --tests` call | locally before a PR |
| `cargo xtask check --pr <base>` | fmt, Clippy, lint, the `architecture` and `e2e` suites, unit and integration tests of the packages that `affected --base <base>` selects; when only documentation changed (see `affected`), it runs `cargo xtask docs` instead | `pr.yml` (Linux, Windows) |
| `cargo xtask docs` | conflict-marker and decision-ID lints, then a relative-link check (`docs/**/*.md`, `README.md`, `AGENTS.md`; every relative link and image target exists; anchors are not checked) | docs-only PRs, local |
| `cargo xtask affected --base <ref>` | the affected packages, `*` for all, `docs` when only `*.md`, `docs/**` (not `docs/site/**`), `LICENSE` or `.github/pull_request_template.md` changed | `pr.yml`, `check --pr` |
| `cargo xtask check` | everything: all test targets and `sync-metadata --check` | `main.yml` and `release.yml` on three systems |
| `cargo xtask report` | prints `target/check-result.txt` (written by every `check`) as Markdown for the PR body | local |
| `cargo xtask-art locale-diff [<lang>] [--ref <git-ref>]` | missing, extra and placeholder-mismatched keys of a locale against `en-US` (plural-aware) | local, translations |

The commands that need the application crates or the SVG renderer (`icon`, `locale-diff`, `screenshots`) sit behind
the `art` feature of `xtask` so every other command builds without `oxysol-tui`, `oxysol-core`, `ratatui` and `resvg`
(O-67 F4). Run them with `cargo xtask-art <command>` (alias of `cargo run -p xtask --features art --`); without the
feature they fail with that hint. `cargo xtask check` builds and tests `xtask` with the feature, so the icon check
(a unit test) still runs in every gate. `fonts` needs only Python and stays light.

Definitions used by `xtask lint`:

- **LOC:** lines that are neither blank nor comment-only.
- **Nesting depth:** the number of nested `if`, `match`, `for`, `while`, `loop`, block (`{}`,
  `unsafe`, `async`) and block-bodied closure constructs inside a function body. The function body
  is level 0. `else if` stays at the level of its `if`. A `match` arm adds no level.
