# Phase 7 (0.7) — Hungarian cards and Scorpion

Goal: the third new card style after phase 4 (Hungarian suits) and the next new game (Scorpion). Version 0.7.x.

Exit criteria:

- P7-04 and P7-05 are DONE, each with its rules confirmed by Opus and its names checked against a cited source;
- the screenshot renderer and batch runner P7-06 to P7-09 are DONE (decision T-43).

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
