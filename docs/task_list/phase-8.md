# Phase 8 (0.8) — Player documentation, Russian cards and Canfield

Goal: the player documentation (the wiki pages and the player site) and the stable release procedure are written against the features built so far; the Russian card style and the Canfield game are added. Version 0.8.x.

Exit criteria:

- P7-02 and P7-03 are DONE, and the micro cards M-30, M-31, M-32 and M-33 are DONE (M-32 covers the card styles);
- P8-04 and P8-05 are DONE.

Order of work: the documentation cards first, because they describe what the game does; the micro cards M-30…M-33 (`micro.md`) run alongside them. The card style and the game last.

Micro cards in this phase (micro.md): M-30, M-31 and M-33 (READY), M-32 (DRAFT until P5-02 is merged, which phase 4 does).

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

### P8-04 — Card style: Russian suits (Unicode)

- **Owner:** Haiku
- **Depends on:** P7-04
- **Files:** `assets/cards/fronts/russian/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The fifth new card style: the Russian suit names Крести, Пики, Черви and Бубны, with the court labels Валет, Дама and
    Король. The names and labels are checked against a cited source named in the PR. The suit symbols may be the French
    symbols when the symbols are the same; the style is then distinct by its names and labels, which is the point of it.
  - The same rules as P4-08 (ASCII letters, colour tokens, no font glyphs needed, colour never the only signal).
- **Acceptance:** the same tests as P4-08; the cited source is in the PR description; both locales carry the style name.

### P8-05 — Game: Canfield

- **Owner:** Sonnet
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a public-domain rules source the owner names)
- **Depends on:** P7-05
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/canfield/**`, `crates/core/src/solver/canfield*` (only if
  the solver is in this card), the variant list of `oxysol-core` where the games are matched,
  `crates/tui/src/screens/main_menu/**` (the entry only), `locales/*/rules.toml`, `locales/*/menu.toml`,
  `docs/wiki/rules/canfield.md` (new), `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:** Canfield as a module of `oxysol-core` on the game contract, with its reserve pile, its stock and the
  foundation rule as in the confirmed rules text. A solver only if it stays within the budget of T-29; otherwise the
  winnable option stays off for Canfield and the PR says so. Entry, rules tab and rules page as in P4-10.
- **Acceptance:** engine tests for the reserve, the stock and the foundations; the same seed gives the same deal; the
  variant is stored and read back; both locales.
