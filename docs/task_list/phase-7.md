# Phase 7 (0.7) — Hungarian cards and Scorpion

Goal: the third new card style after phase 4 (Hungarian suits) and the next new game (Scorpion); alongside, the solver,
hint and interface cards moved from phase 6. Version 0.7.x.

Exit criteria:

- P7-04 and P7-05 are DONE, each with its rules confirmed by Opus and its names checked against a cited source;
- the screenshot renderer and batch runner P7-06 to P7-09 are DONE (decision T-43).

The cards moved from phase 6 (see "Moved from phase 6") are not exit criteria.

Order of work: the card style, then the game.

### P7-04 — Card style: Hungarian suits (Unicode)

- **Owner:** Small
- **Depends on:** P6-04
- **Files:** `assets/cards/fronts/hungarian/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The fourth new card style: the Hungarian suits Makk, Tök, Zöld and Piros, drawn with Unicode symbols. The suit names
    and the court labels (Alsó, Felső, Király) are taken from a cited source named in the PR; the card does not rely on
    memory.
  - The same rules as P4-08 (symbol coverage, ASCII letters, colour tokens, no font glyphs needed).
- **Acceptance:** the same tests as P4-08 for this style; the cited source for every name is in the PR description; both
  locales carry the style name.

### P7-05 — Game: Scorpion

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a public-domain rules source the owner names)
- **Depends on:** P6-05
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/scorpion/**`, the variant list of `oxysol-core`
  where the games are matched, `crates/tui/src/screens/main_menu/**` (the entry only), `locales/*/rules.toml`,
  `locales/*/menu.toml`, `docs/wiki/rules/scorpion.md` (new), `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:** Scorpion as a module of `oxysol-core` on the game contract, the rules as in the confirmed rules text, the
  entry in the game menu, the rules tab and the rules page. A solver is out of this card; the winnable option stays off
  for Scorpion until a solver card exists.
- **Acceptance:** engine tests on scripted deals (legal and illegal moves, the face-down cards, the foundations); the
  same seed gives the same deal; the variant is stored and read back; the menu entry and the rules tab in both locales.

### P7-10 — FreeCell family: Baker's Game and a choice of 1 to 4 cells

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a cited public-domain rules source)
- **Priority:** P2. Owner request 2026-11-26 (the FreeCell family has a single game).
- **Depends on:** P6-05 (the pattern of a new variant), P6-12
- **Files:** `crates/core/src/games/freecell/**` (the rules parameter), `crates/core/src/solver/freecell/**`, the
  variant list of `oxysol-core`, `crates/tui/src/screens/game_select/catalog.rs`, `locales/*/{menu,rules}.toml`,
  `docs/wiki/rules/freecell.md`, `docs/tech/architecture.md` (the games list), tests
- **Scope:**
  - **Baker's Game:** FreeCell with columns built down **in the same suit** (traditional, the ancestor of FreeCell); one
    rules parameter of the FreeCell engine and solver, not a copy.
  - **Cells:** a variant option for FreeCell and Baker's Game with 1, 2, 3 or 4 free cells (4 by default); the maximum
    run that may be moved follows the number of free cells and empty columns as today.
- **Options (phase 6 review 2026-11-26):** the cell count is a field of `GameOptions` (T-53) with its Settings row, the
  board header and the history column, like the P6-06 options; no separate per-variant mechanism.
  - Rules tab, menu entry under the FreeCell family, curated seeds (P5-06) and the solver table for seeds 0-49 per
    variant and cell count in the PR.
- **Acceptance:** engine tests for same-suit building and for each cell count (including the run limit); the same seed
  gives the same deal; the solver verdicts for seeds 0-49 recorded in T-29; menu and rules in every offered locale.

### P7-11 — FreeCell family: Eight Off and Seahaven Towers

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a cited public-domain rules source)
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** P7-10
- **Files:** as P7-10, plus `crates/tui/src/layout/columns.rs` (eight cells or ten columns), tests
- **Scope:**
  - **Eight Off:** eight columns of six cards, eight cells of which four start filled, columns built down in suit, only a
    King on an empty column.
  - **Seahaven Towers:** ten columns of five cards, four cells of which two start filled, columns built down in suit,
    only a King on an empty column.
  - Both on the FreeCell engine with its rules parameters (cells, columns, the starting cells, the empty-column rule);
    layout of the extra cells or columns at 100 × 30 and wider.
- **Acceptance:** engine tests for each rule above; deals by seed; layout without overflow at 100 × 30, 130 × 40 and
  200 × 60; solver table for seeds 0-49; menu and rules in every offered locale.

### P7-12 — TriPeaks family: no K–A wrap, and Four Peaks

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a cited public-domain rules source)
- **Priority:** P2. Owner request 2026-11-26 (the TriPeaks family has a single game).
- **Depends on:** P6-05
- **Files:** `crates/core/src/games/tripeaks/**`, `crates/core/src/solver/tripeaks*`, the variant list of `oxysol-core`,
  `crates/tui/src/layout/tripeaks.rs`, `crates/tui/src/screens/game_select/catalog.rs`, `locales/*/{menu,rules}.toml`,
  `docs/wiki/rules/tripeaks.md`, tests
- **Scope:**
  - **No wrap:** a TriPeaks option where a King and an Ace are not adjacent (harder); the default stays as today.
  - **Four Peaks:** four peaks dealt in the same pattern, with the stock and waste of TriPeaks; the layout fits at
    100 × 30 with the card size falling back as today.
  - One engine with a rules parameter (wrap, peak count), not a copy.
- **Options (phase 6 review 2026-11-26):** the no-wrap rule is a field of `GameOptions` (T-53) with its Settings row, the
  board header and the history column, like the P6-06 options; no separate per-variant mechanism.
- **Acceptance:** engine tests for the wrap rule and the four-peak deal; deals by seed; the solver table for seeds 0-49
  per variant; layout without overflow at 100 × 30, 130 × 40 and 200 × 60; menu and rules in every offered locale.

### P7-13 — Solver for the game options of P6-06

- **Owner:** Medium
- **Priority:** P2. Follow-up of P6-06 (owner rule 2026-11-25: new solver features go to the next phase).
- **Depends on:** P6-06, P6-14
- **Files:** `crates/core/src/solver/{klondike,spider,golf}/**`, `crates/core/src/solver/mod.rs` (the entry with
  options), `crates/tui/src/app/deals.rs`, `crates/tui/src/screens/game_select/**` (the "winnable search is off" note
  goes away for options the solver covers), `crates/core/tests/solver_perf.rs`, `docs/tech/decisions.md` (T-29), tests
- **Scope:** the solver models every option of `GameOptions` (Klondike passes through the stock and any card on an
  empty column, Spider deal with an empty column, Golf King-to-Ace wrap and a Queen on a King), so the winnable-deals
  search, the winnable check and the solver hints work for non-default options too. Curated seeds stay for the default
  options only.
- **Acceptance:** model tests against the engine move by move for every option; the solver table for seeds 0-49 per
  option recorded in T-29; the game choice offers the winnable search for every option the solver covers.

## Moved from phase 6 (review 2026-11-26)

These cards were planned for phase 6 and were not its exit criteria; they keep their IDs. They are not exit criteria of
phase 7 either and run alongside the cards above, in this order. Each step of the solver and hint order makes the next
one cheaper to measure and needs no rework later.

1. Interface, no solver: O-116 → O-117; O-120 → O-121 (O-121 is a DRAFT with the owner's open questions).
2. Solver speed and quality: P6-14 → P6-16 → P7-13 (P7-13 needs P6-14; its Golf part changes the rank-group dead test
   of P6-16 for the wrap option, and its Klondike part drops the talon key as a set when the passes are limited, as the
   P6-14 card states). P6-13 and P6-17 at any point, P6-15 after P6-14.
3. In-play solver and hints: P6-11 → O-115 → O-119; P6-10 last.

### O-116 — The bottom bar: `a`, `e` and `s` by preference, only the ones that are available

- **Owner:** Medium
- **Priority:** P2. Owner report 2026-11-25 (follow-up of O-110); scope clarified by the owner 2026-11-26.
- **Depends on:** O-110
- **Files:** `crates/tui/src/screens/board/{moves_panel,guidance,guidance_parts,priority}.rs`, `locales/*/board.toml`
  (only if a label is missing), tests
- **Problem:** the `Do:` row has little room, and the quick move (`a`), the waste quick move (`e`, O-110) and the stock
  (`s`) are three keys for the same kind of thing: "play the obvious move" in different situations. Today they compete
  for the room by a fixed drop order (`QUICK = 9`, `STOCK = 10`, `WASTE_QUICK = 11`), so `e` is the first to go, and the
  row can spend its room on a key that does nothing at the moment.
- **Fix:**
  - The `Do:` row does not reserve room for all three. It offers **only the ones that are available now**, in the order
    of preference **`a` → `e` → `s`**, as many as fit after the other parts. A key that cannot do anything (no movable
    card under the cursor for `a`, no waste card with a target for `e`, a stock that cannot be dealt or turned for `s`)
    is not shown at all and leaves its room to the next one.
  - Two keys that would make the same move are shown once, by the first in the preference: when the cursor is on the
    waste, `a` already plays the waste card, so `e` is not shown.
  - The drop order follows the preference: `QUICK`, then `WASTE_QUICK`, then `STOCK` (`s` goes first when the row is
    narrow, then `e`, then `a`). The other numbers in `priority.rs` keep their relative order.
  - Quick move off (O-114) hides `a` and `e`; `s` then stands alone.
  - The possible moves panel and window (`v`) list the waste quick move with its key `e` when the waste card can move,
    and show it unavailable with the existing reasons otherwise (no waste card, no target, quick move off).
- **Acceptance:** tests for Klondike draw 1 and draw 3, Pyramid and Australian Patience: only available keys are shown;
  with all three available the row at 130 × 40 shows `a`, `e`, `s` in that order; at 100 × 30 and narrower `s` drops
  before `e` and `e` before `a`; with `a` not available, `e` and `s` take its room; with the cursor on the waste `e` is
  not shown; a stock that cannot be dealt or turned hides `s`; quick move off hides `a` and `e`; the moves panel and
  window list `e`; every offered locale fits.

### O-117 — A setting for `e`: off, semi-automatic (to the focused pile) or automatic

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-25.
- **Depends on:** O-116
- **Files:** `crates/tui/src/settings.rs`, `crates/tui/src/settings/**` (the aids), `crates/tui/src/screens/settings/**`
  (Gameplay tab), `crates/tui/src/widgets/overlay/quick_options.rs`, `crates/tui/src/screens/board/{moves,commands,choices}.rs`
  and the guidance/moves panel files touched by O-116, `locales/*/{settings,board}.toml`, `docs/tech/architecture.md`,
  tests
- **Problem:** `e` always sends the waste card to the best target, like `a`. The owner also wants a way to take the waste
  card and play it exactly where the player is pointing.
- **Fix:** a setting **Waste key (`e`)** with three values, in the Gameplay tab and in Quick options (current game, with
  "Save as default"):
  - **Off**: `e` does nothing (refused with a status line, like quick move off), and its hints disappear.
  - **Semi-automatic**: `e` takes the top card of the waste and drops it on the pile under the cursor (the focused
    pile) when that move is legal; when it is not, nothing moves and the status line says why (no waste card, or the
    focused pile does not accept it). The cursor does not move.
  - **Automatic** (default, today's behaviour): `e` sends the waste card to its best target, like `a`.
  - The labels in the `Do:` row, the guide, the moves panel/window and the help say what `e` does in the chosen mode
    (for example `e: waste to here` in semi-automatic). The general **Quick move** setting off (O-114) still turns `e`
    off whatever this setting says.
- **Acceptance:** tests for each mode in Klondike draw 1 and draw 3, Pyramid and Australian Patience: Off refuses;
  Semi-automatic plays onto a legal focused pile (column and foundation) and refuses an illegal one with the right
  reason, without moving the cursor; Automatic equals today's result; the labels change with the mode; an old settings
  file loads as Automatic; every offered locale fits at 100 × 30.

### O-120 — Settings: clear game history and statistics (soft delete)

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** —
- **Files:** `src/storage/{schema,sqlite,history}.rs`, the storage trait in `crates/tui` it implements,
  `crates/tui/src/screens/settings/**` (a row and its confirmation), `crates/tui/src/screens/stats.rs` (it reads only
  rows that are not deleted), `locales/*/settings.toml`, `tests/integration/migration.rs`, tests
- **Problem:** the player cannot clear the memory of played games and the statistics.
- **Fix:**
  - A **soft delete**, owner rule: no data migration. The `games` table gets one column
    `is_deleted INTEGER NOT NULL DEFAULT 0`, added by a single `ALTER TABLE ... ADD COLUMN` appended to `MIGRATIONS`
    (that list is the only way this schema adds a column). No row is rewritten, copied or removed, and nothing else in
    the schema changes.
  - **Clear history and statistics** sets `is_deleted = 1` on every game row in one statement. Every read of history,
    statistics, the played-seed marks and the stored proofs ignores deleted rows; new games are written as before.
  - The saved game in progress is not touched (Resume still works).
  - In Settings (Gameplay tab) a row **Clear history and statistics** opens a confirmation modal that says what is
    cleared and that the game cannot undo it; keyboard and mouse.
- **Acceptance:** tests: an old database opens and gets the column with every row not deleted; clearing marks every
  row; statistics, history, played-seed marks and proofs are empty afterwards and fill again with new games; the saved
  game survives; the confirmation can be cancelled; every offered locale fits at 100 × 30.

### O-121 — Achievements: a menu entry and the latest one in the top right corner

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** O-120 (deleted games do not count)
- **Decision:** T-50 (the implementer completes the row with the trait, the storage and the list)
- **Files:** `crates/core/src/achievements/**` (new: the rules, pure, no clock or files), `src/storage/{schema,sqlite}.rs`
  (a new table), `crates/tui/src/screens/achievements.rs` (new), `crates/tui/src/screens/{mod,main_menu}.rs` (the menu
  entry), `crates/tui/src/screens/board/header.rs` and `crates/tui/src/screens/main_menu.rs` (the latest one in the top
  right corner), `locales/*/achievements.toml` (new), `docs/tech/architecture.md`, `docs/tech/decisions.md` (T-50), tests
- **Problem:** the game has no achievements.
- **Fix:**
  - **Design (SOLID):** one small trait per rule, for example
    `trait Achievement { fn id(&self) -> AchievementId; fn earned(&self, event: &GameEvent, history: &HistoryView) -> bool; }`,
    each achievement in its own type, and a registry that is a list of them. Adding an achievement means adding one type
    and one entry in the list, without touching the other rules or the checker (single responsibility, open/closed). The
    checker gets the events (game started, game won, game restarted, …) and a read-only view of the history; it knows
    nothing of the terminal or the database (the application passes the storage in behind a trait).
  - When a rule is met for the first time, the achievement is **stored** with its time in a new table
    `achievements (id TEXT PRIMARY KEY, earned_at INTEGER NOT NULL)` and is never earned twice. Clearing the history
    (O-120) does not remove earned achievements.
  - **First list:**
    - **First game:** the first game started.
    - **First win:** the first game won.
    - **Every game of a group won:** one achievement per game family (Klondike, Spider, FreeCell, Pyramid, the Yukon
      family, Golf, …) for winning each of its variants at least once.
    - **Every game started:** every variant started at least once.
    - **Restart:** the first restarted game.
  - **Menu:** an **Achievements** entry in the main menu opens a screen with every achievement: earned ones with the
    date, the others with what is needed. Earned and not earned differ by a sign as well as colour.
  - **Top right corner:** the latest earned achievement (sign and name) stands in the top right corner of the main menu
    and the board header when there is room, and is the first thing dropped when there is not. A new one is also
    announced once on the status line.
- **Open questions for the owner (before dispatch):** the rest of the list (the last item of the request was left
  blank); whether a game won with hints, the winnable check or undo counts as a win; whether clearing the history should
  also clear achievements.
- **Acceptance:** core tests per rule; adding a rule in a test needs no change to the checker; an achievement is stored
  once and survives a restart of the app; the screen lists earned and not earned with signs; the corner shows the latest
  and is dropped first at 100 × 30; every offered locale fits.

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

### O-119 — Quick options always offer New game, Restart and the winnable check

- **Owner:** Medium
- **Priority:** P2. Owner request 2026-11-26.
- **Depends on:** O-115 (the check), O-117 (it edits the same dialog first)
- **Files:** `crates/tui/src/widgets/overlay/quick_options.rs` (237 LOC; split the buttons out if it passes 300),
  `crates/tui/src/widgets/overlay/{dialog,render_tests,focus_tests}.rs`, `crates/tui/src/screens/board/commands.rs`,
  `locales/*/board.toml`, tests
- **Problem:** the `Do:` row drops Restart and New game when it is narrow, and the quick options dialog (`o`) offers
  only the aids, "Save as default" and Close, so there is no one place where these are always reachable.
- **Fix:** the quick options dialog always has, above "Save as default" and Close, a row of three action buttons:
  - **New game (`n`)**: the same confirmation as `n` on the board.
  - **Restart (`r`)**: the same confirmation as `r` on the board.
  - **Winnable? (`w`)**: the check of O-115, both questions at once: **this deal** (can the seed be won) and **from
    here** (can the current position still be won). The answers appear in the dialog with their signs (`✓`, `✗`, `?` and
    the ASCII forms) and the wording of O-115; a variant without a solver says the check is not available. The check
    runs in the background and the dialog stays responsive; closing the dialog does not cancel it, a move does.
  - The buttons join the focus ring (Tab, arrows) and have mouse targets; the keys `n`, `r` and `w` work while the
    dialog is open.
- **Acceptance:** tests: the three buttons are always present, at 100 × 30 too, in every variant; each starts the same
  effect as its board key; the winnable answers appear for the deal and for the position with their signs; a variant
  without a solver shows "not available"; focus order and mouse targets; every offered locale fits.

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

## Developer testing infrastructure: screenshots and the batch regression runner

Owner request 2026-11-21, built on the dev bridge of phase 6 (P6-07 to P6-09). The bridge writes real SVG and PNG
screenshots of the interface. It does this in-process and headless, with no terminal, GUI or system fonts, so a
multimodal agent can look at the screen in WSL2, CI, Linux, Windows or macOS (decision T-43). A JSON-lines scenario
file runs as a batch with numbered frames, a log and a manifest. A pack of baseline scenarios covers every screen.

Two parts of this already exist in `xtask/src/screenshots/`, behind `cargo xtask-art screenshots`, which renders the
player-site images:
- a converter from a Ratatui `Buffer` to a colour grid that resolves truecolour, 256-colour and 16-colour values
  (`grid.rs`);
- a monospace SVG writer that draws box-drawing and block glyphs as shapes, so the grid holds in any font (`svg.rs`,
  `shapes.rs`).

It also has its own scripted driver (`driver.rs`), which does the same job as `Headless` from P6-07. These cards move
the renderer into the dev feature and make `xtask` use it and `Headless`. Nothing is written twice.

### P7-06 — Screenshots: one SVG renderer for the dev bridge and the player site

- **Owner:** Medium
- **Depends on:** P6-07
- **Files:**
  - `crates/tui/src/headless/svg/**` (new, feature `dev`; moved from `xtask/src/screenshots/{grid,shapes,svg}.rs`)
  - `xtask/Cargo.toml` (feature `art` enables `oxysol-tui/dev`)
  - `xtask/src/screenshots/**` (uses the moved renderer, and `Headless` instead of `driver.rs`)
  - `docs/site/src/img/**` (only if the regenerated images change)
  - `docs/tech/architecture.md`
  - tests
- **Scope:**
  - `svg::render(buffer, options) -> String` is a pure function, with no files and no clock:
    - the exact `cols × rows` geometry and a fixed cell size;
    - theme background and colours resolved from truecolour, 256 and 16 colours;
    - `bold`, `dim`, `underline` and `reversed`;
    - box-drawing and block elements drawn as shapes, and card suit glyphs and ASCII mode as text.
  - `options` holds the font family, defaulting to `"oXySol Mono"` with a monospace fallback stack, the cell size, and
    an optional title.
  - The output is byte-identical for the same buffer and options: elements and attributes come in a fixed order, and
    numbers are formatted the same way every time.
  - The player-site animations stay in `xtask` and call the shared still renderer frame by frame.
- **Acceptance:**
  - `cargo xtask-art screenshots` regenerates the site images. They are byte-identical, or every difference is shown
    in the PR and explained.
  - Tests:
    - the SVG of a known buffer has `cols × rows` cells of the stated size;
    - bold, dim, underline and reversed produce the expected attributes;
    - an indexed and a 16-colour cell resolve to the expected RGB;
    - two renders of the same buffer are byte-identical.
  - `xtask/src/screenshots/driver.rs` is gone.

### P7-07 — Screenshots: deterministic PNG and image paths in the bridge `snapshot`

- **Owner:** Medium
- **Depends on:** P7-06, P6-08
- **Files:**
  - `Cargo.toml` (feature `dev-bridge` adds `resvg`, which is already a workspace dependency, with its `text` feature)
  - `src/dev_bridge/**`
  - `deny.toml` (only if a new transitive licence must be allowed; say which)
  - `docs/tech/architecture.md`
  - `tests/integration/dev_bridge.rs`
  - tests
- **Scope:**
  - The SVG of P7-06 is rasterized to PNG with `resvg`. The font database holds only the embedded `oXySol Mono`
    (`assets/fonts/oXySolMono-Regular.ttf`, `include_bytes!`) and never system fonts, so the same frame gives the same
    pixels on every OS.
  - The image size is the fixed cell size times the grid size, at an optional integer scale.
  - `snapshot` accepts `"svg_path"` and/or `"png_path"`, plus an optional `"scale"`. Each file is written atomically:
    to a temporary file in the same folder, then renamed.
  - The response adds an `images` list: the resolved path, the format, and the width and height in pixels.
  - A path that cannot be written is a command error (`ok: false`), not a crash.
  - Optional per-step capture: a `"capture": {"svg_path": ..., "png_path": ...}` field on any command writes the frame
    after that step.
- **Acceptance:**
  - Tests:
    - the PNG starts with the PNG signature and has the stated pixel size;
    - the SVG parses and has the stated `viewBox`;
    - the same frame written twice gives identical SVG bytes and identical PNG dimensions;
    - a failed write gives `ok: false`.
  - The PNG of a Klondike deal is attached to the PR. The owner checks by eye that the layout and colours match the
    terminal.
  - `cargo deny` passes.

### P7-08 — Batch runner: `--dev-script` and `--dev-out`

- **Owner:** Medium
- **Depends on:** P7-07, P6-09
- **Files:** `src/cli.rs`, `src/dev_bridge/**`, `docs/tech/architecture.md`, `tests/integration/dev_bridge.rs`, tests
- **Scope:**
  - `oxysol --dev-script <scenario.jsonl> --dev-out <dir>` (dev builds only) runs the file through the same bridge
    engine as `--dev-bridge`, with no `stdin` and no terminal.
  - Every `snapshot` saves numbered frames, `<dir>/frames/0001-<name>.svg` and `.png`. The name comes from an optional
    `"name"` field, else the `frame_id`.
  - The full structured log (P6-09) goes to `<dir>/logs.jsonl`.
  - `<dir>/manifest.json` holds:
    - the scenario path, seed, viewport sizes, frame list (file, `frame_id`, size) and errors;
    - the elapsed virtual time;
    - the build version.
  - A scenario with a failing command still writes the manifest, records the error, and exits with a non-zero code.
  - Output is byte-identical across runs, except the PNG bytes, which are only required to keep the same dimensions.
- **Acceptance:**
  - Integration tests run a short scenario and find:
    - the frames, with valid SVG and PNG headers and non-zero sizes;
    - `logs.jsonl` and `manifest.json`, with the expected fields.
  - A second run gives identical SVG files and an identical manifest.
  - A failing scenario gives a manifest with the error and a non-zero exit.

### P7-09 — Baseline scenario pack and the viewport runner

- **Owner:** Medium
- **Depends on:** P7-08
- **Files:** `dev/scenarios/*.jsonl` (new), `dev/scenarios/README.md` (new), `xtask/src/scenarios.rs` (new, `cargo xtask
  scenarios`), `xtask/src/main.rs`, `docs/tech/workflow.md` (the command), `.gitignore` (the output folder), tests
- **Scope:**
  - Scenarios, all with fixed seeds:
    - main menu, plain and with the language list open;
    - Help centred, docked right and docked left, on `Keys`, `Rules` and `Command line`;
    - Settings, Statistics, Choose a game;
    - a game of Klondike, Spider, FreeCell, Pyramid and Yukon, each with the side panels open and with `Quick options`
      open.
  - `cargo xtask scenarios [--viewport <cols>x<rows>]... [--out <dir>]` runs the pack through `--dev-script`, using
    the dev build. It is an `xtask` command, not a shell script (T-08). The default viewports are:
    - `80×24`, `120×40` and `160×50` from the request;
    - `100×30`, the real minimum (T-11).

    At `80×24` every scenario shows the too-small notice. That is the expected baseline there.
  - The output goes to `target/scenarios/<viewport>/<scenario>/`, which is not committed.
- **Acceptance:**
  - `cargo xtask scenarios` runs the whole pack at the four viewports without errors.
  - A test runs one scenario at one viewport and checks its manifest.
  - Running the pack twice gives identical SVG files.
  - The README lists every scenario and what it shows.

## Done before the 2026-11 replan

### O-30 — Player site: screenshots, both languages, latest release, build from source

- **Owner:** Sonnet
- **Depends on:** —
- **Files:** `docs/site/**`, `xtask/**` (a screenshot command), `.github/workflows/release.yml` (site job only),
  `docs/tech/workflow.md`, `README.md` (install section only)
- **Scope:** owner request 2026-11-19: the GitHub Pages site (`docs/site`, mdBook) becomes a real landing page:
  several screenshots and a few short animations of the game (generated reproducibly from the TUI, e.g. by an
  `xtask` command that renders scripted scenes to SVG; vector per AGENTS.md), English and Polish with a language
  switch on every page, a link to the newest build (pre-releases included, injected at build time), and for
  Windows, Linux and macOS how to download the latest source with `curl` in a terminal and build it, stating that
  Rust (rustup, the toolchain in `rust-toolchain.toml`; on Windows the MSVC Build Tools) is required.
- **Acceptance:** `mdbook build docs/site` succeeds locally; the screenshots regenerate with one command; every
  page exists in both languages; the build commands verified on Windows (say how) and written for Linux and
  macOS; the release link resolves to the newest pre-release.
- **Done:** #110
- **Scope added (owner, 2026-11-19):** the build instructions use the `release` branch; a Features section (themes, High Contrast for low vision, color never the only signal, languages); the main README rewritten as a GitHub landing page.
- **Deviation (accepted):** `xtask` depends on the workspace crates `oxysol-tui`, `oxysol-core` and `ratatui` for the screenshot command; animations are CSS-animated SVG; the rules pages link to the English wiki; Linux and macOS commands are marked not verified.
