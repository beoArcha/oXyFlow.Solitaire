# Interface review: usability and readability (U-04)

Reviewer: Claude Fable 5.1 for the owner, 2026-11-16. Reviewed by Opus. Scope: every screen of
`oxysol-tui` at the state of `main` after O-19 (`60c4ddb`; P2-07 and O-19 changed the engine and
the launcher only, no screen), both locales, both glyph modes, the four terminal sizes the owner
named. No product code was changed; this document and the refined task cards are the deliverable.

Method: a throwaway example (not committed) drove `App` on a Ratatui `TestBackend` through the
public API, the way the test harness in `crates/tui/src/test_support.rs` does, and printed the
text buffer of every screen at 80 × 24, 100 × 30, 130 × 40 and 200 × 60, in en-US and pl-PL, in
Unicode and ASCII mode: main menu, game selection, seed entry, Klondike/Spider/FreeCell/Pyramid
mid-game (browsing, hint, carrying, after two draws, panel hidden, paused, confirm dialog),
settings, statistics, the three help tabs, the win dialog. Excerpts below are trimmed (columns
cut, empty rows removed). Colors do not show in a text buffer; the color findings come from the
palette files and the ANSI mapping, not from a screenshot.

Owner priorities applied in that order: readability first, then ease and intuitiveness: a
newcomer must see at once what is on the board, what is selected and what to do next, without
reading help; it must work on an old laptop at low and high resolution, in the classic Windows
console (WGL4 glyphs only), Windows Terminal, Linux and macOS terminals, in ASCII mode and on
16-color terminals.

## 1. Executive summary: the ten problems by impact

| # | Problem | Where | Fix (short) | Card |
|---|---|---|---|---|
| 1 | The block-art rank of P3-13 is less readable than the corner index. `D`/`0`, `W`/`H`, `7`/`1` and `10` blur at half-block resolution; it is the first thing a newcomer looks at on a medium or large card. | `widgets/card/rank_font.rs`, `face.rs` | Remove the pixel font. The centre shows the rank as **plain bold text next to the suit symbol** on one row, with the suit pictogram beneath it; the corner indices stay. See §4. | P3-14 |
| 2 | The bottom of the board shows the same fact up to three times: the announcer line (focus), the `Now:` line (focus again), the guide panel (stock counter again), the footer (keys that `Possible:` already lists). Five chrome rows at 100 × 30 leave 25 for the board. | `screens/board/view.rs`, `layout/mod.rs::chrome_with` | One **game-information** place (header row, or the top box of the side panel when it is shown) and one **controls** place (two rows: `Now:` with the last event right-aligned, `Do:` with the keys). Drop the footer and the announcer row on the board. See §3. | P3-15 |
| 3 | The guide panel has no frame: one `│` edge, no top/bottom border, same background as a dialog; it also disappears silently below 130 columns although the setting says "On". | `widgets/guide.rs`, `layout/mod.rs` | A framed panel with a title in its border; below 130 columns `g` opens the same content as a window and the settings row names the requirement. | P3-15, P3-18 |
| 4 | At the compact size (7 × 5 cards) a column of face-down cards draws only its top border rows, which is exactly what stacked empty slots look like; the `▒` fill never shows. A Spider deal at 100 × 30 reads as a wall of empty frames. | `layout/fan.rs` (down step 1), `widgets/card/mod.rs` | Draw the face-down pattern in the top border row of a covered card (`┌▒▒▒▒▒┐`), or collapse the face-down prefix to one row with a count whenever the down step is 1. | P3-14 |
| 5 | Selection, target and hint marks are one character each (`●`, `+`, `?`) in the top border, and the target halo is a ring of `·`. A newcomer does not see where the picked run may go, nor what the hint points at. | `widgets/card/frame.rs`, `aura.rs`, `glyphs.rs` | Targets get a visible slot treatment (border style plus a filled marker word on empty slots), the hint marks both source and target with a two-cell marker, and the halo symbols are chosen per size. See §4.4. | P3-14, P3-21 |
| 6 | Tab does nothing on the menus, the settings, the statistics buttons, the seed entry buttons and the dialogs; buttons are reachable only by mouse or by the arrows in two dialogs. | `widgets/menu_list.rs`, every screen's `handle` | A focus ring per screen: Tab/Shift+Tab cycle it in reading order, Enter/Space activate; the help window keeps Tab for its tabs. See §5. | P3-17 |
| 7 | Zoom: nothing in Help says how to make the cards larger; `+`/`-` are bound to "select more/fewer cards", which Up/Down already do inside a column. Ctrl+Plus/Minus and Ctrl+wheel are taken by every modern terminal. | `input.rs`, help locales | `+`/`=`, `-`, `0` zoom the card size (setting "Card size", default Auto); run size moves to Shift+Up/Down; Help documents the terminal's own zoom; the classic Windows console also changes its font. See §5.2. | P3-16 |
| 8 | Polish: pile names are injected in the nominative into sentences that need the accusative: `Przeglądasz Kolumna 1` ("Browsing Column 1"), `Trzymasz … z Talia`. The announcer form `Kolumna 1: siódemka trefl` is correct. | `screens/board/guidance.rs`, `locales/*/board.toml` | Templates of the form `{pile}: {what}` only (no verb before a pile name), or declined pile names per template. | P3-22 (new) |
| 9 | 16-color terminals: the Dark theme maps focus, target and the focus halo to `Color::Yellow`, which the classic Windows console and xterm draw as dark olive; borders are `DarkGray` on `Black`. The High Contrast theme draws the table and the chrome both pure black, so the board has no edge. | `theme/dark.rs`, `high_contrast.rs`, `palette.rs::TEXT_PAIRS` | `LightYellow` for the focus tokens, `Gray` for borders in the ANSI column; a one-cell border around the table in High Contrast; add the mark-on-table pairs to the contrast test. See §6. | P3-23 (new) |
| 10 | Menus and settings: the focus markers sit at the full width of the longest label (`► New game           ◄`); settings values are not aligned and not grouped; the undo toggle on the game selection is drawn in the focus style although it is not focused; the status line is cut mid-word at 100 columns (`…press Esc to cance`). | `widgets/menu_list.rs`, `screens/settings/*`, `game_select.rs`, `announce.rs` | Marker next to the label, right-aligned values in two columns, section headings, a button style for toggles, and no sentence longer than 60 columns in a one-row area. | P3-24 (new) |

Everything else found is listed per screen in §2 and carries a lower impact.

## 2. Findings per screen

### 2.1 Board, compact size (100 × 30, 7 × 5 cards)

Evidence, Klondike Draw 3 after two draws, en-US Unicode, trimmed (rows 1–18 of 30):

```text
                Klondike, Draw 3 | seed 42 | 12:34 | Moves: 2
┌─────┐   ┌Q♥─┌9♥─┌3♣───┐     ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
│▒▒▒▒▒│   │   │   │     │     │     │   │     │   │     │   │     │
│▒▒▒▒▒│   │  ♥│  ♥│  ♣  │     │     │   │     │   │     │   │     │
└───18┘   └───└───└───3♣┘     └─────┘   └─────┘   └─────┘   └─────┘

┌7♣───┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
│     │   ┌K♣───┐  ░┌─────┐░  ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
│  ♣  │   │     │  ░╔A♣══►╗░  ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
│     │   │  ♣  │  ░║     ║░  ┌5♣───┐   ┌─────┐   ┌─────┐   ┌─────┐
└───7♣┘   │     │  ░║  ♣  ║░  │     │   ┌6♥───┐   ┌─────┐   ┌─────┐
          └───K♣┘  ░║     ║░  │  ♣  │   │     │   ┌7♦───┐   ┌─────┐
                   ░╚═══A♣╝░  │     │   │  ♥  │   │     │   ┌10♠──┐
                   ░░░░░░░░░  └───5♣┘   │     │   │  ♦  │   │     │
                                        └───6♥┘   │     │   │  ♠  │
                                                  └───7♦┘   │     │
                                                            └──10♠┘
                           (8 empty rows)
                Tableau 3: ace of clubs. Dealt from the stock.
Now: Browsing Tableau 3, ace of clubs | Turned over: 0
Possible: Enter: pick up | a: send to foundation | s: turn stock | Arrows: move | h: hint | u: undo
           Enter Select   u Undo   r Redo   h Hint   s Stock   p Pause   ? Help   Esc Back
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| Face-down cards (`┌─────┐` rows under `K♣`, `A♣`, `5♣`) are indistinguishable from empty slots: with a down step of 1 only the top border shows. | `layout/size.rs::down_step` is 1 at 7 × 5; `widgets/card/mod.rs::draw` fills the interior, never the border row. | When the visible part of a face-down card is one row, draw it as the pattern row (`┌▒▒▒▒▒┐`, ASCII `+#####+`), or collapse the face-down prefix with a count (`▒▒ x3 ▒▒`) whenever the step is 1. The collapsed form exists already (`draw_collapsed`); it is used only when the fan does not fit. | High / small |
| The waste fan shows fragments: `┌Q♥─┌9♥─┌3♣───┐` on top, `└───└───└───3♣┘` at the bottom. | `layout/columns.rs::waste` fans three cards with `waste_step` = 4 at width 7. | Fan covered waste cards by their label width + 1 (`10♥` needs 4 cells, `Q♥` 3), draw a covered card's bottom row blank under the next card, and show only the playable card's bottom index. At 7 × 5 fan two cards, not three. | Medium / small |
| Eight empty rows under the tableau while the cards are the smallest size. | `columns.rs::fits` reserves `FAN_BUDGET` = 14 rows for a king-to-ace run plus two card heights. 9 × 7 needs 29 rows, the body has 25. | Keep 7 × 5 here (the next size does not fit with any sane budget), but give the rows back by taking two chrome rows away (§3) and by letting the fan use a larger step than `up_step` when there is room (`fan.rs` only shrinks). | Medium / small |
| The stock badge `18` sits in the bottom border where a rank index sits on face-up cards, so it reads as a card. | `paint_bottom_row` draws the badge like an index. | Prefix it: `x18`, the way the collapsed row does; on 9 × 7 and up draw `18 cards` as a label under the stock (the row is free in every layout; see `counters::beside_stock`). | Low / small |
| Empty foundations are blank frames; a newcomer does not know that aces go there. | `Face::Slot` draws nothing inside. | A muted placeholder in empty slots: `A` in a foundation, nothing in a cell or column, `↑` (WGL4 U+2191, ASCII `^`) in an empty stock that can be recycled, `x` when no pass is left. | Medium / small |
| The status row and `Now:` say the same thing; the status row is cut mid-word at 100 columns while carrying: `Tableau 3: ace of clubs selected, 4 targets. Cards picked up. Choose a target, or press Esc to cance`. | `status.rs::focus_text` + `announce::announce` joins focus and event; `view.rs` centres it in one row; `guidance.rs` builds `Now:` from the same state. | See §3: the focus sentence lives only in `Now:`; the event goes to the right end of that row (short texts, at most 40 columns, with the Error/Success tokens); the sentence `Cards picked up. Choose a target…` is dropped because `Do:` says `Enter: drop · Esc: put back`. | High / medium |
| The footer repeats keys that `Possible:` lists and adds nothing a newcomer needs (`r Redo`, `p Pause` are in Help). | `view.rs::FOOTER`. | Drop the footer on the board; `Do:` ends with `? keys` and `Esc menu`. | Medium / small |
| The stock counter is appended to `Now:` (`| Turned over: 0`) because no free row exists beside the stock in a column layout. | `counters::beside_stock`. | It is game information: put it in the header (compact) or the Game box of the panel (standard and up), never in `Now:`. | Medium / small |
| Pyramid: the name `Pyramid row 7, position 1` is long; the `Now:` row becomes `Browsing Pyramid row 7, position 1, two of clubs | Pass 1 of 3`. | `announce::pile_name`. | `Row 7, card 1` (and the Polish equivalent); the counter moves out of `Now:`. | Low / small |

Hint display, Klondike 100 × 30:

```text
┌─────┐   ┌─────┐             ┌────?┐   ┌─────┐
…
          ┌K♣───┐   ┌─────┐   ┌─────┐
          │     │   ┌A♣──?┐   ┌─────┐
```

The hint is a single `?` in the top-right corner of the source and of the target. On a 7 × 5 card
that is one cell among 35. Fix (P3-14): the hint uses the hint border style on both cards (the
Unicode hint set is identical to the normal set today: `hint: edges(["┌", "┐", "└", "┘"], "─", "│")`
in `frame.rs`) and a two-cell marker `?→`/`→?` (`→` is WGL4 U+2192; ASCII `?>` `>?`), and the
`Now:` row names the move: `Hint: ace of clubs to Foundation 1`.

### 2.2 Board, standard size (130 × 40, 11 × 9 cards, guide panel)

Evidence, en-US Unicode, trimmed to the first three columns and the panel:

```text
                       Klondike, Draw 3 | seed 42 | 12:34 | Moves: 0
┌─────────┐   ┌─────────┐      ┌─────────┐   ┌─────────┐   │ Guide (g: hide)
│▒▒▒▒▒▒▒▒▒│   │         │      │         │   │         │   │ Goal
│▒▒▒▒▒▒▒▒▒│   │         │      │         │   │         │   │ Build the four foundations
└───────24┘   └─────────┘      └─────────┘   └─────────┘   │ from ace to king, by suit.
                                                           │ Turned over: 0
░░░░░░░░░░░░░                                              │
░╔7♣══════►╗░  ┌─────────┐   ┌─────────┐                   │ Keys
░║   ▀▀█   ║░  │▒▒▒▒▒▒▒▒▒│   │▒▒▒▒▒▒▒▒▒│                   │ ► Arrows  move
░║    █    ║░  ┌K♣───────┐   ┌─────────┐                   │   Enter   pick up / drop
░║    ▀    ║░  │   █ █   │   │▒▒▒▒▒▒▒▒▒│                   │   Esc     put back
░║         ║░  │   █▀▄   │   ┌A♣───────┐                   │   a       quick move
░║   ▄█▄   ║░  │   ▀ ▀   │   │   ▄▀▄   │                   │   +/-     run size
░║  ▄█▀█▄  ║░  │         │   │   █▀█   │                   │ ► s       turn stock
░║    █    ║░  │   ▄█▄   │   │   ▀ ▀   │                   │   u/r     undo / redo
░╚═══════7♣╝░  │  ▄█▀█▄  │   │         │                   │ ► h       hint
░░░░░░░░░░░░░  │    █    │   │   ▄█▄   │                   │   c       copy seed
               └───────K♣┘   │  ▄█▀█▄  │                   │ ► i       how to play
                             │    █    │                   │ ► ?       all keys
                             └───────A♣┘                   │
                                                           │ Mouse
                                                           │ Click: pick up, drop
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| The three-row rank is blurry: `7` is `▀▀█ / █ / ▀`, `K` is `█ █ / █▀▄ / ▀ ▀`, `A` is `▄▀▄ / █▀█ / ▀ ▀`. The Polish `D` (`██▄ / █ █ / ▀▀▀`) differs from `0` by one pixel column, `W` from `H` by one pixel row (`rank_font.rs` bitmaps `LETTER_D` vs `ZERO`, `LETTER_W`). `10` is nine cells of half blocks. | P3-13 pixel font, 3 × 5 pixels in 3 rows of half blocks. The ASCII form (`^ _ #`) is worse: `#^_ / ^ ^` for `K`. | Remove `rank_font.rs`. The centre row reads `K ♣` in bold (rank text, one space, suit symbol) with the pictogram beneath. See §4. | High / small (deletion) |
| The panel has one edge and no title bar; its first line `Guide (g: hide)` looks like text. It carries `Turned over: 0` that the `Now:` row also shows. | `guide.rs::draw` paints a left edge only; `view.rs` passes the counter to both. | Frame it (`┌─ Guide ──┐`, WGL4 light box, ASCII `+- Guide -+`), one line of padding, a `Game` box on top with the counters, and remove the counter from `Now:`. | High / small |
| The footer is centred under the board and the panel together; the `Now:`/`Possible:` rows run under the panel too. | `chrome_with` splits only the middle band. | The side panel spans header to bottom (full height); the controls strip is as wide as the board only. | Medium / small |
| Hint marker, selection marker and the focus marker are still one cell at 11 × 9. | As in 2.1. | As in 2.1; at 11 × 9 and 13 × 11 the top border also shows the word `TARGET`/`HINT`? No: words need localization and width. Use the two-cell markers and the border style; the panel explains them once in a legend row (`►` focus, `●` picked, `+` target, `?` hint). | Medium / small |

### 2.3 Board, large size (200 × 60, 13 × 11 cards)

Evidence, en-US Unicode, first two columns:

```text
░╔7♣════════►╗░  ┌───────────┐
░║    ▀▀█    ║░  │▒▒▒▒▒▒▒▒▒▒▒│
░║     █     ║░  ┌K♣─────────┐
░║     ▀     ║░  │    █ █    │
░║           ║░  │    █▀▄    │
░║   ▄███▄   ║░  │    ▀ ▀    │
░║  ▄▄█▀█▄▄  ║░  │           │
░║  ███▀███  ║░  │   ▄███▄   │
░║    ▄█▄    ║░  │  ▄▄█▀█▄▄  │
░║           ║░  │  ███▀███  │
░╚═════════7♣╝░  │    ▄█▄    │
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| The rank is the same 3-row glyph as at 11 × 9, now surrounded by more empty space; the 7 × 4 pictogram is good. | `face.rs::scale_for` picks `Scale::Large`, the rank is always `rank_font::HEIGHT` rows. | §4: plain bold text; the free rows go to padding so the pictogram sits in the lower half, the text in the upper half. | High / small |
| The board uses 36 of 55 body rows and 140 of 170 columns; the rest is table. | `size.rs::SIZES` stops at 13 × 11; `spacing` caps the gap at 3. | Add a 15 × 13 size (large pictogram 9 × 5) for ≥ 180 columns and ≥ 56 rows; let `MAX_GAP` grow to 5 at that class. Optional; the owner's zoom (P3-16) lets the player pick. | Low / medium |
| The panel is 30 columns at every size; at 200 columns the key names are still abbreviated (`u/r undo / redo`). | `GUIDE_WIDTH` constant. | 30 columns at 130–179, 40 at ≥ 180 with full labels and the legend. | Low / small |

### 2.4 Board, ASCII mode

Evidence, Klondike 100 × 30 carrying (`A♣` picked, four foundation targets):

```text
+-----+   +-----+            .+~~~%?+. .+~~~~%+. .+~~~~%+. .+~~~~%+.
|#####|   |     |            .!     !. .!     !. .!     !. .!     !.
+---24+   +-----+            .+~~~~~+. .+~~~~~+. .+~~~~~+. .+~~~~~+.
                             ......... ......... ......... .........
+7C---+   +-----+   +-----+   +-----+
|     |   +KC---+  =+-----+=  +-----+
|  C  |   |     |  =*AC*>?*=  +-----+
|     |   |  C  |  =*     *=  +5C---+
+---7C+   |     |  =*  C  *=  |     |
          +---KC+  =*     *=  |  C  |
                   =****AC*=  |     |
                   =========  +---5C+
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| ASCII target borders `~` and `!` and the hint border `.`/`:` are the most legible marks in the whole review; the Unicode set has no equivalent (target and hint use the normal border). | `frame.rs::UNICODE_EDGES` reuses the normal set for `target` and `hint`. | Give the Unicode target the double-line set is taken by focus; use `╒═╕`/`╘═╛` for targets (WGL4 U+2552…), keep the focus `╔═╗` and the selection `╒`? No: selection and target must differ. Proposal: focus `╔═╗║`, selection `█` edge cells (`▐`,`▌` U+2590/258C are WGL4) with `═`, target `┌┐` corners with `╌`? Not WGL4. Final: target border = normal set drawn in the Target token plus the `+` marker in both top corners and a filled `+ + +` bottom row on empty slots; hint = `?` in both top corners. | Medium / small |
| ASCII suits are capital letters: with the Polish ranks `D` (dama) and the suit `D` (diamonds) the queen of diamonds is `DD`; `DH`, `DC`, `DS` read as two letters of equal weight. | `glyphs.rs::ASCII.suits = ["C","D","H","S"]`, `locales/pl-PL/board.toml [rank] queen = "D"`. | Lowercase ASCII suits: `c d h s` (`Dd`, `10h`, `Kc`). Rank and suit then differ in case in both locales, and no rank letter is a suit letter. | High (Polish ASCII) / tiny |
| The ASCII pixel rank (`#^_`) is unreadable. | `rank_font.rs::ASCII_HALVES`. | Removed with the pixel font. | High / none |
| The ASCII pictograms are good (`/###\`, `## ##`) and stay. | | | |

### 2.5 Board, Polish

Evidence, 100 × 30:

```text
                 Kolumna 1: siódemka trefl
Teraz: Przeglądasz Kolumna 1, siódemka trefl | Przełożono: 0
Możliwe: s: dobierz z talii | Strzałki: ruch | h: podpowiedź | ?: klawisze | i: jak grać
    Enter Wybierz   u Cofnij   r Ponów   h Podpowiedź   s Talia   p Pauza   ? Pomoc   Esc Wstecz
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| `Przeglądasz Kolumna 1` and `Trzymasz … z Talia` are ungrammatical (nominative after a verb that needs the accusative or genitive). | `board.action.browsing = "Przeglądasz {pile}, {what}"` injects `pile_name` (nominative). | Change the templates in both locales to the colon form the announcer already uses: `Kolumna 1: siódemka trefl`, `Trzymasz: siódemka trefl (z: Kolumna 1), cele: 2`. Rule for all locales: a pile name is never the object of a verb; it stands after a colon or in parentheses. | High for pl-PL / tiny |
| `Enter Wybierz` (Select) on the board, where Enter picks up or drops. | `help.action.select` is one label for every screen. | Board-specific label keys for `Select` and `Back` in the `Do:` row (already there: `weź`, `połóż`, `odłóż`); the footer goes away. | Low / none after P3-15 |
| Help key names are English (`Up / k`, `Down / j`, `Left`, `Right`, `Space`). | `KeySpec::display` is not localized. | Localize the key names of the key table (`Góra`, `Dół`, `Lewo`, `Prawo`, `Spacja`) through `help.key.*`. | Medium / small |

### 2.6 Main menu, game selection, seed entry

Evidence, 100 × 30, en-US:

```text
                                       ► New game           ◄
                                         New game from seed
                                         Statistics
…
                                            [ Undo: on ]
                       u or a click switches undo on or off for the next game
…
                                      [ _                     ]
                                       [ Continue ]   [ Back ]
                 Digits type the seed   Backspace Delete   Enter Continue   Esc Back
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| The focus markers frame the width of the longest label, so `►` and `◄` are 11 cells apart from `New game`; the focused row is the only one with a background, which a 16-color terminal shows as a yellow bar. | `menu_list.rs::draw` pads every label to `inner`. | Marker directly before the label (`► New game`), the whole row in the focus style; the `◄` is not needed (one marker reads as well and matches dialogs' buttons on the left side). Keep `◄` only in dialogs where buttons sit in a row. | Medium / tiny |
| The menu has no title and no hint what the game is; the logo is 48 columns of block art. | `main_menu.rs::draw_items`. | Add the tagline (`common.app.tagline`) under the logo and the version in the bottom-right corner; a `Resume` row shows the saved game (`Resume: Klondike, Draw 3, 12:34, Moves 42`). | Low / small |
| The undo toggle `[ Undo: on ]` is drawn in the focus style though it is not focused; it cannot be reached by Tab. | `game_select.rs::draw_toggle` uses `theme.focus()`. | A button style (`[ Undo: on ]` in Accent on Background, reversed when focused), part of the Tab ring (§5). | Medium / tiny |
| Seed entry: `Continue` and `Back` buttons are not keyboard-focusable; the key line explains it instead. | `seed.rs` has no focus state. | Tab ring: field → Continue → Back. | Medium / small |
| Eight variants in one flat list; a newcomer does not know what `Draw 3` or `2 suits` means. | `game_select.rs`. | A one-line description under the list for the focused variant (from `rules.toml` Goal) and a difficulty word (`easy`, `medium`, `hard`). | Medium / small |

### 2.7 Settings

Evidence, 100 × 30:

```text
                                   ► Language: English          ◄
                                     Theme: Dark
                                     Front style: international
                                     Back style: default
                                     Symbols: Unicode
                                     Colors: Auto
                                     Sound style: Default
                                     Sound cues: On
                                     Music: Off
                                     Volume: 70%
                                     Guide panel: On
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| Values are not aligned, there are no groups, and nothing says that Left/Right change a value (only Enter cycles forward). | `settings.rs::draw`, `options.rs::line`. | Two columns (`Language ……… ◄ English ►`), groups `Display`, `Sound`, `Board`, and a description row for the focused option. New rows from this review: `Card size: Auto` (P3-16), `Aura: Full / Focus only` (P3-21), `Full screen: Off` (O-16). The `Guide panel` row says `On (needs 130 columns)` when the terminal is narrower (P3-18). | Medium / small |
| `Front style: international`, `Back style: default` show folder ids, not names. | `value_text` returns the id. | Names from the style manifests (`style.toml`), as the sound styles do. | Low / small |
| `Symbols: Font` is offered but renders as Unicode (`Glyphs::new` maps Font to Unicode). | `glyphs.rs`. | Hide the `Font` value until P5-02 exists. | Low / tiny |

### 2.8 Statistics

Evidence, 100 × 30:

```text
  Game                        Played  Won  Win rate  Best time  Fewest moves  Streak  Best streak
  Klondike, Draw 1 (undo)          1    1      100%       1:35           120       1            1
…
                          History, newest first, times in UTC   Game 1 of 9
      Date (UTC)        Game              Seed                  Result           Time   Moves
    ► 1970-01-01 00:20  FreeCell          777                   In progress         -       - ◄
…
                         [ Play this seed again ]   [ Copy seed ]   [ Back ]
                Up Down Select   Enter Play this seed again   c Copy seed   Esc Back
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| The Polish summary table is 100 columns wide and starts at column 0 (`Gra … Maks. seria`), the history list is 97 columns. At 100 × 30 nothing is left for margins; narrower labels are needed. | `stats/format.rs` fixed column widths. | Shorter Polish headings (`Serie` for streak columns), hide `Best streak` below 110 columns, and the seed column shrinks to 20 digits. | Medium / small |
| The buttons are mouse-only; keys are `Enter`, `c`, `Esc`. | `stats.rs::handle`. | Tab ring (§5). | Medium / small |
| Times are UTC; the owner plays locally. | No time-zone dependency (P4-04 decision). | Keep, but say `UTC` once in the heading only (already) and drop it from the column header. | Low / tiny |

### 2.9 Help window (Keys, Rules, Command line)

The window is the best-designed part of the interface: tabs are marked by brackets and reverse
video, the key table is generated from `KEY_TABLE`, the text scrolls with a position indicator.
Findings:

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| The Keys tab lists keys by group, not by task; a newcomer looks for "how do I move a card". | `help.rs::sections`. | Put the `Rules` tab's `Controls` section first on the Keys tab (`Pick up: Enter / Space`, `Drop: Enter`, …) and keep the groups below. | Low / small |
| Tab switches the help tabs; after P3-17 Tab is the focus key everywhere else. | `overlay/mod.rs::tab_target`. | Keep: inside the help window the tabs are the focusable elements, so Tab cycling them is the same rule. Left/Right switch tabs on Keys and Command line; on Rules they switch games, which stays. | None |
| The `Close` button is always focused; Esc and Enter close; fine. | | | |
| Zoom and full screen are not documented. | | A `Display` section on the Keys tab: card size keys, the terminal's own zoom (Ctrl+wheel, Ctrl+Plus/Minus, F11, Alt+Enter) per terminal, full screen (O-16). | Medium / small |

### 2.10 Dialogs: pause, confirm, win, alert

Evidence, 100 × 30:

```text
╔ Please confirm ═══════════════════════╗        ╔ You won ═════════════════╗
║ Give up this game and deal a new one? ║        ║    The game is solved.   ║
║                                       ║        ║        Time: 5:12        ║
║              Yes    ► No ◄            ║        ║        Moves: 134        ║
╚═══════════════════════════════════════╝        ║  ► New game ◄    Menu    ║
                                                 ╚══════════════════════════╝
```

| Problem | Cause | Fix | Impact / effort |
|---|---|---|---|
| Buttons move with Left/Right/Up/Down only; Tab does nothing. | `overlay/mod.rs::handle_for`. | Tab/Shift+Tab cycle the buttons too (§5). | Medium / tiny |
| The win dialog does not say whether the time or move count is a personal best, and offers no `Play this seed again`. | `win.rs`. | Add `Best time for Klondike, Draw 1!` when the statistics say so, and a third button `Same seed`. | Low / small |
| The pause dialog covers the board but the header still shows the running clock value (stopped) and the bottom rows show the last event; fine. The win dialog leaves the stale `Possible: n: new game …` row and the footer under it. | | With P3-15 the `Do:` row under a modal shows the dialog's keys. | Low / tiny |

### 2.11 Too-small notice (80 × 24)

```text
                               Terminal too small
                              Current size: 80 x 24
                             Required size: 100 x 30
                   Enlarge the window or reduce the font size.
```

Correct and sufficient. Two refinements: name the terminal's zoom-out key when it is known
(`Ctrl+Minus` in Windows Terminal and GNOME Terminal, `Cmd+Minus` on macOS), and allow the help
window (`?`) so the Command line tab can be read even here. Lowering the minimum below 100 × 30 is
not recommended: a 7 × 5 Klondike needs 69 columns and 25 rows for the board alone, and FreeCell
needs 8 columns of cards (71 columns); 80 × 24 would force 5 × 4 cards with no interior, which
fails the readability priority.

## 3. Recommended layout per screen-size class

Classes are decided from the terminal size at every draw (`Input::Resize` already reaches the
board). The guide panel setting and the card-size setting (P3-16) only narrow the choice.

| Class | Columns | Rows | Card size (auto) | Side panel | Chrome rows |
|---|---|---|---|---|---|
| Compact | 100–129 | 30–35 | 7 × 5 (9 × 7 when rows ≥ 34) | none; `g` opens the guide as a window | 1 header + 2 controls |
| Standard | 130–179 | 36–49 | 9 × 7 or 11 × 9 | 30 columns, framed, full height | 0 header (game box in the panel) + 2 controls |
| Wide | ≥ 180 | ≥ 50 | 11 × 9 or 13 × 11 | 40 columns, framed, full height | same as Standard |
| Large | ≥ 200 | ≥ 60 | 13 × 11 (15 × 13 if added) | 40 columns | same as Standard |

Rows ≥ 36 with fewer than 130 columns (a tall narrow window) is Compact; columns ≥ 130 with fewer
than 36 rows (a wide short window) is Standard with 9 × 7 cards.

### 3.1 Areas and what goes where

```text
Compact (100 × 30)                          Standard and up (130 × 40)
┌──────────────────────────────────────┐    ┌──────────────────────────┬─ Guide ───────────┐
│ Klondike, Draw 3 · seed 42 · 12:34   │    │                          │ Game              │
│ Moves 2 · Turned over 0 · No undo    │    │                          │ Klondike, Draw 3  │
├──────────────────────────────────────┤    │                          │ seed 42 (c: copy) │
│                                      │    │                          │ time 12:34        │
│               board                  │    │          board           │ moves 2           │
│                                      │    │                          │ turned over 0     │
│                                      │    │                          │ undo on           │
│                                      │    │                          │ best 5:12         │
├──────────────────────────────────────┤    │                          ├───────────────────┤
│ Now: Tableau 3: ace of clubs    Dealt│    │                          │ Goal …            │
│ Do: Enter pick up · a to foundation …│    │                          │ Keys … (marked)   │
└──────────────────────────────────────┘    │                          │ Mouse …           │
                                            ├──────────────────────────┤ Legend ► ● + ?    │
                                            │ Now: …            Dealt  │                   │
                                            │ Do: …                    │                   │
                                            └──────────────────────────┴───────────────────┘
```

- **Game information** (one place): variant, seed (click or `c` copies; underlined), time,
  moves, the stock counter, the undo badge, the best time of this variant and mode from the
  statistics. Compact: the header row, in one line; when it does not fit, the best time and then
  the counter are dropped first. Standard and up: the `Game` box at the top of the panel; the
  header row is removed and the board gains it.
- **Board**: the body between. Always at least 100 columns wide; the panel never shrinks it below
  that (`BOARD_MIN_WIDTH` stays).
- **Controls** (one place, two rows under the board, as wide as the board):
  `Now:` = what is focused or carried, in the colon form (`Tableau 3: ace of clubs`,
  `Carrying: ace of clubs (from Tableau 3), 4 targets`, `Hint: ace of clubs to Foundation 1`,
  `Game won`, `No legal moves`), and at the right end of the same row the last event in the
  Success or Error token (`Dealt from the stock.`, `Illegal move: the cards cannot go there.`),
  cleared by the next action. `Do:` = the actions of this exact state with their keys, dropping
  parts from the right as today, always ending with `? keys` and `Esc menu`.
- **Guide panel** (Standard and up): framed, title in the top border, sections `Game`, `Goal`,
  `Keys` (usable ones marked, as today), `Mouse`, `Legend`. It never repeats a fact that is in
  the controls strip. Below 130 columns (P3-18): the setting stays on, `g` opens the same
  content as a window over the board (reusing `dialog::boxed`), the `Do:` row shows `g: guide`,
  and the settings row reads `Guide panel: On (needs 130 columns, now 100)`.
- **Footer and announcer row**: removed on the board. Other screens keep their one-row footer.
- **Modals**: unchanged (centred boxes); `Do:` under an open modal shows the modal's keys.

How the board scales inside its body: the card size is the largest of `SIZES` that fits (as
today), the gap grows to 3 (5 at Wide), the top row and the tableau are centred horizontally,
the tableau fan uses the natural step and may grow it by one when the body has spare rows (a
new `fan` plan before shrinking), and the pyramid is centred vertically (as today).

## 4. Card-face recommendation per size

Principles: the rank is text, never a picture; the suit is a symbol next to the rank and a
pictogram in the middle; every mark that depends on state has a symbol, a border style and a
color, so any one of the three can fail; everything is WGL4 or ASCII.

### 4.1 Faces

| Size | Interior | Top-left and bottom-right corners | Centre | Pictogram |
|---|---|---|---|---|
| 7 × 5 | 5 × 3 | `10♥` top-left (as today), mirrored bottom-right | row 2: suit symbol centred (as today) | none |
| 9 × 7 | 7 × 5 | same | row 1: `K ♣` bold, centred | rows 3–4: Small 5 × 2 |
| 11 × 9 | 9 × 7 | same | row 1: `K ♣` bold | rows 3–5: Medium 5 × 3 |
| 13 × 11 | 11 × 9 | same | row 1: `K  ♣` bold (two spaces) | rows 4–7: Large 7 × 4 |
| 15 × 13 (new, optional) | 13 × 11 | same | row 1: `K  ♣` bold | rows 4–8: a 9 × 5 set to draw |

What replaces the block-art rank: the one-row text `K ♣`. On the three larger sizes it is
drawn with `Modifier::BOLD` in the suit color; it uses the same localized label as the corner,
so Polish shows `W`, `D`, `K`. A face card gets no picture (no WGL4 drawing reads as a jack).
The pictograms of `art.rs` stay unchanged in both symbol sets; they read well at every scale.

Why not a bigger text: a 2-row or 3-row glyph font from box-drawing or half blocks is what
P3-13 tried; at the cell resolution of a terminal, letters with a single distinguishing pixel
(`D`/`0`, `W`/`H`, `8`/`B`) cannot be told apart. Reading three repetitions of a plain glyph
(two corners plus the centre) is faster than decoding one blurry picture. The owner's note that
the small face "was hard to notice" is answered by bold type, the empty row around the text and
the pictogram, not by size.

Test for P3-14: for every size, locale and glyph mode, the rendered rows of two different cards
differ, and the corner label plus the centre row of every card contain the rank label and the
suit symbol as text.

### 4.2 Glyph choices

| Element | Unicode (WGL4) | ASCII | Note |
|---|---|---|---|
| Suits | `♣ ♦ ♥ ♠` (U+2663, 2666, 2665, 2660) | `c d h s` lowercase | Lowercase separates the suit from the Polish rank `D`; uppercase `C D H S` today gives `DD` for the queen of diamonds |
| Ranks | `A 2 … 10 J Q K` / `A 2 … 10 W D K` | same | never a pixel font |
| Face-down fill | `▒` (U+2592) | `#` | also in the top border row of a covered card (`┌▒▒▒▒▒┐`) |
| Empty foundation | muted `A` centred | `A` | placeholder, Slot token |
| Empty stock, recyclable | `↑` (U+2191) | `^` | `x` / `x` when no pass is left |
| Focus | border `╔═╗║╚═╝`, marker `►` (U+25BA) | `@`/`=` border, `>` | as today |
| Picked run | border `╒═╕│╘═╛`, marker `●` (U+25CF) | `*` border and marker | as today; the carry halo `▓`/`=` stays |
| Target | normal border in the Target token, `+` in both top corners, and on an empty slot a centred `+` | `~`/`!` border (today), `%` → `+` | one symbol for targets in both sets |
| Hint | `?→` on the source, `→?` on the target (U+2192), hint border token | `?>` / `>?` | both cards marked |
| Halos | focus `░`, carry `▓`, target `·` | `:` `=` `.` | the target ring is weak at 7 × 5; the slot marker above carries it |

Color independence: red suits keep the symbol and the pictogram shape; focus, selection,
target and hint each have a distinct border set or marker in both symbol sets. The test
`the_three_auras_differ_in_both_sets` extends to the four card marks.

### 4.3 The Polish ranks

`W` and `D` are plain text in this design, so the owner's `W`/`H` and `D`/`0` confusions
disappear with the pixel font. The ASCII suit change (`Dd`, `Wh`) is the only other Polish-specific
item. A test renders every rank of every suit in both locales and both modes at every size and
asserts that no two differ only in pictogram rows.

### 4.4 Marks for a newcomer

What the player must see at once, and how each is carried:

| Question | Carrier (symbol / border / color) | Text |
|---|---|---|
| Where am I? | `►` marker, double border, focus halo `░`, Yellow | `Now: Tableau 3: ace of clubs` |
| What did I pick up? | `●` marker, `╒═╕` border, carry halo `▓`, Cyan | `Now: Carrying: ace of clubs …, 4 targets` |
| Where can it go? | `+` in the top corners of every target, `+` centred in empty target slots, target ring `·`, Yellow | `… 4 targets`, `Do: Enter: drop` |
| What does the hint say? | `?→` / `→?`, hint border, Orange | `Now: Hint: ace of clubs to Foundation 1` |
| What do I do next? | | `Do:` row; the panel marks the usable keys with `►` and bold |

## 5. Keyboard model

### 5.1 Tab focus order (P3-17)

Rule: every screen and dialog owns a focus ring in reading order (top to bottom, left to right).
Tab moves forward, Shift+Tab backward, both wrap; Enter and Space activate the focused element;
Up/Down keep their current meaning inside a list, Left/Right inside a value or a button row. The
focused element shows `►` and reverse video. Rings:

| Screen or dialog | Ring |
|---|---|
| Main menu | the items (Tab = Down, Shift+Tab = Up) |
| Game selection | the variants list (as one element; Up/Down inside it) → `[ Undo: on ]` |
| Seed entry | the field → `[ Continue ]` → `[ Back ]` |
| Settings | the rows (Tab = next row; Left/Right change the value; Enter cycles forward) |
| Statistics | the history list → `[ Play this seed again ]` → `[ Copy seed ]` → `[ Back ]` |
| Board | no ring: Tab cycles the piles in order (Stock, Waste, Foundations, Cells, Tableau 1…n), Shift+Tab backwards; a cheap way to reach a pile when the geometric arrows stall |
| Confirm, Win, Alert, Pause | the buttons (Tab = next, Shift+Tab = previous; Left/Right keep working) |
| Help window | the three top tabs (as today); on Rules, Left/Right switch games and Tab still switches the top tabs |

`Tab` and `BackTab` become `Action::FocusNext` and `Action::FocusPrevious` in `KEY_TABLE`
(group Navigation) so Help lists them. In the seed field Tab is not text (`is_text_key` excludes
it already, Tab is not `KeyCode::Char`).

### 5.2 Zoom keys (P3-16)

Facts about the terminals in scope:

- Windows Terminal, GNOME Terminal, Konsole, iTerm2, Terminal.app, WezTerm and Alacritty all
  bind Ctrl+Plus / Ctrl+Minus / Ctrl+0 (Cmd on macOS) and Ctrl+mouse wheel to their own font
  zoom. Those keys never reach the game. Binding them in the game would work only in the classic
  Windows console, where nothing intercepts them.
- The classic Windows console has no zoom key at all; Ctrl+wheel does nothing. Its font can be
  changed by the program through `GetCurrentConsoleFontEx` / `SetCurrentConsoleFontEx`
  (`windows-sys`, feature `Win32_System_Console`, one `unsafe` block each).
- `+` and `-` are bound today to `RunMore` / `RunLess`, which Up and Down already perform
  inside a column (`cursor.rs::deeper` / `shallower` are what `Action::Up`/`Down` call first).
- `=` is the unshifted `+` on US and Polish keyboards.

Decision:

| Key | Action | Where |
|---|---|---|
| `+`, `=` | `ZoomIn`: next larger card size that fits; at the largest, nothing (status: `Largest size that fits`) | board |
| `-` | `ZoomOut`: next smaller size; the smallest is 7 × 5 | board |
| `0` | `ZoomAuto`: back to the largest that fits (the default) | board |
| `Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0` | the same three, and in the classic Windows console additionally the console font: +2 / −2 pixels / the font at start | only reaches the game in the classic console; documented as the terminal's own zoom elsewhere |
| `Shift+Up`, `Shift+Down` | `RunMore` / `RunLess` (moved from `+`/`-`); Up/Down keep doing it inside a column | board |
| Ctrl+wheel | `ZoomIn`/`ZoomOut` when the terminal delivers it (`MouseEventKind::ScrollUp` with `CONTROL`) | board |

The card size is a setting `card_size: Auto | Small | Medium | Large | Huge` (serde default
`Auto`), shown in Settings as `Card size`, changed by the keys and remembered. A size larger than
what fits falls back to the largest that fits and the header says `(auto)`. The Help window's
Keys tab gets a `Display` section that names the keys above and, under it, the terminal's own
zoom: `Windows Terminal, Linux: Ctrl+Plus, Ctrl+Minus, Ctrl+0, Ctrl+mouse wheel · macOS: Cmd+Plus,
Cmd+Minus, Cmd+0 · Classic Windows console: the keys above change the font`. The guide panel's
Keys section lists `+/-/0 card size`.

Conflicts checked: `0` is free; `=` is free; `Shift+Up`/`Shift+Down` are free (the key table
ignores Shift only for character keys, `KeySpec::matches`). The settings screen keeps Left/Right
for values; the statistics screen's `c` stays.

### 5.3 Other key notes

- `R` (Shift+r) restarts while `r` redoes: a slip of Shift restarts a game behind a confirm
  dialog, which is fine, but Help should show `Shift+R`, not `R`.
- `Esc` on the board opens the menu after a confirm; a newcomer expects Esc to cancel the pick
  first (it does) and then to pause. Keep, and show `Esc menu` in `Do:`.
- `m` (menu) and `Esc` do the same; `q` quits with a confirm; `Ctrl+C` quits at once. Fine.

## 6. Colour and contrast notes per theme

The contrast test (`theme/tests.rs`) checks `TEXT_PAIRS` only. The pairs that carry the marks
are not in it: `Target`, `Selection`, `Hint`, `BorderFocus`, `Slot` on `Table`, and `Border` on
`Surface`. Add them at a 3.0 target (graphical objects), and add `Error` and `Success` on
`Background` (they will be used in the controls strip).

| Theme | True color | 16 colors (`ansi16` column) | Notes |
|---|---|---|---|
| Dark | Text 232,236,244 on 24,28,38 (≈ 15:1); table 24,92,58 with white text; focus halo 255,232,130 on the table (≈ 6:1); hint 255,170,60 on the table (≈ 4.5:1). Good. | `border` = `DarkGray` on `Black` (invisible on most 16-color palettes); `border_focus`, `focus_bg`, `target`, `aura_focus` = `Yellow`, which the classic Windows console and xterm render as dark olive (128,128,0) on the green table; `selection`/`aura_carry` = `LightCyan` (fine); `table` = `Green` (0,128,0) with `White` text (fine). | Use `Gray` for `border`, `LightYellow` for the four focus/target tokens, `LightRed` for `card_red` only if `Red` is dark on the terminal (it is not on Windows: keep `Red`). |
| Light | Text 24,28,36 on 244,246,248; accent 18,76,160; table 32,114,70 with 245,252,248 text; `slot` 130,190,155 on the table (≈ 2:1, faint). | `background` = `White`, `text` = `Black`, `focus_bg` = `Blue` with `White` text (fine); `slot` = `LightGreen` on `Green` (≈ 2:1). | Raise `slot` to 200,230,210 (≈ 3.5:1) and `White` in 16 colors; the placeholder `A` in empty foundations makes slots readable without contrast. |
| High contrast | Pure black everywhere, white text; table black. The board, the header and the controls strip have the same background, so the board has no edge. | `table` = `Black`, `background` = `Black`; `card_black` on `card_face` = `Black` on `White` (fine). | Draw a one-cell `Border` line around the table in this theme (a `Block::bordered()` on the body when `theme.id() == HighContrast`), and set `surface` to 16,16,16 so dialogs stand out at true color. |
| All themes, 256 colors | `rgb_to_ansi256` nearest-cube; the dark background 24,28,38 maps to 233/234 (fine). | | The Dark table 24,92,58 maps to 22 (0,95,0): darker than intended; the focus halo stays readable. No change needed. |

Dark theme at 16 colors is the case the owner will meet first in the classic console (`detect`
reports true color on Windows, but a player who starts with `--colors 16` or a `TERM=xterm` SSH
session gets this column).

Color is never the only carrier: every mark has a symbol and a border style (§4.2), the
pictograms differ in shape, the focused list item has `►`, the dialogs' buttons have `► ◄`.

## 7. Prioritized implementation order

1. **P3-14 Readable card faces** (Sonnet): delete the pixel font, text centre, face-down border
   row, lowercase ASCII suits, placeholders in empty slots, two-cell hint marks, target markers.
   Highest readability gain, mostly deletion, no layout change.
2. **P3-15 Screen layout** (Sonnet): the size classes, the two-row controls strip, the game
   information place, the framed panel with the `Game` box, footer and announcer row removed
   on the board, the colon-form `Now:` templates (which also fixes the Polish grammar of §2.5,
   so P3-22 only covers the remaining strings).
3. **P3-18 Guide below 130 columns** (Sonnet, after P3-15): the panel as a window, the settings
   row text, `g: guide` in `Do:`.
4. **P3-17 Tab focus** (Haiku): focus rings, `FocusNext`/`FocusPrevious` in the key table, the
   marker next to the label.
5. **P3-16 Zoom** (Sonnet): `card_size` setting, the keys, the `Display` help section, the
   classic-console font change behind a decision for `windows-sys`.
6. **P3-21 Aura options and reshuffle** (Haiku): `aura: Full | FocusOnly`, reshuffle key.
7. **P3-23 Colors** (Haiku): the 16-color swatches, the mark pairs in the contrast test, the
   High Contrast table border.
8. **P3-22 Localization templates and key names** (Haiku): the remaining pile-name templates,
   localized key names, `Row 7, card 1`.
9. **P3-24 Menus, settings and statistics polish** (Haiku): aligned values, groups, descriptions,
   the toggle button style, style names, narrower Polish statistics.
10. **O-16 Full screen** (Sonnet): last; it changes no readability, and the zoom work settles
    what "larger" means first.

P3-19 (auto-finish) and P3-20 (winnable deals) are unaffected by this review except that their
options join the `Board` group of the settings screen.

## 8. Decisions recorded for the cards

- D-U04-1: no multi-row rank glyphs in any mode; rank and suit are text in the corners and in
  the centre row; pictograms stay.
- D-U04-2: one game-information place and one controls place per size class (§3.1); the
  announcer row and the footer leave the board.
- D-U04-3: the guide panel is framed and never repeats the controls strip; below 130 columns it
  is a window on `g`.
- D-U04-4: `+`/`=`/`-`/`0` are zoom; run size moves to Shift+Up/Down; Ctrl+Plus/Minus/0 are
  documented as the terminal's own zoom and act only where they arrive (the classic console).
- D-U04-5: Tab/Shift+Tab move a focus ring on every screen and dialog; the help window's tabs
  are that ring.
- D-U04-6: ASCII suits are lowercase.
- D-U04-7: aura setting `Full` (default) or `Focus only`; targets keep their `+` markers in
  both.
- D-U04-8: full screen never relaunches the game; it asks the terminal where an API exists and
  otherwise names the terminal's own key once.
