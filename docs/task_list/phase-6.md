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

### O-115 — Is it winnable? A check for the deal and for the current position

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-25.
- **Depends on:** P5-05 (merged); moved to phase 6 on 2026-11-25 (owner: new solver features go to the next phase)
- **Decision:** T-47
- **Files:** `crates/core/src/solver/mod.rs` and the per-game `Position::from_piles` paths it already uses (a new entry
  point that solves the current position), `crates/tui/src/solver.rs`, `crates/tui/src/app/deals.rs` or a new
  `crates/tui/src/app/winnable.rs`, `crates/tui/src/input/table.rs`, `crates/tui/src/widgets/guide/**`,
  `crates/tui/src/screens/board/{guide_view,moves_panel}.rs`, `crates/tui/src/widgets/overlay/{guide_window,moves_window}.rs`,
  `src/` (the worker that answers the new query), `docs/tech/architecture.md` (solver contract),
  `docs/tech/decisions.md` (T-47), `locales/*/{board,help}.toml`, tests
- **Shared files:** `crates/tui/src/input.rs` if the key table needs it, `locales/en-US/*.toml`, `docs/tech/decisions.md`
- **Problem:** the player cannot ask whether the deal is winnable, nor whether the game can still be won from where they
  are now.
- **Fix:**
  - One action **Check: winnable?** that answers two questions at once:
    - **This deal:** the stored proof of the deal (T-30) when there is one, otherwise a fresh check from the start.
    - **From here:** a check of the current position (core gets an entry point that solves from the current piles and
      counters, with the same models, budgets and replay rule as T-29).
  - Each answer is **winnable**, **not winnable** (only after an exhausted search) or **unknown** (budget reached), with a
    sign as well as colour (`✓`, `✗`, `?` and ASCII forms).
  - It runs in the background through the `Solver` trait and never blocks input; a move, undo or new game cancels the
    check of the old position and clears the answer. A variant without a solver says the check is not available.
  - Where: a button in the guide panel and window (`g`) and in the moves panel and window (`v`), and the key `w` on the
    board. The button and the key are listed in the guide, the help keys and the footer hints where they fit.
  - The text says that "winnable" means solvable with every card known (T-29), so it is not a promise for a player who
    cannot see the face-down cards.
- **Open question for the owner (before dispatch):** does using the check mark the game in statistics (like a hint), or
  not at all?
- **Plan (solver review 2026-11-25):** a core entry point `solve_from(variant, seed, played: &[Move], budget)`:
  deal, replay `played`, build the model from `piles()` (plus `stock_status()` for the Pyramid recycles; nothing else is
  missing: Klondike foundation slots and the talon order are in the piles), search, and verify by replaying `played`
  and the line on a fresh deal; `solve` becomes `solve_from(.., &[], ..)`. This avoids cloning a `Box<dyn Game>`.
  In-play queries use their own worker lane, so they never wait behind a winnable-deal batch.
- **Shared core for P6-10 (review 2026-11-25):** split an internal search over a model built from `Piles`
  (`search_from_piles(variant, &piles, stock_status, budget)`, which can also return the best first move found when a
  best-first search runs out of budget) from the public `solve_from(...)`, which adds the replay of `played` and the final
  verification on a fresh deal. P6-10 needs the internal one for synthetic (sampled) worlds that match no real seed.
- **Acceptance:** core tests: the new entry point returns the same verdict as `solve` on a fresh deal, finds a won line
  from a mid-game position and replays it on the engine, and returns unknown on a tiny budget; TUI tests: `w` and both
  buttons start the check, the answer appears with its sign, a move clears it, an answer for an old position is ignored;
  the input never waits for the solver; every complete locale.


### P6-10 — Hint and in-play check knowledge: full, or the player's with a stock memory

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-25.
- **Depends on:** P5-07 (hint profiles, `HintKnowledge`), O-115 (in-play winnable check)
- **Decision:** the orchestrator assigns one at dispatch
- **Files:** `crates/core/src/games/progress.rs`, `crates/core/src/game.rs` (a hint call that takes the knowledge),
  `crates/core/src/solver/**` (only the in-play position check of O-115), `crates/tui/src/settings.rs`,
  `crates/tui/src/screens/settings/**` (Gameplay section), `crates/tui/src/widgets/overlay/quick_options.rs`,
  the board hint call, `locales/*/{settings,board}.toml`, `docs/tech/architecture.md`, `docs/tech/decisions.md`, tests
- **Shared files:** `crates/tui/src/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/en-US/*.toml`
- **Problem:** a hint in play (P5-07) always uses the player's knowledge with a fixed stock memory of 3; the owner wants
  it to be a choice.
- **Fix:**
  - Judging a seed, before or after a game (winnable deals, statistics check, the "this deal" answer of O-115), always
    uses full knowledge. Unchanged.
  - In play (hints and the "from here" answer of O-115) a setting chooses: **full knowledge** (every card, stock order
    included) or **the player's knowledge** with a **stock memory** of 1 to N cards (default 3): the hint may use the
    identity of at most that many stock cards the player has already seen, and nothing else hidden.
  - A stock with fewer cards than the memory, or no stock at all, is handled: the memory is capped by what was seen.
  - The setting is in Settings (Gameplay) and Quick options; an old settings file reads as the player's knowledge, 3.
- **Plan (solver review 2026-11-25):** depends also on P6-11. `Full` (and any position with nothing unknown, such
  as FreeCell) asks `solve_from` with a small budget in the background and offers its first move.
  `Player { stock_memory }`: the session keeps a FIFO of the last N stock cards the player saw (empty without a stock,
  capped by a short stock); the hint samples K completions of the unknown cards with a deterministic PRNG seeded only by
  a hash of the visible state (never the deal seed), solves each, and votes on the first move. Target: FreeCell hints
  win most deals, Spider 2/4 above 0 %.
- **Sampled worlds and stock memory (review 2026-11-25):**
  - Each sampled world is searched with `search_from_piles` of O-115 on the determinised `Piles`; the voted first move is
    checked against `game.legal_moves()` of the real game before it is offered.
  - The session's `StockMemory` records, for each of the last N stock cards seen, its identity **and its position in the
    talon**, so Klondike draw 3 keeps the phase of its triples when the unknown cards are sampled.
  - Undo also rolls back `StockMemory` (a snapshot per move beside the position history), so undoing a stock draw never
    leaves a card "from the future" in the memory; the no-leak test covers undo.
- **Acceptance:** core tests: with the player's knowledge no hint depends on a face-down card or an unseen stock card
  (two deals that differ only in hidden cards give the same hint); the memory of 1, 3 and N with a long, short and
  empty stock; full knowledge may differ. TUI tests: the setting changes the hint; every complete locale. The hints-only
  table of T-48 measured for both knowledge modes.

## Order of the solver and hint cards

Each step makes the next one cheaper to measure and needs no rework later:

1. P6-12 → P6-13: speed without any change of behaviour; `solver/tests/api.rs` stays as it is.
2. P6-14 → P6-15 → P6-16 → P6-17: the solver improvements, with one round of new baseline measurements.
3. P6-11 → O-115: `guidance()` off the redraw path, the in-play worker lane, `search_from_piles` split from `solve_from`.
4. P6-10: solver-backed hints in the background, `Full` / `Player` knowledge with a stock memory of 1..N.

### P6-11 — Hints off the redraw path, and no repeated positions

- **Owner:** Medium
- **Priority:** P2. Solver review 2026-11-25 (item 2).
- **Depends on:** P5-07
- **Files:** `crates/tui/src/screens/board/guidance.rs` (355 LOC, over the warning: split the hint part out),
  `crates/tui/src/screens/board/{mod,commands}.rs`, `crates/core/src/session.rs` (205 LOC),
  `crates/core/src/games/progress.rs` (270 LOC), tests
- **Problem:** `Board::guidance()` runs on every redraw (`view.rs:119`) and calls `legal_moves()` and `hint()`
  (`guidance.rs:246`, `:276`), so a hint must stay near a millisecond and can never use the solver. Following the hints
  repeats positions (seeds 0-199: Klondike 103, FreeCell 113, Pyramid Relaxed 111 runs end in a repeat), mostly turning
  the stock round with nothing to play.
- **Fix:**
  - The footer asks only whether a hint exists, cached per session revision (move count plus undo position); the hint
    itself is computed when `h` is pressed or the revision changes, never per frame.
  - `Session` keeps a short ring of recent position keys (the engine's `piles()` hashed), rolled back by undo;
    `Game::hint` stays pure, and the session-level hint skips a move whose result is in the ring, falling back to the
    next ranked move. A full stock cycle with no play in between counts as a repeat.
- **Interface between `Game` and `Session` (review 2026-11-25):** `Game` stays stateless, but a single
  `Option<Move>` is not enough: if `Session` rejects a looping move the player would get no hint although a second good
  move exists. `Game` therefore offers its ranked candidates (`hint_candidates()`, best first; `hint()` is the first of
  them), and `Session` takes the first candidate whose result is not in its ring of recent positions.
- **Acceptance:** redraws call neither `hint()` nor `legal_moves()` more than once per revision (test with a counting
  fake game); the hints-only benchmark (`crates/core/tests/hint_perf.rs`), driven through the `Session` hint with its filter (not `Game::hint` alone), shows repeats at most 5 of 200 per variant and
  no variant wins fewer deals than in T-48; undo restores the ring.

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

### P6-13 — Winnable deals: a small thread pool, same seed as one thread

- **Owner:** Medium
- **Priority:** P2. Solver review 2026-11-25 (item 4).
- **Depends on:** —
- **Files:** `src/solver.rs` (225), `crates/tui/src/app/deals.rs` (221), `crates/tui/src/solver.rs` (144), tests
- **Problem:** one worker solves up to 40 seeds (`SEED_CAP`) one after another for "winnable deals only".
- **Fix:** `ThreadSolver` runs `min(available_parallelism - 1, 4)` workers (at least one) on `std::thread`, sharing the
  generation counter; answers carry their query id; the TUI accepts the first `Winnable` **in submission order**,
  waiting for earlier queries still running, so the chosen seed is exactly the one-thread result. No change in
  `crates/core`.
- **Memory check (review 2026-11-25):** once P6-15 has made Australian Patience best-first, measure the peak memory
  of four workers each running best-first at 300k positions; record it here and cap the pool if it is too high.
- **Acceptance:** a test with a stub solver that answers out of order picks the same seed as in order; cancel drops
  every worker's pending jobs; time to a proven deal for Spider 4 suits, Yukon and Australian Patience (20 searches
  each) measured before and after.

### P6-14 — Klondike solver: best-first, a canonical draw-1 talon, safe cards stay home

- **Owner:** Medium
- **Priority:** P2. Solver review 2026-11-25 (item 6, Klondike part).
- **Depends on:** P6-12
- **Files:** `crates/core/src/solver/klondike/{mod,position,steps}.rs` (161, 231, 184; add `heuristic.rs` rather than
  grow `position.rs`), `crates/core/src/solver/mod.rs` (strategy choice), `solver/tests/api.rs`,
  `docs/tech/decisions.md` (T-29), tests
- **Fix:**
  - `score()` (work left: cards not home, face-down cards weighted by depth, buried next-foundation cards) and
    `Strategy::BestFirst` for Klondike. An exhausted best-first search is still complete, so `Unwinnable` stays a proof.
  - Draw 1 (unlimited passes): the talon key is the **set** of talon cards, not the stock/waste split, because every
    talon card stays reachable and a play does not reorder the others. Draw 3 keeps the full order.
  - No `Down` step for a card that `is_safe` (nothing can need it as a base).
- **Check while implementing (review 2026-11-25):** confirm that Klondike draw 1 has no hidden limit on passes
  through the stock (today `stock_status().recycles_left` is `None`, so unlimited); the talon key as a set of cards is
  only correct under that condition.
- **Acceptance:** seeds 0-199 draw 1 and draw 3: no verdict flips between `Winnable` and `Unwinnable`; Unknown falls
  (target on seeds 0-49 at 100k: draw 1 below 15 %, draw 3 below 18 %); every `Winnable` replays; pinned node counts
  updated knowingly; best-first memory at 100k nodes reported.

### P6-15 — Yukon family and Pyramid Relaxed: best-first and complete pruning

- **Owner:** Medium
- **Priority:** P2. Solver review 2026-11-25 (item 6, Yukon and Pyramid part).
- **Depends on:** P6-12, P6-14 (same approach)
- **Files:** `crates/core/src/solver/yukon/{mod,position,steps}.rs` (153, 264, 180; new `heuristic.rs`),
  `crates/core/src/solver/pyramid/{mod,steps}.rs` (99, 137), `crates/core/src/solver/mod.rs`, `solver/tests/api.rs`,
  `docs/tech/decisions.md` (T-29, T-41), tests
- **Fix:** `score()` and `BestFirst` where it lowers Unknown; drop the `Down` step in the same-suit-down variants
  (Russian Solitaire, Australian Patience: a card moved back could only take the lower card of its suit, which is
  already home); keep `Down` in Yukon, Alaska and Moosehide, where a group can build onto it.
- **Acceptance:** seeds 0-199: no verdict flips; Unknown on seeds 0-49 lower for Yukon (32 %), Australian Patience
  (34 %) and Pyramid Relaxed (22 %); T-29 and T-41 tables replaced.

### P6-16 — Golf solver: a proof by rank groups, a packed key, a lower budget

- **Owner:** Medium
- **Priority:** P3. Solver review 2026-11-25 (item 7, Golf part).
- **Depends on:** P6-12
- **Files:** `crates/core/src/solver/golf/{mod,position,steps}.rs` (69, 117, 65), `crates/core/src/solver/mod.rs`
  (`DEFAULT_GOLF_NODES`), `docs/tech/decisions.md` (T-45), tests
- **Fix:** a position is dead when the remaining tableau ranks form more contiguous rank groups than chains can still
  start (`groups > stock_len + can_start_now`, where `can_start_now` is 1 when the current waste rank is adjacent to some
  rank still on the table, else 0): a chain plays ranks one apart, so it stays inside one group, and each chain starts
  from the waste or a stock card. Rank groups follow the engine's `games::golf::ranks_adjacent`: in oXySol a king and an
  ace are **not** adjacent (T-45), so groups do not wrap from king to ace. Pack the key (seven heights, stock size, waste rank) into one `u64`.
- **Acceptance:** the dead test checked against an exhaustive engine search on 50 small constructed positions; seeds
  0-999 give the same verdicts as today with fewer nodes; the default budget lowered so seeds 0-49 stay at 0 % Unknown.

### P6-17 — Spider 2 and 4 suits: a beam stratified by deals left

- **Owner:** Medium
- **Priority:** P3. Solver review 2026-11-25 (item 7, Spider part).
- **Depends on:** —
- **Files:** `crates/core/src/solver/spider/multi.rs` (306 LOC, already over the warning: move the beam into
  `spider/beam.rs`), `crates/core/src/solver/spider/multi_moves.rs` (178), `solver/tests/api.rs`,
  `docs/tech/decisions.md` (T-29), tests
- **Fix:** the beam keeps its width per stratum of deals left (`stock_len / 10`), so positions that dealt early do not
  crowd out ones that still hold the stock. Spider 2/4 still never answers `Unwinnable`.
- **Acceptance:** seeds 0-49 at 150k: Spider 4 suits Winnable up from 80 % (target about 90 %), Spider 2 stays 100 %,
  average time not higher than today; pinned node counts updated knowingly.

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
