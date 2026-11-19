# X-01 — Spike: card font in the classic console

Status: **done for the classic console on Windows 10.** Windows Terminal is not installed on the owner's
machine, so it was not tested. macOS, Linux, PuTTY and SSH were not tested either. Their answers come
from documentation and are marked as such.

Tested on 2026-11-19. Owner tests were recorded by the reviewer (Opus) with the owner. The prototype
(fonts, SVGs, build script, test-sheet program) stays on the branch `spike/X-01-card-font` and is not
merged.

Markers used below:

- **(verified)**: observed in the classic console by the owner, see the screenshots in section 4.
- **(docs)**: from documentation or reviewer notes. Not exercised by the prototype.
- **(measured)**: measured on the development machine, not on the owner's machine.
- **(not tested)**: not run anywhere.

## 1. Scope

| Terminal | Status |
|---|---|
| Classic console (conhost) on Windows 10 | **tested**, section 4 |
| Windows Terminal | **not tested**: not installed on the owner's Windows 10 machine |
| macOS Terminal | **not tested** (out of scope) |
| Linux terminal | **not tested** (out of scope) |
| PuTTY | **not tested** (out of scope) |
| Over SSH | **not tested** (out of scope; rendering happens in the local terminal) |

## 2. The prototype on the branch `spike/X-01-card-font`

The branch is never merged and gets no pull request. It is kept for reference only.

| Path on the branch | What |
|---|---|
| `spikes/x-01/svg/*.svg` | six hand-written SVGs: four suits, back tile, 2 × 2 mosaic |
| `spikes/x-01/build_font.py` | font builder (Python, fontTools and skia-pathops) |
| `spikes/x-01/fonts/oXySolCardsSpike.ttf` | **variant P**: card glyphs and a space only |
| `spikes/x-01/fonts/oXySolCardsSpikeMono.ttf` | **variant M**: the card glyphs inside a copy of Cascadia Mono 2102.025, a complete monospace font |
| `spikes/x-01/x01_sheet.rs` | test-sheet program, `std` only |
| `spikes/x-01/install-font.ps1` | optional install and remove helper |

Glyph box: one cell is 1200 × 2380 units, baseline at y = 1900 from the top. In the font that is UPM 2048,
advance 1200, ascent 1900, descent 480, line gap 0, the metrics of Cascadia Mono (measured).

| n | Glyph | BMP | Astral twin |
|---|---|---|---|
| 0 | spade | U+E6C0 | U+10E6C0 |
| 1 | heart | U+E6C1 | U+10E6C1 |
| 2 | diamond | U+E6C2 | U+10E6C2 |
| 3 | club | U+E6C3 | U+10E6C3 |
| 4 | back tile | U+E6C4 | U+10E6C4 |
| 5–8 | mosaic top-left, top-right, bottom-left, bottom-right | U+E6C5…E6C8 | U+10E6C5…10E6C8 |

The mosaic SVG is 2400 × 4760 and is sliced into four tiles by a boolean intersection, so it is one drawing.

## 3. Test setup (classic console)

- The test sheet prints the font it reads back from the console and the cell size. The recorded runs
  read `oXySol Cards Spike Mono`, cell **6 × 12 px** and **10 × 24 px** (verified).
- The font was installed machine-wide with `install-font.ps1 -AllUsers -ConsoleRegistry` in an
  administrator shell. It was registered under `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Console\TrueTypeFont`
  (verified). The Properties dialog listed it only after that registry step.
- The per-user install was not tested in the classic console (not tested).
- Setting variant P (glyph-only) as the console font turned **all** text into squares, because it has
  no letters (verified).
- The Properties dialog offered only small sizes for this font. The 24 px size was set through the
  registry key (verified).
- Launch method for the recorded screenshots (double-click, CMD or PowerShell) and the Windows build and
  display scale were not recorded.

## 4. Results: classic console

**Configuration: classic console (conhost) · font `oXySol Cards Spike Mono` (variant M) · Windows 10 ·
build and display scale not recorded · cell sizes 6 × 12 px and 10 × 24 px (printed by the test sheet)**

| Section | What must be true | 6 × 12 px | 10 × 24 px | Screenshots | Notes |
|---|---|---|---|---|---|
| 1 Alone | each glyph between its brackets, nothing clipped | **OK** | **OK** | [6×12](x-01-screenshots/x01-ch-M-s1-6x12.png), [10×24](x-01-screenshots/x01-ch-M-s1-10x24.png) | All nine glyphs sit one cell wide, nothing clipped. The suits are drawn small inside the cell. |
| 2 One cell wide | all four rows end in the same column | not recorded | not recorded | — | Section not recorded. |
| 3 Rows | suits evenly spaced; back lattice continues; no gaps between rows | not recorded | not recorded | — | Section not recorded. |
| 4 Mosaic | frame, ring and dot form one picture; no gap, overlap or hairline | **partial** | **partial, close** | [6×12](x-01-screenshots/x01-ch-M-s4-6x12.png), [10×24](x-01-screenshots/x01-ch-M-s4-10x24.png) | 6 × 12: the mosaic reads as two boxes stacked; frame and ring are split. 10 × 24: the frame is one continuous rectangle; the ring is split into top and bottom halves; a short bar shows at the horizontal seam inside the ring. Horizontal joins fine at both sizes. |
| 5 Box drawing | glyph height and baseline agree with box-drawing characters | not recorded | not recorded | — | Section not recorded. |
| 6 Wide text | CJK, full-width, Polish and emoji neighbours do not move or clip the glyphs | not recorded | not recorded | — | Section not recorded. |
| 7 Astral | U+10E6C0… look like the BMP glyphs and take one cell | **fail** | not recorded | [6×12](x-01-screenshots/x01-ch-M-s7-6x12.png) | The astral twins render as squares. The astral row is wider than the BMP row. |
| 8 Colour | foreground colour; reverse video; tiles joined on a background | not recorded | not recorded | — | Section not recorded. |
| Sizes | section 1 and 4 at several sizes | 6 × 12 done | 10 × 24 done | — | The planned 10 / 16 / 24 pt steps were not run as such. Cell sizes 6 × 12 and 10 × 24 px were run. 16 px was not recorded. |

Other configurations:

| Configuration | Result |
|---|---|
| Windows Terminal, all profiles (WT-A to WT-D) | **not tested**: Windows Terminal is not installed on the owner's Windows 10 machine |
| Classic console, CMD and PowerShell windows | **not recorded** (the screenshots above do not state the launch method) |
| Classic console, variant P (glyph-only) as the font | **fail** (verified): all text became squares |

Screenshots of this run (the cell size is printed in the window header):

- [s1, 6 × 12 px](x-01-screenshots/x01-ch-M-s1-6x12.png): section 1
- [s1, 10 × 24 px](x-01-screenshots/x01-ch-M-s1-10x24.png): section 1
- [s4, 6 × 12 px](x-01-screenshots/x01-ch-M-s4-6x12.png): section 4 (mosaic)
- [s4, 10 × 24 px](x-01-screenshots/x01-ch-M-s4-10x24.png): section 4 (mosaic)
- [s7, 6 × 12 px](x-01-screenshots/x01-ch-M-s7-6x12.png): section 7 (astral)

## 5. Final answers to the six questions

### Q1. Where does the terminal pick up the font, and where must the player configure it?

**Answer:** In the classic console nothing picks the font up by itself. The font must be installed, and
it must be listed in the console's font registry before it can be selected. A per-user install was not
tested in the classic console. Windows Terminal lets the player choose a font per profile (docs, not tested).

- The console has one font for the whole window and no fallback list. The complete patched font (variant M)
  had to be selected; variant P (glyph-only) turned all text into squares (verified).
- The owner's runs selected variant M only after the registry step in section 3 (verified).
- The game can set the console font itself with `SetCurrentConsoleFontEx`, which also serves the zoom
  work of P3-16 (docs; not exercised by the prototype). The reviewer's notes add that the per-application
  console key can carry the face name and size (`FaceName`, `FontFamily`, `FontSize`) (docs; not exercised).
- Windows Terminal: `font.face` is set per profile. From 1.20.11381 (Preview 1.21) it may be a
  comma-separated list, and later faces are used for glyphs missing from the first (docs; not tested).

**Recommendation:** The player installs the font once. The game sets the console font itself on the
Windows classic console, as an opt-in in Settings, and only when the font is installed. Detecting that
it is installed is a proposal to check in P5-02 (for example, read back the face name after setting it).
Otherwise the game falls back to Unicode or ASCII glyphs.

### Q2. Are glyphs aligned to the cell, correctly scaled, one cell wide?

**Answer:** Yes for single glyphs in the classic console (verified, section 1). Each BMP glyph sits one cell
wide between its brackets, at 6 × 12 and 10 × 24 px, with nothing clipped. Row alignment (section 2) was
not recorded. The suits are drawn small inside the cell (verified), so they are not "correctly scaled"
for the whole cell yet.

**Recommendation:** Use single-cell glyphs for suits and the back tile. Make the artwork fill more of the
glyph box (the suits look small in the cell, verified) before P5-01 fixes the box. Record the row-alignment
check (section 2) at the next opportunity.

### Q3. Do multi-cell tiles join without gaps? Are full-card illustrations feasible?

**Answer:** Partly. Horizontal joins are fine at both sizes (verified). Vertical joins are visibly broken
at 6 × 12 px (two boxes stacked, frame and ring split). At 10 × 24 px the frame is one continuous
rectangle, but the ring is split and a short bar shows at the seam (verified). Full-card illustrations are
not feasible: only 68 BMP code points are free and plane 16 does not render (section 4, Q4).

**Recommendation:** Small mosaic decorations only (a 2 × 2 ornament, for example in a card corner), and only
at cell sizes of about 10 × 24 px or larger. No full-card illustrations. Test an overlap (bleed) of a few
units at the tile edges to close the vertical seam before relying on it (docs-based idea, not tested).

### Q4. Which code point range avoids collisions with common icon fonts?

**Answer:** In the BMP, use **U+E6BC–U+E6FF** (68 code points). Do **not** use plane 16. The prototype uses
U+E6C0–U+E6C8 inside this range.

- Plane 16 (U+10E6C0…) renders as squares in the classic console and the row is wider than the BMP row
  (verified, section 7).
- The collision scan (measured on the development machine) found E6BC–E6FF free of Nerd Fonts, Font Awesome 6,
  and the Windows-bundled Segoe MDL2 Assets and Segoe UI Symbol. The other free pockets (E0E5–E130,
  E959–EA5F, EC85–ECFF) are taken at least in part by Segoe fonts on Windows 10, so they are not safe to use.
- 68 code points hold about seven full sets of nine glyphs (four suits, back, four mosaic tiles). That is
  enough for suits, backs and small decorations only.

**Recommendation:** Reserve E6BC–E6FF in `codepoints.toml` (P5-01) and treat it as a fixed contract. Recheck
the range against the icon font versions the game ships, because Seti-UI and Font Awesome grow.

### Q5. Is a patched complete monospace font needed for terminals without font fallback?

**Answer:** Yes, for the classic console (verified). The console has one face and no list. Variant P
failed (all text squares); variant M rendered everything (verified). Windows Terminal 1.20.11381 or
newer with a face list does not need patching (docs; not tested).

**Licence note (not a legal review):** Variant M is a modified copy of Cascadia Mono 2102.025. The
installed font's name table (nameID 13) states an SIL Open Font License (OFL) style licence that allows
modification and redistribution if the licence text stays in the file and no Reserved Font Name is used.
A shipped patched font must keep the copyright and licence text, must not use a Reserved Font Name, and
needs a `NOTICE` file. Consolas is Microsoft's and must not be modified or redistributed. Confirm the base
font's licence before shipping any derivative.

**Recommendation:** Yes, ship a complete patched monospace font for the classic console. Choose the base
font for its licence. Metrics are copied from the base, so each base font needs its own build, and the
font must be rebuilt whenever the card art changes.

### Q6. Which build tool: a font library called from a script, or pure Rust in `xtask`?

**Answer:** Python with fontTools, called from `cargo xtask fonts`. This is a recommendation, not a
decision: it adds a second toolchain and must be approved in F-02.

What the prototype needed (`build_font.py`, about 300 lines):

1. SVG path parsing with arcs (`fontTools.svgLib.path`).
2. Union of overlapping contours and boolean intersection to slice the mosaic (`skia-pathops`).
3. Cubic to quadratic conversion for TrueType (`fontTools.pens.cu2quPen`).
4. A valid font: `cmap` (format 12 for astral code points), metrics, OS/2, name, post (`FontBuilder`).
5. Injecting glyphs into an existing font for variant M (`TTFont`).

| | Python with fontTools | Pure Rust in `xtask` |
|---|---|---|
| Effort | built and rendered in the prototype (measured) | not prototyped (estimate, not tested) |
| Curves, arcs, cubic to quadratic | built in | `kurbo`, `svgtypes` (not tested) |
| Boolean ops | `skia-pathops` wheel | a polygon-clipping crate or hand-written code (not tested) |
| Font writing | `FontBuilder` | `write-fonts` (not tested) |
| Patching an existing font | `TTFont` edits, works | no mature tool; the weakest point |
| New dependencies | Python, fontTools and skia-pathops (to be pinned), also in CI on three OSes | at least three crates; each needs approval under AGENTS.md |
| Failure mode | a missing Python or wheel fails the build on that platform | uniform, compile-time |

**Recommendation:** Python with fontTools behind `cargo xtask fonts`, because it builds and patches fonts
with the least code. Before adopting it, F-02 must approve Python in the toolchain and pin the
versions. If the owner refuses Python, pure Rust is possible only if the patched font is dropped, because
patching has no mature answer in Rust yet.

## 6. Recommendation for F-02 and D-06

Proposal for the owner and F-02:

1. Ship a **complete patched monospace font** with the card glyphs in the BMP private use area,
   **U+E6BC–U+E6FF** (no plane 16).
2. On the **Windows classic console**, the game sets the console font itself when the font is installed.
   This is an opt-in in Settings. Otherwise it falls back to Unicode or ASCII glyphs.
3. Scope of the artwork: **single-cell suit and back glyphs, plus small mosaic decorations**. **No
   full-card illustrations.**
4. Build with **Python fontTools from `cargo xtask fonts`**, subject to F-02's approval.
5. Before shipping: choose the base font and confirm its licence (section 5, Q5); test Windows Terminal
   on a real machine if it will be supported (not tested here); decide the glyph box with the suit size
   in mind (section 5, Q2).

What F-02 still has to decide:

- approval of Python in the toolchain (Q6);
- the base font and its licence (Q5);
- whether a derivative of the base font is published with the game;
- the glyph box and the suit size (Q2);
- whether Windows Terminal is a supported target, and if so, a test on a real machine.

## 7. What was not recorded

- Sections 2, 3, 5, 6 and 8 were not recorded in the classic console.
- Section 7 was recorded only at 6 × 12 px.
- The planned 16 px and 24 px sizes were not recorded (10 × 24 px was). Recorded cells were 6 × 12 and 10 × 24 px.
- Windows Terminal was not tested (not installed).
- The per-user font install in the classic console was not tested.
- `SetCurrentConsoleFontEx`, and the per-application console key, were not exercised by the prototype.
- The launch method (double-click, CMD, PowerShell), the Windows build and the display scale were not
  recorded for the screenshots.

## 8. Known limitations of the prototype

- No hinting. At small sizes diagonal edges look soft.
- The mosaic seams have no bleed (see Q3).
- The back tile is a plain lattice, and the suits are simple outlines, not artwork.
- Regular weight only. Bold text would use synthetic bold on the card glyphs.
- Variant M is built from Cascadia Mono 2102.025. Other versions may differ.
- The build steps and the full procedure are on the branch `spike/X-01-card-font`
  (`spikes/x-01/`). They are not repeated here.
