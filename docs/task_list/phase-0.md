# Phase 0 — Bootstrap

Goal: a GitHub repository where `cargo xtask check` passes, the PR flow works end to end, and every
shared file already has its final shape so later tasks do not collide.

### U-01 — Decisions, master prompt, repository

- **Owner:** User
- **Scope:**
  1. Done: the original master prompt is saved as `docs/tech/master_prompt.md` (Polish). P0-02
     turns it into the English reference.
  2. Done: license (MIT), visibility (public), Pyramid (two games), default deck style
     (International) — recorded as R-10…R-13 in `docs/tech/decisions.md`.
  3. Done: the repository is `github.com/beoArcha/oXyFlow.Solitare` (R-12).
  4. Still open: review the assumed defaults in `decisions.md`.
- **Blocks:** nothing.
- **Done:** before PR numbering

### P0-01 — Repository skeleton and workspace

- **Owner:** Haiku
- **Depends on:** U-01
- **Files:** the whole repository except `docs/`, `AGENTS.md`, `CLAUDE.md`
- **Scope:**
  1. `git init -b main`, `.gitignore` (`target/`, `*.log`, editor folders). Push only once the user
     has named the remote.
  2. `metadata.toml` with `[project]`: `organization = "oXyFlow"`, `repository = "oXyFlow.Solitare"`,
     `product = "oXySol"`, `binary = "oxysol"`, `version = "0.1.0"`, `description`, `license`, `repository_url`.
  3. `rust-toolchain.toml`: the installed stable channel (1.99.1) with `rustfmt` and `clippy`.
  4. Root `Cargo.toml`: workspace (`.`, `crates/core`, `crates/tui`, `xtask`), edition 2024,
     `[workspace.package]`, `[workspace.dependencies]` for what phases 0–4 need (see the package table
     in `architecture.md`; leave `midly` and `cpal` to phase 6) plus dev-dependencies `tempfile` and `toml`.
     Root package `oxysol` has `src/lib.rs` and `src/main.rs`.
  5. `[workspace.lints]`, inherited by every crate:
     - rust: `unsafe_code = "deny"`, `unused_must_use = "deny"`;
     - clippy, all `deny`: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`,
       `let_underscore_must_use`, `allow_attributes_without_reason`, `undocumented_unsafe_blocks`.
  6. `clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`,
     `allow-dbg-in-tests` set to `true`.
  7. Compiling crates `crates/core` (`oxysol-core`) and `crates/tui` (`oxysol-tui`). `src/main.rs`
     prints `oxysol <version>` from `CARGO_PKG_VERSION`.
  8. `.cargo/config.toml`: alias `xtask = "run -p xtask --"`.
  9. `xtask` subcommands:
     - `check [--quick]` — the steps listed under "Quality gates" in `architecture.md`; `--quick`
       skips the `architecture` and `e2e` test targets and `sync-metadata --check`;
     - `lint` — collects `.rs` files from `src/`, `crates/*/src`, `crates/*/tests`, `xtask/src`,
       `tests/`, runs every rule, prints `LEVEL rule path:line message`, exits 1 on any error.
       Types: `Severity { Warning, Error }`, `Finding { rule, severity, file, line, message }`,
       `SourceFile { path, text }`. Exceptions come from `lint-exceptions.toml`
       (`[[exception]] rule, path, reason`); an entry without `reason` is itself an error;
     - `sync-metadata [--check]` and `version` — stubs that succeed.
  10. Declared, empty modules that micro tasks fill in without touching shared files:
      `xtask/src/lint/{loc,nesting,comments,err_arms,err_types}.rs` (each
      `pub fn check(file: &SourceFile) -> Vec<Finding>` returning an empty vector),
      `xtask/src/sync_metadata.rs`, `tests/architecture/main.rs`, `tests/integration/main.rs`,
      `tests/e2e/main.rs`, `crates/tui/src/caps.rs`, `crates/tui/src/theme/{mod,ansi,contrast}.rs`,
      `crates/tui/src/widgets/{mod,logo}.rs`, `src/cli.rs`.
  11. `README.md` — name, one-line description, how to run the check.
  12. `LICENSE`: MIT (R-10), copyright holder `oXyFlow`.
- **Acceptance:** `cargo xtask check` and `cargo xtask check --quick` pass; `cargo run` prints
  `oxysol 0.1.0`; first commit on `main`; `release` branch created from it.
- **Done:** before PR numbering

### P0-02 — Master prompt in English with amendments

- **Owner:** Haiku
- **Depends on:** P0-01 (the Polish original must be in git history first)
- **Files:** `docs/tech/master_prompt.md`
- **Scope:** translate the file to English in place, faithfully, keeping section numbers. Then
  apply amendments R-01…R-09 from `decisions.md` in the sections they name, marking each changed
  passage with its amendment ID. Do not add requirements of your own.
- **Acceptance:** every original section present; every amendment applied exactly once and
  traceable; no statement contradicts `decisions.md`.
- **Done:** #1

### P0-03 — CI: pull requests, main, promotion

- **Owner:** Haiku
- **Depends on:** P0-01
- **Files:** `xtask/src/{main.rs,check.rs,affected.rs,error.rs}`, `.github/workflows/{pr.yml,main.yml}`, `docs/tech/workflow.md`
- **Scope (R-16, R-17, T-19, T-22, T-23):**
  - `check --quick`: run every test target except `architecture` and `e2e` (today it runs only
    `--lib --bins --test integration`, so tests under `crates/*/tests` are skipped).
  - `affected.rs`: a pure function from changed paths to `Affected` (`All`, or a set of package
    names) implementing T-22, and `cargo xtask affected --base <ref>`, which takes the paths from
    `git diff --name-only <ref>...HEAD` and prints one package per line (`*` for all).
  - `check --pr <base>`: fmt, Clippy, lint, `cargo test --workspace --test architecture --test e2e`,
    then the unit and integration tests of each affected package (all its targets except those two);
    no package tests when none is affected.
  - `pr.yml`: `pull_request` to `main` (`opened`, `synchronize`, `reopened`); Ubuntu;
    `fetch-depth: 0`; toolchain from `rust-toolchain.toml`; `Swatinem/rust-cache`;
    `cargo xtask check --pr origin/${{ github.base_ref }}`; superseded runs of the same PR cancelled.
  - `main.yml`: push to `main` and `workflow_dispatch`; matrix Ubuntu, Windows, macOS running
    `cargo xtask check`; then `promote` (needs the matrix; `concurrency` one at a time, never
    cancelled): stop if `release` already contains the commit; otherwise push the tested SHA to
    `release` over SSH with secret `RELEASE_DEPLOY_KEY` as a plain (non-force) push, so a stale run
    cannot move `release` backwards; then `gh workflow run release.yml --ref release`, skipped with a
    notice while `release.yml` does not exist.
  - `workflow.md`: branch naming, PR title format, the three workflows and what each runs, versions
    (T-21), the deploy key, rulesets and Pages settings, and how to reproduce each CI step locally.
- **Acceptance:** unit tests for `affected` (docs only → none; `crates/core/src/x.rs` → core, tui,
  oxysol; `crates/tui/…` → tui, oxysol; `src/…` and `tests/…` → oxysol; `xtask/…` → xtask;
  `Cargo.lock` → all; mixed input); workflows are valid YAML; the PR's own `pr.yml` run is green.
- **Done by Opus after merge:** deploy key and secret, rulesets for `main` and `release`, Pages source.
- **Done:** #9

### P0-04 — Release workflow: build, tag, publish, site

- **Owner:** Haiku
- **Depends on:** P0-03
- **Files:** `.github/workflows/release.yml`, `xtask/src/{main.rs,version.rs}`, `docs/site/**`, `docs/tech/workflow.md`
- **Scope (R-17, T-18, T-21):**
  - `cargo xtask version` prints the version from `metadata.toml`; `cargo xtask version --tag
    [--stable]` prints the T-21 tag. The tag is computed by a pure function of version, commit
    count and `stable`.
  - `release.yml`: on push to `release` and `workflow_dispatch` (boolean input `stable`, default
    false); refuses any ref but `release`; one run at a time.
    1. full check on Ubuntu, Windows, macOS;
    2. release builds for `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`,
       `x86_64-unknown-linux-musl` (`musl-tools`), `x86_64-apple-darwin`, `aarch64-apple-darwin`;
       each archived (`.zip` on Windows, `.tar.gz` elsewhere) with `README.md` and `LICENSE` as
       `oxysol-<tag>-<target>`, uploaded as workflow artifacts;
    3. compute the tag; fail clearly if it exists; create it on the tested SHA;
    4. a GitHub Release with every archive and a `SHA256SUMS` file; `prerelease` unless `stable`;
    5. build the site from `docs/site/` with mdBook (prebuilt binary or action, no Rust
       dependency added), insert version and release link, deploy with `actions/deploy-pages`.
  - `docs/site/`: `book.toml`, `SUMMARY.md`, `index.md` (what oXySol is, status: pre-release,
    download link to the latest release, link to the repository). P7-02 grows it into the player wiki.
- **Acceptance:** tests for `version` and the tag function; workflow is valid; after merge Opus
  states whether the first promotion produced a pre-release with five archives and an updated page.
- **Done:** #13

### O-01 — Record roles, CI, seed and SQLite decisions (R-14…R-19)

- **Owner:** not recorded (follow-up)
- **Depends on:** U-01
- **Files:** `AGENTS.md`, `docs/task_list/{README,micro,phase-0,phase-1,phase-3,phase-4,phase-5,phase-7,phase-8}.md`, `docs/tech/{architecture,decisions}.md`
- **Scope:** Records R-14 (Opus orchestrates and reviews, Fable on escalation), R-15 (small PRs), R-16 (PR CI with the full architecture and E2E targets, incremental unit and integration), R-17 (main to release promotion; release builds, tags, publishes and updates the site), R-18 (seeds shown, typed and recorded) and R-19 (SQLite and text logs). Revises T-02 and T-19, adds T-21…T-24, and rewrites the affected cards (P0-03, the new P0-04, P4-01 on SQLite, P7-02, M-18 with `--seed`, M-06).
- **Acceptance:** Docs only, no code changed; the decisions are in `decisions.md` and the cards are rewritten to match (PR #5).
- **Done:** #5

### O-03 — CI actions on Node 24

- **Owner:** not recorded (follow-up)
- **Depends on:** P0-03
- **Files:** `.github/workflows/{main,pr,release}.yml`
- **Scope:** Updates the third-party actions to Node 24 versions: `actions/checkout` v7, `upload-artifact` v7, `download-artifact` v8, `upload-pages-artifact` v5, `deploy-pages` v5. `Swatinem/rust-cache` stays at v2 (v2.9.3), which already targets Node 24. Workflow triggers and the `promote` steps are unchanged.
- **Acceptance:** No workflow input changes were needed (PR #40).
- **Done:** #40

### O-04 — Release builds only by manual dispatch

- **Owner:** not recorded (follow-up)
- **Depends on:** P0-04
- **Files:** `.github/workflows/{main,release}.yml`, `docs/tech/{decisions,workflow}.md`
- **Scope:** Owner decision R-20 (2026-11-15): `release.yml` loses its `push` trigger and keeps `workflow_dispatch` with the `stable` input. `main.yml` `promote` still fast-forwards `release` after a green full check but no longer starts the release. R-17, T-21 and T-23 are updated; `workflow.md` says how to start a release (`gh workflow run release.yml --ref release`, with `-f stable=true` for a stable one).
- **Acceptance:** A push to `release` no longer starts a release build; `workflow_dispatch` does (PR #38).
- **Done:** #38

### O-08 — Release targets: Windows x64, Linux x64/arm64 musl, macOS arm64

- **Owner:** not recorded (follow-up)
- **Depends on:** O-04
- **Files:** `.github/workflows/release.yml`, `docs/tech/{decisions,workflow}.md`
- **Scope:** Release matrix: `x86_64-pc-windows-msvc` (windows-latest), `x86_64-unknown-linux-musl` (ubuntu-latest, `musl-tools`), `aarch64-unknown-linux-musl` (ubuntu-24.04-arm, native ARM runner), `aarch64-apple-darwin` (macos-latest). `x86_64-unknown-linux-gnu` and `x86_64-apple-darwin` are removed. Each target gets a normal archive and a `-portable` archive that contains `oxysol.portable`. Decision T-25.
- **Acceptance:** Matrix and archive names as listed in PR #44.
- **Done:** #44

### O-22 — No-sound CI build, cache tuning, author and third-party licences

- **Owner:** not recorded (follow-up)
- **Depends on:** —
- **Files:** `.github/workflows/{main,pr,release}.yml`, `Cargo.toml`, `LICENSE`, `about.toml`, `about.hbs`, `build.rs`, `crates/{core,tui}/Cargo.toml`, `src/{about,lib,main}.rs`, `tests/e2e/main.rs`, `tests/integration/audio.rs`, `locales/*/cli.toml`, `docs/tech/{architecture,decisions,workflow}.md`
- **Scope:** CI builds and tests the no-sound configuration (`--no-default-features`, as the Linux musl release) in `pr.yml` and in the ubuntu job of `main.yml`. Rust cache is read-only in `pr.yml` and in the release check, and no longer used by the release tag and build jobs. `authors` in `[workspace.package]`, a copyright line in `LICENSE`, Windows `CompanyName` and `LegalCopyright`, and a legal second line in `oxysol --version`. Release archives contain `THIRD-PARTY-LICENSES.md`, generated per target by cargo-about 0.9.3. Decision T-31.
- **Acceptance:** `cargo xtask check --quick`, the clippy run and the `--no-default-features` tests pass locally, and cargo-about generation was run locally; the release workflow was not run (PR #85).
- **Done:** #85

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

### O-36 — Version scheme: patch counts merges, minor counts phases, the plan ends at 0.9

- **Owner:** Sonnet
- **Depends on:** —
- **Files:** `metadata.toml`, `Cargo.toml`, `build.rs`, `crates/tui/build.rs`, `src/about.rs`, `xtask/src/version*`,
  `.github/workflows/release.yml`, `docs/tech/{decisions,workflow,architecture}.md`
- **Scope:** owner decision 2026-11-19: every merge to `main` raises the patch by one (derived from git: first-parent
  commits since the `version` line of `metadata.toml` last changed; no bot commits); the minor counts completed plan
  phases (phases 0–3 done: 0.4.0); the whole plan ends at 0.9.0 and 1.0.0 is a later decision. The binary, the menu,
  the Windows file version and the release tags (`v0.4.P`) use the effective version; without git it is `0.4.0+src`.
- **Acceptance:** unit and temporary-repository tests; `cargo xtask version` and `oxysol --version` shown in the PR.
- **Done:** #119

### O-37 — Mouse path for Pause and Possible moves on every screen width

- **Owner:** Haiku
- **Depends on:** P3-16, P3-28
- **Files:** `crates/tui/src/screens/board/guidance.rs`, the board footer and moves panel, locales, tests
- **Scope:** P3-09 finding: `p` (pause) has no clickable control on the board and `v` (possible moves) has none
  below 130 columns, so both are keyboard-only (AGENTS.md: every action by keyboard and mouse). Give both a
  clickable entry (the `Do:` row or the footer, following its drop priorities) at every supported width.
- **Acceptance:** click tests at 100 × 30 and 130 × 40 for both actions; both locales.

### O-38 — Settings save failure: alert and log once per session

- **Owner:** Haiku
- **Depends on:** —
- **Files:** `crates/tui/src/app/effects.rs`, `crates/tui/src/screens/settings/tests.rs`, tests
- **Scope:** P3-09 finding, decided by the orchestrator 2026-11-20: `apply_settings` alerts and logs on every
  failed settings write, unlike game writes. Make it alert and log once per session, like game writes; later
  failures stay silent until a write succeeds again. Update the test that pins the old behaviour.
- **Acceptance:** tests for the first failure (alert and log), a second failure (silent) and recovery.

### O-39 — Split files over 300 LOC found by the P3-09 gate

- **Owner:** Antigravity
- **Depends on:** —
- **Files:** `crates/tui/src/app.rs`, `crates/tui/src/input.rs`, the oversized test files named in #121
- **Scope:** P3-09 finding (warnings only): `app.rs` (399) and `input.rs` (337) above 300 LOC; several test files
  above 300 and three card tests nested 5–6 deep. Split by responsibility without behaviour changes.
  `glyphs/wgl4.rs` is a data table and stays.
- **Acceptance:** no LOC or nesting warnings for these files; `cargo xtask check --quick` passes.

### O-40 — Player site in nine languages and a refreshed README

- **Owner:** Opus
- **Depends on:** O-30
- **Files:** `docs/site/**`, `README.md`
- **Scope:** owner request 2026-11-20: the site in German, French, Italian, Brazilian Portuguese, Ukrainian, Dutch and
  Spanish next to English and Polish, reusing the English screenshots and saying the game interface is in English and
  Polish today with the others in progress; README updated for P3-16, P3-37 and P5-01.
- **Done:** #128 (mdBook 0.4.52 build and a link check of 39 pages; translations not reviewed by native speakers)

### O-41 — Seven more interface languages in code, offered once translated

- **Owner:** Opus
- **Depends on:** P3-37
- **Files:** `crates/tui/src/i18n*`, the language picker, settings, CLI locale handling, `locales/*/meta.toml`,
  placeholder `locales/<code>/` copies, `docs/tech/{architecture,decisions}.md`, tests
- **Scope:** owner request 2026-11-20: locales de-DE, fr-FR, it-IT, pt-BR, uk-UA, nl-NL, es-ES with flags, CLDR plural
  rules and layout tests over every complete locale. A locale is offered only when its `meta.toml` says
  `complete = true`; a translation PR (O-42…O-48) replaces the ten files and sets that flag.
- **Acceptance:** see the PR; `cargo xtask check --quick` passes.
- **Done:** #131 (only complete locales offered; Settings row shows own names; T-36)

### O-42 — Interface translation: German (de-DE)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/de-DE/*.toml`, `docs/tech/glossaries/de.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.
- **Done:** #129 (fit fixes by Haiku; stats test reads texts from the locale)

### O-43 — Interface translation: French (fr-FR)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/fr-FR/*.toml`, `docs/tech/glossaries/fr.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.
- **Done:** #132 (fit fixes by Haiku)

### O-44 — Interface translation: Italian (it-IT)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/it-IT/*.toml`, `docs/tech/glossaries/it.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.
- **Done:** #134 (menu test matches Esc as a whole word)

### O-45 — Interface translation: Brazilian Portuguese (pt-BR)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/pt-BR/*.toml`, `docs/tech/glossaries/pt.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.

### O-46 — Interface translation: Ukrainian (uk-UA)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/uk-UA/*.toml`, `docs/tech/glossaries/uk.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.

### O-47 — Interface translation: Dutch (nl-NL)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/nl-NL/*.toml`, `docs/tech/glossaries/nl.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.

### O-48 — Interface translation: Spanish (es-ES)

- **Owner:** Antigravity (text only), checked by Opus
- **Depends on:** O-41
- **Files:** `locales/es-ES/*.toml`, `docs/tech/glossaries/es.md`
- **Scope:** owner request 2026-11-20: translate the ten en-US files; same keys and order, every placeholder kept, short
  labels, one term per concept (glossary). Translations run one language at a time.
- **Acceptance:** key and placeholder parity checked by script; after O-41, `complete = true` and the layout tests pass.

### O-49 — Workflow review: CI, release and agent workflows

- **Owner:** Opus
- **Depends on:** —
- **Files:** `docs/tech/workflow-review-2026-11.md`
- **Scope:** owner request 2026-11-20: review of the workflows for safer, better, more convenient and faster work.
- **Done:** #133 (26 findings, cards O-50…O-67; the documentation-only items B3a, B5, the C4 rule, C5 and F1 were
  applied directly in AGENTS.md, `workflow.md`, `architecture.md` and `decisions.md`)

### O-50 — Require up-to-date branches and auto-merge on main (repository setting)

- **Owner:** Owner
- **Depends on:** O-49
- **Files:** repository settings only
- **Scope:** S1 (P1): branch protection on `main` requires branches to be up to date before merging; enable auto-merge so a PR merges itself once green. Land first: every later PR is then tested against the latest `main`.
- **Acceptance:** The setting is visible in Settings → Branches; an out-of-date PR cannot merge until updated.

### O-51 — `check --quick` runs every test target in one cargo call

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/check*`
- **Scope:** B1 (P1): `--quick` skips the `e2e` and `architecture` targets although they cost under 0.1 s, and the split run compiles core and the TUI twice. Run all test targets in one cargo call.
- **Acceptance:** An e2e failure (like the O-36 version test) fails `--quick` locally; a warm `--quick` is not slower.

### O-52 — Lint: conflict markers in every tracked file

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/lint*`, tests
- **Scope:** B2 (P1): conflict markers are caught only in `.rs` files by the compiler. Add a lint over the tracked text files that fails on conflict-marker lines (seven `<`, `=` or `>` at the start of a line), naming file and line.
- **Acceptance:** A test with a fixture; the rule runs in `check --quick` and passes on `main`.

### O-53 — Lint: decision ID uniqueness and references

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/lint*`, tests, `docs/tech/decisions.md` (ordering only)
- **Scope:** B3 (P1): parse the table rows of `decisions.md`; fail on a duplicate `R-`/`T-`/`D-` ID; warn on a reference under `docs/` to an ID that does not exist.
- **Acceptance:** Tests with fixtures; `check --quick` passes on `main`.

### O-54 — `check` writes a result file; `cargo xtask report`; PR template

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/check*`, `xtask/src/main.rs`, `.github/pull_request_template.md`
- **Scope:** C1 (P1): `check` writes `target/check-result.txt` with the commit SHA, mode, steps, result and the last lines of a failing step; `cargo xtask report` prints it as Markdown for the PR body. A PR template with What / Files / Check result / Deviations / Limitations.
- **Acceptance:** Unit tests for the file format; the template shows on a new PR.

### O-55 — Pin GitHub Actions by SHA and add Dependabot

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/*.yml`, `.github/dependabot.yml`
- **Scope:** S3 (P1): pin every action by full commit SHA with the version as a comment; Dependabot for `github-actions` and `cargo`, weekly and grouped.
- **Acceptance:** Workflows still pass; the Dependabot config validates.

### O-56 — cargo-deny on pull requests

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `deny.toml`, `.github/workflows/pr.yml`, `docs/tech/decisions.md` (ID assigned in the card)
- **Scope:** S2 (P1): licenses (the T-31 list), advisories and sources checked on every PR, not only at release.
- **Acceptance:** `pr.yml` runs it; a disallowed license fails it.

### O-57 — English fallback for incomplete locales and `cargo xtask locale-diff`

- **Owner:** Sonnet
- **Depends on:** O-49, O-41
- **Files:** `crates/tui/src/i18n*`, `xtask/src/**`, locale tests, `docs/tech/architecture.md`
- **Scope:** B6 (P1): an incomplete locale need not mirror every English key; missing keys fall back to en-US, so a new English key no longer touches nine folders. Complete locales keep full parity. `cargo xtask locale-diff <lang>` lists missing, extra and placeholder-mismatched keys.
- **Acceptance:** Parity enforced for complete locales only; locale-diff tests.

### O-58 — Check pull requests on Windows as well

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/pr.yml`
- **Scope:** S6 (P2): Windows-only code (console font, full screen, widening) is compiled only on `main`. Add a cached Windows job to `pr.yml`.
- **Acceptance:** A PR shows both jobs; the owner updates the required checks.

### O-59 — Tests derive row counts instead of hard-coding them

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `crates/tui/src/accessibility_tests.rs`, `crates/tui/src/screens/settings/tests/**`
- **Scope:** B7 (P2): settings and menu tests break whenever a row is added (fixed counts and step loops). Derive counts from `FIELDS` and find rows by field.
- **Acceptance:** Adding a settings row needs no test edit.

### O-60 — `cargo xtask worktrees`: list and remove worktrees of merged branches

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/**`, `docs/tech/workflow.md`
- **Scope:** C4 (P2): list worktrees whose branch is merged into `origin/main` or whose PR is closed, `--remove` to delete them, and report orphan directories.
- **Acceptance:** Tests with a temporary repository.

### O-61 — main: re-run jobs that never got a runner before failing

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/main.yml`
- **Scope:** B4 (P2): macOS jobs cancelled after about 15 minutes without a runner turn `main` red and skip promotion. Retry once, or tell a never-started job from a failed one.
- **Acceptance:** A documented retry path.

### O-62 — Build the site on pull requests; check screenshots are current

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `.github/workflows/pr.yml`, `xtask/src/screenshots/**`
- **Scope:** B8 (P2): build the mdBook site on PRs that touch `docs/site/**`; add a staleness check for committed screenshots and the font.
- **Acceptance:** A broken link in a test PR fails the check.

### O-63 — Take the version from the binary instead of a TUI build script

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `crates/tui/build.rs`, `build.rs`, `src/**`, the TUI version display
- **Scope:** F2 (P2): the git-derived version recompiles `oxysol-tui` after every commit, even docs-only ones. Compute it in the binary crate and pass it to the TUI.
- **Acceptance:** A docs-only commit does not rebuild `oxysol-tui`; the version shown is unchanged.

### O-64 — `cargo xtask preflight` and a local CI mode

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `xtask/src/**`, `docs/tech/workflow.md`, `AGENTS.md`
- **Scope:** C2, C3 (P2): `preflight [--card <ID>]` fetches `origin/main`, reports how far behind the branch is, runs the marker and decision-ID lints and lists changed files the card does not name.
- **Acceptance:** Tests with a temporary repository.

### O-65 — Release provenance, SBOM, immutable releases; pinned host key

- **Owner:** Sonnet
- **Depends on:** O-49
- **Files:** `.github/workflows/release.yml`, the promotion step, `docs/tech/workflow.md`
- **Scope:** S4, S5 (P2): provenance attestations and an SBOM per archive; immutable releases; a pinned GitHub SSH host key for promotion instead of a fresh `ssh-keyscan`.
- **Acceptance:** A release dry run shows the attestations; promotion works with the pinned key.

### O-66 — Optimize the engine in test builds

- **Owner:** Haiku
- **Depends on:** O-49
- **Files:** `Cargo.toml` (profiles)
- **Scope:** F3 (P2): solver tests dominate debug test time; raise `opt-level` for `oxysol-core` in the dev/test profile.
- **Acceptance:** Test time before and after in the PR.

### O-67 — Small cleanups: pycache, site redeploy, xtask split, apt cache

- **Owner:** Antigravity / Haiku
- **Depends on:** O-49
- **Files:** as named in the review (B9, C7, F4, F5)
- **Scope:** P3 items of `docs/tech/workflow-review-2026-11.md`, one PR each.
- **Acceptance:** Each item as in the review.
