# Phase 9 (0.9) — Final tests, polish and the stable release

Goal: the final round before the plan ends. A UI/UX test plan with keyboard and mouse journeys per screen and per game; locale fit at the minimum size; an accessibility audit; the E2E scenarios; the terminal matrix on real machines (U-03); the last card style (Portuguese) and the last game (La Belle Lucie); new themes; fixes and polish; the final review; and the stable 0.9 release that ends the plan. Version 0.9.x. The plan ends when this phase is done: nothing raises the minor after it, and 1.0.0 is a later decision of the owner (R-25).

Exit criteria:

- the U-03 table is filled in by the user, and every finding is fixed or listed as a known limitation in the player FAQ;
- the P8-02 report and the F-03 verdict are go;
- a `v0.9.<patch>` tag exists as a stable release with binaries for Windows, Linux and macOS, and the card fonts.

Order of work: the test plan first; then the E2E scenarios and the journeys; the fit and accessibility audits; the terminal matrix; the last card style, game and themes; then the fixes and polish, the final review, and the release.

Micro cards in this phase (micro.md): M-28 and M-29 (E2E scenarios, DRAFT until P7-01 is merged).

### P9-01 — UI/UX test plan

- **Owner:** Medium
- **Depends on:** P8-04, P8-05, P7-03
- **Files:** `docs/tech/ui-test-plan.md` (new)
- **Scope:** a plan, not code. For every screen (main menu, game menu, the board of every game, Settings and each of its
  sections, How to play with its three tabs, Statistics, the overlays: pause, win, dialogs and confirmations, the size
  notice) and for every game, write journeys with an ID (J-…): the start state, the keys, the clicks, the expected
  result. Each action that exists by keyboard and by mouse gets both paths. Each journey is marked automated (P9-02)
  or manual (U-03). The plan also names the minimum size (100 × 30), the 130 × 40 and 200 × 60 sizes, the nine locales
  (P9-03) and the accessibility audit items (P9-04).
- **Acceptance:** the plan lists every screen and every game; every journey has an ID, a start state and an expected
  result; Opus reviews it before P7-01 and P9-02 start.

### P7-01 — E2E harness and core scenarios

- **Owner:** Small
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

### P9-02 — Keyboard and mouse journeys per screen and per game

- **Owner:** Medium
- **Depends on:** P9-01, P7-01
- **Files:** `tests/e2e/**` (the journeys), `crates/tui/src/**/tests*` only where a journey needs a test hook
- **Shared files:** `crates/tui/src/app.rs` (only if a hook must be added there)
- **Scope:** implement the automated journeys of the plan as in-process E2E tests on the harness of P7-01. Every journey
  keeps its ID in the test name, so a failure names the journey.
- **Acceptance:** every automated journey of the plan passes in `cargo xtask check` on the three systems; no journey
  depends on real time or on the home directory.

### P9-03 — Locale fit at the minimum size, all nine languages

- **Owner:** Small
- **Depends on:** P9-01, O-57
- **Files:** `crates/tui/src/**/tests*` (layout tests over every complete locale), `locales/*/*.toml` (text fit only)
- **Shared files:** `locales/en-US/*.toml`, `locales/pl-PL/*.toml` (only if the source text itself must change)
- **Scope:** every screen and overlay of the plan is drawn at 100 × 30 in each of the nine complete locales and checked for
  truncation, overflow and overlapping labels. A fix shortens the text of that locale only, as O-42…O-48 did; a change to
  the English source is named in the PR and must not change the meaning.
- **Acceptance:** a test covers every complete locale on every screen of the plan; the PR lists each fix per locale;
  nothing is truncated at 100 × 30.

### P9-04 — Accessibility audit

- **Owner:** Medium
- **Depends on:** P9-01, P3-34, P3-35
- **Files:** `docs/tech/accessibility-audit.md` (new), tests, and the files named by the findings
- **Scope:** audit the game against the accessibility rules of AGENTS.md, item by item: every action by keyboard and by
  mouse (from the journeys of P9-01); focus and selection visible without colour in every theme and every colour-vision
  mode; colour is never the only carrier of information; every text pair and every graphic pair meets its target (4.5:1,
  3:1, and 7:1 for the low-vision mode); no information is given by sound alone; every control has a text label. Screen
  readers are not claimed. Each finding names the rule and the location. Small fixes are made in this card; larger ones
  become cards.
- **Acceptance:** the audit document lists every rule with its status (met, partial, missing) and evidence (a test or a
  file); every finding is fixed or listed as a known limitation in the player FAQ (M-33).

### U-03 — Terminal matrix

- **Owner:** User
- **Depends on:** P7-01
- **Scope:** run the game in each environment and record the result (OK, or the problem with a
  screenshot). Agents cannot verify this.

  | Environment | Start and exit | Symbols and colors | Keyboard | Mouse | Resize | Sound |
  |---|---|---|---|---|---|---|
  | Windows 10/11 CMD | | | | | | |
  | Windows PowerShell | | | | | | |
  | Windows Terminal | | | | | | |
  | EXE by double click (own window, no second window) | | | | | | |
  | Linux graphical terminal | | | | | | |
  | macOS Terminal | | | | | | |
  | SSH to Linux (musl build) | | | | | | n/a |
  | PuTTY | | | | | | n/a |

  In one environment also: every theme, both locales, every front and back style in each glyph
  mode, 16-color mode, a 100 × 30 window and a maximized one, both sound styles, keyboard-only and
  mouse-only play of one full game, terminal state after exit and after a forced panic.
- **Acceptance:** the table filled in here; problems described well enough to reproduce.

### P9-05 — Card style: Portuguese suits (Unicode)

- **Owner:** Small
- **Depends on:** P8-04
- **Files:** `assets/cards/fronts/portuguese/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:** the last new card style of the plan: the Portuguese suits Copas, Espadas, Ouros and Paus, with the court
  labels Valete, Dama and Rei. The names and labels are checked against a cited source named in the PR. The same rules as
  P4-08.
- **Acceptance:** the same tests as P4-08; the cited source is in the PR description; both locales carry the style name.

### P9-06 — Game: La Belle Lucie

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a public-domain rules source the owner names)
- **Depends on:** P8-05
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/belle_lucie/**`, the variant list of `oxysol-core`
  where the games are matched, `crates/tui/src/screens/main_menu/**` (the entry only), `locales/*/rules.toml`,
  `locales/*/menu.toml`, `docs/wiki/rules/la-belle-lucie.md` (new), `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:** La Belle Lucie as a module of `oxysol-core` on the game contract, with its fans and its foundation rule as in
  the confirmed rules text. No solver in this card; the winnable option stays off for it. Entry, rules tab and rules page
  as in P4-10.
- **Acceptance:** engine tests for the fans and the foundations; the same seed gives the same deal; the variant is stored
  and read back; both locales.

### P9-07 — New themes

- **Owner:** Small
- **Status:** DRAFT (the owner names the two themes and their public source before dispatch)
- **Depends on:** P3-36
- **Files:** `assets/themes/<name>.toml` (two new files), `docs/tech/decisions.md` (the credit, as T-32 gives it)
- **Scope:** owner request, phase 9: two more themes after the themes-as-data change of P3-36, each based on a public colour
  scheme whose licence allows the use, credited in `decisions.md` as T-32 credits Monokai and Dracula. Each theme maps every
  token, including the card faces, the aura, the 16-color fallback and the four-colour deck.
- **Acceptance:** both themes selectable and remembered; the contrast tests pass for them; renders of the board in both
  themes; both locales.

### P8-01 — Fixes and visual polish

- **Owner:** Medium
- **Depends on:** U-03
- **Files:** as required by the U-03 findings
- **Scope:** fix the matrix findings, each with a reproducing test where possible. Then a visual
  pass against the product priorities: readability of board and cards, consistent borders and
  spacing, hierarchy of header and footer, visibility of focus, the look at 100 × 30 and on a large window.
- **Acceptance:** every U-03 finding closed or documented as a known limitation in the wiki FAQ;
  `cargo xtask check` passes.
- **Out of scope:** new features.

### P8-02 — Final review

- **Owner:** Medium
- **Depends on:** P8-01
- **Files:** fixes across the repository
- **Scope:** review the whole project against `master_prompt.md` and `decisions.md`: architecture
  boundaries, error handling and logging (one log at the boundary, `source` kept), justification of
  every exception (`#[allow]`, `unsafe`, `lint-exceptions.toml`), comments, naming, needless
  abstractions and dependencies, accessibility rules, test coverage of MVP requirements. Every
  finding names a rule and a location; style preferences are not findings.
- **Acceptance:** a report: each MVP requirement with a status (met / partial / missing) and
  evidence (a test or a file); a list of known limitations; `cargo xtask check` passes.

### F-03 — Checkpoint: release sign-off

- **Owner:** Large (Huge only on escalation — R-14)
- **Depends on:** P8-02
- **Scope:** read the P8-02 report and the U-03 table, not the code. Decide: release, or the list
  of blocking items. Record post-MVP items (configurable per-game options, streaming music, raster
  graphics, mobile, more deck styles) as a short backlog in `decisions.md`.
- **Acceptance:** a go / no-go verdict with reasons.

### P8-03 — Stable 0.9 release that ends the plan

- **Owner:** Small + User
- **Depends on:** F-03, P7-02
- **Files:** `metadata.toml`, `Cargo.toml`, `Cargo.lock`
- **Scope:** Haiku: `metadata.toml` reads `0.9.0` from the PR that closed phase 8 (R-23, R-25); the version
  released is the current `0.9.<patch>` (patch = merges since then). User: wait for `main.yml` to promote to `release`, run
  `release.yml` by hand with `stable: true`, download the artifacts and start the binary on at least one system.
- **Acceptance:** tag `v0.9.<patch>` (stable); a GitHub Release with binaries for Windows, Linux and macOS and the
  card fonts; the web page updated.
