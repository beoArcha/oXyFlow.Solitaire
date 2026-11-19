# Phase 7 — E2E tests, documentation, deployment

Goal: the full player journey covered by tests on `main`, documentation for players and
developers, and a one-click deployment from `release`.

### P7-01 — E2E harness and core scenarios

- **Owner:** Haiku
- **Depends on:** P3-09, P4-02
- **Files:** `tests/e2e/**`
- **Scope:** in-process tests (decision T-12): a `Harness` combining `App`, a 120 × 40
  `TestBackend`, `SqliteStorage` in a temp dir, a recording `Audio` double and a fixed seed, with
  helpers to press keys, click cells and read the screen. Core scenarios:
  1. start → menu → quit;
  2. Klondike: new game → legal moves → undo → redo → hint;
  3. an illegal move shows a message and leaves the board unchanged;
  4. save → new instance → resume the same game; the same seed gives the same deal;
  5. a corrupt save → alert → menu;
  6. pause stops the clock;
  7. binary smoke test: `--version`, `--help`, `--seed`, an unknown argument (exit code 2).
  Finalize micro cards M-28 and M-29 with a worked example and mark them `READY`.
- **Acceptance:** all scenarios pass in `main.yml` on the three systems; tests depend neither on
  real time nor on the home directory.

### P7-02 — Player site and stable release procedure

- **Owner:** Haiku
- **Depends on:** P0-04, P5-01, M-30…M-33
- **Files:** `docs/site/**`, `.github/workflows/release.yml`, `docs/tech/workflow.md`
- **Scope:** the build, tag and release pipeline already exists (P0-04). Grow `docs/site/` into the
  player wiki (rules from `docs/wiki/rules/`, installation, controls, decks, accessibility, FAQ),
  add the card fonts to the release assets, and describe the stable release procedure in
  `workflow.md`: version in `metadata.toml` → `main` → promotion → `release.yml` with `stable: true`.
- **Acceptance:** the site builds locally with mdBook; the font files appear in a pre-release made
  from the PR's merge; state plainly what was observed on GitHub.
- **Out of scope:** a browser version of the game.

### P7-03 — Technical documentation and README

- **Owner:** Haiku
- **Depends on:** P3-09
- **Files:** `docs/tech/**` (except `master_prompt.md` and `spikes/`), `README.md`
- **Scope:** reconcile `architecture.md` with the code (signatures, layout). Add `conventions.md`
  (code rules, error handling, localization, themes, accessibility rules, lint exceptions) and
  `testing.md` (test categories, what runs on a PR and what on `main`, how to run each locally).
  Keep `decisions.md` current. `README`: description, installation, running, key summary, links to
  the wiki and releases.
- **Acceptance:** every signature and path in the documents exists in the code; nothing is stated
  twice across files; `README` promises nothing outside MVP.

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
