# Phase 5 (0.5) — Themes and colour vision, Spanish cards, Golf and the Spider solver

Goal: the table, the card face and the aura are set apart from the theme; the low-vision mode and the colour-vision modes are built on the Interface section of phase 4; the second new card style (Spanish) and the second new game (Golf, with its solver) are added; the Spider solver gets its 2-suit and 4-suit results improved. This is the phase worked on now, during version 0.5.x.

Exit criteria:

- P3-32, P3-33, P3-34, P3-35, P5-03, P5-04 and P5-05 are DONE;
- every new theme option passes the contrast tests, and the four-colour deck works at every card size;
- Golf's solver verdicts replay on the engine for seeds 0 to 49, and the Spider table is in T-29.

Order of work: the Interface options first, then the card style and the games, which do not touch those files.

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

### P5-03 — Card style: Spanish suits (Unicode)

- **Owner:** Haiku
- **Depends on:** P4-08
- **Files:** `assets/cards/fronts/spanish/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The second new card style, Spanish: the suits Oros, Copas, Espadas and Bastos, drawn with Unicode symbols, with
    the court labels Sota, Caballo and Rey.
  - The same rules as P4-08: no font glyphs needed (Font mode draws the Unicode symbol), the symbol coverage recorded in
    the PR, ASCII letters (suggested O, C, E, B), colour tokens, and colour never the only signal.
- **Acceptance:** the same tests as P4-08 for this style; both locales carry the style name; the symbol table with its
  coverage is in the PR description.

### P5-04 — Game: Golf

- **Owner:** Sonnet
- **Depends on:** P4-10a
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/golf/**`, `crates/core/src/solver/golf*` (new, with
  the solver), the variant list of `oxysol-core` where the games are matched, `crates/tui/src/screens/main_menu/**`
  (the entry only), `locales/*/rules.toml`, `locales/*/menu.toml`, `docs/wiki/rules/golf.md` (new),
  `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:**
  - Golf (traditional, public domain): seven columns of five cards dealt face up, a stock and one waste pile. A card of a
    column may be played onto the waste when its rank is one higher or one lower than the waste top; the stock turns one
    card onto the waste. Whether King and Ace are adjacent is stated in the rules tab text.
  - A solver in this card, depth-first with a visited set and a budget in positions as T-29 describes, so the
    winnable-deals option (P3-20) works for Golf.
  - Rules tab, menu entry and the rules page as in P4-10.
- **Acceptance:** engine tests for each legal and illegal move and for the stock; on 50 seeds (0 to 49) every solver
  verdict of "winnable" replays on the engine, and the timing is recorded in the PR; both locales.

### P5-05 — Solver: Spider with 2 and 4 suits

- **Owner:** Sonnet
- **Depends on:** P2-08
- **Files:** `crates/core/src/solver/spider/**`, `crates/core/src/solver/mod.rs` (only if the budget type changes),
  `docs/tech/decisions.md` (T-29 numbers and any budget change)
- **Scope:** T-29 measured Spider with 2 suits at 14 winnable, 0 unwinnable and 86 % unknown, and with 4 suits at 2
  winnable and 98 % unknown, on seeds 0 to 49. Improve the search so that fewer deals stay unknown at the same or a
  stated budget; the 1-suit result must not get worse. Any budget change is recorded in T-29, and the measured table is
  replaced in the same PR.
- **Acceptance:** before and after tables for 2 and 4 suits on seeds 0 to 49 (winnable, unwinnable, unknown, average
  and slowest time); every "winnable" replays on the engine; no "unwinnable" without an exhausted search; the 1-suit row
  does not get worse.

### P5-06 — New game from seed: game first, then a seed from played games, random replay, five difficulty levels

- **Owner:** Sonnet
- **Depends on:** P4-09, P2-08
- **Files:** the new-game-from-seed screen under `crates/tui/src/screens/**`, `crates/core` (curated seed table per
  game, if stored there), `locales/*` (all nine), tests, `docs/tech/architecture.md` (screen flow)
- **Scope:** owner request 2026-11-20. Today "New game from seed" asks for a number first.
  - Flow: choose the game (and variant) first, then the seed.
  - The seed screen offers: type a seed (as today); pick one from the seeds already played in that game (from the
    history, newest first, with result and time); "random replay" that picks one of the played seeds at random.
  - Five difficulty levels with four curated seeds each, per game and variant: very easy, easy, medium, hard,
    very hard. Difficulty comes from solver metrics (e.g. solution length, branching, number of solver nodes) and is
    computed offline by an `xtask` step that writes the table; every curated seed is proven winnable.
- **Acceptance:** keyboard and mouse tests for the flow; the curated table has 4 × 5 seeds for every game variant
  with a proof of winnability; the levels are ordered by the chosen metric (documented in the card's PR).

## Done before the 2026-11 replan

### X-01 — Spike: card font in real terminals

- **Owner:** Sonnet (prototype) + User (testing)
- **Depends on:** P0-01
- **Files:** `docs/tech/spikes/x-01-card-font.md`; the prototype stays on its branch and is not merged
- **Scope:**
  - Build a throwaway font from six hand-written SVGs: four suits, one back tile, and a 2 × 2 tile
    set that should join into one picture. Glyphs go to Private Use Area code points; the advance
    width equals one terminal cell.
  - A small program prints the test card: each glyph alone, a row of glyphs, the 2 × 2 mosaic,
    glyphs next to box-drawing characters and next to wide text.
  - The user installs the font and runs the program in Windows Terminal, the classic console (CMD
    and PowerShell), macOS Terminal, one Linux terminal, PuTTY, and over SSH; screenshots go in the report.
- **Questions the report must answer:**
  1. Where does the terminal pick up the font by itself, and where must the player configure it?
  2. Are glyphs aligned to the cell, correctly scaled, one cell wide?
  3. Do multi-cell tiles join without gaps — are full-card illustrations feasible?
  4. Which code point range avoids collisions with common icon fonts?
  5. Is a patched complete monospace font needed for terminals without font fallback?
  6. Which build tool: a font library called from a script, or pure Rust in `xtask`?
- **Acceptance:** a results table per terminal with screenshots, and a recommendation for each question.
- **Scope (2026-11-19):** the owner tests on Windows 10 only (Windows Terminal, classic console); macOS, Linux, PuTTY and SSH are answered from documentation and marked not tested. Prototype on branch `spike/X-01-card-font`.
- **Results so far (owner, classic console, 2026-11-19):** BMP PUA glyphs (U+E6C0…) render one cell wide with the patched complete font; plane-16 twins show as squares and break the width; the glyph-only font turns all text into squares; the 2×2 mosaic joins horizontally, the vertical seam needs a check at a larger font size.
- **Done:** report `docs/tech/spikes/x-01-card-font.md` (#113); the decision it fed is recorded by F-02 (this PR, #115).

### F-02 — Checkpoint: graphics delivery

- **Owner:** Opus (Fable only on escalation — R-14)
- **Depends on:** X-01
- **Scope:** decide D-06 with the user from the spike report: the delivery method, the code point
  range, the glyph box and metrics, single-cell versus mosaic scope, the build tool. Record it in
  `decisions.md` (T-16 becomes final), write the artwork style guide into micro cards M-19…M-24 and
  mark them `READY`, adjust P5-01 and P5-02.
- **Acceptance:** decision recorded; cards updated; no production task of this phase starts before it.
- **Decided (owner, 2026-11-19):** D-06 and T-16 in `decisions.md`: own complete font (Cascadia Mono base, OFL 1.1,
  different family name), optional console switch on the classic Windows console, Python fontTools behind
  `cargo xtask fonts`, Windows Terminal supported from documentation, single-cell glyphs plus 2 × 2 mosaics,
  code points U+E6BC–U+E6FF. The glyph box, the artwork rules and the code point table are the style guide on
  the card of M-19…M-24; M-19…M-24 are `READY`; P5-01 and P5-02 are adjusted below; follow-ups O-34 (here)
  and O-35 (`phase-0.md`).
- **Done:** X-01 report (#113) and this PR (#115).

### P5-07 — Better hints: a hint that leads toward a win

- **Owner:** Sonnet
- **Depends on:** P4-10a
- **Files:** the hint code of each game (`crates/core/src/games/*/mod.rs` and a new `hints.rs` per game where it
  grows), its tests, `docs/tech/decisions.md` (only for a decision the orchestrator names)
- **Scope:** owner request 2026-11-21: the hints never repeat a position, but in Yukon playing only hints won none of
  200 deals. Rank hints so they make progress toward a win in every game: uncover face-down cards, empty a column
  when it helps, avoid moves that block a card the foundations need next. Where a solver answers within the hint time
  budget, a hint may take the first move of its solution. Measure before and after.
- **Acceptance:** for each variant, a table on seeds 0 to 199: deals won by playing hints only, before and after; no
  variant gets worse; following hints still never repeats a position; a hint never takes longer than the existing
  budget.

### P5-01 — Font builder and preview sheet

- **Owner:** Haiku
- **Depends on:** F-02
- **Files:** `xtask/src/fonts.rs`, `xtask/fonts/{build_font.py,requirements.txt}`,
  `assets/cards/codepoints.toml`, `assets/fonts/**`, `docs/tech/architecture.md` (the fonts paragraph only)
- **Scope (D-06, T-16; the style guide on the card of M-19…M-24 is the contract for glyph files):**
  - `cargo xtask fonts` finds Python 3 (`python3`, then `python`), runs `xtask/fonts/build_font.py` and
    fails with a message naming what was tried when Python or a package is missing. `requirements.txt`
    pins exact versions of `fonttools` and `skia-pathops`. Python is used only by this command, never by
    `cargo build`, `cargo test` or `cargo xtask check`.
  - Input: every `assets/cards/{fronts,backs}/<style>/glyphs/*.svg`. Each file is validated against the
    style guide: allowed elements only, exact viewBox, fill only, ink inside the box (inside the safe area
    for front glyphs).
  - Base font: Cascadia Mono, committed as `assets/fonts/base/CascadiaMono.ttf` with the licence text
    (`assets/fonts/OFL.txt`); the version used is written in `assets/fonts/NOTICE.md`. The glyph box is the
    Cascadia Mono cell from X-01: UPM 2048, advance 1200, ascent 1900, descent 480, line gap 0. The builder
    reads these from the base font and fails if they differ. An SVG point (x, y) in the viewBox 0 0 1200
    2380 becomes the font point (x, 1900 − y); every added glyph has advance 1200.
  - Output, committed: `assets/fonts/oXySolMono-Regular.ttf`, the base font with the card glyphs added.
    Family name `oXySol Mono` (name IDs 1, 4, 6, 16), no Cascadia name and no Reserved Font Name; the
    copyright and licence entries of the base (name IDs 0, 13, 14) are kept and one line is added for the
    new glyphs. The glyphs of the base are kept untouched, so the font stays a complete monospace font.
  - Outlines: SVG arcs and curves read with fontTools, overlapping contours united and cubic curves
    converted to quadratic (TrueType) with skia-pathops and `cu2quPen`. `mosaic.svg` (viewBox 0 0 2400
    4760) is one drawing sliced into four glyphs `mosaic-tl|tr|bl|br` by intersection with the four
    cell boxes.
  - `assets/cards/codepoints.toml` is the registry, one data file the TUI reads too (P5-02): a key
    `<fronts|backs>/<style>/<name>` and a code point per glyph, plus the reserved ranges, initially the
    table of the style guide. An assigned code point never changes; a new glyph takes a reserved slot of
    its group or the first spare one. Every code point lies in U+E6BC–U+E6FF and is used once, else the
    command fails. An SVG without a registry entry fails (message names the file); a registry entry
    without an SVG is skipped with a note (the art is not drawn yet).
  - `target/fonts/preview.html` (not committed): a contact sheet drawn with the built font through
    `@font-face`. Every glyph with key, style and code point, solid black on white at 6 × 12 px,
    10 × 24 px and large; the suit sets side by side; each mosaic assembled in 2 × 2 cells. This is what
    M-19…M-24 and U-02 review by eye.
  - The output is reproducible: fixed font timestamps, so two runs give identical bytes and identical
    registry contents.
- **Acceptance:** an end-to-end test builds a font from two fixture SVGs and reads it back (glyph count,
  code points, advance 1200, metrics 2048 / 1900 / 480, family name without "Cascadia"); it needs Python,
  so it is `#[ignore]` with a reason and run with `cargo test -p xtask -- --ignored fonts`, outside
  `--quick`. Unit tests that need no Python cover registry parsing, the range and duplicate checks and
  Python discovery. An invalid SVG fails with a message naming the file and the rule; the committed font
  contains every code point of the base font; running the command twice yields identical bytes.
- **Done:** #124 (Cascadia Mono 2407.24 static Regular; name ID 7 removed; xtask main/error touched)
