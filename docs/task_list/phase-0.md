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

### P0-02 — Master prompt in English with amendments

- **Owner:** Haiku
- **Depends on:** P0-01 (the Polish original must be in git history first)
- **Files:** `docs/tech/master_prompt.md`
- **Scope:** translate the file to English in place, faithfully, keeping section numbers. Then
  apply amendments R-01…R-09 from `decisions.md` in the sections they name, marking each changed
  passage with its amendment ID. Do not add requirements of your own.
- **Acceptance:** every original section present; every amendment applied exactly once and
  traceable; no statement contradicts `decisions.md`.

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
