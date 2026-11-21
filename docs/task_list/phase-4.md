# Phase 4 (0.4) — Workflow, fixes, first card style and first game

Goal: the workflow is hardened first (the P1 and P2 findings of the November review), then the open fixes are closed, the card-style setting and the first new card style are in place, the game menu gets its structure, and the first new game (Yukon) is added. This is the phase worked on now, during version 0.4.x.

Exit criteria:

- every card below is DONE (`BLOCKED` with the owner's decision on the card if the owner stops it);
- O-34 (Windows Terminal, tested by the user on a real machine) and U-02 (artwork review) are confirmed by the user;
- `cargo xtask check` passes on `main` after each merge of this phase;
- the workflow cards O-50 to O-67 are all merged, so the P1 findings of the review are closed.

Order of work: the workflow cards first (P1 before P2 before P3, as the review ranks them), then the open fixes, then P5-02, the card-style setting that the first card of this phase introduces and O-34 tests, then the cards of the phase's own work. P5-02 moved here from phase 5 (R-25) because it is the card-style setting.
Micro cards in this phase (micro.md): M-19, M-20, M-21, M-22, M-23, M-24 (artwork, READY; reviewed by U-02).

### O-50 — Require up-to-date branches and auto-merge on main (repository setting)

- **Owner:** Owner
- **Depends on:** O-49
- **Files:** repository settings only
- **Scope:** S1 (P1): branch protection on `main` requires branches to be up to date before merging; enable auto-merge so a PR merges itself once green. Land first: every later PR is then tested against the latest `main`.
- **Acceptance:** The setting is visible in Settings → Branches; an out-of-date PR cannot merge until updated.
- **Done:** owner, 2026-11-20 (up-to-date branches, auto-merge, `check-windows`, `deny` and `site` required; Immutable releases on; one deploy key)

### O-51 — `check --quick` runs every test target in one cargo call

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/check*`
- **Scope:** B1 (P1): `--quick` skips the `e2e` and `architecture` targets although they cost under 0.1 s, and the split run compiles core and the TUI twice. Run all test targets in one cargo call.
- **Acceptance:** An e2e failure (like the O-36 version test) fails `--quick` locally; a warm `--quick` is not slower.
- **Done:** #143 (one `cargo test --workspace --tests` call; docs updated in this PR)

### O-52 — Lint: conflict markers in every tracked file

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/lint*`, tests
- **Scope:** B2 (P1): conflict markers are caught only in `.rs` files by the compiler. Add a lint over the tracked text files that fails on conflict-marker lines (seven `<`, `=` or `>` at the start of a line), naming file and line.
- **Acceptance:** A test with a fixture; the rule runs in `check --quick` and passes on `main`.
- **Done:** #142 (`=======` flagged only inside an open block)

### O-53 — Lint: decision ID uniqueness and references

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/lint*`, tests, `docs/tech/decisions.md` (ordering only)
- **Scope:** B3 (P1): parse the table rows of `decisions.md`; fail on a duplicate `R-`/`T-`/`D-` ID; warn on a reference under `docs/` to an ID that does not exist.
- **Acceptance:** Tests with fixtures; `check --quick` passes on `main`.
- **Done:** #155 (also accepts ID headings such as `### D-06`; unknown references warn)

### O-54 — `check` writes a result file; `cargo xtask report`; PR template

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/check*`, `xtask/src/main.rs`, `.github/pull_request_template.md`
- **Scope:** C1 (P1): `check` writes `target/check-result.txt` with the commit SHA, mode, steps, result and the last lines of a failing step; `cargo xtask report` prints it as Markdown for the PR body. A PR template with What / Files / Check result / Deviations / Limitations.
- **Acceptance:** Unit tests for the file format; the template shows on a new PR.
- **Done:** #153 (cargo output piped, so no colour or progress bar during `check`)

### O-55 — Pin GitHub Actions by SHA and add Dependabot

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/*.yml`, `.github/dependabot.yml`
- **Scope:** S3 (P1): pin every action by full commit SHA with the version as a comment; Dependabot for `github-actions` and `cargo`, weekly and grouped.
- **Acceptance:** Workflows still pass; the Dependabot config validates.
- **Done:** #145 (Dependabot weekly, grouped)

### O-56 — cargo-deny on pull requests

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `deny.toml`, `.github/workflows/pr.yml`, `docs/tech/decisions.md` (ID assigned in the card)
- **Scope:** S2 (P1): licenses (the T-31 list), advisories and sources checked on every PR, not only at release.
- **Acceptance:** `pr.yml` runs it; a disallowed license fails it.
- **Done:** #152 (T-37; transitive unmaintained crates do not block)

### O-57 — English fallback for incomplete locales and `cargo xtask locale-diff`

- **Owner:** Sonnet
- **Depends on:** O-49, O-41
- **Files:** `crates/tui/src/i18n*`, `xtask/src/**`, locale tests, `docs/tech/architecture.md`
- **Scope:** B6 (P1): an incomplete locale need not mirror every English key; missing keys fall back to en-US, so a new English key no longer touches nine folders. Complete locales keep full parity. `cargo xtask locale-diff <lang>` lists missing, extra and placeholder-mismatched keys.
- **Acceptance:** Parity enforced for complete locales only; locale-diff tests.
- **Done:** #151 (T-38; all nine locales complete, so new keys still go to every locale unless one is set `complete = false`)

### O-58 — Check pull requests on Windows as well

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/pr.yml`
- **Scope:** S6 (P2): Windows-only code (console font, full screen, widening) is compiled only on `main`. Add a cached Windows job to `pr.yml`.
- **Acceptance:** A PR shows both jobs; the owner updates the required checks.
- **Done:** #157 (`check-windows` job; the owner adds it to the required checks)

### O-59 — Tests derive row counts instead of hard-coding them

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `crates/tui/src/accessibility_tests.rs`, `crates/tui/src/screens/settings/tests/**`
- **Scope:** B7 (P2): settings and menu tests break whenever a row is added (fixed counts and step loops). Derive counts from `FIELDS` and find rows by field.
- **Acceptance:** Adding a settings row needs no test edit.
- **Done:** #144 (`pub(crate) mod options`; a new main menu item still needs a `MENU_KEYS` outcome, by design)

### O-60 — `cargo xtask worktrees`: list and remove worktrees of merged branches

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/**`, `docs/tech/workflow.md`
- **Scope:** C4 (P2): list worktrees whose branch is merged into `origin/main` or whose PR is closed, `--remove` to delete them, and report orphan directories.
- **Acceptance:** Tests with a temporary repository.
- **Done:** #159 (`cargo xtask worktrees [--remove [--force]]`; the T3 worktree folder is not scanned for orphans)

### O-61 — main: re-run jobs that never got a runner before failing

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/main.yml`
- **Scope:** B4 (P2): macOS jobs cancelled after about 15 minutes without a runner turn `main` red and skip promotion. Retry once, or tell a never-started job from a failed one.
- **Acceptance:** A documented retry path.
- **Done:** #154 (macOS runs as `check-macos`, outside `promote`'s needs; re-run by hand)

### O-62 — Build the site on pull requests; check screenshots are current

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/pr.yml`, `xtask/src/screenshots/**`
- **Scope:** B8 (P2): build the mdBook site on PRs that touch `docs/site/**`; add a staleness check for committed screenshots and the font.
- **Acceptance:** A broken link in a test PR fails the check.
- **Done:** #160 (`site` job builds mdBook and checks relative links when `docs/site/**` changes; screenshot staleness not done)

### O-63 — Take the version from the binary instead of a TUI build script

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `crates/tui/build.rs`, `build.rs`, `src/**`, the TUI version display
- **Scope:** F2 (P2): the git-derived version recompiles `oxysol-tui` after every commit, even docs-only ones. Compute it in the binary crate and pass it to the TUI.
- **Acceptance:** A docs-only commit does not rebuild `oxysol-tui`; the version shown is unchanged.
- **Done:** #162 (`crates/tui/build.rs` removed; `Deps::version` from the binary)

### O-64 — `cargo xtask preflight` and a local CI mode

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/**`, `docs/tech/workflow.md`, `AGENTS.md`
- **Scope:** C2, C3 (P2): `preflight [--card <ID>]` fetches `origin/main`, reports how far behind the branch is, runs the marker and decision-ID lints and lists changed files the card does not name.
- **Acceptance:** Tests with a temporary repository.
- **Done:** #161 (`cargo xtask preflight [--card] [--base] [--ci]`)

### O-65 — Release provenance, SBOM, immutable releases; pinned host key

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `.github/workflows/release.yml`, the promotion step, `docs/tech/workflow.md`
- **Scope:** S4, S5 (P2): provenance attestations and an SBOM per archive; immutable releases; a pinned GitHub SSH host key for promotion instead of a fresh `ssh-keyscan`.
- **Acceptance:** A release dry run shows the attestations; promotion works with the pinned key.
- **Done:** #163 (attestations, CycloneDX SBOM per target, pinned `.github/known_hosts`; the owner enables Immutable releases)

### O-66 — Optimize the engine in test builds

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `Cargo.toml` (profiles)
- **Scope:** F3 (P2): solver tests dominate debug test time; raise `opt-level` for `oxysol-core` in the dev/test profile.
- **Acceptance:** Test time before and after in the PR.
- **Done:** #146 (warm core tests 2m41s → 11s; warm `--quick` 1m58s → 28s)

### O-67 — Small cleanups: pycache, site redeploy, xtask split, apt cache

- **Owner:** Antigravity / Haiku
- **Depends on:** O-49
- **Files:** as named in the review (B9, C7, F4, F5)
- **Scope:** P3 items of `docs/tech/workflow-review-2026-11.md`, one PR each.
- **Acceptance:** Each item as in the review.
- **Progress:** B9, C7 and F5 done in #165 (manual `site.yml`); F4 (xtask compiles the TUI before the first gate) open
- **F4:** moved into P4-14 (light `xtask`)

### O-37 — Mouse path for Pause and Possible moves on every screen width

- **Owner:** Haiku
- **Depends on:** P3-16, P3-28
- **Files:** `crates/tui/src/screens/board/guidance.rs`, the board footer and moves panel, locales, tests
- **Scope:** P3-09 finding: `p` (pause) has no clickable control on the board and `v` (possible moves) has none
  below 130 columns, so both are keyboard-only (AGENTS.md: every action by keyboard and mouse). Give both a
  clickable entry (the `Do:` row or the footer, following its drop priorities) at every supported width.
- **Acceptance:** click tests at 100 × 30 and 130 × 40 for both actions; both locales.
- **Done:** #158 (`p: pause` and `v: moves` in the `Do:` row; the `Now:` row shows the ones the `Do:` row drops)

### O-38 — Settings save failure: alert and log once per session

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/app/effects.rs`, `crates/tui/src/screens/settings/tests.rs`, tests
- **Scope:** P3-09 finding, decided by the orchestrator 2026-11-20: `apply_settings` alerts and logs on every
  failed settings write, unlike game writes. Make it alert and log once per session, like game writes; later
  failures stay silent until a write succeeds again. Update the test that pins the old behaviour.
- **Acceptance:** tests for the first failure (alert and log), a second failure (silent) and recovery.
- **Done:** #148 (own `settings_write_failed` flag in `App`; a successful write re-arms the alert)

### O-39 — Split files over 300 LOC found by the P3-09 gate

- **Owner:** Antigravity
- **Depends on:** —
- **Files:** `crates/tui/src/app.rs`, `crates/tui/src/input.rs`, the oversized test files named in #121
- **Scope:** P3-09 finding (warnings only): `app.rs` (399) and `input.rs` (337) above 300 LOC; several test files
  above 300 and three card tests nested 5–6 deep. Split by responsibility without behaviour changes.
  `glyphs/wgl4.rs` is a data table and stays.
- **Acceptance:** no LOC or nesting warnings for these files; `cargo xtask check --quick` passes.
- **Done:** #164 (`app/event.rs`, `input/table.rs`, `input/tests.rs`; the oversized test files stay)

### O-35 — Release archives carry the font licence (OFL)

- **Owner:** Haiku
- **Depends on:** P5-01 (the font, `assets/fonts/OFL.txt` and `assets/fonts/NOTICE.md` exist)
- **Files:** `.github/workflows/release.yml`, `about.hbs`, `docs/tech/{decisions,workflow}.md`
- **Scope:** D-06 point 2. The game ships the oXySol Mono font, a modified copy of Cascadia Mono under the SIL
  Open Font License 1.1, and the licence must travel with it. `THIRD-PARTY-LICENSES.md` is generated by
  `cargo-about` (T-31), which lists crates and not fonts, so it does not cover the font: this card closes that
  gap. Every archive (normal and `-portable`, all four targets) gets a `fonts/` folder with
  `oXySolMono-Regular.ttf`, `OFL.txt` and `NOTICE.md` (copyright of the base font and of the additions, the
  family-name change, the base version). `about.hbs` gains a short fixed "Fonts" section naming Cascadia Mono, its
  copyright holder and the OFL and pointing to `fonts/OFL.txt`. The release job fails when one of the three
  files is missing from an archive. `decisions.md` T-31 gets a sentence that fonts are covered here, and
  `workflow.md` lists the new files among the archive contents.
- **Acceptance:** the archive listing of a local run of the packaging step shows the three font files in both
  archives of one target; the generated `THIRD-PARTY-LICENSES.md` contains the Fonts section; the check fails
  when `OFL.txt` is removed; the release workflow itself was not run (say so).
- **Done:** #150 (by Antigravity Dad; `cargo about` not run locally, release workflow not run)

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
- **Done:** #170, #171 (styles from `assets/cards/*/*/style.toml`; console font option and graphics check; the real console switch still to be checked by hand; Font mode has no glyph art until M-19…M-24)

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
- **Done:** owner, 2026-11-20 (oXySol Mono in Windows Terminal: menu, language list and statistics render correctly)

### U-02 — Artwork review

- **Owner:** User
- **Depends on:** P5-02, M-19…M-24
- **Scope:** open `target/fonts/preview.html`, then play one game in Font mode with each front and
  back. Accept each style or list the glyphs to redraw.
- **Acceptance:** every MVP style accepted, or a redraw list attached to the micro cards.
- **Done:** owner review 2026-11-20, LGTM (M-19…M-24 art); font rebuilt with 25 card glyphs in this PR

### P4-07 — Site: language landing page

- **Owner:** Sonnet
- **Depends on:** O-40, O-62
- **Files:** `docs/site/**`, `docs/site/theme/**` (only if the landing page needs a template or a stylesheet)
- **Scope:**
  - Today the player site (mdBook, https://beoarcha.github.io/oXyFlow.Solitare/) opens on the English
    introduction, and the sidebar starts with the English pages ("Introduction", "1. Features", …) without an
    "English" heading, while each of the other eight languages has its own heading. Make it consistent.
  - Add a language-neutral start page, listed first in `SUMMARY.md` and the page the site opens on: an
    animation of the nine flags (vector SVG under `docs/site/src/img/`), each flag with the word for solitaire in
    its language: English Solitaire, Polski Pasjans, Deutsch Patience, Español Solitario, Português (Brasil)
    Paciência, Українська Пасьянс, Nederlands Patience, Italiano Solitario, Français Réussite. Each word is checked
    by a native speaker before merge.
  - In the centre, a short description of the game in the language chosen from `navigator.languages` (the first
    entry that matches one of the nine by its language part; English when none matches), with links to that
    language's pages and to all other languages.
  - The English pages move under an `English` part heading in `SUMMARY.md`, in the same form as the other eight
    parts. No file is renamed unless mdBook needs it.
  - Without JavaScript the page is a static list of the nine languages with the English text. The animation
    respects `prefers-reduced-motion`: the flags stop and show their first frame.
  - Audit the nine language versions and fix what differs: page sets, link texts, the latest-build line, the
    language bar at the top of each page, and screenshot labels (`docs/site/src/img/en/menu.svg`, `help.svg` and
    the Polish images still show `v0.1.0`; the screenshots are regenerated by `cargo xtask screenshots`, not edited
    by hand). List every finding and its fix in the PR description.
- **Acceptance:** `mdbook build docs/site` passes (mdBook 0.4.52, as in O-40) and the link check passes; without
  JavaScript the start page lists nine languages in English; with JavaScript a browser set to Polish shows the Polish
  description (say how it was checked); `prefers-reduced-motion` stops the animation; no horizontal scroll at 320 px
  width; the audit list is in the PR description.
- **Done:** #166 (landing page with nine flags; words and descriptions corrected after a native review (German Solitär, French Solitaire))

### P3-29 — Settings: an Interface section (colors, aura, size, font)

- **Owner:** Sonnet
- **Depends on:** P3-16, P3-21, P3-24, P3-27
- **Files:** `crates/tui/src/settings.rs` and the settings screen, `crates/tui/src/widgets/aura.rs`,
  `crates/tui/src/layout/**`, `src/` (console font on Windows), locales, tests
- **Scope:** owner request 2026-11-19: Settings gets an "Interface" (UI/UX) section that gathers
  everything about look and readability in one place: theme (interface colors, P3-30), card back
  (P3-31), aura color, aura style (halo, frame, marks only) and aura size (thin or thick), interface
  size (card size and spacing, the setting of P3-16), font size and font choice. Font size and font face
  can only be changed by the program in the classic Windows console (`SetCurrentConsoleFontEx`, shared
  with P3-16; offer the fonts installed that cover WGL4, e.g. Consolas, Lucida Console, Cascadia Mono).
  In Windows Terminal, Linux and macOS terminals the rows show the terminal's own shortcut instead of a
  value and say why. Every change is previewed at once and kept only after confirmation.
- **Acceptance:** tests for each row (value, persistence, immediate preview, revert on cancel); the
  unsupported-terminal note; the console font change verified by hand in the classic console (say
  where); both locales.
- **Done:** #185, #188

### P4-06 — Automatic moves in statistics: failed attempts and streaks

- **Owner:** Sonnet
- **Depends on:** P4-05
- **Files:** `crates/core/src/session*` (counters only), `src/` (storage, schema migration),
  `crates/tui/src/screens/stats*`, `crates/tui/src/screens/board/**` (game facts), locales, tests
- **Scope:** owner request 2026-11-19: detect a game played mostly by the automatic moves. Count per game
  the automatic moves made (quick move, auto-move to the foundations, auto-finish counted apart),
  the failed automatic attempts (a quick move with no target), the manual moves, and the longest streak
  of automatic moves in a row. Store them with the game (schema migration with defaults). Statistics and
  history show the share of automatic moves, the failed attempts and the longest automatic streak, and
  mark a game as "assisted" when the share is above a threshold (a constant, e.g. 80 %, named in the
  decision log). The counters are information only; they never block a move.
- **Acceptance:** tests for each counter on scripted games, the streak, the migration, the "assisted"
  mark at the threshold; both locales.
- **Done:** #174 (schema v5 assist counters; assisted = quick moves above 80% of at least 10 moves (T-39))

### P4-08 — Card style: Swiss suits (Unicode)

- **Owner:** Haiku
- **Depends on:** P5-02
- **Files:** `assets/cards/fronts/swiss/style.toml` (new), `crates/tui/src/card_style.rs` (the rule for a front without
  font keys), `crates/tui/src/widgets/card/**` (tests only), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The first new card style of phase 4, on the card-style setting that P5-02 introduces: the Swiss suits Rosen,
    Schellen, Schilten and Eicheln, drawn with Unicode symbols, with the court labels Under, Ober and König.
  - A style may have no font glyphs. Font mode then draws the style's Unicode symbol and never a Private Use Area
    code point. P5-02's validation accepts a front without font keys; a test proves it.
  - Research the symbols: each suit needs a Unicode symbol that is in WGL4 or in the monospace fonts common to
    Windows Terminal, the classic console, macOS Terminal and GNOME Terminal. Record the coverage of each symbol in
    the PR description. Each suit also has an ASCII letter (suggested R, S, H, E) and a colour token. Colour is never
    the only way to tell two suits apart.
  - The style is selectable like the others. Adding it needs no change to the card drawing code.
- **Acceptance:** snapshot tests front × back × glyph mode × size; the four suits differ by shape in Unicode and in
  ASCII; the rank is readable at the smallest size; Font mode falls back to Unicode for this style; no colour outside
  the tokens; both locales carry the style name; the symbol table with its coverage is in the PR description.

### P4-09 — Game menu: stars, favourites, families and search

- **Owner:** Sonnet
- **Depends on:** P3-04, P3-17, O-59
- **Files:** `crates/tui/src/screens/main_menu.rs`, `crates/tui/src/screens/main_menu/**`, `crates/tui/src/settings.rs`
  (the favourites list only), `locales/*/menu.toml`, tests
- **Shared files:** `crates/tui/src/settings.rs`, `crates/tui/src/input.rs` (only if a key is added there),
  `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:** owner request, phase 4 plan: the game list grows, so it needs structure before the next game is added.
  - Games are grouped by family (the Klondike family, the Spider family, the FreeCell family, the Pyramid family and
    the families of the games added later), in a fixed order.
  - A star marks the most popular games (Unicode ★, ASCII `*`). The initial stars are Klondike, Spider, FreeCell and
    Pyramid; the owner may change the list before dispatch.
  - Favourites: `f`, or a click on the game's mark, puts a game in a Favourites group at the top. The list is kept in
    settings.
  - Search: `/` opens a search field; the list filters by game name and variant while the player types; Esc clears the
    field first, a second Esc leaves the menu as before.
  - Every action works by keyboard and by mouse. The focus and the selection are visible without colour.
  - No file goes over 300 lines.
- **Acceptance:** tests for the grouping order, the star marks in Unicode and ASCII, toggling a favourite and its
  persistence, the search filter and clearing it, and the keyboard and mouse paths; both locales; renders at 100 × 30
  and 130 × 40.
- **Done:** #173 (families, stars, favourites (`f`), search (`/`); no footer buttons for `f` and `/`)

### P4-10a — Game: Yukon (classic)

- **Owner:** Opus
- **Depends on:** P4-09, P1-02
- **Files:** `crates/core/src/games/yukon/**`, `crates/core/src/games/mod.rs` and the variant list where games are matched, `crates/tui/src/screens/game_select/catalog.rs` (the `Family` / variant entry), `locales/*/{common,rules,menu}.toml` (all nine locales), `docs/wiki/rules/yukon.md`, `docs/tech/architecture.md` (games list), tests
- **Scope:** Rules sources: [Yukon (solitaire)](https://en.wikipedia.org/wiki/Yukon_(solitaire)), [Russian Solitaire](https://en.wikipedia.org/wiki/Russian_Solitaire) and the PySolFC rule descriptions (owner choice, 2026-11-20).
  - One deck, no stock. Seven columns: column 1 one face-up card; columns 2–7 have 1…6 face-down cards with five
    face-up cards on top (21 face down, 31 face up).
  - Four foundations, ace to king by suit. Columns build down in alternating colours.
  - Any face-up card moves together with every card above it, in or out of sequence; only the bottom card of the
    moved group must fit. An uncovered face-down card turns face up. An empty column takes only a king or a group
    with a king at the bottom. No redeal.
  - Built so that the building rule and the deal are parameters of the Yukon module: P4-10b…e add variants
    without new move code. Undo, hints (P3-26 possible moves), auto-finish when forced (P2-07), statistics and
    autosave work as for the other games. Winnable deals stay off until P4-10f.
  - Menu entry in a new Yukon family (P4-09), a rules tab in How to play, and `docs/wiki/rules/yukon.md`.
- **Acceptance:** engine tests on scripted deals (group moves out of sequence, illegal moves, a face-down card
  turned over, king to an empty column, foundations, win); the same seed gives the same deal; storage round trip
  of the variant; the menu entry and rules tab in all nine locales; board fit at 100 × 30; architecture games list
  updated.

### P4-10b — Yukon variant: Russian Solitaire

- **Owner:** Haiku
- **Depends on:** P4-10a
- **Files:** as P4-10a (variant only)
- **Scope:** Yukon layout and moves; columns build down **in the same suit**. A variant of the Yukon family in the
  menu, with its rules tab section and wiki section. No new move code beyond the building rule parameter.
- **Acceptance:** tests for the same-suit rule (legal and illegal), storage round trip, menu and rules in all nine
  locales.

### P4-10c — Yukon variant: Alaska

- **Owner:** Haiku
- **Depends on:** P4-10b
- **Files:** as P4-10a (variant only)
- **Scope:** Yukon layout and moves; columns build **up or down in the same suit**. Menu entry, rules tab and wiki
  section as P4-10b.
- **Acceptance:** tests for building up and down by suit, storage round trip, menu and rules in all nine locales.

### P4-10d — Yukon variant: Moosehide

- **Owner:** Haiku
- **Depends on:** P4-10c
- **Files:** as P4-10a (variant only)
- **Scope:** Yukon layout and moves; columns build down in **any suit except the card's own**. Menu entry, rules tab
  and wiki section as P4-10b.
- **Acceptance:** tests for the rule (same suit refused, other suits accepted), storage round trip, menu and rules in
  all nine locales.

### P4-10e — Yukon variant: Australian Patience (stock, one pass)

- **Owner:** Sonnet
- **Depends on:** P4-10d
- **Files:** as P4-10a, plus the stock and waste drawing the board already has for Klondike (reuse, no copy)
- **Scope:** seven columns of four face-up cards; the remaining 24 cards form a stock dealt one card at a time to a
  waste pile, one pass only, no redeal; columns build down by suit; Yukon group moves; an empty column takes only a
  king (or a group with a king at the bottom). Stock counter as in Klondike (P2-06, P3-12). Menu entry, rules tab and
  wiki section.
- **Acceptance:** tests for the deal, the single pass, waste to column and foundation, group moves, storage round
  trip; board fit at 100 × 30; menu and rules in all nine locales.

### P4-10f — Solver for the Yukon family: winnable deals and the statistics check

- **Owner:** Opus
- **Depends on:** P4-10e, P2-08
- **Files:** `crates/core/src/solver/**` (or the solver module of P2-08), `crates/core/src/games/yukon/**`, the
  winnable-deals option and the statistics check wiring where a game opts in, `docs/tech/decisions.md` (the node
  budget and the unknown share, ID assigned by the orchestrator), tests
- **Scope:** a solver for Yukon, Russian Solitaire, Alaska, Moosehide and Australian Patience on the P2-08 solver
  contract (proof of a win or of no win within a node budget; unknown otherwise). Yukon deals are open except the
  face-down cards: the solver works on the hidden information the player has, as the Klondike solver does (T-29).
  Enables the winnable-deals option (P3-20) and the statistics check (P4-12) for the family. Measure the share of
  winnable / unwinnable / unknown deals per variant on a sample of seeds and record it.
- **Acceptance:** tests on known winnable and unwinnable seeds per variant; a benchmark (ignored test) with the shares
  and the time per deal; winnable deals on for the family; the decision row with the budget.

### P4-11 — Language selector: stray glyph, reachable with Up, Tab hint in the footer

- **Owner:** Haiku
- **Depends on:** P3-37
- **Files:** the main menu and language selector code under `crates/tui/src/screens/main_menu*`, the main menu footer, `locales/*/menu.toml` (all nine), tests
- **Scope:** owner finding 2026-11-20 (Windows Terminal, oXySol Mono):
  - With the drop-down list open, a stray `◄` is drawn left of the flag in the top-left corner, in addition to the
    selector's own `◄ flag ► ▼`; remove the extra glyph.
  - The selector cannot be reached with Up from the first menu row; Up on the first row (or on "Wznów") moves the
    focus to the language selector, and Down from the selector returns to the menu (wrap rules stay as they are
    otherwise).
  - The main menu footer does not say that Tab moves between the menu and the language selector; add a `Tab` entry
    (e.g. "Tab Język" / "Tab Language") following the footer's drop priorities.
- **Acceptance:** a render test with the list open shows no extra `◄`; key tests for Up to the selector and Down
  back; the footer shows the Tab entry at 100 × 30 in every offered locale.
- **Done:** #172 (the stray mark was the selector's focus marker; focus stays visible through the tooltip and the list's `►`)

### P4-12 — Statistics: a clear winnable column and a check for unchecked deals

- **Owner:** Haiku
- **Depends on:** P4-04, P2-08
- **Files:** `crates/tui/src/screens/stats/**`, `locales/*/stats.toml` (all nine), tests; the solver call goes through
  `Session`/core APIs only
- **Scope:** owner finding 2026-11-20: the history table's winnable column has only a `?` header, and almost every
  game shows `·` (not checked), so the player cannot see whether a deal was winnable.
  - Give the column a readable header (e.g. "Do wygr." / "Winnable") and keep the legend line visible below the table
    at every supported height.
  - Add an action on the statistics screen (key and mouse button, e.g. `w: check winnable`) that runs the solver for
    the selected game, or for all unchecked games in the background with a visible progress and a time limit per deal,
    and stores the result like the winnable-deals option does (T-29). Results: ✓ winnable, ✗ not winnable,
    ? unknown (limit reached).
- **Acceptance:** tests for the header and legend at 100 × 30; a test that the action turns `·` into ✓/✗/? for a known
  seed; the UI stays responsive while checking (background or step-wise).

### P4-13 — Process priority: `--priority` flag, Settings row, solver threads below the UI thread

- **Owner:** Sonnet
- **Depends on:** P3-29 (Settings rows; start after it is merged)
- **Files:** `src/` (argument parser, a new `src/console/priority.rs` or `src/priority.rs` next to the kernel32 FFI of
  `src/console/font.rs`), `crates/tui/src/settings*` and the settings screen, the solver worker in
  `crates/tui/src/app/deals.rs` (thread priority only), `locales/*` (all nine), `docs/tech/architecture.md`, tests
- **Scope:** owner request 2026-11-20: under heavy CPU load from other programs the game reacts slowly.
  - Five levels: Highest, High, Normal (default), Low, Lowest. Windows: `SetPriorityClass` with
    `HIGH_PRIORITY_CLASS`, `ABOVE_NORMAL_PRIORITY_CLASS`, `NORMAL_PRIORITY_CLASS`, `BELOW_NORMAL_PRIORITY_CLASS`,
    `IDLE_PRIORITY_CLASS`. Never `REALTIME_PRIORITY_CLASS`. Linux and macOS: `setpriority` with nice 0 / 5 / 10 for
    Normal / Low / Lowest; High and Highest need privileges, so the row says so and the priority stays unchanged
    (a failed call is reported once, not an error log per frame).
  - `oxysol --priority <highest|high|normal|low|lowest>` sets it for this run and wins over the setting; Help lists the
    flag with the other command-line options.
  - A "Process priority" row in Settings (Interface or a System group), saved and applied at start; a change applies at
    once (preview and confirm like the other rows of P3-29).
  - The solver threads (winnable deals, the statistics check of P4-12) run below the UI thread (Windows
    `SetThreadPriority(THREAD_PRIORITY_BELOW_NORMAL)`, Unix a higher nice for the thread where supported), so input and
    drawing stay responsive while a solver runs.
  - Behind a small trait (like `Console`) so tests use a fake; the FFI follows the existing `// SAFETY:` style.
- **Acceptance:** tests through the fake for each level on Windows and Unix mappings, the privilege note, the flag
  parsing (valid, invalid, flag beats setting), the setting persistence and the solver thread priority call; the real
  priority checked by hand in Task Manager (Details → Priority) — say what to check; all nine locales.
- **Done:** #189

### P4-14 — Docs-only pull requests run a docs check instead of the full build; light `xtask` (O-67 F4)

- **Owner:** Sonnet
- **Depends on:** O-62, O-64
- **Files:** `.github/workflows/pr.yml`, `xtask/Cargo.toml`, `xtask/src/**`, `xtask/src/affected.rs`,
  `docs/tech/workflow.md`, `docs/tech/architecture.md` (the xtask/CI table only)
- **Scope:** owner request 2026-11-20: a PR that changes only documentation (`*.md` anywhere, `docs/**` except
  `docs/site/**`, which the `site` job already builds; `LICENSE`, `.github/pull_request_template.md`) should not
  build and test the whole workspace.
  - `cargo xtask affected` reports a third state, docs-only. `check --pr <base>` then runs only `cargo xtask docs`:
    the conflict-marker and decision-ID lints (O-52, O-53) and a relative-link check over `docs/**/*.md`, `README.md`
    and `AGENTS.md` (every relative link and image target exists; anchors are not checked).
  - The required checks keep their names: `check` and `check-windows` still run and report success on a docs-only
    PR, they just take the short path (no `cargo build`, no Clippy, no tests). A PR that touches any other file runs
    the full check as today.
  - O-67 F4: `xtask` no longer compiles `oxysol-tui`, `oxysol-core`, `ratatui` and `resvg` for every command. Move the
    commands that need them (`screenshots`, `fonts`, `icon`) behind a cargo feature or a second binary, so `lint`,
    `docs`, `affected`, `preflight`, `report`, `version` and the `check` driver build quickly; `check` still builds the
    workspace itself through cargo. Measure a cold `cargo xtask lint` before and after.
- **Acceptance:** tests for the docs-only classification (`*.md`, `docs/**`, mixed with code → full); a test PR that
  changes one `.md` file shows `check` and `check-windows` passing in well under a minute; the link check fails on a
  broken relative link (fixture); cold `cargo xtask lint` time before and after in the PR.

## Done before the 2026-11 replan
- **Done:** #169 (`w` / `W` check one or all unchecked deals off the UI thread; header `Win?`)
- **Done:** #186, #190

### P4-01 — SQLite persistence

- **Owner:** Sonnet
- **Depends on:** P3-01
- **Files:** `Cargo.toml`, `Cargo.lock`, `src/{lib.rs,paths.rs}`, `src/storage/**`, `tests/integration/**`, `tests/architecture/**`
- **Scope (R-18, R-19, T-02):**
  - Add `rusqlite` (feature `bundled`) to the root package only.
  - `paths.rs`: directories from `directories::ProjectDirs::from("", "oXyFlow", "oXySol")`;
    `OXYSOL_HOME` points everything at one directory (tests, portable use).
  - `SqliteStorage` implements `oxysol_tui::Storage` over `oxysol.db` with the tables in
    `architecture.md`; schema in `PRAGMA user_version`, migrations as an ordered list of SQL
    strings applied in one transaction; every write in a transaction.
  - `start_game` inserts a `games` row (variant, seed, UTC start time) and returns its id;
    `finish_game` sets outcome, end time and elapsed time; `save_game` keeps one row tied to a game id.
  - Errors: a newer `user_version` → `StorageError::Incompatible`; not a database → `Corrupt` and the
    file is renamed to `oxysol.db.corrupt`; SQLite and I/O errors keep their `source`. Storage logs nothing.
  - Architecture test: `rusqlite` appears only in the root `Cargo.toml`.
- **Acceptance:** temp-dir tests: settings and game round trip; missing rows → `Ok(None)`; history
  rows with seed and outcome; a newer schema version; a corrupt file (error, file renamed, next open
  starts empty); a read-only location (error with `source`); `clear_game`; reopening the database
  keeps everything.
- **Done:** #31

### P4-02 — Logging, panic hook, entry point

- **Owner:** Haiku
- **Depends on:** P3-01, M-18
- **Files:** `src/{main.rs,lib.rs,logging.rs,panic_hook.rs,error.rs}`, `locales/*/cli.toml`, `tests/architecture/**`
- **Scope:**
  - `main`: parse arguments with `cli::parse` (M-18); `--version` and `--help` print localized text
    and exit; `--seed <u64>` starts straight into a new game with that seed (R-18); an argument error prints usage and exits with code 2; `--ascii` and `--colors`
    override detected capabilities.
  - `logging.rs` (T-24): `tracing-subscriber` writing to `oxysol.log` in the data directory; the
    file is truncated at start when above 1 MB; level from `OXYSOL_LOG` (default `info`). If logging cannot
    start: one warning on stderr before the TUI starts, and the app continues.
  - `panic_hook.rs`, in this order: `oxysol_tui::restore_terminal()`, a short stderr message with
    the panic location and the log path, a diagnostic record through a file handle opened earlier.
    No large allocations; the hook itself must not panic. Ignoring a write error inside the hook is
    the one permitted exception to the ignored-result rule, with a `reason`.
  - `error.rs`: `AppError` aggregating layer errors (`thiserror`). `main` returns `ExitCode`:
    0 success, 1 runtime failure, 2 usage error. A critical error is `tracing::error!` with target
    `oxysol::critical`.
  - No `windows_subsystem`, no `AttachConsole`/`AllocConsole` (decision T-10).
  - Architecture test: `src/main.rs` installs the panic hook before starting the TUI.
- **Acceptance:** a child-process test shows the hook leaving a log record and a non-zero exit code;
  `oxysol --version` prints the version from `metadata.toml`; an unknown argument exits with 2.
- **Done:** #37

### P4-03 — Autosave and resume

- **Owner:** Sonnet
- **Depends on:** P3-03, P4-01
- **Files:** `crates/tui/src/app.rs`, `crates/tui/src/screens/board/**`, `tests/integration/**`, `locales/*/common.toml`
- **Scope:**
  - A new game calls `start_game` (variant, seed) and logs the deal at `info`. Save the game
    (`SessionRecord` + elapsed time) after every session change and when leaving the board. A won
    game calls `finish_game(Won)` and removes the save; starting another game over a save calls
    `finish_game(Abandoned)`.
  - Startup: load settings (missing or failing → defaults and an alert). Resume rebuilds the session
    with `Session::restore` and restores the clock.
  - A corrupt, incompatible or unreplayable save: alert, remove the save, return to the menu.
  - A write failure during play: one alert per session, one log record, the game continues.
- **Acceptance:** integration tests with `SqliteStorage` in a temp dir: play → drop `App` → new `App`
  → Resume gives identical `piles()`, history position, seed and time; the history table holds the seed and outcome; a corrupt save leads to an alert
  and the menu; a storage that fails on write neither stops the game nor alerts twice.
- **Done:** #45

### P4-05 — Restarts counted separately in games without undo

- **Owner:** Sonnet
- **Depends on:** P4-04, P3-12
- **Files:** `src/` (storage, schema migration), `crates/tui/src/screens/stats*`,
  `crates/tui/src/screens/board/**` (header counter), locales, tests
- **Scope:** owner decision 2026-11-19: Restart stays allowed in games without undo, but it is counted
  separately. Each game record keeps a restart count (schema migration with a default of 0). The board
  header shows the count in no-undo games once it is above zero. Statistics show no-undo results
  without restarts apart from those with restarts (for example "wins without restart" next to all
  wins), and the history lists the restart count. Games with undo are unchanged.
- **Acceptance:** migration test from the current schema; restart increments the count only in no-undo
  games; statistics and history show it; both locales.
- **Done:** #96
- **Deviation (accepted):** the file list was widened to `crates/tui/src/storage.rs`, `storage/memory.rs`, `app/{effects,persist}.rs`, their tests and `architecture.md`; `App` calls `Storage::record_restart` for the board; the stats tables use a one-space gap to fit 100 columns; the Polish column is `Czyste`.

### P4-04 — Statistics, game history and seed copying

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-03
- **Files:** `crates/core/src/session{.rs,/tests.rs}`, `crates/tui/src/{clipboard,storage,app/deps,app/effects,app/persist,app/persist_tests}.rs`, `crates/tui/src/screens/{stats/**,main_menu.rs}`, `src/{clipboard.rs,storage/**}`, `locales/*/stats.toml`, `tests/integration/{migration,sqlite_storage}.rs`, `docs/tech/{architecture,decisions}.md`
- **Scope:** Schema v2 with a forward migration: `games.moves` (NULL for older rows) and `games.undo_allowed`; `Storage::start_game(variant, seed, undo_allowed)`, `Storage::finish_game(id, outcome, elapsed_ms, moves)` and `Storage::history()`. `Session::moves_made()` counts executed moves, undo, redo and restart. A Statistics item in the main menu shows a summary per variant and undo mode, and the history newest first, with "Play this seed again" and "Copy seed" (`c`). Copying uses `arboard` with an OSC 52 fallback. Dates are shown in UTC. Decision T-27.
- **Acceptance:** `cargo xtask check --quick` passed; the architecture and e2e suites also passed locally (PR #61).
- **Done:** #61

### O-05 — Keep a double-clicked console open after a startup error

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-02
- **Files:** `src/{console,lib,main}.rs`, `locales/{en-US,pl-PL}/cli.toml`, `docs/tech/decisions.md`
- **Scope:** After a failed run (usage, paths, storage, terminal or output error), a process started by double-click waits for Enter before its console closes. `console::owns_window()` detects a console owned by the process (`GetConsoleProcessList`, declared locally, no new dependency). `console::should_pause(failed, owns_window, stdin_is_terminal)` is a pure function with a unit test. Success, a run from an existing terminal and piped stdin never wait. Localized prompt `cli.press_enter`. T-10 gets one sentence on the exception.
- **Acceptance:** E2E: child processes with piped stdin never pause (4 passed). Manual on Windows: a run with `--bogus` opened its own console and stayed open; a run from an existing PowerShell exited with code 2 without waiting (PR #39).
- **Done:** #39

### O-07 — Portable mode: data next to the executable

- **Owner:** not recorded (follow-up)
- **Depends on:** P4-01
- **Files:** `src/paths.rs`, `docs/tech/{architecture,decisions}.md`, `README.md`
- **Scope:** Decision R-21. A copy is portable when a regular file `oxysol.portable` sits beside the executable; its database and log then live in `data/` beside it, and nothing is written to system folders. Precedence in `AppPaths::detect()`: `OXYSOL_HOME` (when set and not empty), then the portable marker, then the platform data directory. A failing `current_exe()` means not portable.
- **Acceptance:** Unit tests in `src/paths.rs`: marker gives `data/`, no marker gives the platform directory, `OXYSOL_HOME` wins, and a directory named `oxysol.portable` is not a marker; `cargo xtask check --quick` passed (PR #41).
- **Done:** #41

### O-09 — Application icon for the Windows executable

- **Owner:** not recorded (follow-up)
- **Depends on:** P0-01
- **Files:** `build.rs`, `Cargo.toml`, `Cargo.lock`, `assets/icon/**`, `xtask/src/{main,check,error}.rs`, `xtask/src/icon/**`, `xtask/Cargo.toml`, `docs/tech/decisions.md`
- **Scope:** The icon as vector sources (`assets/icon/oxysol.svg`, simplified `oxysol-32.svg` and `oxysol-16.svg`). `cargo xtask icon` renders them with resvg to 16…256 px and writes the committed `assets/icon/generated/oxysol.ico` and `oxysol-256.png`; `cargo xtask icon --check` compares pixels with a tolerance of 2 and runs in `cargo xtask check`. Root `build.rs` embeds the ICO and a version resource with winresource, only for Windows targets. New dependencies: resvg 0.50.0 (xtask) and winresource 0.1.34 (build dependency). Decision T-26.
- **Acceptance:** `cargo xtask check --quick` passes; on Windows the built executable shows the icon and version information; `icon --check` fails after a source SVG is edited without regenerating. Linux and macOS builds were not verified (PR #51).
- **Done:** #51

### O-11 — `--seed` starts the game directly; typed keys stay in text fields

- **Owner:** not recorded (follow-up)
- **Depends on:** P3-10
- **Files:** `src/{main,seed}.rs`, `crates/tui/src/{app,input,test_support}.rs`, `crates/tui/src/app/tests.rs`, `crates/tui/src/screens/main_menu.rs`, `locales/*/cli.toml`, `tests/integration/{audio,autosave}.rs`, `docs/tech/architecture.md`
- **Scope:** `oxysol --seed N` opens the game selection for seed N; picking a variant deals that game. The interim override that made only the first new game use N is removed; later new games get fresh seeds. While the seed field is open, a character key is text: `?` and `i` no longer open overlays there. F1 and Ctrl+C still work everywhere (`MainMenu::captures_text`, `is_text_key`). Not included: Backspace in `KEY_TABLE` (item 3 of O-11), because it needs a new `Action` variant that would change the board while P3-11 edits it; the seed field handles Backspace directly.
- **Acceptance:** Tests: `--seed` opens the games of that seed; Enter deals with the chosen variant; a later new game gets a fresh seed; Esc keeps the seed; `?` and `i` are typed inside the field and open overlays outside it; F1 still opens Help. `cargo xtask check --quick` passed (PR #58).
- **Done:** #58

### O-15 — `--new-window` flag and the command-line options in Help

- **Owner:** not recorded (follow-up)
- **Depends on:** O-05, O-14
- **Files:** `src/{cli,error,lib,main,new_window,new_window/tests}.rs`, `crates/tui/src/app/modal.rs`, `crates/tui/src/widgets/overlay/**`, `locales/*/{cli,help}.toml`, `docs/tech/{architecture,decisions}.md`
- **Scope:** `--new-window` (also `-n`) relaunches oXySol in a new console window with the same arguments minus the flag; the original process exits with code 0 at once. If no window can be started, a localized error names what was tried and the process exits with code 2. Windows uses `CREATE_NEW_CONSOLE`; Linux tries `$TERMINAL -e`, `x-terminal-emulator -e`, `gnome-terminal --`, `konsole -e` and `xterm -e`; macOS runs `osascript`. Command construction is pure and unit-tested; the spawn is not. `--help` is regrouped and fully translated to Polish. Help gets a third tab, "Command line", showing the `cli.help` text; Tab now cycles Keys, Rules and Command line. Decision T-28; the architecture contract changes.
- **Acceptance:** `cargo xtask check --quick` passed. Manual on Windows 10 (release build): a second console opened with the game, the parent exited with code 0, and the O-05 pause works in the relaunched console. Linux and macOS: construction tests only (PR #70).
- **Done:** #70

### O-19 — `--new-window`: alacritty, kitty, wezterm and foot on Linux

- **Owner:** not recorded (follow-up)
- **Depends on:** O-15
- **Files:** `src/new_window.rs`, `src/new_window/tests.rs`
- **Scope:** Adds alacritty (`-e`), kitty (no `-e`), wezterm (`start --`) and foot to the Linux fallback list, between konsole and xterm. The tests check the whole attempt sequence of all eight built-in terminals and the exact program names and arguments.
- **Acceptance:** `cargo xtask check --quick` passed (PR #72).
- **Done:** #72
