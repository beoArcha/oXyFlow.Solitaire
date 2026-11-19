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
3. `tag` computes the tag with `cargo xtask version --tag` (or `--tag --stable`) and checks whether
   it already exists:
   - a stable tag that exists fails the run;
   - a dev tag that exists at a different commit fails the run;
   - a dev tag at this commit with a GitHub Release already present is a notice, and the run ends
     without publishing (the same commit was already released);
   - a dev tag at this commit without a Release (a previous run stopped before publishing) is
     published again on the same tag.
4. `build` makes release builds with `--locked` for `x86_64-pc-windows-msvc`,
   `x86_64-unknown-linux-musl` (`musl-tools`), `aarch64-unknown-linux-musl` (`musl-tools`)
   and `aarch64-apple-darwin`. Each target produces two archives: a normal archive
   `oxysol-<tag>-<target>` (with `oxysol` or `oxysol.exe`, `README.md`, `LICENSE` and
   `THIRD-PARTY-LICENSES.md`) and a portable archive `oxysol-<tag>-<target>-portable` with the same
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

Tags: dev `v<version>-dev.<n>`, stable `v<version>`, both from `metadata.toml` (T-21).

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

## Versions (T-21)

`metadata.toml` holds the next stable version. Each `release.yml` run publishes a
pre-release `v<version>-dev.<n>`, where `n` is `git rev-list --count HEAD`. With
`stable: true` it publishes `v<version>` and fails if that tag already exists.

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
