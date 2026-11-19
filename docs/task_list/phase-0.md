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
