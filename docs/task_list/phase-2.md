# Phase 2 — Games

Goal: four games whose rules match the descriptions below. Klondike comes first and sets the module
pattern the other three follow.

## Requirements shared by P2-01…P2-04

- The game type implements `Game`; fields are private. Replace the P1-02 stub without changing
  names or `game.rs`.
- `apply` = validation + execution + automatic effects. On `Err` the state is unchanged. A pile that
  does not exist gives `MoveError::NoSuchPile`, never a panic. An unsupported move kind gives
  `MoveError::Unsupported`.
- `legal_moves()` returns exactly the set `apply` accepts — including every empty foundation for an
  ace and every free cell.
- `piles()` in a fixed order: `Stock`, `Waste`, `Foundation(0..)`, `Cell(0..)`, `Tableau(0..)`, `Pyramid`.
- Use `crate::rules`, `crate::deck`, `crate::rng`. Do not copy shared logic; report a missing predicate.
- Files: `mod.rs` (state, deal, `impl Game`), `rules.rs` (validation and execution), `moves.rs`
  (move generation and hint), `tests.rs`. Each at most 300 LOC.
- Unit tests: the deal (counts, face-up cards), every rule positive and negative, automatic effects,
  the win condition (state built by a `#[cfg(test)]` constructor), the hint.
- `cargo test -p oxysol-core`, including `tests/playout.rs`, passes for the game's variants.

### P2-01 — Klondike (Draw 1, Draw 3)

- **Owner:** Sonnet
- **Depends on:** F-01
- **Files:** `crates/core/src/games/klondike/**`
- **Rules:**
  - 52 cards. 7 tableau columns: column *i* holds *i*+1 cards, top one face up. The other 24 form
    the stock. 4 foundations.
  - Foundations: by suit, ascending from ace to king. An ace goes to any empty foundation.
  - Tableau: descending, alternating colors. Any correctly ordered run of face-up cards may move.
    An empty column takes only a king (or a run starting with one).
  - `Move::Stock`: draws 1 or 3 cards to the waste (fewer if fewer remain). With an empty stock and
    a non-empty waste it turns the waste back into the stock in the original order; passes are
    unlimited. Both empty → `MoveError::StockEmpty`.
  - Only the top waste card is playable, also in Draw 3.
  - Allowed: waste → tableau/foundation, tableau → foundation, foundation → tableau (one card).
  - After cards leave a column, a face-down card on top flips automatically.
  - Win: 52 cards on the foundations.
- **Hint order:** to a foundation → a tableau move that flips a face-down card or empties a column →
  waste to tableau → a king to an empty column if it reveals a face-down card → `Stock`. Never
  suggest: foundation → tableau, moving a king from the bottom of a column to an empty column,
  moving part of a run.
- **Done:** #20

### P2-02 — Spider (1, 2, 4 suits)

- **Owner:** Sonnet
- **Depends on:** P2-01
- **Files:** `crates/core/src/games/spider/**`
- **Rules:**
  - 104 cards: 1 suit — 8 × spades; 2 suits — 4 × spades and 4 × hearts; 4 suits — 2 full decks.
  - 10 columns: the first 4 hold 6 cards, the other 6 hold 5 (54 in total), top cards face up.
    Stock of 50. 8 foundations.
  - Building: descending by rank, any suit. Only a descending same-suit run may move as a group.
    An empty column takes any card or run.
  - `Move::Stock`: one face-up card onto each of the 10 columns. Not allowed while any column is
    empty (its own `MoveError` variant) or when the stock is empty.
  - A complete same-suit K→A run on top of a column moves automatically to the first free
    foundation; then the card beneath flips. This also applies to a run completed by a stock deal.
  - No moves from foundations.
  - Win: 8 complete runs on the foundations.
- **Hint order:** a move extending a same-suit run → a move flipping a face-down card → a move
  emptying a column → building on a different suit → `Stock`. Never suggest moving a whole column
  to an empty column.
- **Done:** #24

### P2-03 — FreeCell

- **Owner:** Haiku
- **Depends on:** P2-01
- **Files:** `crates/core/src/games/freecell/**`
- **Rules:**
  - 52 cards, all face up. 8 columns: the first 4 hold 7 cards, the other 4 hold 6. 4 cells, 4 foundations.
  - Foundations: by suit, ascending from the ace. Tableau: descending, alternating colors.
  - A cell holds one card. An empty column takes any card or run.
  - Moving *n* cards between columns needs a valid run and
    *n* ≤ (free cells + 1) × 2^(empty columns); an empty target column does not count as empty.
  - Allowed: column ↔ cell, column → foundation, cell → foundation, column → column. No moves from
    foundations.
  - `Move::Stock` → `MoveError::Unsupported`.
  - Win: 52 cards on the foundations.
- **Hint order:** to a foundation → a column move extending a run → cell to column → column to an
  empty column → column to a cell. Never suggest moving a column's only card to another empty column.
- **Done:** #27

### P2-04 — Pyramid (Relaxed, Classic)

- **Owner:** Haiku
- **Depends on:** P2-01
- **Files:** `crates/core/src/games/pyramid/**`
- **Variants:** one implementation with two fixed rule sets (`PyramidRules`), differing only in the
  number of stock passes and the win condition. Relaxed: 3 passes, win when the pyramid is empty.
  Classic: 1 pass, win when all 52 cards are on `Foundation(0)`. No further options in MVP.
- **Rules:**
  - 52 cards. A pyramid of 28 face-up cards in 7 rows (row *r* holds *r*+1 cards), addressed as
    `PileRef::Pyramid { row, index }`. The other 24 form the stock; its top card is face up.
  - Pyramid card `(r, i)` is available once both `(r+1, i)` and `(r+1, i+1)` are gone; the last row
    is available from the start.
  - Available cards: available pyramid cards, the top of the stock, the top of the waste.
  - Values (a function in this module): ace 1, 2–10 face value, jack 11, queen 12, king 13.
  - `Move::Remove { first, second: Some(_) }`: two different available cards summing to 13 go to
    `Foundation(0)`. `Remove { first, second: None }`: an available king. Stock + waste is a valid pair.
  - `Move::Stock`: the top stock card moves to the waste. With an empty stock and passes left, the
    waste returns to the stock in the original order. After the last pass: `MoveError::StockEmpty`.
  - `Move::Transfer` → `MoveError::Unsupported`.
  - Passes and win condition: as given under Variants.
- **Hint order:** a king → a pair of two pyramid cards → pyramid + stock/waste → stock + waste → `Stock`.
- **Done:** #25

### P2-05 — Gate: game rules and flow tests

- **Owner:** Sonnet
- **Depends on:** P2-01…P2-04
- **Files:** `crates/core/tests/flows.rs`, fixes in `crates/core/src/games/**`
- **Scope:**
  - Review the games against the rules above and `AGENTS.md`: rule fidelity, atomic `apply`, no
    panics, `legal_moves` agreeing with `apply`, no duplicated logic, sensible hints. Each finding
    names the rule and the location; fix small ones directly.
  - `flows.rs`: per variant, a fixed-seed scenario through `Session` — deal, a series of moves,
    undo/redo, `restart`, `SessionRecord` to JSON and back.
  - Golden deals: fixed seed → hard-coded initial layout of every variant.
  - Check wiki pages M-07…M-10 against the implemented rules.
  - Update the Status column in `README.md` and `micro.md`.
- **Acceptance:** review report; `cargo xtask check` passes; `architecture.md` matches the code.
- **Done:** #28

### P2-07 — Forced-win detection and the finishing moves

- **Owner:** Sonnet
- **Depends on:** P2-06
- **Files:** `crates/core/src/games/**`, `crates/core/src/game.rs`, tests
- **Scope:** a `Game` method that reports when the rest of the game is a certain win and returns the
  sequence of moves that finishes it (e.g. Klondike: stock and waste empty and every card face up; FreeCell
  and Spider: a greedy finish that provably reaches the foundations; Pyramid where it applies). Pure,
  deterministic and fast (no search beyond a small bound). Owner request 2026-11-16: the game should know it
  is won and finish laying out the cards by itself (as an option, see P3-19).
- **Acceptance:** tests per game on positions that are and are not forced wins; the returned moves are
  legal and end in `is_won()`; no false positives on golden deals.
- **Done:** #71

### P2-08 — Solver for winnable deals

- **Owner:** Sonnet (Fable may be consulted for the search design)
- **Depends on:** P2-07
- **Files:** `crates/core/src/solver/**` (new), tests, `docs/tech/decisions.md`
- **Scope:** a bounded solver per game that answers "winnable", "unwinnable" or "unknown within the budget"
  for a deal, with a node/time budget passed in (core stays clock-free: the budget is a node count).
  Klondike (draw 1 and 3), FreeCell, Spider (1, 2, 4 suits) and Pyramid; use well-known techniques (move
  ordering, state hashing, safe auto-moves to foundations) and document the limits. Used by P3-20 to offer
  winnable deals only.
- **Acceptance:** known winnable and unwinnable deals per game answered correctly; the budget respected;
  determinism; a benchmark note in the PR (typical time per deal on this machine).
- **Deviation (accepted):** the Spider default budget is 60,000 positions; Spider 2 and 4 suits mostly answer Unknown.
- **Done:** #78

### P2-06 — Stock pass and deal counters in the engine

- **Owner:** not recorded (follow-up)
- **Depends on:** P2-05
- **Files:** `crates/core/src/{game.rs,lib.rs,session/tests.rs}`, `crates/core/src/games/{freecell,klondike,pyramid,spider}/**` (per-game `tests/stock.rs`), `crates/core/tests/flows/{main,stock}.rs`, `docs/tech/architecture.md`
- **Scope:** Public `StockStatus { recycles, recycles_left, deals_left }` in `oxysol_core` and `Game::stock_status()`, which returns `None` for FreeCell. Klondike counts recycles in its game state; Pyramid derives its count from `recycles_left` and the rule set; Spider reports `deals_left`. No TUI, storage or locale change; the save format is unchanged because sessions rebuild games by replay.
- **Acceptance:** Per-game stock tests and `Session` flow tests, including undo, redo, restart and restore; golden deals and pinned record JSON unchanged; `architecture.md` documents `StockStatus` in the same PR (PR #60).
- **Done:** #60

### O-02 — Phase 2 gate follow-ups

- **Owner:** not recorded (follow-up)
- **Depends on:** P2-05
- **Files:** `crates/core/src/game.rs`, `xtask/src/lint/mod.rs`
- **Scope:** Follow-ups 3 and 7 from the P2-05 review (#28): the `PyramidRules` doc comment said Relaxed allows waste pairs, which contradicts R-11 and P2-04 (the rule sets differ only in passes and win condition), so it is corrected; `#![allow(dead_code)]` is removed from `xtask/src/lint/mod.rs` because every rule is implemented.
- **Acceptance:** Clippy is clean without the allow; `cargo xtask check --quick` passed (PR #29).
- **Done:** #29
