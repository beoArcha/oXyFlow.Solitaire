# Phase 5 — Card graphics

Goal: card fronts and backs drawn as vector art, delivered to the terminal as oXySol's own complete
monospace font (D-06, T-16), with a Unicode and an ASCII rendering for everyone who does not install it.
The delivery method was confirmed by spike X-01 and decided at checkpoint F-02.

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

### P5-02 — Style-driven cards and graphics check

- **Owner:** Haiku
- **Depends on:** P5-01, P3-05; the `Console` trait of P3-16 in `oxysol` (`src/`), which reads and sets the
  console font. If P3-16 is not merged when this card starts, say so in the report.
- **Files:** `crates/tui/build.rs`, `crates/tui/src/{card_style.rs,widgets/card.rs,screens/graphics_check.rs}`,
  the Font mode row of the settings screen, `src/` (console font, with the P3-16 trait),
  `assets/cards/**/style.toml`, `locales/*/{cards,settings}.toml`
- **Scope (D-06):**
  - `style.toml` for a front: per suit a name key, the font glyph (a registry key of `codepoints.toml`,
    resolved to its code point at build time), the Unicode symbol, the ASCII letter and a color token;
    labels for ace and court ranks. For a back: the tile glyph and the mosaic keys, the Unicode and ASCII
    patterns and color tokens.
  - A build script embeds every style folder and `codepoints.toml`; the settings screen lists what was
    found.
  - The card widget draws any front and any back in the three glyph modes at every size step. Font mode
    draws the Private Use Area glyphs of the oXySol font: suits, court emblems and back tiles. The 2 × 2
    mosaic decoration is drawn only at the Large and Huge card sizes (and, on the classic console where
    the cell size is known, only from 10 × 24 px); elsewhere the single tile is used.
  - Write the manifests for fronts `international` (the default), `french`, `german`, `italian` and
    the two backs `lattice` and `ornament`. A skin may reuse glyphs of another style: `french` uses the
    International suits with its own court labels; German and Italian use the International court emblems.
  - **Classic Windows console, console font option:** a Settings option, part of the Font render mode,
    "Use the oXySol font in this console". The game checks that the font is registered (set the face with
    `SetCurrentConsoleFontEx` and read it back with `GetCurrentConsoleFontEx`; the exact check is chosen
    here and verified by hand, X-01 did not exercise it), keeps the previous face and size, and restores
    them when the option is turned off and at exit. The size stays as it is. Not registered, or the console
    does not accept the face: the option says so, points to the installation page of the wiki and the mode
    falls back (below). The game never installs the font.
  - **Fallback:** when the font is not in use (not registered, not accepted, not the classic console and the
    player has not confirmed Font mode in the graphics check) the game draws the Unicode glyphs, never
    Private Use Area code points, which would show as squares.
  - **Windows Terminal** (environment variable `WT_SESSION`): the graphics-check screen and the Settings row
    show a localized hint to set the profile font face to `oXySol Mono`, or to a list that ends with it
    (`"font": { "face": "Cascadia Mono, oXySol Mono" }`; a list works from Windows Terminal 1.20.11381).
    This is from documentation, tested later in O-34.
  - Graphics-check screen: shows sample glyphs in Font mode and asks whether they look right; "yes"
    switches glyph mode to Font, "no" keeps Unicode and points to the installation page of the wiki.
- **Acceptance:** snapshot tests front × back × glyph mode × size; the four suits differ by shape in
  every mode; a rank is readable at the smallest size; adding a style folder needs no code change
  (a test style folder proves it); no color outside the tokens; every font key in a `style.toml` exists in
  `codepoints.toml` and lies in U+E6BC–U+E6FF; the console font decision (registered, not registered,
  rejected, restored) is tested through a fake `Console`; the Windows Terminal hint appears only with
  `WT_SESSION`; the real switch is verified by hand in the classic console (say where); both locales.

### U-02 — Artwork review

- **Owner:** User
- **Depends on:** P5-02, M-19…M-24
- **Scope:** open `target/fonts/preview.html`, then play one game in Font mode with each front and
  back. Accept each style or list the glyphs to redraw.
- **Acceptance:** every MVP style accepted, or a redraw list attached to the micro cards.

### O-34 — Windows Terminal: test the card font on a real machine

- **Owner:** User + Sonnet
- **Depends on:** P5-02
- **Files:** `docs/tech/spikes/o-34-windows-terminal.md` (new); the hint texts in `locales/*/{cards,settings}.toml`
  and the installation page of the wiki (M-30) only if the test shows them wrong
- **Scope:** D-06 point 4: Windows Terminal is a supported target from documentation, not yet tested because it
  is not installed on the owner's Windows 10 machine. When the owner installs it: install the font, set the
  profile font face as the in-game hint says (a single face, and a face list on 1.20.11381 or newer), and
  run the graphics check and a game in Font mode with each front and back. Record: glyph alignment and one-cell
  width, the 2 × 2 mosaic seams (vertical ones were the weak point in conhost), text next to box-drawing
  characters, the behavior without the font.
- **Acceptance:** a results table with screenshots in the form of X-01 section 4, this time with the Windows
  Terminal version, Windows build, display scale, font size and launch method written down; the hint is
  confirmed or corrected; Windows Terminal is marked tested or documented-only in `decisions.md` D-06.
