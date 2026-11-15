# Workflow

Contract for branches, checks, workflows and releases. Decisions: `decisions.md` (R-07, R-16,
R-17, T-19, T-21, T-22, T-23). Gates: `architecture.md`, section "Quality gates".

## Branches and pull requests

- Branch from `main` as `<type>/<ID>-<slug>`, type `feat`, `fix`, `refactor`, `chore`, `docs`,
  `test` or `ci`. Example: `feat/M-02-lint-nesting`.
- Open a pull request to `main` titled `<type>(<scope>): <summary> [<ID>]`.
- Pull requests are squash-merged. Nothing is pushed to `main` or `release` directly.

## Local checks

| Command | What it runs | CI |
|---|---|---|
| `cargo xtask check --quick` | fmt check, Clippy `-D warnings`, lint, every test target except `architecture` and `e2e` | run before opening a PR |
| `cargo xtask check --pr origin/main` | fmt, Clippy, lint, `architecture` and `e2e`, then unit and integration tests of the affected packages | `pr.yml` |
| `cargo xtask check` | everything: all test targets, then `sync-metadata --check` | `main.yml` on Linux, Windows, macOS |
| `cargo xtask affected --base origin/main` | prints the affected packages (`*` means all) | used by `check --pr` |

Affected packages (T-22): `crates/core` selects `oxysol-core`, `oxysol-tui` and `oxysol`;
`crates/tui` selects `oxysol-tui` and `oxysol`; `src/` and `tests/` select `oxysol`; `xtask/`
selects `xtask`; `metadata.toml` selects `xtask` and `oxysol`. `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `clippy.toml`, `.cargo/`, `.github/`, `locales/` and `assets/` select every
package. Changes under `docs/` and to `*.md` select none.

## Workflows

| File | Trigger | Runs |
|---|---|---|
| `.github/workflows/pr.yml` | pull request to `main` opened, updated or reopened | `cargo xtask check --pr origin/<base>` on Ubuntu; a newer push cancels the older run of the same PR |
| `.github/workflows/main.yml` | push to `main`, manual dispatch | `cargo xtask check` on Ubuntu, Windows and macOS; then `promote` (main only) |
| `.github/workflows/release.yml` | push to `release`, manual dispatch | full check, builds, tag, GitHub Release, site (P0-04, not yet present) |

`promote` runs one at a time and is never cancelled. It stops when `release` already contains the
commit. Otherwise it pushes the tested commit to `release` over SSH with the deploy key from the
secret `RELEASE_DEPLOY_KEY`, as a plain push (no force), so an old run cannot move `release`
backwards. It then runs `gh workflow run release.yml --ref release`. While `release.yml` does not
exist, that step prints a notice and nothing is dispatched.

If the secret is empty, `promote` fails with a message that points here.

## Versions (T-21)

`metadata.toml` holds the next stable version. Each automatic `release.yml` run publishes a
pre-release `v<version>-dev.<n>`, where `n` is `git rev-list --count HEAD`. A manual dispatch with
`stable: true` publishes `v<version>` and fails if that tag already exists.

## One-time setup (repository owner)

1. Deploy key for promotion. Run in a scratch directory and delete the private key afterwards:

   ```sh
   ssh-keygen -t ed25519 -C "oxysol release promotion" -f release_deploy -N ""
   gh repo deploy-key add release_deploy.pub --repo beoArcha/oXyFlow.Solitare --allow-write --title "release promotion"
   gh secret set RELEASE_DEPLOY_KEY --repo beoArcha/oXyFlow.Solitare < release_deploy
   ```

2. Rulesets (Settings, Rules, Rulesets):
   - `main`: require a pull request before merging; required status check `check` (the job of
     `pr.yml`, available after its first run); allow squash merges only in the repository's merge
     settings; block force pushes and deletion.
   - `release`: restrict updates; block force pushes and deletion; add the deploy key from step 1
     as the only bypass actor.

3. Pages (Settings, Pages): source "GitHub Actions" (used by `release.yml`).

## Limitations

- `promote` trusts the `github.com` host key obtained with `ssh-keyscan` on each run.
- Windows and macOS run the full check only on `main` and `release`, not on pull requests (R-16).
