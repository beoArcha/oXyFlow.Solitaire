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
| `cargo xtask check --quick` | fmt check, Clippy `-D warnings`, lint (including conflict markers in every tracked file), every test target in one cargo call | run before opening a PR |
| `cargo xtask check --pr origin/main` | fmt, Clippy, lint, `architecture` and `e2e`, then unit and integration tests of the affected packages | `pr.yml` |
| `cargo xtask check` | everything: all test targets, then `sync-metadata --check` | `main.yml` on Linux, Windows, macOS |
| `cargo xtask affected --base origin/main` | prints the affected packages (`*` means all) | used by `check --pr` |
| `cargo xtask screenshots` | regenerates the SVG screenshots and animations of the player site (see "Player site") | run by hand; commit the result |

Affected packages (T-22): `crates/core` selects `oxysol-core`, `oxysol-tui` and `oxysol`;
`crates/tui` selects `oxysol-tui` and `oxysol`; `src/` and `tests/` select `oxysol`; `xtask/`
selects `xtask`; `metadata.toml` selects `xtask` and `oxysol`. `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `clippy.toml`, `.cargo/`, `.github/`, `locales/` and `assets/` select every
package. Changes under `docs/` and to `*.md` select none.

## Workflows

| File | Trigger | Runs |
|---|---|---|
| `.github/workflows/pr.yml` | pull request to `main` opened, updated or reopened | `cargo xtask check --pr origin/<base>` on Ubuntu, then Clippy and tests of `oxysol` with `--no-default-features` (the configuration of the Linux release); a newer push cancels the older run of the same PR. The rust cache is only read (`save-if: false`) |
| `.github/workflows/main.yml` | push to `main`, manual dispatch | `cargo xtask check` on Ubuntu, Windows and macOS, plus the no-sound Clippy and tests on Ubuntu; then `promote` (main only). It is the only workflow that saves the rust cache |
| `.github/workflows/release.yml` | manual dispatch only (R-20), input `stable` (default `false`) | `guard`, `check`, `tag`, `build`, `publish`, `site` (see "Release workflow") |

`promote` runs one at a time and is never cancelled. It stops when `release` already contains the
commit. Otherwise it pushes the tested commit to `release` over SSH with the deploy key from the
secret `RELEASE_DEPLOY_KEY`, as a plain push (no force), so an old run cannot move `release`
backwards. It starts no release: building and publishing is the owner's decision (R-20).

If the secret is empty, `promote` fails with a message that points here.

## Release workflow

Start it from the Actions tab ("Release", "Run workflow", branch `release`) or with:

```sh
gh workflow run release.yml --ref release
```

`release.yml` runs one at a time (`concurrency: release`, never cancelled). Its jobs:

1. `guard` fails unless the ref is `refs/heads/release`.
2. `check` runs `cargo xtask check` on Ubuntu, Windows and macOS.
3. `tag` checks out the full history, computes the tag with `cargo xtask version --tag` (`v<major>.<minor>.<patch>`,
   see Versions) and checks whether it already exists:
   - a tag at a different commit fails the run;
   - a tag at this commit with a GitHub Release already present is a notice, and the run ends
     without publishing (the same commit was already released); but with `stable: true` and an existing
     pre-release, the `stabilize` job turns that pre-release into a full release;
   - a tag at this commit without a Release (a previous run stopped before publishing) is
     published again on the same tag.
4. `build` checks out the full history (the version needs it) and makes release builds with `--locked` for `x86_64-pc-windows-msvc`,
   `x86_64-unknown-linux-musl` (`musl-tools`), `aarch64-unknown-linux-musl` (`musl-tools`)
   and `aarch64-apple-darwin`. Each target produces two archives: a normal archive
   `oxysol-<tag>-<target>` (with `oxysol` or `oxysol.exe`, `README.md`, `LICENSE`,
   `THIRD-PARTY-LICENSES.md` and `fonts/` with `oXySolMono-Regular.ttf`, `OFL.txt` and
   `NOTICE.md`) and a portable archive `oxysol-<tag>-<target>-portable` with the same
   files plus `oxysol.portable`:
   `.zip` on Windows, `.tar.gz` elsewhere. Archives are workflow artifacts named
   `archive-<target>`. `THIRD-PARTY-LICENSES.md` is generated per target by a pinned `cargo-about`
   (`about.toml`, `about.hbs`) with the features of that target's build, so it lists the crates that
   the binary includes. `tag` and `build` use no rust cache: published binaries come from clean builds;
   only `check` reads the cache of `main`.
5. `publish` writes `SHA256SUMS`, creates the tag on the tested commit and a GitHub Release with
   every archive and `SHA256SUMS`. It is a pre-release unless `stable` is set.
6. `site` builds `docs/site` with a pinned mdBook (v0.4.52, SHA-256 checked), inserts the tag and the
   release link into every page (`{{VERSION}}` and `{{RELEASE_URL}}`, English and Polish, and fails if a
   placeholder is left), and deploys GitHub Pages. The link points at the release that this run just
   published, so it is always the newest build, pre-releases included (`/releases/latest` ignores them).

Tag: `v<major>.<minor>.<patch>`, the same for a pre-release and a stable release (T-21).

A stable release adds the input `stable`:

```sh
gh workflow run release.yml --ref release -f stable=true
```

When the run is skipped at `tag` or fails before `publish`, `publish` and `site` do not run, so
the site keeps the content of the last successful run.

## Player site

The site is an mdBook in `docs/site`: English pages in `docs/site/src`, Polish pages in
`docs/site/src/pl` with the same file names, one `SUMMARY.md` with a Polish part, and one build. Every page
starts with a link to its twin in the other language. Text lives in the pages; the release link is injected by
the `site` job. To check the site locally, install mdBook (the release binary of the pinned version, or
`cargo install mdbook --version 0.4.52 --locked`) and run `mdbook build docs/site`. Locally the placeholders
`{{VERSION}}` and `{{RELEASE_URL}}` stay as they are.

### Screenshots

`cargo xtask screenshots` renders scripted scenes of the real application: it builds an `App` with in-memory
services, drives it with key presses on a Ratatui `TestBackend` terminal and writes each frame buffer as SVG
(background runs as rectangles, glyphs as text with an explicit x per glyph, box and block characters as paths so
they join in any font). It writes both languages (`en-US` and `pl-PL`) to `docs/site/src/img/en/` and
`docs/site/src/img/pl/`: the `menu`, `klondike`, `spider`, `freecell`, `pyramid`, `settings`, `help` and `guide`
screens, the board in each theme (`theme-dark`, `theme-light`, `theme-high-contrast`, `theme-monokai`,
`theme-dracula`) and three animations (`anim-move`, `anim-auto-finish`, `anim-win`). An animation is one SVG:
the first frame, then for each next frame only the changed runs of cells, shown by CSS keyframes in a loop (with
`prefers-reduced-motion` it shows the last frame and does not move).

The scenes use fixed seeds and a fixed clock, so the output does not change between runs. Run the command after any
change to the look of the game, and commit the files it writes. The scene scripts are in `xtask/src/screenshots/`
(`stills.rs`, `motion.rs`); alternative texts are in `labels.rs`. The command depends on `oxysol-tui`,
`oxysol-core` and `ratatui`; the TUI exposes nothing extra for it.

## Versions (T-21, R-23)

The version is `major.minor.patch`.

- **Patch: automatic.** It is derived from git, never stored. It is the number of first-parent commits on
  `main` since the commit that last changed the `version` line of `metadata.toml`. Squash merges give one
  commit per merge, so every merge to `main` raises the patch by 1. Nobody edits it and no bot commits.
- **Minor: raised by hand when a plan phase closes.** The minor counts completed phases: completing phase N
  gives minor N+1 (phases 0–3 done: 0.4.x; the plan ends at 0.9 with phase 9, see R-25; 1.0.0 is a later owner decision).
  One PR does it: set `version = "0.<N+1>.0"` in `metadata.toml`, run `cargo xtask sync-metadata` (it updates
  `[workspace.package]` in `Cargo.toml`), run `cargo build` so `Cargo.lock` follows, and change nothing else.
  Once that PR is merged the patch counts again from 0.
- **Where it shows.** `cargo xtask version` prints the effective version (`cargo xtask version --base` the
  one in `metadata.toml`, `--tag` the release tag). `build.rs` computes the same value through the same code
  (`xtask/src/version_scheme.rs`) and exports it as `OXYSOL_VERSION`: `--version`, the main menu label and
  the Windows file version use it. `Cargo.toml` and `Cargo.lock` keep `major.minor.0`. On a build that
  cannot read git history (a source archive, a shallow clone, no `git`) the version is `0.4.0+src`; the
  release workflow always checks out the full history.
- **Release.** Each `release.yml` run publishes the tag `v<effective version>` as a pre-release; with
  `stable: true` it publishes the same tag as a full release (or promotes the pre-release of the same
  commit).

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

## Parallel work and shared files

Two tasks run in parallel only when their file lists do not overlap. Some files are touched by most
tasks (measured over 60 PRs in `workflow-review-2026-11.md`, C5); a card that edits one of them names
it on a **Shared files** line, and the orchestrator either serializes such cards or merges the first
before dispatching the second. Current hotspots:

- `crates/tui/src/app.rs`, `crates/tui/src/settings.rs`, `crates/tui/src/input.rs`
- `crates/tui/src/screens/settings/**` and the settings row-count tests in `accessibility_tests.rs`
- `locales/en-US/*.toml` (every new key must reach every locale folder, see below)
- `docs/tech/decisions.md` (IDs are assigned by the orchestrator in the card)
- `docs/task_list/**` (edited by the orchestrator only)

A merged PR changes the base of every open one: before merging the next PR the orchestrator merges
`main` into it (or asks its agent to) and runs `cargo xtask check --quick` again.

## Translations

A locale is offered to players only when `locales/<lang>/meta.toml` says `complete = true`
(architecture: "Adding or translating a locale"). Translation tasks are text-only and run one
language at a time: the translator replaces the ten area files of a locale, keeps keys, order and
`{placeholders}`, sets `complete = true` and writes `docs/tech/glossaries/<lang>.md`. The
orchestrator then merges `main`, compares the keys with `en-US`, and runs the check; the fit tests
(footer, `Do:` row, header, statistics, help, `Now:` row) run for every offered locale, and texts
that do not fit are shortened in that locale only. A test that hard-codes English or Polish text is
a test bug: it reads the expected text from the locale instead.

## Worktrees

Every agent task gets its own git worktree under `~/.t3/worktrees/oXyFlow.Solitare/`. After its PR
is merged or closed, the orchestrator removes it: `git worktree remove <path>`, `git worktree prune`,
`git branch -D <branch>` (GitHub deletes the remote branch on merge). A worktree whose agent stopped
with uncommitted work is committed as `wip(...)` and pushed before removal, so another agent can
finish it.

## Local setup on Windows

Each worktree has its own `target/`, so a first `cargo xtask check --quick` compiles every
dependency again (about 7 minutes cold, 2 minutes warm). Two machine settings, not part of the
repository, make this much faster:

1. **sccache:** `cargo install sccache --locked`, then set the user environment variable
   `RUSTC_WRAPPER=sccache` (System Properties → Environment Variables → User variables → New).
   Dependencies then compile once for all worktrees. CI is unaffected.
2. **Windows Defender exclusions:** Windows Security → Virus & threat protection → Manage settings →
   Exclusions → Add an exclusion → Folder: `%USERPROFILE%\.t3\worktrees`, the repository folder and
   `%LOCALAPPDATA%\Mozilla\sccache`. Real-time scanning of `target/` is a large slowdown.

## Limitations

- `promote` trusts the `github.com` host key obtained with `ssh-keyscan` on each run.
- Windows and macOS run the full check only on `main` and `release`, not on pull requests (R-16).
