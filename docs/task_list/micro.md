# Micro tasks — Antigravity queue

Small, self-contained cards for Beo, Oxy and Son; Dad may take any `READY` card nobody has started.
Read `AGENTS.md` and your card only. A `DRAFT` card is not workable yet: the task named under
"Finalized by" fills in the exact interface and marks it `READY`. "Needs" must be merged first.

| ID | Card | Lane | Needs | Status |
|---|---|---|---|---|
| M-01 | Lint rule: file length | Beo | P0-01 | DONE |
| M-02 | Lint rule: nesting depth | Oxy | P0-01 | DONE |
| M-03 | Lint rule: empty `Err` arms | Son | P0-01 | DONE |
| M-04 | Lint rule: vague error types | Beo | P0-01 | DONE |
| M-05 | Metadata sync | Oxy | P0-01 | DONE |
| M-06 | Architecture tests | Son | P0-01 | DONE |
| M-07 | Wiki: Klondike rules | Beo | P0-01 | DONE |
| M-08 | Wiki: Spider rules | Oxy | P0-01 | DONE |
| M-09 | Wiki: FreeCell rules | Son | P0-01 | DONE |
| M-10 | Wiki: Pyramid rules | Beo | P0-01 | DONE |
| M-11 | Theme: Light palette | Beo | P3-01 | DONE |
| M-12 | Theme: High Contrast palette | Oxy | P3-01 | DONE |
| M-13 | RGB to 256-color conversion | Oxy | P0-01 | DONE |
| M-14 | Contrast ratio helper | Son | P0-01 | DONE |
| M-15 | Terminal capability detection | Beo | P0-01 | DONE |
| M-16 | Title logo | Oxy | P0-01 | DONE |
| M-17 | Polish translation, part 1 | Son | P3-03, P3-04 | DONE |
| M-18 | Command-line parser | Son | P0-01 | DONE |
| M-19 | Artwork: International suits | Beo | P5-01 | READY |
| M-20 | Artwork: German suits | Oxy | P5-01 | READY |
| M-21 | Artwork: Italian suits | Son | P5-01 | READY |
| M-22 | Artwork: court emblems | Beo | P5-01 | READY |
| M-23 | Artwork: back style 1 | Oxy | P5-01 | READY |
| M-24 | Artwork: back style 2 | Son | P5-01 | READY |
| M-25 | Sound style 1: cues | Beo | P6-03 | DONE |
| M-26 | Sound style 2: cues | Oxy | P6-03 | DONE |
| M-27 | Music loops | Son | P6-03 | DONE |
| M-28 | E2E: Spider and FreeCell | Beo | P7-01 | DRAFT |
| M-29 | E2E: Pyramid and settings | Oxy | P7-01 | DRAFT |
| M-30 | Wiki: installation and requirements | Son | P3-09 | READY |
| M-31 | Wiki: controls | Beo | P3-09 | READY |
| M-32 | Wiki: decks, themes, sound | Oxy | P5-02, P6-02 | DRAFT |
| M-33 | Wiki: accessibility and FAQ | Son | P3-09 | READY |
| M-34 | Polish translation, part 2 (#62) | Beo | P5-02, P6-02 | DONE |
| M-35 | Polish copy edit and glossary (external review of 2026-11-19) (#84) | Son | — | DONE |

## Tooling

### M-01 — Lint rule: file length

- **Branch:** `feat/M-01-lint-loc` · **File:** `xtask/src/lint/loc.rs`
- **Do:** implement the existing `pub fn check(file: &SourceFile) -> Vec<Finding>`. Count lines that
  are neither blank nor comment-only (a line starting with `//` after trimming, or lying wholly
  inside a `/* … */` block). More than 300 → one `Warning`; more than 500 → one `Error` instead.
  Rule name `"loc"`, line 1, message with the count and the limit.
- **Done when:** unit tests in the same file pass for 300 lines (nothing), 301 (warning), 500
  (warning), 501 (error), and show that blank and comment-only lines are not counted.
- **Done:** #2

### M-02 — Lint rule: nesting depth

- **Branch:** `feat/M-02-lint-nesting` · **File:** `xtask/src/lint/nesting.rs`
- **Do:** implement `check` with a `syn` visitor. For every function and method compute the deepest
  nesting of `if`, `match`, `for`, `while`, `loop`, plain/`unsafe`/`async` blocks and closures with
  a block body. The function body is level 0. `else if` stays at the level of its `if`. A `match`
  arm adds no level. A nested function is measured on its own. Depth 5 or 6 → `Warning`; 7 or more
  → `Error`. Rule name `"nesting"`, the line of the deepest point, message with the function name.
- **Done when:** tests pass for: `match` in `for` in `if` = 3; an `else if` chain = 1; a
  block-bodied closure adds one; a nested function does not add to its parent; depth 4 (nothing),
  5 (warning), 7 (error).
- **Done:** #3

### M-03 — Lint rule: empty `Err` arms

- **Branch:** `feat/M-03-lint-err-arms` · **File:** `xtask/src/lint/err_arms.rs`
- **Do:** implement `check` with `syn`. `Error` for a `match` arm whose pattern is `Err(..)` (any
  inner pattern) and whose body is `{}` or `()`, and for `if let Err(..) = … {}` with an empty
  block. Rule name `"err-arms"`.
- **Done when:** tests pass for both forms, and show no finding for `Err(e) => return Err(e.into())`,
  `Err(e) => log(e)`, and `Ok(_) => {}`.
- **Done:** #4

### M-04 — Lint rule: vague error types

- **Branch:** `feat/M-04-lint-err-types` · **File:** `xtask/src/lint/err_types.rs`
- **Do:** implement `check` with `syn`. `Error` for every `pub fn` (free function or method) whose
  return type is `Result<_, String>`, `Result<_, &str>`, `Result<_, ()>`, `Result<_, Box<dyn …>>`,
  or any type whose path starts with `anyhow`. Rule name `"err-types"`.
- **Done when:** tests pass for each forbidden form, and show no finding for a private function
  with `Result<_, String>` or a public one with `Result<_, MoveError>`.
- **Done:** #6

### M-05 — Metadata sync

- **Branch:** `feat/M-05-sync-metadata` · **File:** `xtask/src/sync_metadata.rs`
- **Do:** add a pure function that takes the text of `Cargo.toml` and of `metadata.toml` and
  returns the new `Cargo.toml` text with `[workspace.package]` `version`, `description`, `license`
  and `repository` set from `[project]` `version`, `description`, `license`, `repository_url`, using
  `toml_edit` so everything else is preserved byte for byte. Keep the signature of the existing
  entry function: without `--check` it rewrites the file; with `--check` it changes nothing and
  fails when the result would differ.
- **Done when:** tests on strings pass: values are copied, comments and ordering survive, already
  synced input is returned unchanged, a missing `[project]` key is an error.
- **Done:** #12

### M-06 — Architecture tests

- **Branch:** `test/M-06-architecture` · **File:** `tests/architecture/main.rs`
- **Do:** four tests reading files relative to `env!("CARGO_MANIFEST_DIR")`:
  1. `[dependencies]` of `crates/core/Cargo.toml` contains nothing but `serde` and `thiserror`;
  2. `[dependencies]` of `crates/tui/Cargo.toml` contains none of `oxysol`, `serde_json`,
     `directories`, `cpal`, `midly`, `rusqlite`;
  3. no `.rs` file under `crates/core/src` contains `std::fs`, `std::io`, `std::env`, `std::time`,
     `std::net`, `std::process`, `std::thread`, `println!` or `eprintln!`;
  4. `crates/core/src/lib.rs` does not contain `pub mod games`.
- **Done when:** `cargo test --test architecture` passes (this target is not part of `--quick`,
  run it explicitly), and each test fails when you temporarily break its rule — say so in the report.
- **Done:** #11

### M-18 — Command-line parser

- **Branch:** `feat/M-18-cli-parser` · **File:** `src/cli.rs`
- **Do:** no dependencies, no printing, no text for users:

  ```rust
  pub enum ColorArg { Ansi16, Ansi256, TrueColor }
  pub struct Cli { pub show_version: bool, pub show_help: bool, pub ascii: bool, pub colors: Option<ColorArg>, pub seed: Option<u64> }
  pub enum CliError { UnknownArgument(String), MissingValue(&'static str), InvalidValue { flag: &'static str, value: String } }
  pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, CliError>;
  ```

  Flags: `--version`, `--help`, `--ascii`, `--colors <16|256|truecolor>`, `--seed <u64>` (decimal). `CliError` derives
  `thiserror::Error`. `args` excludes the program name.
- **Done when:** tests pass for no arguments, each flag, all three color values, `--colors` without
  a value, `--colors 8`, `--seed 42`, `--seed 18446744073709551615`, `--seed -1`, `--seed abc`,
  `--seed` without a value, and an unknown argument.
- **Done:** #10

## Wiki: rules

### M-07, M-08, M-09, M-10 — Rules of Klondike, Spider, FreeCell, Pyramid

- **Branch:** `docs/M-0X-wiki-<game>` · **File:** `docs/wiki/rules/<game>.md` (one game per card)
- **Read also:** the card of that game in `docs/task_list/phase-2.md` — it is the only source.
- **Do:** explain the game to a player in plain English under the headings Goal, Layout, Moves,
  Stock, Winning, Variants. Use card-table words, never code names such as `Move::Stock`. Do not add
  a rule that is not in the source card and do not omit one.
- **Done when:** every rule of the source card appears once; nothing else is claimed.
- **Done:** M-07 #22, M-08 #23, M-09 #26, M-10 #19

## Terminal and theme helpers

### M-13 — RGB to 256-color conversion

- **Branch:** `feat/M-13-ansi256` · **File:** `crates/tui/src/theme/ansi.rs`
- **Do:** `pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8` returning the nearest color by squared
  distance among the 6 × 6 × 6 cube (indexes 16–231, channel levels 0, 95, 135, 175, 215, 255) and
  the gray ramp (indexes 232–255, value 8 + 10·i). Never return 0–15.
- **Done when:** tests pass: (0,0,0) → 16; (255,255,255) → 231; (255,0,0) → 196; (128,128,128) → 244;
  (95,135,175) → 67.
- **Done:** #8

### M-14 — Contrast ratio helper

- **Branch:** `feat/M-14-contrast` · **File:** `crates/tui/src/theme/contrast.rs`
- **Do:** `pub fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f32` using the WCAG 2 relative
  luminance formula for sRGB.
- **Done when:** tests pass: black on white = 21.0 (± 0.01); identical colors = 1.0; the result is
  symmetric; (119,119,119) on white is 4.48 (± 0.01).
- **Done:** #15

### M-15 — Terminal capability detection

- **Branch:** `feat/M-15-caps` · **File:** `crates/tui/src/caps.rs`
- **Do:**

  ```rust
  pub enum ColorDepth { Ansi16, Ansi256, TrueColor }
  pub enum Platform { Windows, MacOs, Linux }
  pub struct TerminalCaps { pub color: ColorDepth, pub unicode: bool, pub remote: bool }
  pub fn detect(env: &dyn Fn(&str) -> Option<String>, platform: Platform) -> TerminalCaps;
  ```

  Color: `COLORTERM` equal to `truecolor` or `24bit` → `TrueColor`; otherwise Windows → `TrueColor`;
  otherwise `TERM` containing `256color` → `Ansi256`; otherwise `Ansi16`. Unicode: Windows → true;
  otherwise true when the first non-empty of `LC_ALL`, `LC_CTYPE`, `LANG` contains `utf-8` or `utf8`
  in any case. Remote: `SSH_CONNECTION` or `SSH_TTY` is set. The function reads nothing but `env`.
- **Done when:** a table test passes for: Windows with no variables; Linux `xterm-256color` with
  `LANG=en_US.UTF-8`; Linux `TERM=xterm` with `LANG=C` (16 colors, no Unicode); macOS
  `xterm-256color` UTF-8; `COLORTERM=truecolor`; `LC_ALL` overriding `LANG`; an SSH session; no
  variables at all on Linux.
- **Done:** #17

### M-16 — Title logo

- **Branch:** `feat/M-16-logo` · **File:** `crates/tui/src/widgets/logo.rs`
- **Do:** `pub fn logo(unicode: bool) -> &'static [&'static str]` returning the word `oXySol` drawn
  in block letters, 5 or 6 rows, at most 60 columns, every row the same width. The Unicode version
  uses only `█`, `▀`, `▄` and spaces; the ASCII version only `#` and spaces.
- **Done when:** tests pass for equal row widths, the width limit and the allowed characters of
  both versions; the six letters are legible when printed.
- **Done:** #18

### M-11, M-12 — Theme palettes: Light, High Contrast

- **Status:** READY · **Finalized by:** P3-01
- **Branch:** `feat/M-11-theme-light` (Light) · `feat/M-12-theme-high-contrast` (High Contrast)
- **File:** `crates/tui/src/theme/light.rs` (M-11) or `crates/tui/src/theme/high_contrast.rs` (M-12) only
- **Interface:** the file already exists and holds `pub fn palette() -> Palette` returning the Dark
  palette as a placeholder. Replace its body with a full `Palette { … }` literal; copy the shape of
  `crates/tui/src/theme/dark.rs`. Each field is `swatch(r, g, b, Color::<Named>)`: the RGB value for
  true color (256-color terminals are derived from it) and one named ANSI color from
  `ratatui::style::Color` (`Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `Gray`,
  `DarkGray`, `LightRed`, `LightGreen`, `LightYellow`, `LightBlue`, `LightMagenta`, `LightCyan`,
  `White`) for 16-color terminals. Never use `Color::Rgb`, `Color::Indexed` or `Color::Reset`.
- **Tokens (22), all must be set:** `background` (screen), `surface` (dialogs), `text`, `text_muted`,
  `accent`, `border`, `border_focus`, `focus_fg` / `focus_bg` (the focused list item), `error`,
  `success`, `table` (playing field), `table_text`, `slot` (empty pile outline, on `table`),
  `card_face`, `card_back`, `card_back_mark` (pattern on `card_back`), `card_red`, `card_black`
  (rank and suit on `card_face`), `target` (legal move), `selection` (selected card), `hint`
  (hinted card; the last three are drawn on `table`). Light: dark text on light surfaces. High
  Contrast: maximum separation, no mid-tones for text.
- **Pairs that must pass** (`theme::palette::TEXT_PAIRS`, foreground on background):
  `text`/`background`, `text`/`surface`, `text_muted`/`background`, `text_muted`/`surface`,
  `accent`/`background`, `accent`/`surface`, `focus_fg`/`focus_bg`, `error`/`surface`,
  `success`/`surface`, `table_text`/`table`, `card_red`/`card_face`, `card_black`/`card_face`,
  `card_back_mark`/`card_back`. Ratio at least 4.5 (Light) or 7.0 (High Contrast).
- **Do not touch** any other file. The tests already exist in `crates/tui/src/theme/tests.rs`.
- **Done when:** `cargo test -p oxysol-tui theme` passes. It checks every pair above against the
  `ThemeId::contrast_target()` of the theme (4.5 or 7.0), that `card_red` differs from `card_black`
  (RGB and 16-color), that `focus_bg` differs from `background` (RGB and 16-color), and that every
  16-color value is a named color.
- **Done:** M-11 #32, M-12 #33

## Translation

### M-17, M-34 — Polish translation

- **Status:** M-17 DONE (finalized by P3-03), M-34 DONE (finalized by P3-09)
- **Branch:** `docs/M-XX-pl-translation` · **Files:** part 1 — `locales/pl-PL/{common,menu,board}.toml`;
  part 2 — the remaining `locales/pl-PL/*.toml`
- **M-17 keys:** `common.toml` (all tables); `board.toml`: `[status]`, `[error]` and `[confirm]` (move
  rejections, hints, undo/redo messages, confirmation prompts); keep the structure of `[header] format`;
  `[rank]` uses the Polish letters (`W` jack, `D` queen; `A`, `K` and the digits stay, which is the
  one allowed exception to "no value identical to English"); `menu.toml` is filled by P3-04, which
  must be merged before this card starts.
- **Do:** translate every value from the matching `locales/en-US/` file into natural Polish. Keep
  keys, placeholders such as `{name}` and key order unchanged. Avoid wording that needs a plural
  form of a number.
- **Done when:** the locale parity test passes; no value is identical to the English one unless it
  is a proper name.
- **Done:** M-17 #54, M-34 #62

## Artwork

### M-19 … M-24 — Vector glyphs

- **Status:** READY · **Finalized by:** F-02 (D-06, T-16: glyph box, rules, naming, code points) · **Needs:** P5-01
  (`cargo xtask fonts` checks the files and builds the preview sheet)
- **Branch:** `feat/M-XX-art-<style>` · **Files:** only the SVG files of your card, listed below
- **Cards and files** (names are fixed by the code point table below):
  - M-19 International suits: `assets/cards/fronts/international/glyphs/{spade,heart,diamond,club}.svg`
    (the French skin reuses them).
  - M-20 German suits: `assets/cards/fronts/german/glyphs/{acorn,leaf,heart,bell}.svg`.
  - M-21 Italian suits: `assets/cards/fronts/italian/glyphs/{baton,sword,cup,coin}.svg`.
  - M-22 court emblems: `assets/cards/fronts/international/glyphs/{jack,queen,king}.svg` (the other
    skins reuse them).
  - M-23 back style 1, `lattice`: `assets/cards/backs/lattice/glyphs/{tile,mosaic}.svg`.
  - M-24 back style 2, `ornament`: `assets/cards/backs/ornament/glyphs/{tile,mosaic}.svg`.
- **Do:** draw each glyph as one hand-written SVG that follows the style guide below. Shapes must stay
  recognizable at one terminal cell and differ from each other by silhouette, not by color. Do not touch
  `codepoints.toml`, `style.toml` or any other file.
- **Done when:** `cargo xtask fonts` accepts every file and the glyphs appear in
  `target/fonts/preview.html`; attach a screenshot of the sheet to the PR and say that you checked the
  6 × 12 px row.

#### Style guide (shared by M-19…M-24)

**1. Glyph box.** The font is built on the Cascadia Mono cell: units per em 2048, advance 1200, ascent 1900,
descent 480, line gap 0. One terminal cell is therefore **1200 × 2380 font units**, about one to two.

- Single glyph: `viewBox="0 0 1200 2380"`. The y axis points down. The baseline is at y = 1900; the centre of
  the cell is (600, 1190). Draw in SVG coordinates and never flip anything: the builder maps (x, y) to the
  font point (x, 1900 − y).
- Mosaic (section 6): `viewBox="0 0 2400 4760"`, two cells wide and two high.
- No `width` or `height` attribute on `<svg>`.
- Scale to keep in mind: at the smallest cell, 6 × 12 px, **one pixel is about 200 units**; at 10 × 24 px it is
  about 120 units wide and 100 high.

**2. Safe area and padding.**

- Front glyphs (suits, court emblems): all ink inside **x 60…1140, y 60…2320** (60 units of padding on every
  side, so neighbouring cells never touch). Center the visual mass on (600, 1190).
- Fill the width. The X-01 prototype drew suits too small. A round glyph (heart, diamond, club, spade,
  acorn, bell, cup, coin) is **1000 to 1080 units wide**; a tall glyph (baton, sword, leaf, emblems) is
  at least 1000 units wide or at least 1700 units high. Height follows the shape; the glyph is centered
  vertically, not placed on the baseline.
- Back tiles are full bleed: the ink may reach the box edge, 0…1200 × 0…2380. A tile must be **seamless in
  both directions**: a shape that touches the left edge at height y touches the right edge at the same
  height and with the same thickness, and likewise top and bottom, so a grid of tiles forms one pattern.
- Nothing may lie outside the box. The builder rejects ink outside the box or, for front glyphs, outside
  the safe area.

**3. Fill, strokes and minimum feature size.**

- Allowed elements: `<svg>` with `xmlns` and `viewBox`, and one or more `<path d="…"/>`. Nothing else: no
  `<g transform>`, `<use>`, `<rect>`, `<circle>`, `<text>`, `<image>`, `<defs>`, gradients, clip paths,
  masks, filters, `style` blocks or `transform` attributes. Convert every shape to a path (in Inkscape:
  Object to Path, Stroke to Path).
- **Fill only, no strokes.** No `stroke…` attribute anywhere. A line is a thin closed shape.
- One color. `fill` is omitted or `#000`; no opacity. The theme colors the glyph in the terminal.
- Holes: a subpath wound opposite to the shape it cuts, with the default (nonzero) fill rule; do not use
  `fill-rule="evenodd"`. Overlapping subpaths of the same winding are fine, the builder unites them.
- **Minimum feature size, so that the shape survives at 6 × 12 px:**
  - every line-like part (rim, stem, blade, frame, bar) is at least **200 units** wide;
  - every hole and every gap that must show is at least 200 × 200 units; a gap smaller than that fills in,
    so either close it completely or open it to 200 or more;
  - a tip or corner narrower than 200 units for more than 150 units of its length disappears: blunt it to a
    width of 200 units or more (a pointed spade is fine because it widens quickly);
  - nothing is drawn smaller than 100 units, ever. Detail between 100 and 200 units may exist only as
    decoration that can vanish without changing what the glyph is.
- No hinting is applied; keep curves smooth and avoid hairline spikes.

**4. Silhouettes that differ without color.**

- Color is never the carrier. Hearts and diamonds must not be told apart from clubs and spades by red and
  black. Every glyph is judged as a solid black shape at 6 × 12 px (the preview sheet shows that row).
- The glyphs of one set (the four suits; jack, queen, king) must be told apart at 6 × 12 px as solid black
  shapes. They differ by outline: at least two of the width at the top, the width at the base, the
  height, a stem or none.
- Interior detail is optional and never what tells two glyphs apart.
- Suggested silhouettes (not binding): spade pointed top with a stem; heart; diamond as a rhombus; club with
  three lobes and a stem; acorn as a cup over an oval nut; leaf as a pointed oval with a stalk; bell flared
  with a clapper; baton as an upright knobbed stick; sword as a blade with a crossguard; cup as a chalice;
  coin as a disc with a rim; king a crown with three tips; queen a coronet with a round top; jack a cap with
  a plume.

**5. Naming and files.**

- Path: `assets/cards/<fronts|backs>/<style>/glyphs/<name>.svg`.
- `<name>` is lowercase ASCII, `[a-z][a-z0-9-]*`, exactly as in the table below. `<style>`: fronts
  `international`, `german`, `italian`; backs `lattice` (M-23) and `ornament` (M-24).
- The registry key of a glyph is `<fronts|backs>/<style>/<name>`, for example `fronts/german/acorn`. The same
  name may appear in two styles (the German and the International `heart`).
- One glyph, one file. Art that does not exist yet has no file; the builder skips its slot.

**6. The 2 × 2 mosaic.**

- Only the two back styles have a mosaic: `mosaic.svg`, `viewBox="0 0 2400 4760"`. It is **one drawing**
  that fills a block of 2 × 2 cells, used as a decoration at the larger cell sizes (about 10 × 24 px and up);
  the single `tile` must look good on its own.
- The builder cuts it into four glyphs by intersecting the drawing with the four cell boxes, at x = 1200 and
  y = 2380: top-left (x 0…1200, y 0…2380), top-right, bottom-left, bottom-right. They get the names
  `mosaic-tl`, `mosaic-tr`, `mosaic-bl`, `mosaic-br`. You draw the picture once and never draw or cut the
  four parts yourself.
- Keep the 60-unit padding from the outer edge of the whole 2400 × 4760 box.
- A feature either **crosses a seam with at least 200 units on both sides, or stays at least 200 units away
  from it**. Never start or end a shape exactly on a seam: that makes hairlines and gaps.
- The vertical seam (y = 2380) was the weak point in X-01: a ring and a frame split there. Make every part
  that crosses it at least 300 units thick, and prefer shapes that are wider than they are tall.

**7. Code point table.** Range **U+E6BC…U+E6FF, 68 code points** (BMP Private Use Area, nothing in plane 16).
A slot never changes once assigned. A new glyph takes a reserved slot of its group, then a spare one.
`assets/cards/codepoints.toml` (P5-01) holds this table; the prototype's slots E6C0…E6C8 are not carried over.

| Code point | Key | Card |
|---|---|---|
| E6BC | `fronts/international/spade` | M-19 |
| E6BD | `fronts/international/heart` | M-19 |
| E6BE | `fronts/international/diamond` | M-19 |
| E6BF | `fronts/international/club` | M-19 |
| E6C0 | `fronts/german/acorn` | M-20 |
| E6C1 | `fronts/german/leaf` | M-20 |
| E6C2 | `fronts/german/heart` | M-20 |
| E6C3 | `fronts/german/bell` | M-20 |
| E6C4 | `fronts/italian/baton` | M-21 |
| E6C5 | `fronts/italian/sword` | M-21 |
| E6C6 | `fronts/italian/cup` | M-21 |
| E6C7 | `fronts/italian/coin` | M-21 |
| E6C8…E6CF | reserved: more suit sets (8) | — |
| E6D0 | `fronts/international/jack` | M-22 |
| E6D1 | `fronts/international/queen` | M-22 |
| E6D2 | `fronts/international/king` | M-22 |
| E6D3…E6D7 | reserved: more court emblems (5) | — |
| E6D8 | `backs/lattice/tile` | M-23 |
| E6D9 | `backs/lattice/mosaic-tl` | M-23 |
| E6DA | `backs/lattice/mosaic-tr` | M-23 |
| E6DB | `backs/lattice/mosaic-bl` | M-23 |
| E6DC | `backs/lattice/mosaic-br` | M-23 |
| E6DD | `backs/ornament/tile` | M-24 |
| E6DE | `backs/ornament/mosaic-tl` | M-24 |
| E6DF | `backs/ornament/mosaic-tr` | M-24 |
| E6E0 | `backs/ornament/mosaic-bl` | M-24 |
| E6E1 | `backs/ornament/mosaic-br` | M-24 |
| E6E2…E6FF | spare: further back styles and decorations (30) | — |

Count: 12 suit glyphs + 3 court emblems + 10 back glyphs = **25 assigned**; reserved 8 + 5 and spare 30 = **43**;
25 + 43 = 68, the whole range.

## Sound

### M-25, M-26, M-27 — Sound styles and music

- **Status:** READY · **Finalized by:** P6-03
- **Branch:** `feat/M-XX-sound-<style>` · **Files:** `assets/audio/<style>/{style.toml,scores/*.txt,*.mid}`
- **Notation:** header lines `tempo <bpm>` and `program <n>` (0–127), then whitespace-separated tokens `<note>:<length>` (e.g. `C4:8 E4:8 G4:4`; notes C D E F G A B with optional `#` or `b`, octave -1…9; length 1, 2, 4, 8, 16, 32, optional trailing `.` for dotted), rests `R:4`, chords `C4+E4+G4:2`. Lines starting with `#` are comments.
- **Cue list:** pick, place, flip, deal, illegal, undo, hint, win, menu move, menu select.
- **Length limits:** sound cues under 1.0 s (win cue under 3.0 s); looping music track at most 60 s.
- **Cards:** M-25 and M-26 — one sound style each: a score for every cue (pick, place, flip, deal,
  illegal, undo, hint, win, menu move, menu select) and the timbres in `style.toml`. M-27 — one
  looping music track of at most 60 seconds for each style.
- **Do:** write the scores, compile them with `cargo xtask midi <style>`, commit scores and `.mid`
  files. Cues are short (under one second, the win cue under three) and distinguishable from each
  other by contour, not only by pitch.
- **Done when:** the command compiles without errors and the style loads in the game's settings.
- **Done:** M-25 #52, M-26 #53, M-27 #52, #53

## Tests

### M-28, M-29 — Additional E2E scenarios

- **Status:** DRAFT · **Finalized by:** P7-01 (harness API and one worked example)
- **Branch:** `test/M-XX-e2e-<topic>` · **File:** one new file under `tests/e2e/`
- **Cards:** M-28 — Spider (deal a row, move a run, undo) and FreeCell (cell, foundation, a
  multi-card move, a rejected oversized move). M-29 — Pyramid (a pair, a king, stock passes) and
  settings (language and theme persist across a restart).
- **Do:** write the scenarios with the harness, on fixed seeds, by copying the worked example.
- **Done when:** `cargo test --test e2e` passes.

## Wiki: player pages

### M-30 … M-33 — Player documentation

- **Status:** M-30, M-31 and M-33 READY (finalized by P3-09, 2026-11-20); M-32 DRAFT until P5-02 merges (the card
  styles and the graphics check do not exist yet).
- **Branch:** `docs/M-XX-wiki-<page>` · **File:** one page under `docs/wiki/` (the Polish copy follows the
  glossary in `docs/tech/pl-glossary.md` when one is written)
- **Cards:** M-30 installation and requirements; M-31 controls (keyboard and mouse, from the key table);
  M-32 decks, themes and sound; M-33 accessibility (what is supported and the limits of a TUI — no claim of
  screen-reader support) and FAQ.
- **Do:** describe only what the named sources show. No feature that is not in the game; a planned one (color-vision
  modes, the card font, card styles) is not described as present. Quote a number only from its source.
- **Done when:** every statement can be traced to a source named in the card.

**M-30 sources (final behavior).**

- Screen: the terminal must be at least 100 × 30 cells, otherwise only a notice with the current and the required
  size is drawn (`crates/tui/src/widgets/too_small.rs`, `locales/en-US/common.toml` `[size]`). The side panels
  (guide, possible moves) need 130 columns; below that the same content opens in windows (`docs/tech/architecture.md`,
  "Board guidance" and "Size classes"). A double-clicked Windows build widens its window to 130 columns or goes full
  screen (`README.md`, "What it is").
- Install: the download, the build from source and the Rust and C toolchain notes are in `README.md` ("Install");
  Linux builds have no sound (`architecture.md`, "Audio"). Archive names come from `.github/workflows/release.yml`.
- Starting: `oxysol --help` is the text of `locales/en-US/cli.toml` `help`; `src/cli.rs` for the options (`--ascii`,
  `--colors`, `--seed`, `--fullscreen`, `--no-fullscreen`, `--new-window` / `-n`, `--version`, `--help`).
- Terminal capabilities and fallbacks (colors 16/256/true color, Unicode or ASCII): `crates/tui/src/caps.rs`,
  `crates/tui/src/glyphs/`. Windows double-click, console and full screen: `src/console.rs`, `src/console/display.rs`,
  `architecture.md` ("Full screen").
- Files: `OXYSOL_HOME`, the `oxysol.portable` marker with its `data` folder, `oxysol.db`, `oxysol.log` and the
  `OXYSOL_LOG` levels are in `src/paths.rs` and `src/logging.rs`; a corrupt database is renamed to `*.corrupt`
  (`architecture.md`, "Persistence").
- Card font: do not write the installation of the font until P5-02 and O-34 are merged; say only that cards are
  drawn with Unicode or ASCII characters now.

**M-31 sources (final behavior).**

- Keys: every row of `KEY_TABLE` in `crates/tui/src/input.rs` (28 rows), grouped as the Keys tab of the help window
  groups them (`locales/en-US/help.toml` `[group]`, `[action]`, `[key]`). Generate the table from the code, do not
  retype it; a key shown there is the single truth. Tab and Shift+Tab cycle the focus on every screen
  (`architecture.md`, "Focus").
- Mouse: a left click on a card picks it up, a second click on a target drops it, a click on the stock draws, a
  double click moves a card to the best place, a right click puts the picked cards back, press-and-release on another
  pile also moves (`locales/en-US/help.toml` `[mouse]`, `crates/tui/src/mouse.rs`, `crates/tui/src/screens/board/mouse.rs`).
  The wheel scrolls lists and windows; every footer hint and dialog button is clickable
  (`crates/tui/src/widgets/footer.rs`, `crates/tui/src/accessibility_tests/footer.rs`).
- Known gaps to list as limits, not as features (P3-09 findings): `p` (pause) and, below 130 columns, `v` (possible
  moves window) have no button; they are keys only.

**M-33 sources (final behavior).**

- Supported: every action by keyboard and by mouse; the focus marker `►` (`>` in ASCII) and reverse video on every
  screen; selection, targets, hints and the four suits told apart without color; the one-line `Now:` text of the
  controls strip; five themes with contrast targets (4.5, High contrast 7.0); WGL4 characters only in Unicode mode and
  an ASCII mode (`architecture.md`, "Accessibility", "Themes", "Focus"; `crates/tui/src/accessibility_tests/`,
  `crates/tui/src/theme/`, `crates/tui/src/glyphs/`).
- Limits to state plainly: it is a terminal program; no claim of screen-reader support (the `Now:` text is plain text
  in a fixed place, nothing more); color-vision modes and a low-vision mode are planned (P3-34), not built; sound
  duplicates visible feedback and is optional (`architecture.md`, "Accessibility", "Audio"; `locales/en-US/settings.toml`
  `[audio]` for why sound is unavailable).
- FAQ topics, each with its source: where the files are kept (`src/paths.rs`); how to replay or share a deal (seed,
  `c` copies it; `architecture.md` "Copy the seed"); what "Winnable" means and its limits (`architecture.md`
  "Winnable deals"); what Restart does without undo (`architecture.md` "Restarts"); why a screen shows a size notice;
  how to change the language (Settings, `locales/en-US/settings.toml`).

**M-32 sources (will be final with P5-02).** Themes: `crates/tui/src/theme/` and `locales/en-US/settings.toml`
`[theme]`; sound styles and the cue list: `assets/audio/README.md`, `assets/audio/<style>/style.toml`,
`architecture.md` ("Audio"); card styles and the graphics check wait for P5-02.

### M-35 — Polish copy edit and glossary

- **Status:** DONE · **Branch:** `docs/M-35-pl-copy-edit`
- **Files:** `locales/pl-PL/*.toml`, `docs/tech/pl-glossary.md` (new), and only the tests that pin
  the changed Polish strings (render or locale tests).
- **Do:** set the values below exactly (keys and placeholders unchanged; en-US untouched). Then
  write `docs/tech/pl-glossary.md` with the glossary table below and one line saying that new
  Polish texts must use it.
- **Values** (`file: key` → new text):
  - `menu: seed.error_out_of_range` → `To ziarno jest za duże. Maksymalna wartość to 18446744073709551615.`
  - `common: select.undo_hint` → `{key} lub kliknięcie przełącza cofanie w następnej grze`
  - `common: variant.spider_one / spider_two / spider_four` → `Pająk, 1 kolor` / `Pająk, 2 kolory` / `Pająk, 4 kolory`
  - `common: variant.pyramid_relaxed / pyramid_classic` → `Piramida, swobodna` / `Piramida, klasyczna`
  - `common: dialog.ok` → `OK`
  - `common: error.storage` → `Zapis do bazy danych nie powiódł się. Gra toczy się dalej bez zapisu.`
  - `settings: value.on / value.off` → `Tak` / `Nie`
  - `settings: audio.remote` → `Dźwięk jest niedostępny: sesja zdalna.`
  - `settings: audio.not_built` → `Ta wersja programu nie zawiera obsługi dźwięku.`
  - `settings: audio.style_failed` → `Dźwięk jest niedostępny: nie można wczytać wybranego stylu dźwięku.`
  - `stats: footer` → `Góra Dół Wybór   Enter Zagraj ponownie   c Kopiuj ziarno   Esc Wstecz`
  - `stats: empty` → `Nie rozegrano jeszcze żadnej gry. Rozpocznij nową w menu głównym.`
  - `stats: copy_failed` → `Nie udało się skopiować ziarna: {seed}`
  - `board: name.foundation` → `Stos końcowy {number}`
  - `board: status.picked` → `Wzięto karty. Wybierz cel lub naciśnij Esc, aby odłożyć.`
  - `board: status.hint` → `Podpowiedź: podświetlone karty można przenieść.`
  - `board: status.seed_copy_failed` → `Nie udało się skopiować ziarna: {seed}`
  - `board: error.invalid_sequence` → `Niedozwolony ruch: te karty nie tworzą poprawnego ciągu.`
  - `board: error.empty_column_blocks_deal` → `Niedozwolony ruch: zapełnij wszystkie puste kolumny przed rozdaniem rzędu.`
  - `board: error.not_a_pair` → `Niedozwolony ruch: suma wartości tych kart nie wynosi 13.`
  - `board: confirm.restart` → `Rozdać tę grę od początku?`
  - `board: action.quick_foundation` → `{key}: na stos końcowy`
  - `board: action.run` → `{keys}: długość ciągu`; `board: guide.run` → `długość ciągu`
  - `board: guide.goal.klondike` → `Ułóż cztery stosy końcowe od asa do króla, kolorami.`
  - `board: guide.goal.freecell` → `Ułóż stosy końcowe od asa do króla, kolorami.`
  - `help: mouse.click_card` → `Weź, a potem połóż na celu`
  - `help: mouse.right_click` → `Odłóż karty`
  - `help: mouse.drag` → `Przenieś kartę na cel`
  - `help: command_line.footer` → replace `Tab: następna karta` with `Tab: następna zakładka` (rest unchanged)
  - `rules: control.select_pair` → `Wybiera wolną kartę, a potem drugą, aby usunąć parę.`
  - `rules: control.back` → `Odkłada wybrane karty albo opuszcza grę.`
  - `rules: control.quick_move` → `Przenosi karty spod kursora w najlepsze miejsce.`
  - `rules: control.mouse` → `Myszą: kliknij kartę, a potem cel; dwuklik przenosi kartę w najlepsze miejsce; kliknięcie talii dobiera kartę.`
  - `rules`: every `stos bazowy` / `stosy bazowe` / `stosach bazowych` → the same form of `stos końcowy`
    (`klondike.goal`, `klondike.moves_3`, `klondike.tips_2`, `spider.goal`, `freecell.goal`)
  - `rules: klondike.layout_3` → `W trybie „1 karta” dobierasz po jednej karcie, w trybie „3 karty” po trzy, a zagrać można tylko górną kartę stosu odrzuconych.`
  - `rules: klondike.moves_2` → `Uporządkowany ciąg odkrytych kart przenosi się razem. Na pustą kolumnę można położyć tylko króla albo ciąg od króla.`
  - `rules: klondike.tips_4` → `Nie widzisz ruchu? Naciśnij {hint}, aby zobaczyć podpowiedź, albo {undo}, aby cofnąć ruch.`
  - `rules: spider.moves_1` → `Kolumny buduj malejąco, w dowolnym kolorze, na przykład szóstkę kier na siódemce pik.`
  - `rules: freecell.moves_2` → `Odłóż kartę do wolnej komórki, aby nie przeszkadzała, i weź ją stamtąd, gdy będzie pasować.`
  - `rules: freecell.moves_3` → `Stosy końcowe buduje się kolorami od asa. Kart nie można z nich zabrać.`
  - `rules: pyramid.layout_4` → `Wariant swobodny pozwala przejść przez talię 3 razy i kończy się, gdy piramida jest pusta. Wariant klasyczny pozwala na 1 przejście i kończy się, gdy usuniesz wszystkie 52 karty.`
  - `rules: pyramid.tips_2` → `Zanim usuniesz kartę, sprawdź, czy druga karta nie jest potrzebna do innej pary.`
  - `cli: help` → in the `OXYSOL_HOME` line replace `(zawsze wygrywa)` with `(ma pierwszeństwo)`
  - `cli: terminal_failed` → `oXySol zakończył działanie z powodu błędu terminala. Szczegóły w pliku dziennika.`
  - `cli: output_failed` → `oXySol nie może zapisywać do terminala.`
  - `cli: log_unavailable` → `oXySol nie może zapisać dziennika w pliku {path}. Działa bez dziennika.`
  - `cli: panic` → `oXySol niespodziewanie zakończył działanie ({location}). Szczegóły w pliku {path}.`
- **Glossary** (English → Polish): foundation → stos końcowy; tableau / column → kolumna; stock →
  talia; waste → stos odrzuconych (short: Odrzucone); free cell → wolna komórka; run → ciąg; run
  size → długość ciągu; face-up / face-down → odkryta / zakryta; pick up / drop / put back → weź /
  połóż / odłóż; undo / redo → cofnij / ponów; seed → ziarno; Spider / Pyramid → Pająk / Piramida;
  Klondike / FreeCell → unchanged; Draw 1 / Draw 3 → 1 karta / 3 karty; Relaxed / Classic →
  swobodny / klasyczny (swobodna / klasyczna with Piramida); tab (UI) → zakładka.
- **Not in this card:** `board: action.browsing`, `action.browsing_run`, `action.carrying`,
  `action.cards` and `settings: value.unavailable` (placeholder grammar) belong to P3-22.
- **Done when:** the values are set, a key named above that does not exist is listed in the report
  (not invented), `cargo xtask check --quick` passes, the PR is open.
- **Deviation (accepted):** four board messages kept their short forms (40-column event limit), and the `stats: longest` heading became `Rekord`.
- **Done:** #84
