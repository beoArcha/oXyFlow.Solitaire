# Phase 5 (0.5) — Themes and colour vision, Spanish cards, Golf and the Spider solver

Goal: the table, the card face and the aura are set apart from the theme; the low-vision mode and the colour-vision modes are built on the Interface section of phase 4; the second new card style (Spanish) and the second new game (Golf, with its solver) are added; the Spider solver gets its 2-suit and 4-suit results improved. This is the phase worked on now, during version 0.5.x.

Exit criteria:

- P3-32, P3-33, P3-34, P3-35, P5-03, P5-04 and P5-05 are DONE;
- the visual fixes O-69 to O-107 are DONE, or `BLOCKED` with the owner's decision on the card;
- every new theme option passes the contrast tests, and the four-colour deck works at every card size;
- Golf's solver verdicts replay on the engine for seeds 0 to 49, and the Spider table is in T-29.

Order of work: the Interface options first, then the card style and the games, which do not touch those files. The visual fixes (O-69 to O-107) go by priority, P1 before P2 before P3, respecting each card's dependencies; Settings cards run one at a time in the order O-86, O-94, O-102, O-103, O-96, because they share the Settings screen.

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

### O-68 — Help window: a mouse click on the Keys and Command line tabs switches to them

- **Owner:** Haiku
- **Depends on:** none
- **Files:** `crates/tui/src/widgets/overlay/{mod.rs,nav.rs,tabs.rs,help_keys.rs,command_line.rs}`,
  `crates/tui/src/widgets/overlay/how_to_play/**`, the screen that opens Help from the main menu if it routes the mouse
  itself, tests
- **Scope:** owner report 2026-11-21 (screenshot, main menu, Help open on Rules, v0.5.0): clicking the top tabs `Keys` or
  `Command line` with the mouse does nothing; the keyboard switches tabs fine. Every top tab must switch on a left click
  from every other tab, from the main menu and from the board, at every dock position (the screenshot shows
  `Dock: centre`).
- **Acceptance:** tests that click each top tab (Keys, Rules, Command line) from each of the others, opened from the main
  menu and from the board, with the dock at least at its default and centre positions; keyboard behaviour unchanged.
- **Done:** #207

## Visual fixes and improvements

Owner visual review of 0.5.0 (2026-11-21; batch 1: O-69 to O-81, batch 2: O-82 to O-99 — Settings, Statistics, New game and Klondike; batch 3: O-100 to O-107 — Spider, FreeCell, Pyramid, Yukon and the Settings redesign). Backlog only: no card here starts until the owner says so. The cards are grouped by priority.

**P1 — Visual bugs and layout collisions**

### O-69 — Help window: one height for every tab; optional backdrop dimming

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/**`, `crates/tui/src/widgets/overlay/dock/**`, tests
- **Problem:** switching between `Keys`, `Rules` and `Command line` makes the window height jump; on `Rules` (How to play) the window is a little shorter, so the top of the `oXySol` logo shows above the yellow border.
- **Fix:** one height for every Help tab (a share of the terminal height with a minimum and a maximum); the logo never shows above the border. Optionally dim the background behind a centred window (theme tokens only).
- **Acceptance:** a test renders each tab at 100 × 30 and 200 × 60 and finds the same window rectangle; no logo cell above the border; colours only through theme tokens.
- **Done:** #211

### O-70 — Language tooltip disappears when a window opens

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/language_picker/**`, `crates/tui/src/screens/main_menu.rs`, tests
- **Problem:** the `English (US)` tooltip under the flag in the top-left corner stays on screen while Help or How to play is open.
- **Fix:** clear the language widget's hover and focus tooltip whenever any window or overlay opens.
- **Acceptance:** a test hovers or focuses the flag, opens Help, and finds no tooltip text in the buffer; closing the window does not bring it back until the next hover or focus.
- **Done:** #213

### O-71 — Help window: a separator between the scrolling text and the footer
- **Done:** #223

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/**`, tests
- **Problem:** on `Command line` (and the other scrolling tabs) the last text line (for example `Window:`) touches the window footer (`Tab: next tab…`) with nothing between them.
- **Fix:** a horizontal rule (`─`, ASCII `-`) or at least one empty row between the scrolling area and the footer, the same on every tab.
- **Acceptance:** a test on every Help tab finds the separator row between the last text row and the footer, in Unicode and ASCII mode.

### O-72 — Docked Help: a margin between the version label and the panel

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/footer.rs`, `crates/tui/src/widgets/overlay/dock/**`, `crates/tui/src/screens/main_menu.rs`, tests
- **Problem:** with Help docked right (`Dock: right`) the version (`v0.5.0`) in the bottom-right corner of the main view touches the double yellow border of the panel.
- **Fix:** keep one or two columns between the main view's footer and the docked panel border.
- **Acceptance:** a test docks Help left and right and finds at least one empty column between the version label (and any footer text) and the panel border.

### O-82 — Statistics footer: a space between `Copy seed` and `Esc Back`

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/stats.rs`, `crates/tui/src/screens/stats/**`, `crates/tui/src/widgets/footer.rs`, tests
- **Problem:** the bottom key bar of Statistics reads `c Copy seedEsc Back`, with no space between the two entries.
- **Fix:** the same separator between every footer entry; find why this pair loses it (width budget or a missing join).
- **Acceptance:** a test renders Statistics at 100 × 30 and 200 × 60 in all nine locales and finds at least one space between every pair of footer entries.
- **Done:** #212

### O-83 — Board footer: the `Do:` prefix is clipped when a side panel is open

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/widgets/footer.rs`, tests
- **Problem:** in Klondike with the right side panel open (`Guide` or `Possible moves`) the second footer line starts `ow: s: turn stock…` instead of `Do: s: turn stock…`.
- **Fix:** compute the footer width from the table area that is left when a side panel takes its columns, and clip from the end, never the start.
- **Acceptance:** a test opens each side panel in every variant at 100 × 30 and 130 × 40 and finds both footer rows starting with their full prefix.
- **Done:** #210 (the clipped prefix did not reproduce; the PR adds the regression test only, owner 2026-11-21: likely a cropped screenshot)

### O-84 — Board footer and `Now:` row: no overflow into the status line; the source column kept

- **Owner:** Sonnet
- **Depends on:** O-83
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/widgets/footer.rs`, tests
- **Problem:** with `Possible moves` open and an error shown, the last footer items (`p: pause   v: moves`) run onto the `Now:` row right after `Illegal move: the cards cannot go there.`; `Now: Carrying: eight of diamonds (from: Tab…` cuts off the source column number.
- **Fix:** footer entries never spill into the `Now:` row; when the `Now:` text is too long, shorten the card name first or use a compact column name (for example `T5`) so the source column stays; drop low-priority footer hints first on narrow widths.
- **Acceptance:** tests at the narrowest board width with each side panel open: the `Now:` row holds no footer text, the source column is visible while carrying, and the drop order of footer hints is fixed.
- **Done:** #217

### O-85 — `Turned over` counter: counts what its label says

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**` (header), `crates/core/src/session/**` only if the count lives there, `locales/*/board.toml`, tests
- **Problem:** moving `A ♥` to a foundation uncovers the face-down `2 ♦` in column 4, but the top bar still shows `Turned over: 0`.
- **Fix:** decide from the code what the counter counts: if it counts stock turns, rename it (for example `Stock draws`); if it is meant to count revealed cards, count each face-down tableau card that turns over. Record the choice on the card.
- **Acceptance:** a test reveals a tableau card and checks the counter (or the new label); all nine locales.

### O-86 — Settings: `Font face` shows its own value

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/*/settings.toml`, tests
- **Problem:** both `Font face` and `Font size` show `Ctrl+= / Ctrl+-`.
- **Fix:** `Font face` shows the current face (or that the terminal controls it); only `Font size` names the zoom keys.
- **Acceptance:** a test renders the Interface section with a face set and with the terminal in control and finds the right value on each row; all nine locales.
- **Done:** #209

### O-100 — Spider and Pyramid: no `A` in slots that are not ace foundations

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/widgets/card/**`, `crates/tui/src/widgets/pile.rs`, tests
- **Problem:** Spider (1 suit) draws `A` in all 8 top completion slots, although Spider removes full king-to-ace runs and does not build up from aces; Pyramid (Relaxed) draws `A` in the bottom-right pile of removed cards, where pairs that add up to 13 go.
- **Fix:** a marker that fits the game: in Spider `K`, `✓` or an empty frame; in Pyramid `13`, `×` or an empty frame. Each has a WGL4 and an ASCII form, and the How to play text agrees.
- **Acceptance:** a test renders a fresh Spider and Pyramid deal and finds no `A` in those slots; Klondike, FreeCell and Yukon foundations still show `A`.

### O-101 — Yukon footer: `Enter: pick up` and `a: quick move` when a card can move

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**`, `crates/tui/src/widgets/footer.rs`, tests
- **Problem:** in Yukon, with the focus on a playable face-up card (`J ♥` in column 1), the `Do:` bar shows only `Arrows: move | h: hint | i: how to play…`, without `Enter: pick up` and `a: quick move`.
- **Fix:** the contextual hints offer `Enter: pick up` and `a: quick move` whenever the focused card or group can move, in every Yukon variant (likely the footer still assumes a game without group moves from any face-up card).
- **Acceptance:** a test focuses a movable face-up card in each Yukon variant and finds both hints; a face-down or blocked card does not show them.

### O-102 — Settings: a live preview pane instead of the 4 × 1 swatch

- **Owner:** Sonnet
- **Depends on:** —
- **Files:** `crates/tui/src/screens/settings.rs`, `crates/tui/src/screens/settings/**`, `crates/tui/src/settings.rs`, `crates/tui/src/widgets/card/**`, `crates/tui/src/widgets/pile.rs` (reuse only), tests
- **Problem:** the inline 4 × 1 grey hatch next to `Card back` (theme Dracula) shows neither the theme colours nor the back style (Lattice); `Front style`, `Back style`, `Symbols`, `Card size` and the Aura options have only text or small unlabelled swatches.
- **Fix:** a two-column Settings layout with a live preview pane on the right, drawn on the active theme's table: one face-down card (theme, card back, back style, symbols, card size) and one or two face-up cards showing the focus, carried and target states (front style, symbols, card size, Aura, aura colour, style and size, target marks). It replaces the 4 × 1 swatch and the unlabelled Aura tiles. The cards are drawn by the existing card widget, not a copy. Below the width the two columns need, the preview moves under the list or hides with a note.
- **Acceptance:** a test changes each listed option and finds the preview changed; the 4 × 1 swatch is gone; fits at 100 × 30 (preview below or hidden) and 130 × 40 (side by side) in all nine locales.

**P2 — Focus, navigation and consistency**

### O-73 — Docked Help: dim the selection in the pane without focus

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/menu_list.rs`, `crates/tui/src/widgets/overlay/dock/**`, `crates/tui/src/theme/**` (a token if needed), tests
- **Problem:** with Help docked left or right and focused, the main menu keeps its bright yellow selection (`▶ Help`), which outshines the quiet blue header of the active Help panel.
- **Fix:** when focus is in the docked panel, the other pane's selection uses a muted style (for example a dark grey background); the selection stays visible without colour (the marker stays).
- **Acceptance:** a test moves focus between the panes and checks the selection style of each; the marker is present in both states; contrast tests pass.
- **Done:** #220

### O-74 — Help window: one footer layout and scroll indicator on every tab

- **Owner:** Sonnet
- **Depends on:** O-71
- **Files:** `crates/tui/src/widgets/overlay/**`, `locales/*/help.toml`, tests
- **Problem:** the footer differs by tab: `Keys` has a scrollbar but no line counter (`1-X / Y`) and no scroll hints; `Rules` has no `Tab` hint and says `Up/Down: move focus`, which is unclear; `Command line` shows `Tab: next tab`, the scroll keys and the counter.
- **Fix:** one footer format for `Keys`, `Rules` and `Command line`: the line counter whenever the text overflows, the main tab switch (`Tab`), the game switch on `Rules` (`Left/Right`), scrolling and closing (`Esc`).
- **Acceptance:** a test renders each tab with overflowing text and finds the same footer entries in the same order; all nine locales.

### O-75 — Windows: the active tab and the focused button look different

- **Owner:** Sonnet
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/**`, `crates/tui/src/theme/**`, tests
- **Problem:** the same solid white background with black text marks the active tabs (`[Keys]`, `[Klondike]`, `[Command line]`) and the focused `▶ Close [Esc] ◄` button, so it is unclear what `Enter` will do.
- **Fix:** a separate style for the active tab and for the focused button, so the player sees at once what `Enter` triggers; neither relies on colour alone.
- **Acceptance:** a test checks that the active-tab and focused-button styles differ in every theme and that both keep a non-colour marker; contrast tests pass.

### O-76 — Docking and pane focus keys shown where they apply

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/**`, `locales/*/help.toml`, tests
- **Problem:** the top border shows only `Dock: centre` without the key that changes it; docked mode does not say how to move focus between the panes (`F6`).
- **Fix:** the border label names the key (`Dock: center [d]`, key taken from the key table) and the docked panel footer shows compact `d Dock  F6 Focus` hints.
- **Acceptance:** a test finds the key in the border label and the two hints in the docked footer; keys come from the key table, not literals; all nine locales.

### O-77 — Docked Help: key and description on one line

- **Owner:** Sonnet
- **Depends on:** O-74
- **Files:** `crates/tui/src/widgets/overlay/help_keys.rs`, `crates/tui/src/widgets/overlay/dock/**`, tests
- **Problem:** in the narrow docked panel every shortcut takes two lines (key, then description), which means long scrolling although most pairs fit on one row.
- **Fix:** a two-column row (a key column of about 11 to 12 characters and the description on the same line); only long key combinations (for example `Ctrl +/-/0`) wrap to a second line.
- **Acceptance:** a test at the docking width finds key and description on one row for short keys and a wrap only for long ones; fewer rows than before; all nine locales fit.

### O-87 — Large cards: a spade with a clear stem

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/card/**` (the large pip drawing), tests
- **Problem:** on `Large` cards the spade pip (`9 ♠`, `2 ♠`, `Q ♠`) is a thick symmetric cross with no stem, so its outline is closer to the diamond than to the club.
- **Fix:** redraw the large spade with a pointed top and a stem or foot at the bottom, in Unicode and ASCII mode.
- **Acceptance:** a snapshot of the large pips shows four outlines that differ by shape; the spade has a stem row the diamond does not.
- **Done:** #215

### O-88 — Stock card back: no stray marks in the centre

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/card/**`, `assets/cards/backs/**` only if the style data causes it, tests
- **Problem:** the centre of the face-down stock (`x24`) shows two small offset dark vertical bars that look like a broken glyph.
- **Fix:** a clean symmetric centre detail, or no centre glyph, at every card size and back style.
- **Acceptance:** snapshots of the stock in each back style and size show a symmetric centre without the bars.
- **Done:** #221

### O-89 — Keyboard focus skips face-down cards

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**` (focus movement), tests
- **Problem:** arrow keys can put the focus on face-down tableau cards, and picking one up gives `Illegal move: the card is face down.`
- **Fix:** Up and Down within a column skip face-down cards; entering a column lands on the first face-up card, or on the empty slot.
- **Acceptance:** tests in Klondike and Yukon move through columns with face-down cards and never focus one; the mouse is unchanged.

### O-90 — Quick options: the app's selection colour, one name for Aura, real buttons

- **Owner:** Haiku
- **Depends on:** O-75
- **Files:** `crates/tui/src/widgets/overlay/quick_options.rs`, `locales/*/settings.toml`, tests
- **Problem:** the active row uses a white highlight instead of the yellow selection used everywhere else; the setting is called `Aura` but its description says `Glow:`; the bottom actions (`Save as default`, `Close [Esc]`) have no button style and no focus hints.
- **Fix:** the yellow selection style for the active row, `Aura` in every label and description, and the bottom actions drawn and focused like the other windows' buttons (`[…]`).
- **Acceptance:** a test checks the row style token, the word `Glow` is gone from all nine locales, and the buttons can be focused and pressed by keyboard and mouse.

### O-91 — Possible moves: readable unavailable entries, active ones first

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/moves_window.rs`, `crates/tui/src/widgets/moves.rs`, `crates/tui/src/theme/**` (a token if needed), tests
- **Problem:** unavailable entries and their reasons (`Nothing to pick up here`, `Carrying cards`, `Nothing to undo`) have very low contrast on the dark navy background and take about 70 % of the panel.
- **Fix:** raise the contrast of the unavailable text to pass the contrast tests, and list the available actions first with the unavailable ones grouped or condensed below.
- **Acceptance:** contrast tests cover the unavailable style in every theme; a test finds the available actions above the unavailable ones.
- **Done:** #219 (the moves window only; the side panel order in `screens/board/moves_panel.rs` is a follow-up)

### O-92 — Guide panel: `▶` means focus only

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/guide/**`, tests
- **Problem:** the Guide prefixes the available keys (`▶ Arrows`, `▶ s`, `▶ h`) with `▶`, while the Legend just below says `▶` is the focus.
- **Fix:** a different marker (WGL4 and ASCII form) for available or recommended keys, so `▶` only means focus; the Legend explains both.
- **Acceptance:** a test finds no `▶` in the Guide's key list and finds the new marker in the Legend.
- **Done:** #216

### O-93 — Footer: the `v: moves` highlight clears

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/footer.rs`, `crates/tui/src/screens/board/**`, tests
- **Problem:** `v: moves` stays highlighted in solid white after the `Possible moves` panel is closed.
- **Fix:** hover and click highlights on footer entries clear when the panel is toggled off and when the player goes back to the keyboard.
- **Acceptance:** a test opens and closes the panel by mouse and by key and finds no highlight left on the entry.

### O-94 — Settings: unavailable rows dimmed; no reference to a missing screen

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/*/settings.toml`, tests
- **Problem:** `Console font` shows working arrows `◄ unavailable ►` although it cannot change; the note at the bottom says `(see the graphics check)`, which Settings does not show.
- **Fix:** dim unavailable rows and hide their `◄ ►`; either link the graphics check from Settings or rewrite the note so it does not point at something missing.
- **Acceptance:** a test renders an unavailable row without arrows in the dimmed style; the note text in all nine locales matches what the screen offers.
- **Done:** #218

### O-95 — Statistics: a plain summary line and footer hints for the action bar

- **Owner:** Sonnet
- **Depends on:** O-82
- **Files:** `crates/tui/src/screens/stats.rs`, `crates/tui/src/screens/stats/**`, `locales/*/stats.toml`, tests
- **Problem:** the line `Automatic 27% (7 of 26) · failed 3 · run 1 · finish 0` reads like internal data and does not say what `7 of 26` means next to 123 games; the action bar (`[ Play this seed again ]`, `[ Copy seed ]`, `[ Check ]`, `[ Check all ]`, `[ Back ]`) has no footer hints for `Check`, `Check all` or moving between buttons.
- **Fix:** a line a player understands (what is counted and out of what), and footer hints for every action-bar button and for moving between them.
- **Acceptance:** a test renders the summary and finds the new wording and every action in the footer; fits at 100 × 60 (T-40) in all nine locales.

### O-103 — Settings in three tabs: Audio & Visual, Popular (default), Gameplay

- **Owner:** Sonnet
- **Depends on:** O-102
- **Files:** `crates/tui/src/screens/settings.rs`, `crates/tui/src/screens/settings/**`, `crates/tui/src/settings.rs`, `locales/*/settings.toml`, `docs/tech/architecture.md` (settings screen contract), tests
- **Problem:** the flat list of 23 options is long and mixes display, sound, system and gameplay settings in one column, leaving no room for previews and making the common ones slow to reach.
- **Fix:** three tabs with `Popular` in the middle and active by default (`◄ Audio & Visual │ [Popular] │ Gameplay ►`). **Popular:** the 7 or 8 most changed settings (for example Language, Theme, Card size, Card back, Volume, Aura, Auto-finish) as aliases of their canonical rows; a change in either place shows in both. **Audio & Visual:** Colors, Full screen, Theme, Card back, Front style, Back style, Symbols, Console font, Font face, Font size, Card size, Aura colour, Aura style, Aura size, Sound style, Sound cues, Music, Volume, Process priority. **Gameplay:** Language, Aura, Target marks, Guide panel, Possible moves panel, Auto-finish and later gameplay options. `Tab` / `Shift+Tab` (optionally `[` / `]`) switch tabs from anywhere, so `Left` / `Right` only change values; the footer says `Shift+Tab / Tab: tab`. This replaces the `Performance` section part of O-96 (Process priority goes to Audio & Visual).
- **Acceptance:** tests: Popular is active on open; every setting is reachable in its canonical tab; changing an alias changes the canonical value and the other way round; Tab and Shift+Tab switch tabs and Left/Right never do; keyboard and mouse reach every tab and row; all nine locales fit at 100 × 30.

### O-104 — FreeCell: a gap between the free cells and the foundations

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**` (FreeCell layout), tests
- **Problem:** FreeCell's top row has 8 evenly spaced slots (4 empty free cells, then 4 `A` foundations) with no gap or divider between the two groups.
- **Fix:** a wider gap between slot 4 and slot 5, and optionally a quiet distinct style for the free cells, so the two zones read apart without colour.
- **Acceptance:** a test finds a wider gap between the fourth and fifth slot than between the others at every card size; FreeCell still fits at 100 × 30.

### O-105 — Pyramid: blocked cards look different from playable ones

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**` (Pyramid), `crates/tui/src/widgets/card/**`, `crates/tui/src/widgets/pile.rs`, `crates/tui/src/theme/**` (a token if needed), tests
- **Problem:** all 28 pyramid cards have the same bright white face whether or not cards in the row below cover them, so the player has to trace overlaps to see what is playable.
- **Fix:** a subtle dim or shade (face or border) on covered cards so uncovered, playable cards stand out; it must not rely on colour alone (for example a dimmer frame glyph as well) and must pass the contrast tests.
- **Acceptance:** a test on a fresh deal finds the 7 bottom cards in the playable style and the 21 others in the blocked style; contrast tests pass in every theme.

### O-106 — Covered cards: no clipped big pips (FreeCell, Yukon, Pyramid)

- **Owner:** Sonnet
- **Depends on:** O-87
- **Files:** `crates/tui/src/widgets/card/**`, `crates/tui/src/widgets/pile.rs`, tests
- **Problem:** in dense face-up columns (FreeCell, Yukon) and overlapped Pyramid rows, a covered card shows a clipped half or a 1 to 2 px tip of its large centre pip just under the rank and suit header, which looks ragged.
- **Fix:** when a card is covered from below, do not draw the clipped part of its centre pip (or tidy the visible header strip), so a covered card shows only a clean rank and suit index.
- **Acceptance:** snapshots of covered cards in each of the three games at every card size show no partial pip rows; an uncovered card is unchanged.

### O-107 — Yukon: column 7 never runs off the bottom

- **Owner:** Sonnet
- **Depends on:** —
- **Files:** `crates/tui/src/screens/board/**` (column layout), `crates/tui/src/layout/**`, tests
- **Problem:** at Moves 0 with Large cards the last card of column 7 (`A ♣`) already almost touches the bottom status bar; one or two more cards would push it off screen.
- **Fix:** compress the vertical card spacing of a column as it grows (face-down cards first, then face-up overlap) so the whole column, its last card fully visible, always fits above the status bar; fall back to a smaller card size only when compression is not enough. (The review text for this item was cut off after "Ensure dynamic vertical compression of"; the owner may refine the fix.)
- **Acceptance:** a test builds the longest possible Yukon column (6 face-down + 18 face-up, and longer after moves) at 100 × 30 and 100 × 60 with every card size and finds the last card fully drawn above the status bar; the same test covers Klondike and Spider.

### O-108 — Install scripts that check dependencies first (Linux, macOS, Windows)

- **Owner:** Sonnet
- **Priority:** P1. Building from source fails on a fresh Ubuntu (WSL).
- **Depends on:** —
- **Decision:** T-46
- **Files:**
  - `build/install.sh` (new: Linux and macOS);
  - `build/install.ps1` (new: Windows);
  - `build/README.md` (new);
  - `README.md` ("Build from source");
  - `docs/site/src/install.md` and `docs/site/src/pl/install.md`;
  - `.github/workflows/pr.yml` (a job that runs the scripts' check mode and lints them);
  - `docs/tech/workflow.md` (the new job);
  - `docs/tech/decisions.md` (T-46);
  - tests.
- **Problem:** owner report of 2026-11-21. On Ubuntu under WSL, both `cargo install --git … --branch release oxysol` and the
  `curl` + `cargo build` path from the README fail:
  - first, `pkg-config` is missing;
  - after `sudo apt install pkg-config`, the build stops in `alsa-sys`: `The system library alsa required by crate alsa-sys
    was not found` (`alsa.pc` is missing).

  The README names `build-essential libasound2-dev` but not `pkg-config`. Nothing checks the dependencies before a build
  of several minutes, and the Linux and macOS paths are marked "written, not run".
- **Fix:** one install script per platform in `build/`. Each script checks every dependency before it builds anything,
  then acts by mode:
  - `--check`: stops at the first problem, with a non-zero exit code.
  - `--list`: stops and lists every missing dependency, with the exact command that installs it on this system.
  - `--install`: installs what is missing through the system package manager, after asking. It checks again, and
    builds and installs only if the second check passes.

  Without a mode flag the script asks which of the three to use. In a non-interactive run it behaves like `--list`.

  Dependencies to check:
  - **Rust:** `rustup`/`cargo`. The pinned toolchain is installed by rustup on the first build.
  - **C toolchain:** `cc`/`gcc` on Linux, the Xcode command line tools on macOS, and the MSVC Build Tools with a working
    `link.exe` on Windows.
  - **Linux only:** `pkg-config` and the ALSA headers (`libasound2-dev`, `alsa-lib-devel`, `alsa-lib` …).
  - **Download tools:** `curl` and `tar`.

  Package managers: `apt`, `dnf`, `pacman` and `zypper` on Linux, Homebrew on macOS (only if it is already present;
  otherwise `--list` names the steps), and `winget` on Windows. An unknown distribution falls back to `--list`
  behaviour, with the package names.

  `--no-audio` skips the ALSA check and builds with `--no-default-features`. When only the sound dependencies are
  missing, the scripts offer it as a choice.

  The scripts build the `release` branch and install the binary like `cargo install`: `~/.cargo/bin`, or
  `%USERPROFILE%\.cargo\bin` on Windows. They also work one-line from a URL, with `curl -fsSL …/build/install.sh | sh -s --
  --list` and the PowerShell equivalent.
- **Docs:**
  - README, the site page and its Polish twin describe the scripts first, then the manual steps.
  - The manual Linux line becomes `sudo apt install build-essential pkg-config libasound2-dev`.
  - The "verified on" notes say what was actually run.
- **Acceptance:**
  - In CI, the `pr.yml` job runs `--check` and `--list` on Ubuntu, macOS and Windows runners. On a minimal Ubuntu
    container without the ALSA headers it runs `--list` and checks the output names `libasound2-dev` and
    `pkg-config`.
  - ShellCheck passes on `install.sh`, and PSScriptAnalyzer passes on `install.ps1`.
  - The owner runs `--install` on Ubuntu (WSL), and the PR records the result.
  - Both locales of the site page are updated.

**P3 — Main view polish, widgets and wording**

### O-78 — Main menu: Resume set apart and balanced under the logo

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/main_menu.rs`, `crates/tui/src/screens/main_menu/resume.rs`, tests
- **Problem:** the menu block is aligned to the long `Resume: Klondike, Draw 1, 0:47, Moves 26` entry, so the shorter items look shifted left under the centred `oXySol` logo.
- **Fix:** one empty row between `Resume` and the other entries, and the game details of `Resume` in a muted secondary colour when it is not selected.
- **Acceptance:** a test finds the empty row and the muted style; the menu still fits at 100 × 30 in all nine locales.

### O-79 — Bottom key bar: keys in an accent colour

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/footer.rs`, `crates/tui/src/theme/**` (a token if needed), tests
- **Problem:** in `Enter Open  Tab Language  q Quit  ? Help` keys and labels share one colour, so the bar is hard to scan.
- **Fix:** keys in an accent colour (for example the light blue of the `Keys` tab), labels muted; the key/label split stays readable without colour (spacing as today).
- **Acceptance:** a test checks that key and label styles differ in every theme; contrast tests pass.

### O-80 — Language selector: quieter at rest, clearer when open

- **Owner:** Haiku
- **Depends on:** O-70
- **Files:** `crates/tui/src/widgets/language_picker/**`, tests
- **Problem:** at rest the widget shows three arrows (`◄`, `►`, `▼`), which is noise; in the open list the two-letter codes (`EN`, `PL`, `DE` …) compete with the full language names.
- **Fix:** at rest show only `[flag] PL ▾`; show `◄ ►` only while the widget has focus; dim the two-letter codes in the open list. Keep a WGL4 and an ASCII form of each arrow.
- **Acceptance:** tests of the rest, focused and open states in Unicode and ASCII mode; mouse targets still work.

### O-81 — Window title by context, en-US spelling and wording

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/widgets/overlay/**`, `locales/en-US/help.toml`, `locales/pl-PL/menu.toml`, tests
- **Problem:** opening How to play shows the generic title `Help`; en-US uses the British `centre` (`Dock: centre`, `stays centred`); the Polish `Nowa gra z ziarna` may be unclear to players who do not know the term; Vim users pressing `j`/`k` may hit `h` (Hint) by accident.
- **Fix:** the title follows how the window was opened (`How to play`, or `Help · Rules`); en-US `centre` → `center` and `centred` → `centered`; Polish `Nowa gra z ziarna (seed)` or `Nowa gra z kodu (seed)` (the owner picks); decide whether `h` needs a guard or a note when hints affect the statistics (record the answer on the card; change code only if the owner asks).
- **Acceptance:** tests for both titles; locale parity passes; the Polish line fits at 100 × 30.

### O-96 — Settings: labelled Aura swatches, a Performance section, Left/Right in the footer

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/settings.rs`, `crates/tui/src/screens/settings/**`, `locales/*/settings.toml`, tests
- **Problem:** the three `Aura:` preview swatches have no labels; `Process priority` sits under `Interface` rather than a performance or system group; the footer says only `Enter Change` although the rows also take `◄ ►`.
- **Fix:** label the swatches (focus, carried, target); move `Process priority` into a `Performance` section; add `Left/Right` to the footer hint.
- **Acceptance:** a test finds the three labels, the new section with the priority row, and the footer hint; all nine locales fit at 100 × 30.

### O-97 — Statistics table: results by style, the legend's unchecked mark, local time

- **Owner:** Haiku
- **Depends on:** O-95
- **Files:** `crates/tui/src/screens/stats.rs`, `crates/tui/src/screens/stats/**`, `locales/*/stats.toml`, tests
- **Problem:** every `Result` (`Won`, `Abandoned`, `In progress`) has the same white text; unchecked rows in `Winnable` show `· -` while the legend says `· not checked`; times are shown in UTC.
- **Fix:** a style per result (for example green won, muted abandoned, accent in progress) with the word still carrying the meaning; unchecked cells show exactly the legend mark; local time instead of UTC (or a setting), stated in the header.
- **Acceptance:** tests for the three result styles in every theme (contrast passes), the unchecked mark equal to the legend, and a time rendered in the local offset.

### O-98 — Choose a game: en-US spelling, spacing, a key and wording for Winnable deals, full footer
- **Done:** #222

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/screens/game_select.rs`, `crates/tui/src/screens/game_select/**`, `locales/en-US/menu.toml` and the other locales for new keys, tests
- **Problem:** `English (US)` uses the British `Favourites` / `favourite (f)`; there is no empty row between `Australian` and the game description (`Easy: Build the foundations…`); `[ Winnable deals only: off ]` has no key and its `off` text (`A new game then searches…`) reads oddly; the footer does not list `/ Search`, `f Favorite` and `u Undo`.
- **Fix:** en-US `Favorites` / `favorite (f)`; an empty row above the description (and below `Favorites`); a key for `Winnable deals only` shown on the button and a clear text for each state; the footer lists every key of the screen.
- **Acceptance:** tests for the en-US words, the empty rows, the key toggling the option, and the footer entries; fits at 100 × 30 in all nine locales.

### O-99 — Large cards: consider dropping the centred rank and suit line (optional)

- **Owner:** Haiku
- **Depends on:** O-87
- **Files:** `crates/tui/src/widgets/card/**`, tests
- **Problem:** `Large` cards show rank and suit three times: top-left (`9♠`), a centred line under the top border (`9 ♠`) and bottom-right (`9♠`).
- **Fix:** try the large face without the centred line so the main pip has more room; keep it if a card becomes harder to read in any style. The owner decides from a before and after snapshot.
- **Acceptance:** before and after snapshots for every card style in the PR; the rank stays readable at the large size.

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
