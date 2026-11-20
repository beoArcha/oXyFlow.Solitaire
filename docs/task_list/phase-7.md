# Phase 7 (0.7) — Hungarian cards and Scorpion

Goal: the third new card style after phase 4 (Hungarian suits) and the next new game (Scorpion). Version 0.7.x.

Exit criteria:

- P7-04 and P7-05 are DONE, each with its rules confirmed by Opus and its names checked against a cited source.

Order of work: the card style, then the game.

### P7-04 — Card style: Hungarian suits (Unicode)

- **Owner:** Haiku
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

- **Owner:** Sonnet
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
