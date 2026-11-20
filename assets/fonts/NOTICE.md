# Font notice

## Base font

- Name: Cascadia Mono
- Version: Cascadia Code 2407.24 (GitHub release `v2407.24` of `microsoft/cascadia-code`, font version 2407.024)
- File: `ttf/static/CascadiaMono-Regular.ttf` from `CascadiaCode-2407.24.zip`, committed as
  `assets/fonts/base/CascadiaMono.ttf`. SHA-256: `06520d032ec274fa5040b22c6f4a1d829081b24ba40b2da56dae89bf10c7b481`.
  The variable font `ttf/CascadiaMono.ttf` of the release is not used.
- Licence: SIL Open Font License 1.1. The full text, with the copyright notice of the release, is in
  `assets/fonts/OFL.txt`.
- Metrics the card glyph box is built on: units per em 2048, advance 1200, ascent 1900, descent 480, line gap 0.

## oXySol Mono

`assets/fonts/oXySolMono-Regular.ttf` is a modified version of the base font, built by `cargo xtask fonts`
(D-06, T-16). Changes against the base:

- Card glyphs are added in U+E6BC–U+E6FF. Every base glyph and every base code point is kept unchanged.
- Family name `oXySol Mono` (name IDs 1, 4, 6, 16; unique ID 3). The Reserved Font Name `Cascadia Code` is not used.
  The trademark line that names it (name ID 7) is removed.
- One line for the card glyphs is added to the copyright entry (name ID 0). The licence entries (IDs 13, 14)
  are kept as they are.

The font is licensed under the SIL Open Font License 1.1, as its base (R-10). The card artwork sources are MIT,
see `LICENSE`. Release archives carry this notice and `OFL.txt` (follow-up O-35).
