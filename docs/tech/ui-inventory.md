# UI inventory: colors, sizes and fonts

Reference for the owner and for the backlog cards on themes, interface size, table and card colors,
and low-vision and color-blind variants. It lists what the code does today. It changes no behavior.

**Method.** Every fact was read from the code at `origin/main` 70aa859. Each entry gives its source.
Paths without a prefix are relative to `crates/tui/src/`. `path:N` is a line number. The word
**unknown** marks what the code does not decide. Contrast ratios in section 1.4 are **computed**: they
apply the WCAG 2 formula of `theme/contrast.rs:3-29` to the RGB values listed in section 1.3. No test
produces them. Values for the 16-color fallbacks depend on the player's terminal palette and are
**unknown** here.

**Counts.** 24 color tokens (`theme/palette.rs:10-59`), 3 themes (`theme/mod.rs:21-29`), 4 card sizes
(`layout/size.rs:11-16`), 3 screen size classes (`layout/chrome.rs:104-108`), 3 glyph modes
(`settings.rs:14-21`), 4 color modes incl. Auto (`settings.rs:23-31`), 12 settings rows
(`screens/settings/options.rs:29-42`).

---

## 1. Colors (theme tokens)

### 1.1 Mechanism

- A `Token` is a semantic slot. Components ask for tokens, never for colors. `theme/palette.rs:8-59`.
  `Token::ALL` lists all 24 at `theme/palette.rs:63-88`.
- A `Swatch { rgb, ansi16 }` holds one token's color. `swatch(r, g, b, Color::X)` is the constructor
  (`theme/palette.rs:94-105`). Every palette file is a list of `swatch(...)` calls.
- `Swatch::resolve` picks the emitted color by depth (`theme/palette.rs:109-116`):
  - TrueColor → `Color::Rgb(r, g, b)`;
  - Ansi256 → `Color::Indexed(rgb_to_ansi256(r, g, b))` (`theme/ansi.rs:17-45`);
  - Ansi16 → the named `ansi16` color.
- `theme/ansi.rs` is **not** a theme. It holds the RGB→256-color converter and has no `swatch(...)`.
- Helpers on `Theme`: `style(fg, bg)` (`theme/mod.rs:78-80`), `base()` = Text on Background
  (`theme/mod.rs:83-85`), `surface()` = Text on Surface (`theme/mod.rs:88-90`), `focus()` = FocusFg on
  FocusBg (`theme/mod.rs:93-95`).
- Themes: `ThemeId::ALL = [Dark, Light, HighContrast]` (`theme/mod.rs:29`). No other theme exists.
  `palette()` dispatch is at `theme/mod.rs:32-38`.
- Color depth: `Settings::color_depth` uses the forced `ColorMode` or the detected `caps.color`
  (`settings.rs:97-104`). Detection (`caps.rs:40-48`): **on Windows the depth is always TrueColor**
  (`caps.rs:41`). Elsewhere, `COLORTERM` truecolor/24bit gives TrueColor, `TERM` containing
  `256color` gives Ansi256, otherwise Ansi16 (`caps.rs:50-67`). The code never queries the host
  terminal, so whether a Windows host really renders TrueColor is **unknown**.
- Forced color modes: `ColorMode { Auto, Ansi16, Ansi256, TrueColor }` (`settings.rs:23-31`, offered at
  `screens/settings/options.rs:57-62`).

### 1.2 The tokens: purpose, class, and where they are used

Class: **Meaning** = the color tells a player something the glyphs also tell; **Text** = words;
**Structure** = frames, panels, fills; **Highlight** = titles and button labels. Use-sites are
`file:line` under `crates/tui/src/`.

| # | Token | Purpose (palette doc, `theme/palette.rs`) | Class | Used in |
|---|---|---|---|---|
| 1 | `Background` | Screen background (`:11`) | Structure | `app.rs:393`; `theme/mod.rs:84`; `screens/board/view.rs:69`; `screens/focus.rs:42`; `screens/game_select.rs:192`; `screens/game_select/toggles.rs:64`; `screens/main_menu.rs:249`; `screens/main_menu/seed.rs:215,224,232`; `screens/settings.rs:46,99`; `screens/stats/view.rs:99,128,146,165,171,245`; `widgets/footer.rs:160`; `widgets/too_small.rs:36,53`; `widgets/overlay/mod.rs:106` |
| 2 | `Surface` | Dialog and panel background (`:13`) | Structure | `theme/mod.rs:89`; `screens/board/controls.rs:44,49,61,75`; `screens/board/guide_view.rs:20`; `widgets/guide/keys.rs:133,137,151,173`; `widgets/guide/legend.rs:25`; `widgets/guide/mod.rs:79,178,182,185,190`; `widgets/overlay/dialog.rs:188-224`; `widgets/overlay/guide_window.rs:104`; `widgets/overlay/help.rs:126-199`; `widgets/overlay/how_to_play/view.rs:119-199`; `widgets/overlay/tabs.rs:120,157`; `widgets/overlay/command_line.rs:148` |
| 3 | `Text` | Primary text on Background and Surface (`:15`) | Text | `theme/mod.rs:84,89`; `screens/board/view.rs:49`; `screens/board/controls.rs:44,61,75`; `screens/stats/view.rs:126`; `widgets/guide/keys.rs:151`; `widgets/guide/mod.rs:185`; `widgets/card/face.rs` (via `look.fill`) |
| 4 | `TextMuted` | Secondary text (`:17`) | Text | `screens/settings.rs:46`; `screens/game_select/toggles.rs:64`; `screens/main_menu/seed.rs:232`; `screens/stats/view.rs:171`; `widgets/footer.rs:160` (every key line); `widgets/guide/keys.rs:137,173`; `widgets/overlay/tabs.rs:157` |
| 5 | `Accent` | Titles and highlights (`:19`) | Highlight | `screens/board/view.rs:69` (game-facts row); `screens/focus.rs:42` (button labels, via `button_style` at `screens/focus.rs:41-46`); `screens/game_select.rs:192`; `screens/main_menu.rs:249`; `screens/main_menu/seed.rs:215`; `screens/settings.rs:99`; `screens/stats/view.rs:99,146,165,238`; `widgets/guide/mod.rs:79,182` (section headings); `widgets/overlay/dialog.rs:191` (dialog title); `widgets/overlay/help.rs:132`; `widgets/overlay/how_to_play/view.rs:147` |
| 6 | `Border` | Border of an unfocused element (`:21`) | Structure | `widgets/guide/mod.rs:178,190` (side-panel frame only) |
| 7 | `BorderFocus` | Border of the focused element (`:23`) | **Meaning** (focus) | `widgets/card/mark.rs:29` (focused card frame); `widgets/overlay/dialog.rs:188` (dialog frame) |
| 8 | `FocusFg` | Text of the focused list item (`:25`) | **Meaning** (focus) | `theme/mod.rs:94` (`focus()`); `widgets/menu_list.rs:98`; `screens/main_menu/seed.rs:242`; `screens/stats/view.rs:184` |
| 9 | `FocusBg` | Background of the focused list item (`:27`) | **Meaning** (focus) | as `FocusFg` |
| 10 | `Error` | Error text on Surface (`:29`) | **Meaning** (error) | `screens/main_menu/seed.rs:224`; `screens/stats/view.rs:124,242`; `widgets/too_small.rs:36`; `screens/board/controls.rs:24` (event tone) |
| 11 | `Success` | Success text on Surface (`:31`) | **Meaning** (success) | `screens/board/controls.rs:23` (event tone) |
| 12 | `Table` | Playing-field background (`:33`) | **Meaning** (the field) | `screens/board/view.rs:73` (field fill); `widgets/card/mod.rs:87,90,92,122,135,175`; `widgets/aura.rs:93`; `widgets/card/mark.rs` via `widgets/card/mod.rs:89-91` |
| 13 | `TableText` | Text on Table (`:35`) | Text | `screens/board/view.rs:73`; `widgets/card/mod.rs:175` (bottom row of a fanned card) |
| 14 | `Slot` | Outline of an empty pile slot, on Table (`:37`) | **Meaning** (empty slot) | `widgets/card/mod.rs:87,92,135` |
| 15 | `CardFace` | Face of a card (`:39`) | **Meaning** (card) | `widgets/card/mod.rs:85` (fill and frame of a face-up card; centre text on it) |
| 16 | `CardBack` | Face-down card background (`:41`) | **Meaning** (card back) | `widgets/card/mod.rs:86,181` |
| 17 | `CardBackMark` | Pattern on `CardBack` (`:43`) | **Meaning** (card back) | `widgets/card/mod.rs:86,181` |
| 18 | `CardRed` | Rank and suit of red cards (`:45`) | **Meaning** (suit) | `widgets/card/mod.rs:72` (`color_token`) |
| 19 | `CardBlack` | Rank and suit of black cards (`:47`) | **Meaning** (suit) | `widgets/card/mod.rs:73` |
| 20 | `Target` | Legal move target on Table (`:49`) | **Meaning** (target) | `widgets/card/mark.rs:31`; `widgets/card/mod.rs:122` (target corners); `widgets/aura.rs:104` (target halo) |
| 21 | `Selection` | Selected card on Table (`:51`) | **Meaning** (selection) | `widgets/card/mark.rs:27` |
| 22 | `Hint` | Hinted card on Table (`:53`) | **Meaning** (hint) | `widgets/card/mark.rs:33` |
| 23 | `AuraFocus` | Halo around the focused card or run (`:55`) | **Meaning** (focus halo) | `widgets/aura.rs:107` |
| 24 | `AuraCarry` | Halo around the picked-up run (`:57`) | **Meaning** (carry halo) | `widgets/aura.rs:111` |

Notes from the use-sites:
- The card border of a **marked** card is drawn with the mark token on `Table`, not on the card face:
  `widgets/card/mod.rs:89-91` calls `theme.style(token, Token::Table)`. A focused card's frame cells sit
  on the felt, which matters for contrast (section 1.4).
- Dialog and guide frames use `Surface`; the board field uses `Table`; the screen uses `Background`.
- The Dark `dark.rs` comment at `theme/dark.rs:7` said every text pair reaches 7.0, while the test
  target for Dark is 4.5 (`theme/mod.rs:41-46`). O-26 corrected the comment to 4.5.

### 1.3 Value of every token in every theme

Format: `R,G,B · ansi16`. Source line: the palette file at the line given (`dark.rs`, `light.rs`,
`high_contrast.rs`). Each file lists the 24 tokens in the order of the table: Dark starts at
`dark.rs:10`, Light at `light.rs:45`, High Contrast at `high_contrast.rs:81`.

| Token | Dark (`dark.rs`) | Light (`light.rs`) | High Contrast (`high_contrast.rs`) |
|---|---|---|---|
| Background | 24,28,38 · Black (:10) | 244,246,248 · White (:45) | 0,0,0 · Black (:81) |
| Surface | 36,42,58 · Black (:11) | 255,255,255 · White (:46) | 0,0,0 · Black (:82) |
| Text | 232,236,244 · White (:12) | 24,28,36 · Black (:47) | 255,255,255 · White (:83) |
| TextMuted | 184,194,214 · Gray (:13) | 96,104,116 · DarkGray (:48) | 255,255,255 · White (:84) |
| Accent | 140,190,255 · LightCyan (:14) | 18,76,160 · Blue (:49) | 0,255,255 · Cyan (:85) |
| Border | 110,122,150 · DarkGray (:15) | 192,198,208 · Gray (:50) | 255,255,255 · White (:86) |
| BorderFocus | 255,214,102 · Yellow (:16) | 0,12,40 · Black (:51) | 255,255,0 · Yellow (:87) |
| FocusFg | 20,24,32 · Black (:17) | 255,255,255 · White (:52) | 0,0,0 · Black (:88) |
| FocusBg | 255,214,102 · Yellow (:18) | 28,92,196 · Blue (:53) | 255,255,0 · Yellow (:89) |
| Error | 255,160,150 · LightRed (:19) | 180,24,24 · Red (:54) | 255,128,128 · LightRed (:90) |
| Success | 150,230,160 · LightGreen (:20) | 20,116,48 · Green (:55) | 0,255,0 · LightGreen (:91) |
| Table | 40,120,80 · Green (:21) | 32,114,70 · Green (:56) | 0,0,0 · Black (:92) |
| TableText | 236,246,240 · White (:22) | 245,252,248 · White (:57) | 255,255,255 · White (:93) |
| Slot | 170,220,190 · LightGreen (:23) | 160,210,178 · LightGreen (:58) | 255,255,255 · White (:94) |
| CardFace | 250,250,244 · White (:24) | 255,255,255 · White (:59) | 255,255,255 · White (:95) |
| CardBack | 184,208,248 · White (:25) | 0,14,46 · Black (:60) | 0,160,255 · LightBlue (:96) |
| CardBackMark | 16,20,40 · Black (:26) | 230,240,255 · White (:61) | 0,0,0 · Black (:97) |
| CardRed | 166,20,20 · Red (:27) | 176,24,24 · Red (:62) | 160,0,0 · Red (:98) |
| CardBlack | 16,16,20 · Black (:28) | 20,20,24 · Black (:63) | 0,0,0 · Black (:99) |
| Target | 255,214,102 · Yellow (:29) | 255,214,102 · Yellow (:64) | 0,255,0 · LightGreen (:100) |
| Selection | 150,220,255 · LightCyan (:30) | 80,200,255 · LightCyan (:65) | 0,255,255 · LightCyan (:101) |
| Hint | 255,196,110 · LightYellow (:31) | 255,186,70 · LightYellow (:66) | 255,255,0 · LightYellow (:102) |
| AuraFocus | 255,246,184 · Yellow (:32) | 255,236,140 · Yellow (:67) | 255,255,0 · Yellow (:103) |
| AuraCarry | 210,244,255 · LightCyan (:33) | 172,240,255 · LightCyan (:68) | 0,255,255 · LightCyan (:104) |

Values from before O-26 are kept in the git history (`origin/main` before the O-26 merge: Dark Table
24,92,58; Dark CardBack 28,56,132; Light BorderFocus 28,92,196; Light CardBack 30,64,148; HC CardBack
0,0,128; and the others in the table above that changed). Changes made by O-26 are listed in 1.4.

Observations from the table (facts):
- Dark: `BorderFocus`, `FocusBg` and `Target` are the **same** RGB (255,214,102) (`dark.rs:16,18,29`).
- High Contrast: `BorderFocus`, `FocusBg`, `Hint` and `AuraFocus` are the **same** RGB (255,255,0)
  (`high_contrast.rs:87,89,102,103`). `Table` and `Background` are both (0,0,0) (`high_contrast.rs:81,92`).
- Light: `Target` and `Hint`/`AuraFocus` are near-identical yellows and oranges (`light.rs:64,66,67`).
- Only the three themes exist; there is no user-editable palette file or custom theme (`theme/mod.rs:21-29`).

### 1.4 Contrast: what is computed and what the tests check

**What the tests check.**
- `TEXT_PAIRS` (`theme/palette.rs:120-136`) lists 15 fg/bg pairs. Each must reach
  `ThemeId::contrast_target()` (`theme/mod.rs:41-46`): **4.5** for Dark and Light, **7.0** for High
  Contrast. Checked by `theme/tests.rs:9-19` and `accessibility_tests.rs:212-226` (which also asserts the
  High Contrast target is 7.0).
- `theme/tests.rs:22-31`: red and black cards differ in RGB and in ANSI16 in every theme. This does not
  test hue or contrast between the two.
- `theme/tests.rs:34-40`: `FocusBg` differs from `Background` in RGB and ANSI16.
- `theme/tests.rs:43-52`: a swatch resolves by depth (TrueColor, Ansi256, Ansi16).
- `theme/tests.rs:55-65`: every 16-color value is a named color (not RGB, Indexed or Reset).
  Their contrast is **not** tested; it depends on the player's palette (unknown).
- `theme/tests.rs:68-82`: every token resolves to a non-Reset color at every depth.
- `theme/tests.rs:102-115`: aura pairs are in `TEXT_PAIRS`; `AuraFocus` and `AuraCarry` differ in RGB and
  ANSI16, and both differ from `Table` in ANSI16.
- `theme/tests.rs` `every_graphic_pair_reaches_three_to_one` (O-26): the graphics class `GRAPHIC_PAIRS`
  (`theme/palette.rs`) must reach **3.0** in every theme: `BorderFocus`, `CardBack`, `Slot`, `Target`,
  `Selection` and `Hint` on `Table`. The edge of the field is a per-theme pair, `Palette::field_edge_pair`
  (P3-23), tested by two explicit tests and with no exception list (`GRAPHIC_GAPS` is gone): a theme
  without the field frame (`field_edge: false`: Dark, Light) keeps `Table` on `Background` at 3:1; a theme
  with it (`field_edge: true`: High Contrast) has `Border` on `Background` at 3:1.
  `only_a_theme_whose_felt_is_the_background_draws_the_field_frame` ties the flag to the felt contrast.
- `theme/contrast.rs:35-68`: unit tests of the formula (black/white = 21, identical = 1, symmetry, mid
  gray 119 on white ≈ 4.48).
- `accessibility_tests/board.rs:69-100` and `:207-238` check that focus, selection, targets and halos
  differ **by glyph** in both glyph sets (not by color).

**What is not tested.** Pairs outside `TEXT_PAIRS` and `GRAPHIC_PAIRS` (for example `Error` on
`Background`, used at `screens/main_menu/seed.rs:224` and `widgets/too_small.rs:36`, and `CardFace` on
`Table`), and any 16-color contrast.

**Computed ratios.** WCAG 2 contrast ratio (`theme/contrast.rs:3-29`) of the RGB values in 1.3. `T` marks
pairs in `TEXT_PAIRS` (target 4.5 Dark and Light, 7.0 High Contrast). `G` marks pairs in `GRAPHIC_PAIRS`
(target 3.0). "was" gives the value computed on `origin/main` before O-26 where it changed.

| Pair (fg on bg) | Dark | Light | High Contrast | Tested |
|---|---|---|---|---|
| Text on Background | 14.39 | 15.75 | 21.00 | T |
| Text on Surface | 12.09 | 17.07 | 21.00 | T |
| TextMuted on Background | 9.51 | 5.20 | 21.00 | T |
| TextMuted on Surface | 7.99 | 5.63 | 21.00 | T |
| Accent on Background | 8.86 | 7.55 | 16.75 | T |
| Accent on Surface | 7.45 | 8.18 | 16.75 | T |
| FocusFg on FocusBg | 12.75 | 6.22 | 19.56 | T |
| Error on Surface | 7.31 | 6.81 | 8.65 | T |
| Success on Surface | 9.63 | 5.88 | 15.30 | T |
| TableText on Table | 4.88 (was 7.22) | 5.67 | 21.00 | T |
| CardRed on CardFace | 7.34 | 7.02 | 8.42 | T |
| CardBlack on CardFace | 18.12 | 18.37 | 21.00 | T |
| CardBackMark on CardBack | 11.64 (was 9.07) | 16.58 (was 8.25) | 7.47 (was 16.01) | T |
| AuraFocus on Table | 4.91 (was 6.51) | 4.96 | 19.56 | T |
| AuraCarry on Table | 4.66 (was 5.64) | 4.68 | 16.75 | T |
| Error on Background | 8.70 | 6.28 | 8.65 | — |
| Success on Background | 11.45 | 5.43 | 15.30 | — |
| Border on Surface (panel frame) | 3.33 | **1.72** | 21.00 | — |
| BorderFocus on Surface (dialog frame) | 10.27 | 19.39 (was 6.22) | 19.56 | — |
| Border on Background | 3.96 | **1.59** | 21.00 | — |
| BorderFocus on Table (focused card frame) | 3.87 (was 5.73) | 3.28 (was 1.05) | 19.56 | G |
| Slot on Table (empty slot) | 3.52 (was 3.33) | 3.47 (was 2.75) | 21.00 | G |
| Target on Table | 3.87 (was 5.73) | 4.23 | 15.30 | G |
| Selection on Table | 3.59 (was 4.43) | 3.11 | 16.75 | G |
| Hint on Table | 3.44 (was 4.21) | 3.48 (was 3.00) | 19.56 | G |
| CardFace on Table (face-up card edge) | 5.15 (was 7.62) | 5.90 | 21.00 | — |
| CardBack on Table (face-down card edge) | 3.45 (was 1.35) | 3.23 (was 1.60) | 7.47 (was 1.31) | G |
| Table on Background (field edge; no frame in these themes) | 3.16 (was 2.13) | 5.45 | 1.00 (frame drawn instead, see below) | G (Dark, Light) |
| Border on Background (field frame, High Contrast only, P3-23) | not drawn | not drawn | 21.00 | G (High Contrast) |
| FocusBg on Background | 12.22 | 5.74 | 19.56 | — |

Before O-26, the graphics pairs below 3.0 were Light `BorderFocus` on `Table`, Light `Slot`, all `CardBack`
on `Table`, Dark `Table` on `Background` and High Contrast `Table` on `Background`; Light `Hint` was at 3.00.

**Dark `dark.rs:7` claim.** The comment said every text pair reaches 7.0. The Dark target is 4.5, and after O-26
the lowest Dark text pair is `AuraCarry on Table` at 4.66. The comment now says 4.5.

**High Contrast field edge (closed by P3-23).** `Table` on `Background` is 1.00 (both black), and no black/white
palette can meet both bars: with white text, 7:1 on `Table` requires `Table` luminance ≤ 0.10, while 3:1 against
a black `Background` requires ≥ 0.10. The edge is therefore a drawn frame, not a color change, and it exists
**only in High Contrast** (owner decision 2026-11-19): `Palette::field_edge` is `true` there and `false` in Dark and
Light, whose felt already has 3:1 against the background and which keep their full board area. The frame is a
one-cell `Border` line (`widgets/edge.rs`) around the felt, `┌─┐│└┘` in Unicode and `+-|` in ASCII. When it is
drawn the board geometry is inset by that cell (`layout::chrome_with(.., frame)`, `Chrome::field` is the framed
area, `Chrome::body` the area inside it, `FRAME_INSET`); `BoardScreen` takes the flag from `Theme::field_edge()`, so
drawing, mouse hit-testing and `commands.rs` all use the same `body`, and no card covers the frame and no click
lands on it. `Border` on `Background` is 21.00 in High Contrast. The inset costs High Contrast two columns and two
rows of the board: at 130×40 with the guide panel, Klondike cards are 11×9 there and stay 13×11 in Dark and Light.

---

## 2. Text styles (modifiers)

Source: `ratatui::style::Modifier` uses in non-test code. **No `DIM` and no `ITALIC` is used anywhere**
in `crates/tui/src` (grep). Hover and focus are also shown by color and glyphs, listed here.

| Modifier | Where | For what |
|---|---|---|
| **BOLD** | `widgets/card/face.rs:58` | Rank and suit row in the centre of a large face-up card (only where the scale is Large/Medium/Small with a text band; section 3.1) |
| BOLD | `widgets/guide/mod.rs:80,183` | Section headings in the side panel (Accent on Surface) |
| BOLD | `widgets/guide/keys.rs:134` | Key names in the panel |
| BOLD | `widgets/overlay/help.rs:126` | Help heading |
| BOLD | `widgets/overlay/how_to_play/view.rs:139` | Heading rows of the rules text |
| BOLD | `widgets/overlay/command_line.rs:155` | Command-line text row |
| BOLD \| REVERSED | `widgets/overlay/dialog.rs:221` | **Focused dialog button** |
| BOLD \| REVERSED | `widgets/overlay/tabs.rs:126` | Active tab of the help window |
| BOLD \| REVERSED | `widgets/overlay/how_to_play/view.rs:127` | Active tab of the rules window |
| **REVERSED** | `screens/focus.rs:44` (`button_style`, used at `screens/game_select/toggles.rs:35`, `screens/main_menu/seed.rs:281`, `screens/stats/view.rs:222`) | **Focused button** of the game-selection toggles, the seed entry and the statistics |
| **REVERSED** | `widgets/footer.rs:124,144` | **Mouse hover** on a footer key (pointer only; no keyboard equivalent) |
| **UNDERLINED** | `screens/board/header.rs:142` (`spans`, `:133-146`) | **Seed link**: the seed in the header row and in the side-panel `Game` box. Clicking it copies the seed |

Focus without modifiers: list rows use `FocusFg` on `FocusBg` plus the marker `►` (`widgets/menu_list.rs:98`,
`screens/focus.rs:36-38`). Board cards use a double-line frame and a `►` marker (`widgets/card/frame.rs:36-69`,
`widgets/card/mark.rs:48`) plus the focus halo (section 5).

---

## 3. Sizes

All sizes are terminal cells. There is no font size in the program (section 4.3).

### 3.1 Card size classes

`SIZES` lists four card sizes, largest first (`layout/size.rs:10-16`). They are the only card size
classes; "Compact/Standard/Wide" are screen classes (section 3.5). Derived values come from the formulas
in `layout/size.rs`; the centre face comes from `widgets/card/face.rs:19-28` applied to the interior
(card minus its border).

| Size W×H | Row gap (`size.rs:43-45`) | Up step (`:48-51`) | Down step (`:54-57`) | Pyramid step (`:60-62`) | Waste step (`:66-68`) | Centre face (derived from `face.rs:19-28`) |
|---|---|---|---|---|---|---|
| 13×11 | 2 | 5 | 2 | 5 | 7 | Large: text row + 7×4 suit (`art.rs:18-32`) |
| 11×9 | 2 | 4 | 2 | 4 | 6 | Medium: text row + 5×3 suit |
| 9×7 | 1 | 3 | 1 | 3 | 5 | Small: text row + 5×2 suit |
| 7×5 | 1 | 2 | 1 | 2 | 4 | Compact: one suit symbol (`face.rs:67-70`) |

Text row width: `face.rs:31-35` uses two spaces between rank and suit at interior width ≥ 11 (`face.rs:16`),
one space otherwise. Corner labels (rank and suit) are always on the top border row (`widgets/card/rows.rs:31-33`)
and the bottom row (`rows.rs:55-57`).

**When a size is chosen.** `CardSize::largest_fit` picks the first size in `SIZES` that fits; if none
fits, it takes 7×5 (`layout/size.rs:35-40`, `smallest` at `:30-32`).
- Columns (Klondike, Spider, FreeCell): `layout/columns.rs:84`, with `fits` at `columns.rs:64-69`:
  width = columns·W + (columns−1)·1 ≤ area width (`MIN_GAP = 1`, `columns.rs:11`), and
  height = 2·H + row gap + `FAN_BUDGET` (14, `layout/size.rs:20`) ≤ area height.
  Column count: `columns.rs:54-62`. Klondike Draw Three is the only variant that fans the waste
  (`columns.rs:168`).
- Pyramid: `layout/pyramid.rs:56` with `fits` at `pyramid.rs:17-19`: 7·W ≤ area width and
  (7−1)·pyramid step + 2·H + row gap ≤ area height (`pyramid.rs:13-15`).
- A card is drawn only if it is at least 3×2 (`widgets/card/mod.rs:110-112`); a back row needs 3×1
  (`mod.rs:144`).

### 3.2 Fan steps (`layout/fan.rs`)

- Natural step of face-up cards: `up = max(H/2, 2)` (`size.rs:48-51`) → 5, 4, 3, 2 for the sizes above.
- Natural step of face-down cards: `down = max(up/2, 1)` (`size.rs:54-57`) → 2, 2, 1, 1.
- Growth: when rows are spare, the first plan uses `up + 1` (`fan.rs:78-82`; test `fan.rs:137-146`).
- Shrinking order (`fan.rs:52-84`): face-down steps shrink first (`shrink_down`), then face-up steps
  (`shrink_up`), then the leading face-down cards collapse into one counted row (`collapse`, used when
  the column has ≥2 leading face-down cards and some face-up cards: `fan.rs:101-103`).
- Last resort: one row per card, the newest ranks visible, older cards stack under each other
  (`squeezed`, `fan.rs:86-98`).
- Waste fan: covered waste cards show their left border and corner label; visible width 3 or 4
  (`columns.rs:40-44`, `covered_width`). Three cards are fanned (`FAN_CARDS = 3`, `columns.rs:12`).

### 3.3 Gaps and margins

| Gap or margin | Value | Source |
|---|---|---|
| Column gap | from 1 up to `MAX_GAP` = 3, grows with the window; centred margin | `layout/size.rs:22` (MAX_GAP), `:73-80` (`spacing`) |
| Minimum column gap in the fit test | 1 | `layout/columns.rs:11` |
| Row gap above the tableau | 1 (H < 9) or 2 (H ≥ 9) | `layout/size.rs:43-45`, `columns.rs:87` |
| Pyramid card pitch | W (cards touch) | `layout/pyramid.rs:62` |
| Pyramid base row offset | one extra column between base slots | `layout/pyramid.rs:41` |
| Card border | 1 cell on each side | `widgets/card/mod.rs:115` (interior); `widgets/card/face.rs:80` |
| Face text band | `TEXT_BAND` = 3, `WIDE_INTERIOR` = 11 | `widgets/card/face.rs:15-16`; `art_offset` at `face.rs:44-47` |
| Board header row | 1 row, only without the side panel | `layout/chrome.rs:163` |
| Controls strip | 2 rows (`Now:` and `Do:`) under the board | `layout/chrome.rs:167-168` |
| Guide panel padding | `SIDE` = 2 columns, `FRAME_ROWS` = 2 | `widgets/guide/mod.rs:25-27` |
| Dialog chrome | `CHROME_COLUMNS` 4, `BORDER_ROWS` 2, `BUTTON_ROWS` 2, `BUTTON_GAP` 2, `MARKER_COLUMNS` 4 | `widgets/overlay/dialog.rs:39-43` |
| Dialog text margin | 1 column each side | `widgets/overlay/dialog.rs:200` |
| Help window | `GAP` 4, `KEY_PADDING` 2, `LEFT_TOPICS` 2 | `widgets/overlay/help.rs:20-22` |
| Tabs | `TAB_GAP` 2 | `widgets/overlay/tabs.rs:15` |
| Rules tabs | `TAB_PADDING` 2, `TAB_GAP` 1, `ARROW_COLUMNS` 3 | `widgets/overlay/how_to_play/view.rs:24-26` |
| Key segments in the guide | `KEY_GAP` 2, `MARKER_WIDTH` 2 | `widgets/guide/keys.rs:18-19` |
| Footer buttons | separated by the segments' own spaces (`"   "` between key groups) | `widgets/footer.rs:129-150`; test `footer.rs:183-187` |
| Seed buttons, statistics buttons | `BUTTON_GAP` 3 | `screens/main_menu/seed.rs:21`; `screens/stats/view.rs:24` |
| Menu list | label width + 2 columns, centred; marker is 1 column plus a space | `widgets/menu_list.rs:82-85`; `screens/focus.rs:36-38` |

### 3.4 Screen size classes and constants (`layout/chrome.rs`)

| Constant | Value | Source |
|---|---|---|
| `BOARD_MIN_WIDTH` | 100 columns (board kept beside the panel) | `layout/chrome.rs:92` |
| `STANDARD_MIN_WIDTH` | 130 columns (Standard from here) | `layout/chrome.rs:94` |
| `WIDE_MIN_WIDTH` | 180 columns (Wide from here) | `layout/chrome.rs:96` |
| `GUIDE_WIDTH` | 30 columns, panel of a Standard screen | `layout/chrome.rs:98` |
| `GUIDE_WIDTH_WIDE` | 40 columns, panel of a Wide screen | `layout/chrome.rs:100` |
| Class | Compact < 130; Standard 130–179; Wide ≥ 180 | `layout/chrome.rs:112-120` |
| Panel width | Compact 0; Standard 30; Wide 40 | `layout/chrome.rs:123-129` |
| Header row | present only when there is no panel | `layout/chrome.rs:163,173` |
| Minimum terminal | 100 × 30; below it the "too small" notice shows | `widgets/too_small.rs:13,15`; `is_too_small` at `:18-20`; checked in `app.rs:222` |
| Board size before the first resize | 120 × 40 | `screens/board/mod.rs:45` (`DEFAULT_SCREEN`), set by `resize` at `:228` |

### 3.5 Modal and window sizes

| Window | Size rule | Source |
|---|---|---|
| Text dialogs (confirm, pause, win, search; `overlay/mod.rs:47-70`) | width = max(longest line ≤ 64, buttons, title+2, 24) + 4; height = wrapped text rows + 2 border rows (+ 2 when there are buttons); centred and clamped to the area | `widgets/overlay/dialog.rs:39-45` (constants), `:146-165` (`geometry`), `:122-143` (`boxed`), `:72-81` (`centered`) |
| Help window | width = max(left column + 4 + right column + 4, **80**); body **23** rows; centred | `widgets/overlay/help.rs:20-24`, `:150-155` |
| Guide window (Compact screens, instead of the panel) | box **62** columns (fixed); text **58** columns; **17** text rows, then it scrolls; page 15; wheel 3 | `widgets/overlay/guide_window.rs:22-27`, `:69-71` |
| Rules window (How to play) | width of the help window; **20** text rows (`VIEW_ROWS`); text **76** columns (`TEXT_WIDTH`); chrome 3 rows; page 19; wheel 3 | `widgets/overlay/how_to_play/content.rs:12,14`; `view.rs:23`, `:52-56`; `how_to_play/mod.rs:25-26` |
| Command line | **21** text rows (`VIEW_ROWS`); page 20; wheel 3; rows 1 + fill + 1 | `widgets/overlay/command_line.rs:23,25-26,139-142` |
| Pause | clears the board body; header stays | `widgets/overlay/mod.rs:96-102` |
| Controls strip event | **40-column** limit (`EVENT_WIDTH`), and never more than half the row (`min(40, width/2)`); `Now:` gets the rest | `screens/board/controls.rs:19`, `:36` |
| Seed entry field | text field **22** columns (`FIELD_WIDTH`), **32** characters (`MAX_LENGTH`) | `screens/main_menu/seed.rs:19-20`, `:143` |
| Statistics list | `FIXED_ROWS` 8, `MIN_LIST_ROWS` 5 (history rows always shown), marker 4 columns | `screens/stats/view.rs:20-25` |
| Settings list | one row per field (12) | `screens/settings.rs:91`; `screens/settings/options.rs:29-42` |
| Logo | 5 rows (6 allowed by test), ≤ 60 columns | `widgets/logo.rs:3-17`; test `:30-40` |
| Volume step | 10 percent | `screens/settings/options.rs:63` |

Other numeric constants with a visual effect: `widgets/guide/mod.rs` (panel frame, above); card
`MARGIN` = 2 in a test only (`widgets/card/tests/mod.rs:26`); `layout/hit_tests.rs:5` `LABEL_WIDTH` = 3
(mouse rectangle of a label, `:18`).

---

## 4. Glyphs and fonts

### 4.1 Glyph modes and how they are chosen

- `GlyphMode { Font, Unicode, Ascii }` (`settings.rs:14-21`).
- `Glyphs::new` maps **Font and Unicode to the same set** (`glyphs.rs:83-88`, the match at `:85`). The file
  header says Font is "the oXySol card font … until the font exists" (`glyphs.rs:1-3`). No font exists in
  the repository (`assets/` has only `audio/` and `icon/`; there is no font file).
- Default: Unicode (`settings.rs:68`). Without Unicode, the starting value is ASCII (`settings.rs:84-94`,
  using `caps.unicode`).
- Unicode detection (`caps.rs:69-86`): always true on Windows (`caps.rs:70-71`); elsewhere the first
  non-empty of `LC_ALL`, `LC_CTYPE`, `LANG` must contain `utf-8` or `utf8`.
- Settings row (`screens/settings/options.rs:56,108-114`): cycles Font → Unicode → Ascii.

### 4.2 The symbol sets

Unicode set `glyphs.rs:43-60`, ASCII set `glyphs.rs:62-79`. Every Unicode field is WGL4 (`glyphs/wgl4.rs:4-91`,
checked by `glyphs/wgl4.rs:142-176` and `glyphs.rs:105-126`).

| Field | Unicode | ASCII | Drawn by |
|---|---|---|---|
| `suits` (♣ ♦ ♥ ♠) | U+2663, U+2666, U+2665, U+2660 (`wgl4.rs:81-84`) | `c d h s` (lowercase, `glyphs.rs:64`) | corner labels `widgets/card/mod.rs:66-68`; centre text `face.rs:30-42` |
| `focus_left` ► | U+25BA | `>` | `widgets/card/mark.rs:48`; `screens/focus.rs:36-38` (menu marker) |
| `focus_right` ◄ | U+25C4 | `<` | **defined only** (no rendering use found; `glyphs.rs:19,47,66`) |
| `selected` ● | U+25CF | `*` | `widgets/card/mark.rs:47` |
| Guide available mark • (not a `Glyphs` field) | U+2022 | `*` | `widgets/guide/mod.rs` `available_mark`; Keys section `widgets/guide/keys.rs`, Legend `widgets/guide/legend.rs` |
| `target` + | `+` | `+` | corners `widgets/card/frame.rs:91-99`; empty slot `widgets/card/slot.rs:42` |
| `hint_source` / `hint_target` | `?→` / `→?` (U+2192) | `?>` / `>?` | `widgets/card/mark.rs:42-45` |
| `recycle` (empty stock, can turn) | ↑ (U+2191) | `^` | `widgets/card/slot.rs:28` |
| `spent` (empty stock, no pass) | `x` | `x` | `widgets/card/slot.rs:29` |
| `face_down` | ▒ (U+2592) | `#` | card back fill `widgets/card/mod.rs:118`; collapsed row `:183`; back row `widgets/card/frame.rs:102-115` |
| `aura_focus` | ░ (U+2591) | `:` | `widgets/aura.rs:107` |
| `aura_carry` | ▓ (U+2593) | `=` | `widgets/aura.rs:111` |
| `aura_target` | · (U+00B7) | `.` | `widgets/aura.rs:104` |
| `edge` │ | U+2502 | `\|` | **defined, never drawn** (grep: only `glyphs.rs:121` and `glyphs/wgl4.rs:170`) |
| `border` | `BorderType::Plain` (`glyphs.rs:59,78`) | same | not used for cards (frames below) |

Frames and borders (the cell sets actually drawn):

| Frame | Unicode | ASCII | Source |
|---|---|---|---|
| Card, normal | `┌ ┐ └ ┘` `─` `│` | `+` `-` `\|` | `widgets/card/frame.rs:37`, `:45`, chosen at `:52-69` |
| Card, target | `┌ ┐ └ ┘` `─` `│` plus `+` in both top corners | corners `+`, edges `~` (horizontal) and `!` (vertical) | `widgets/card/frame.rs:40`, `:48`; corners `:91-99` |
| Card, hint | `╓ ╖ ╙ ╜` `─` `║` | `+` corners, `.` horizontal, `:` vertical | `widgets/card/frame.rs:41`, `:49` |
| Card, selected | `╒ ╕ ╘ ╛` `═` `│` | `*` | `widgets/card/frame.rs:38`, `:46` |
| Card, focused | `╔ ╗ ╚ ╝` `═` `║` | `@` corners, `=` horizontal, `@` vertical | `widgets/card/frame.rs:39`, `:47` |
| Dialog | `border::DOUBLE` | `+` `-` `\|` | `widgets/overlay/dialog.rs:242-247`, `ASCII_BORDER` `:47-56` |
| Side panel | `border::PLAIN` | `+` `-` `\|` | `widgets/guide/mod.rs:29-37`, `:68-74` |

Suit pictograms (`widgets/card/art.rs`): Unicode rows use only block characters (▄ █ ▀ ▌ ▐ …); ASCII rows
use `# / \ ^ v < >` and spaces (`art.rs:35-63`); Large (7×4), Medium (5×3), Small (5×2) (`art.rs:18-32`).
Selection of art: `art.rs:66-93`. The four suit shapes differ in both sets (`art.rs:1-3`).

Logo: Unicode logo uses ▄ ▀ █ (`widgets/logo.rs:11-17`); ASCII logo (`:3-9`); chosen by `logo(unicode)` (`:21-24`).

### 4.3 What the program cannot control, and the Windows console

- **Terminal font face and size: the program has no control.** It only places cells. Decision T-13
  (`docs/tech/decisions.md:69`) says scaling means card size and spacing in cells and that the terminal
  controls font size. No code in `crates/` or `src/` sets a font.
- **Classic Windows console font: nothing exists in the code.** A case-insensitive search of the whole
  repository for `SetCurrentConsoleFontEx`, `GetCurrentConsoleFont`, `ConsoleFont`, `CONSOLE_FONT` and
  `font size` finds only planned items in documentation: `docs/task_list/phase-3.md:296-322` (card and
  font size, P3-16), `docs/task_list/phase-3.md:582-592`, `docs/task_list/README.md:136` (P3-16, TODO),
  `docs/tech/ui-review.md:488-501`. The only Windows-console code is window ownership in `src/console.rs`
  (`GetConsoleProcessList` FFI at `src/console.rs:20-29`) and the pause after a failed run
  (`src/console.rs:9-15`, `src/main.rs:32-33`). It does not change fonts.
- The architecture text describes a `Font` mode with Private Use Area glyphs, a "graphics-check screen",
  `assets/cards/` and an `xtask` `fonts` subcommand (`docs/tech/architecture.md:13`, `:37-39`, `:470-471`).
  None of these exists in the code: no graphics-check screen (grep for "graphics" in `crates/tui/src` is empty),
  no `assets/cards/`, no `xtask/src/fonts.rs`, and the xtask usage line lists no `fonts` command
  (`xtask/src/main.rs:20`). This is a documentation mismatch.

---

## 5. Aura: focus, carry and target halos (`widgets/aura.rs`)

- **Shape.** A one-cell ring around the run's rectangle (`ring`, `aura.rs:16-27`). It is drawn only on
  cells not occupied by a card or a drawn empty slot (`occupied`, `aura.rs:56-63`; skipped in `paint`,
  `aura.rs:65-80`). It moves nothing and covers nothing.
- **Focus** (`aura.rs:107`): symbol `aura_focus` (░ / `:`), token `AuraFocus` on `Table`.
- **Carry** (`aura.rs:111`): symbol `aura_carry` (▓ / `=`), token `AuraCarry` on `Table`. Picked-up run.
- **Target** (`aura.rs:104`): symbol `aura_target` (· / `.`), token `Target` on `Table`. Painted first
  (`aura.rs:82-83`), so focus and carry overwrite it where they meet.
- **Thickness:** one cell. No modifier is applied (only foreground and background, `aura.rs:65-80`).
- **Clipping:** the ring is clipped to the board body (`aura.rs:23`). Near an edge it is cut short.
- The halo is in addition to card-level marks (section 4.2 frames, `frame.rs:36-69`) and markers
  (`mark.rs:47-48`).

---

## 6. Configurable today (Settings screen)

Source: `screens/settings/options.rs:29-42` (rows), `settings.rs:33-80` (struct and defaults). A row
changes with ←/→ or Enter; audio rows are locked when sound is unavailable (`screens/settings.rs:36-38`).

| Row (`Field`) | Options | Default | Notes | Source |
|---|---|---|---|---|
| Language | en-US, pl-PL | en-US | `options.rs:93-98` | `settings.rs:64` |
| Theme | Dark, Light, High Contrast | Dark | three compiled palettes | `options.rs:100-106`; `settings.rs:65` |
| Front style | `international` (only one) | international | `options.rs:52` | `settings.rs:66` |
| Back style | `default` (only one) | default | `options.rs:54` | `settings.rs:67` |
| Glyphs | Font, Unicode, Ascii | Unicode | Font renders as Unicode today (`glyphs.rs:85`) | `options.rs:56,108-114`; `settings.rs:68` |
| Colors | Auto, 16, 256, True color | Auto | Auto = detected; Windows detects TrueColor (`caps.rs:41`) | `options.rs:57-62,116-123`; `settings.rs:69` |
| Sound style | list from the audio folders | default | | `settings.rs:70` |
| Cues | on / off | on | | `settings.rs:71` |
| Music | on / off | off | | `settings.rs:72` |
| Volume | 0–100 in steps of 10 | 70 | `options.rs:63,218-220` | `settings.rs:73` |
| Auto-finish | on / off | on | a forced win plays out | `settings.rs:77` |
| Guide panel | on / off | on | beside the board when wide enough | `settings.rs:74` |

Saved in `Settings` but not on this screen: `undo_allowed` (default on, `settings.rs:75`) and
`winnable_only` (default off, `settings.rs:76`); the comments say they are offered in the game selection
(`settings.rs:50-55`).

**Not configurable (no setting or code path found):** card size (automatic, section 3.1), spacing and
margins, fan steps, per-token colors or a custom theme file (themes are compiled in, section 1.3), the
terminal font and font size (section 4.3), aura and frame glyphs, zoom (planned as P3-16 only).

---

## 7. Gaps for accessibility (findings only; nothing is fixed here)

### 7.1 Meaning carried by color alone, or by hue pairs

1. **Focus versus target in Dark.** `BorderFocus`, `FocusBg` and `Target` are the same yellow
   (`dark.rs:16,18,29`). The focused frame (╔ double line and `►`, `mark.rs:48`) and a target (`+` corners,
   `frame.rs:91-99`; halo `·`, `aura.rs:104`) differ by shape. In High Contrast, `BorderFocus`, `FocusBg`,
   `Hint` and `AuraFocus` are the same yellow (`high_contrast.rs:87,89,102,103`). Shape carries it; color does not.
2. **Error and success by hue only.** The event after a move is colored `Error` or `Success`
   (`screens/board/controls.rs:23-24`, `screens/board/status.rs:81-86`). The text is drawn as-is
   (`controls.rs:37-50`); no symbol is added. Dark `Error` (255,160,150) and `Success` (150,230,160) differ
   mostly in hue, a red/green pair. The message wording is the only other carrier. Same for the seed error
   (`screens/main_menu/seed.rs:224`) and the statistics load failure (`screens/stats/view.rs:124`).
3. **Red and black suits.** The suit color is `CardRed`/`CardBlack` (`widgets/card/mod.rs:70-75`). Suit shape
   is also shown (`art.rs:1-3`, the ♥♦♣♠ glyphs), so the suit reads without color. Yet the pair is
   separated mostly by lightness in Dark (`card_red` 166,20,20 vs `card_black` 16,16,20, both dark). For a
   protanope or deuteranope the card color cue is only darkness, not a distinct hue.
4. **Selection, hint and carry: close hues.** Dark `Selection` (102,204,255) and `AuraCarry` (150,228,255) are
   both light cyan. Light `Target` (255,214,102) and `Hint` (255,165,50) are yellow and orange. These pairs
   rely on shapes (`●` and `▓`, `:` and `=`, `?→`).
5. **Field and card edges by hue.** A face-down card against the felt is `CardBack` against `Table`:
   1.35 Dark, 1.60 Light, 1.31 High Contrast before O-26. O-26 made it 3.45 Dark, 3.23 Light and 7.47 High
   Contrast (section 1.4). The face-up card against the felt is 5.15–5.90 Dark and Light, 21.00 High Contrast.

### 7.2 Text and marks that may be too small or too faint

6. **Focused card frame in Light is nearly invisible (fixed by O-26).** Before O-26, `BorderFocus`
   (28,92,196) on `Table` (32,114,70) was **1.05:1** (computed). It is now near-black (0,12,40), 3.28:1. The frame cells of a marked card are drawn on `Table` (`widgets/card/mod.rs:89-91`).
   The `►` marker and double-line shape still show focus, but the frame color itself does not show it.
7. **Light: marks at or below the 3:1 graphics bar (fixed by O-26 except Selection).** Before O-26 the empty
   slot was 2.75, Hint 3.00 and Selection 3.11 (all on `Table`, computed); Slot and Hint are now 3.47 and 3.48.
   Selection (3.11) still passes the bar but has little margin. Slot placeholders
   (`A`, `↑`, `x`, `+`, `widgets/card/slot.rs:24-53`) are text in these colors.
8. **Light: panel and dialog frames.** The side panel frame (`Border`) is 1.72 on Surface and 1.59 on Background
   (computed; `widgets/guide/mod.rs:178,190`). Structure is still shown by glyphs, but faintly.
9. **High Contrast: field edge (closed by P3-23).** `Table` and `Background` are both (0,0,0) (`high_contrast.rs`), so
   the field has no color edge. In this theme only, the edge is now a drawn one-cell `Border` frame
   (`widgets/edge.rs`), 21.00 on `Background`, with an ASCII form (section 1.4).
10. **Halo glyphs are single cells of low ink.** `·` (U+00B7), `:` and `.` (ASCII target) are small. The
    shade `░` and `▓` are dense but one cell. Halos are drawn only on free cells (`aura.rs:56-63`), so a
    halo around a crowded column may be broken.
11. **Suit pictograms are small.** At 9×7 the pictogram is 5×2 and at 7×5 only one suit character remains
    (`art.rs:18-32`, `face.rs:67-70`). The rank appears in the corner labels and, at 9×7 and up, in the
    centre text row. At 7×5 only the corner labels carry the rank (`widgets/card/rows.rs:31-33`).
12. **No text scale.** Text size is the terminal's. The program has no zoom or font setting (section 4.3;
    P3-16 is planned, `docs/task_list/phase-3.md:296`). The minimum terminal is 100×30
    (`widgets/too_small.rs:13,15`), so a large font on a small window can trigger the "too small" notice.
13. **Shape-only fallback in ASCII.** The ASCII set keeps the same meaning with `c d h s` suits and `> <`
    markers, all one character. `ASCII` card frames use `~ !` for target and `. :` for hint
    (`frame.rs:44-50`).

### 7.3 Other findings

14. **Wide and ambiguous characters.** Text width is counted with `chars().count()` everywhere
    (`widgets/card/paint.rs:39-41`, `screens/board/header.rs:39-41`, `widgets/text.rs:8,22`). Many Unicode
    glyphs of the set (► ◄ ● ░ ▒ ▓ ♠♣♥♦ ─│═║ → ↑ ·) are East Asian **Ambiguous** width in Unicode's
    `EastAsianWidth.txt` (**external fact, not checked in this repository**). In a terminal that draws them
    double-width, the card layout would break. Unknown here, because the code does not look at the
    terminal's width.
15. **Windows always assumed TrueColor and Unicode** (`caps.rs:41`, `caps.rs:70-71`). The code does not check
    the host. Hosts without TrueColor get mis-colored output (unknown; the player can force a mode, `options.rs:57-62`).
16. **16-color contrast is untested.** `theme/tests.rs:55-65` checks only named colors. The real contrast
    is set by the player's palette (unknown).
17. **Documentation mismatches found while reading** (not code defects, listed for the owner):
    - `theme/dark.rs:7`: "every text pair reaches 7.0" vs the 4.5 target (fixed by O-26; the comment now says 4.5).
    - `docs/tech/architecture.md:470-471`: Font uses PUA glyphs and a graphics-check screen; the code maps
      Font to Unicode (`glyphs.rs:85`) and has no such screen.
    - `docs/tech/architecture.md:13,37-39`: `xtask/src/fonts.rs` and `assets/cards/` do not exist.
    - `crates/tui/src/glyphs.rs`: `focus_right` and `edge` are defined but not drawn (section 4.2).
18. **Positive findings** (for balance): focus, selection, target and hint each have a glyph and a border
    shape that differ in both glyph sets; a test checks this (`accessibility_tests/board.rs:69-100`).
    No text uses `DIM` or `ITALIC`, so no faint text. Unicode glyphs are WGL4 and tested
    (`glyphs/wgl4.rs:142-176`). Button focus has `REVERSED` as well as color (`screens/focus.rs:44`).

---

## Open questions

- The Windows host's real color and width handling (sections 1.1, 4.2, 7.3 item 14–15): unknown from code.
- 16-color palettes and their contrast: unknown (depends on the player's terminal).
- Whether `focus_right` (◄) and `edge` (│) were meant to be drawn: unknown; no caller exists.
- Whether `DEFAULT_SCREEN` (120×40, `screens/board/mod.rs:45`) is ever shown before the first resize: unknown.
