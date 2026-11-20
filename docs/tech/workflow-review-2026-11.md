# Workflow review — November 2026

Review of the development, CI, release and agent workflows of oXySol as of `main` at `9cdd03a`
(2026-11-20). Read-only review: no workflow or code was changed. Each finding has evidence, a
recommendation, effort (S < ½ day, M ½–2 days, L > 2 days), priority (P1 do next, P2 this phase,
P3 when convenient) and a proposed task card title with the ID left as `O-??`.

## Sources and measurements

- `AGENTS.md`, `docs/tech/workflow.md`, `docs/tech/decisions.md`, `docs/tech/architecture.md`,
  `docs/task_list/README.md`, `.github/workflows/{pr,main,release}.yml`, `xtask/src/**`,
  `Cargo.toml`, `metadata.toml`, `build.rs`, `crates/tui/build.rs`, `.gitignore`, `.cargo/config.toml`.
- `git log --oneline -150`; the last 60 merged PRs (#71…#131, 2026-11-16 to 2026-11-20); the last
  100 workflow runs (2026-11-19 10:18 to 2026-11-20 08:21) with per-job and per-step timings.
- Repository settings through `gh api`: rulesets, merge settings, Actions permissions,
  vulnerability alerts.
- One cold local run of `cargo xtask check --quick` in this fresh worktree on the owner's Windows 10
  machine (no `target/` yet).

| Measure | Value |
|---|---|
| PR workflow (`check`), successful runs | 51 runs; median 1.5 min, max 2.3 min |
| Main workflow, successful runs | 35 runs; median 2.7 min, max 12.2 min |
| Main workflow, failed runs | 4 of 39 (10 %); all four are the macOS job cancelled after ~15 min with no runner assigned |
| PR workflow, failed runs | 2: a stale-base test failure (P3-28) and an `e2e` version test (O-36) that `--quick` does not run |
| Release workflow | 2 runs, 6.9–7.1 min (check 2.3, tag 0.7, build 2–3, publish + site 0.5) |
| PR lead time, open → squash-merged | median 3 min, p90 29 min, max 187 min (60 PRs) |
| PRs that are `docs:` | 22 of 60 |
| Most-touched files in 60 PRs | `architecture.md` 26, `task_list/README.md` 16, `phase-3.md` 15, `accessibility_tests.rs` 14, `locales/{en-US,pl-PL}/board.toml` 14 each, `settings/tests.rs` 12, `app.rs` 10 |
| Local `cargo xtask check --quick`, cold, Windows | **420 s**, passed |
| Worktrees under `~/.t3/worktrees/oXyFlow.Solitare` | 130 directories, 111 registered in `git worktree list`, 98 with a `target/` |

## Summary

CI itself is fast and well structured: a PR is checked in about 1.5 minutes, `main` in under 3
minutes on three systems, and a release takes 7 minutes. Workflow permissions default to
`contents: read`, the release is guarded to the `release` branch, binaries are built cleanly with
`--locked`, mdBook is pinned by SHA-256 and `SHA256SUMS` is published. The weak points are elsewhere:

1. **Merges are not tested against each other.** The `main` ruleset has
   `strict_required_status_checks_policy: false`, PRs merge 3 minutes after opening (median), and
   several agents edit the same few files. Conflict markers have already reached `main` once
   (#96 → removed in #102) and duplicated entries sit in the `architecture.md` status line.
2. **Local checks miss what CI runs.** `--quick` skips `e2e` and `architecture` although together
   they take under 0.1 s, and it compiles core and TUI twice. Agents run the slower, weaker gate.
3. **Hotspot files make parallel work collide by design.** The one-line changelog at the top of
   `architecture.md`, the decision tables with hand-picked IDs, hard-coded row counts in tests and
   the rule that every locale (translated or not) holds every English key force unrelated PRs into
   the same lines.
4. **Supply chain is unguarded at PR time.** No Dependabot alerts, no `cargo-deny`/`cargo-audit`,
   actions pinned by tag, and the license gate (`cargo about --fail`) runs only at release time.
5. **Local builds on Windows are the real bottleneck.** A fresh worktree needs 7 minutes for
   `--quick`; every worktree has its own cold `target/`, and the git-derived version recompiles
   `oxysol-tui` after every commit.

The ten most valuable changes are listed in the roadmap at the end.

## Safer

### S1. Required check does not require an up-to-date branch — P1, S

**Observation.** Ruleset `main` (id 27307339): `required_status_checks: [check]` with
`strict_required_status_checks_policy: false`; `allow_auto_merge: false`. A PR that was green on an
older `main` can be merged after another PR changed the same behavior. Evidence:

- Conflict markers were committed to `docs/tech/architecture.md` by `98b94d6` (#96, P4-05,
  diff lines `+<<<<<<< HEAD … +>>>>>>> 6e1898a`) and stayed on `main` until `adb3579` (#102).
- Run 44699110666 (P3-28 docked modals): `help_lists_every_binding_and_the_gestures_in_a_100_by_30_window`
  failed at `crates/tui/src/widgets/overlay/render_tests.rs:167` after key bindings merged in
  parallel changed the help content. It was caught only because the branch was pushed again.
- `architecture.md:3` lists the P3-16 and P3-37 entries twice, and records `i18n::t_in` as both
  added (P3-37) and removed (O-41) — merge resolutions that kept both sides.

Merge queues are not offered for repositories owned by a personal account, so the remedy is the
strict policy plus a cheap way to update branches.

**Recommendation.** Turn on "Require branches to be up to date before merging" in the `main`
ruleset and enable auto-merge in the repository settings. The orchestrator then runs
`gh pr update-branch <n>` (or `git rebase origin/main && git push --force-with-lease` by the
agent) and `gh pr merge <n> --squash --auto`; GitHub merges when the re-run `check` passes. With
~1.5-minute PR runs the extra cost per merge is small. Document it in `workflow.md`
("Branches and pull requests").

**Card.** `O-?? chore(repo): require up-to-date branches and enable auto-merge on main`

### S2. No dependency vulnerability or license gate before release — P1, M

**Observation.** `gh api repos/…/vulnerability-alerts` returns 404 (Dependabot alerts off). There is
no `.github/dependabot.yml`, no `deny.toml`, no `cargo audit`. The only license check is
`cargo about generate --locked --fail` in `release.yml:164`, so a dependency with a license outside
the accepted list (T-31) is found **after** it is merged, when the owner starts a release.
`Cargo.lock` changes select every package in `affected.rs:272` but run no supply-chain check.

**Recommendation.** Add `deny.toml` with `[licenses] allow` copied from `about.toml` (MIT,
Apache-2.0, BSL-1.0, MPL-2.0, Unicode-3.0, Unlicense, Zlib), `[advisories]` against RustSec,
`[bans] multiple-versions = "warn"` and `[sources] unknown-registry = "deny"`. Install a pinned
`cargo-deny` with `taiki-e/install-action` in `pr.yml` and run `cargo deny check` there (≈10 s), or
only when `Cargo.lock`/`Cargo.toml` change, plus a weekly scheduled run on `main` for new
advisories. Enable Dependabot alerts (security only) in the repository settings. This needs the new
tool in CI only, no workspace dependency.

**Card.** `O-?? ci(deps): cargo-deny for licenses, advisories and sources on pull requests`

### S3. Actions pinned by mutable tags; third-party actions in the release build — P1, S

**Observation.** Every action is referenced by a tag: `actions/checkout@v7`, `Swatinem/rust-cache@v2`,
`taiki-e/install-action@v2` (`release.yml:158`, inside the job that builds the published
binaries), `actions/upload-artifact@v7`, `actions/download-artifact@v8`,
`actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`. Repository Actions permissions:
`allowed_actions: all`, `sha_pinning_required: false`. A moved tag of a third-party action would
run inside the release build with access to the build output.

**Recommendation.** Pin all actions by full commit SHA with the tag as a trailing comment, set
`sha_pinning_required: true` and restrict `allowed_actions` to GitHub-owned plus the two used
third-party actions. Add `.github/dependabot.yml` with `package-ecosystem: github-actions`
(weekly, one grouped PR) so pins stay current without manual work.

**Card.** `O-?? ci(workflows): pin actions by SHA and keep them current with Dependabot`

### S4. Releases have checksums but no provenance, SBOM or immutability — P2, S

**Observation.** `release.yml:224-235` writes `SHA256SUMS` and uploads archives. There is no build
attestation, no SBOM, and nothing stops a later edit of a published release's assets or tag (the
`tag` job only detects a moved tag on the next run, `release.yml:82`). Windows binaries are
unsigned, so SmartScreen warns on first run of a downloaded `oxysol.exe` — relevant because the
owner's primary platform is Windows 10.

**Recommendation.** In `build`, add `actions/attest-build-provenance` for each archive (job
permissions `id-token: write`, `attestations: write`) so users can run
`gh attestation verify oxysol-….zip --repo beoArcha/oXyFlow.Solitare`. Generate a CycloneDX SBOM per
target with a pinned `cargo-cyclonedx` (release-time tool, like `cargo-about`) and attach it.
Enable "immutable releases" in the repository settings so a published tag and its assets cannot be
changed. Code signing (e.g. SignPath's free OSS program) is a separate, owner-level decision; note
it in `decisions.md` as open.

**Card.** `O-?? ci(release): build provenance attestations, SBOM and immutable releases`

### S5. Promotion trusts a fresh `ssh-keyscan`; any deploy key bypasses `release` — P2, S

**Observation.** `main.yml:114` appends `ssh-keyscan -t ed25519 github.com` on every run
(acknowledged in `workflow.md:162`). The `release` ruleset bypass is
`{"actor_type": "DeployKey", "actor_id": null, "bypass_mode": "always"}`, i.e. every deploy key with
write access, not the one created for promotion.

**Recommendation.** Replace the scan with GitHub's published ed25519 host key written verbatim to
`known_hosts`. Keep exactly one write deploy key on the repository and add a check to
`workflow.md`'s one-time setup ("`gh repo deploy-key list` shows one key titled release promotion").

**Card.** `O-?? ci(promote): pin the GitHub host key and document the single deploy key`

### S6. Windows-only code is not compiled on pull requests — P2, M

**Observation.** `pr.yml` runs on `ubuntu-latest` only; Windows and macOS run on `main` and
`release` (`workflow.md:163`, R-16). The classic-console code (T-33 full screen, T-35
`GetCurrentConsoleFontEx`/`SetCurrentConsoleFontEx` through `extern "system"`, T-28
`CREATE_NEW_CONSOLE`, `winresource` in `build.rs`) is behind `cfg(windows)` and is not even
type-checked before merge. The owner plays on that console. The repository is public, so
GitHub-hosted runner minutes are not billed; the "CI budget" reason of R-08 no longer applies.
Windows full check on `main` takes 2.1–2.6 min.

**Recommendation.** Add a `windows` job to `pr.yml` (`cargo xtask check --pr origin/main` on
`windows-latest`, cache read-only). Keep `check` (Linux) as the required context at first, then
make the Windows job required once it has been stable for a week. macOS stays on `main` because of
the queueing seen in B4. Record the change as a revision of R-16 in `decisions.md`.

**Card.** `O-?? ci(pr): check pull requests on Windows as well`

## Better

### B1. `--quick` skips `e2e` and `architecture` although they cost nothing — P1, S

**Observation.** `xtask/src/check.rs:58-79`: `--quick` runs `cargo test --workspace --exclude oxysol --tests`
then `cargo test -p oxysol --lib --bins --test integration`, so `e2e` and `architecture` run only in
CI (`check.rs:81-94`). Run 44622626556 (O-36 README sync) failed in CI on
`version_prints_the_version_and_exits_zero` (`tests/e2e/main.rs:36`), which the agent's local
`--quick` could not see. In CI both suites finish in 0.01–0.02 s. The split also makes cargo
compile `oxysol-core` and `oxysol-tui` twice in the test profile: the local run shows a second
"Compiling oxysol-core / oxysol-tui / oxysol … Finished in 1m 04s" for the second invocation.

**Recommendation.** Make `--quick` one invocation, `cargo test --workspace --all-targets`
(everything `Full` tests, without `sync-metadata --check` and `icon --check`). That closes the gap
and removes the second compile. Update the tables in `workflow.md:17` and `architecture.md`
("Quality gates"). Optionally rename: `--quick` is then "full tests, no asset checks".

**Card.** `O-?? fix(xtask): check --quick runs every test target in one cargo call`

### B2. Conflict markers are only caught where the compiler sees them — P1, S

**Observation.** See S1: markers in `architecture.md` passed every gate. `xtask lint` reads only
`*.rs` under `src`, `tests`, `xtask/src`, `crates/*/{src,tests}` (`xtask/src/lint/mod.rs:154-176`).
Markdown and TOML outside the parsed locale files are never checked.

**Recommendation.** Add a lint rule `conflict-markers` that scans every git-tracked text file
(`git ls-files`, skip binaries by extension: `.ico`, `.png`, `.mid`, `.pyc`) for lines starting with
`<<<<<<< `, `=======` (exactly, alone on the line), or `>>>>>>> `, as an Error. It runs in every
check mode because lint does.

**Card.** `O-?? feat(xtask): lint rule against conflict markers in every tracked file`

### B3. Decision IDs are chosen by hand and clash — P1, S

**Observation.** Two PRs picked the same next ID (T-34, R-22). A stale cross-reference survives:
`decisions.md:129` (T-21) says the minor is "set by hand when a plan phase closes (R-22)", but
R-22 (`decisions.md:93`) is full screen; the phase rule is R-23 (`decisions.md:98`). The R table
is also split: R-01…R-21 at lines 22-47, R-22 at line 93, R-23 at 98, R-24 at 104, T-20 sits after
T-24 (`decisions.md:133`).

**Recommendation.** (a) The orchestrator allocates decision IDs in the task card ("add T-37"), never
the executor. (b) A lint `decision-ids` parses table rows `| <X>-<n> |` in `decisions.md` and fails on a
duplicate ID; it warns on a reference `\b[RTD]-\d+\b` anywhere under `docs/` to an ID that does
not exist. With S1 in place the lint runs against the latest `main`, so a clash fails the second PR.
(c) Fix the T-21 reference in the same PR.

**Card.** `O-?? feat(xtask): decision ID uniqueness and reference lint`

### B4. macOS queueing turns `main` red and skips promotion — P2, S

**Observation.** All four failed Main runs (44556630260, 44556849298, 44566697449, 44570138056)
have `check (macos-latest)` cancelled after ~15 min with `runner_name: ""` and no steps, while
Linux and Windows succeeded. `promote` is skipped (`needs: check`), so `release` lags until the
next merge, and agents reading `gh run list` see a red `main` that is not a code failure.

**Recommendation.** Add a small `rerun-infra` job (or a separate `workflow_run` workflow,
`actions: write`) that re-runs failed jobs once when a job has no runner and no steps.
Alternatively let `promote` depend on Linux and Windows only, and report macOS as a separate
non-blocking job; record the trade-off in `workflow.md`. Either way, document "a macOS job
cancelled with no steps is infrastructure, re-run it" for agents.

**Card.** `O-?? ci(main): re-run jobs that never got a runner before failing main`

### B5. The `architecture.md` status line is a merge-conflict generator — P1, S

**Observation.** 26 of the last 60 PRs edited `docs/tech/architecture.md`; most of them prepend to
the single line 3 ("Status: contract v0.6 (…)"), now one ~2,700-character line. Any two PRs
conflict there by construction, and resolutions duplicated P3-16 and P3-37 (S1).

**Recommendation.** Remove the changelog from line 3 (keep "Status: contract v0.6, see git log").
Contract changes are already traceable by the `[ID]` in each squash commit
(`git log --oneline -- docs/tech/architecture.md`). If a human-readable history is wanted, use one
file per task, `docs/tech/changes/<ID>.md`, which never conflicts. Change the AGENTS.md rule
"If you change a contract in `architecture.md`, update that file" to mean the contract text only.

**Card.** `O-?? docs(architecture): drop the status changelog line that every PR edits`

### B6. Untranslated locales must mirror every English key — P1, M

**Observation.** `crates/tui/src/i18n/tests.rs:101-119` asserts that **every** locale in
`Locale::ALL` has exactly the plain keys of `en-US`. Seven locales are `complete = false` and are
English copies (`locales/de-DE/board.toml` holds `no_undo = "No undo"`). So a feature PR that adds
one English key must edit nine locale files, and every open translation PR (#129 `O-42-de`, #132
`O-43-fr` at review time) conflicts with it or fails the parity test after a rebase — the
"translations started from an older English base" pain. `lookup_falls_back_to_the_key`
(`tests.rs:164`) shows a missing key falls back to the key, not to English.

**Recommendation.** Make lookup fall back to `en-US` text for a missing key, then enforce key
parity only for `complete = true` locales; for incomplete locales assert only that they contain no
keys `en-US` lacks and that their placeholders match (existing `tests.rs:215`). Remove the English
copies from incomplete locales. Add `cargo xtask locale-diff <lang>` that prints missing and
obsolete keys against `en-US`, so a translator who rebases sees exactly what to add. Feature PRs
then touch `en-US` and `pl-PL` only. This changes the i18n contract in `architecture.md` (T-36).

**Card.** `O-?? refactor(i18n): English fallback for incomplete locales and a locale-diff command`

### B7. Tests that count rows break whenever a row is added — P2, S

**Observation.** `crates/tui/src/accessibility_tests.rs:178` `assert_eq!(rows.len(), 18, …)` and
`:186` `assert_eq!(rows_by_tab(&mut h).len(), 18)`; similar fixed sizes in settings and help
tests. `accessibility_tests.rs` (14 PRs) and `screens/settings/tests.rs` (12 PRs) are among the most
edited files, mostly to bump such numbers, so two settings PRs always conflict on the same line.

**Recommendation.** Derive the expected count from the source of truth (the settings row list's
`len()`), and assert the property being tested ("every row is reachable by Tab and has a label")
instead of a number. Add one sentence to AGENTS.md "Code rules": tests do not hard-code the length
of a list the next task will extend.

**Card.** `O-?? test(tui): derive row counts instead of hard-coding them`

### B8. Committed artifacts have no staleness check, the site is not built before release — P2, S

**Observation.** `cargo xtask screenshots` output is committed and must be re-run "after any change
to the look of the game" (`workflow.md:114`) — there is no `--check`, unlike `icon --check`
(`check.rs:48`). `mdbook build docs/site` runs only in `release.yml:274`; O-40 (#128) added seven
site languages that were first built at release time. The placeholder guard
(`release.yml:269`) also exists only there.

**Recommendation.** Add `screenshots --check` (render to memory, compare with the committed SVGs)
to the full check on Linux only (fonts/terminal independent output is already deterministic per
`workflow.md:114`). Add a `site` job to `pr.yml` with a path filter on `docs/site/**` that installs the
pinned mdBook and runs `mdbook build docs/site`.

**Card.** `O-?? ci(site): build the site on pull requests and check screenshots are current`

### B9. Stray file and over-broad ignore — P3, S

**Observation.** `__pycache__/.sub.cpython-314.pyc` is tracked since `67019e4` (#66). `.gitignore`
has no `__pycache__/`, and has a bare `config.toml` that ignores any new `config.toml` at any depth.

**Recommendation.** `git rm` the `.pyc`, add `__pycache__/` and `*.pyc`, and narrow `config.toml`
to the path it was meant for (likely a local app config at the root: `/config.toml`).

**Card.** `O-?? chore(repo): remove the committed pycache and tighten .gitignore`

## More convenient

### C1. No PR template; reports are written from memory — P1, S

**Observation.** `.github/` holds only `workflows/`. AGENTS.md ("Final report") defines the report,
but agents paraphrase the check result, and Antigravity agents have ended a turn while a background
`cargo check` was still running. The orchestrator cannot tell from the PR which commit the claimed
result belongs to.

**Recommendation.** (a) `cargo xtask check` writes `target/xtask/last-check.txt`: mode, commit SHA,
dirty flag, start/end time, each step with status, final verdict. (b) `cargo xtask report` prints a
Markdown block from that file (the verbatim last lines plus SHA), refusing if HEAD moved or the tree
is dirty. (c) `.github/pull_request_template.md` with the AGENTS.md report headings and a slot for
the `xtask report` block. (d) AGENTS.md: "Run checks in the foreground. Never end a turn while a
check runs. Paste `cargo xtask report`; do not summarize it."

**Card.** `O-?? feat(xtask): check writes a result file and report prints it for the PR`

### C2. One `preflight` command for the agent's last mile — P2, M

**Observation.** Recurring manual steps and mistakes: wrong branch names, PR titles out of format,
branches far behind `main` (median PR lives 3 min, but agents start from a `main` that is hours
old), editing files outside the card, hotspot files touched without the orchestrator knowing.

**Recommendation.** `cargo xtask preflight [--card <ID>]`: `git fetch origin main`, report commits
behind and files changed on `main` since the merge base that this branch also touches (early
conflict warning), check the branch name against `<type>/<ID>-<slug>`, run the conflict-marker
and decision-ID lints, and, with `--card`, list changed files that the card does not name. It
prints a suggested PR title. Pure functions + git, no new dependency.

**Card.** `O-?? feat(xtask): preflight command for branch, base and card file checks`

### C3. Docs-only and small PRs pay for the full local test run — P2, S

**Observation.** AGENTS.md requires `--quick` before a PR. 22 of 60 PRs are `docs:`; for them
`affected` selects nothing (`affected.rs:310`), yet `--quick` builds and tests everything (420 s
cold on Windows). `check --pr origin/main` is what CI runs and is cheaper for small diffs.

**Recommendation.** After B1, tell agents to run `cargo xtask check --pr origin/main` (after
`git fetch`) — identical to CI and incremental — and keep `--quick` for a full local pass. For
Antigravity text tasks state it in `micro.md`'s common rules.

**Card.** `O-?? docs(agents): run the CI check mode locally before opening a PR`

### C4. ~100 stale worktrees, no hygiene rule — P2, S

**Observation.** 130 directories under `~/.t3/worktrees/oXyFlow.Solitare`, 111 registered worktrees,
98 with their own `target/` (this worktree's `target/` is 2.4 GB after one `--quick`). Nineteen directories are
no longer registered with git. Nothing in `workflow.md` or AGENTS.md says when a worktree is removed.

**Recommendation.** Add a "Worktrees" section to `workflow.md`: after a PR is merged or closed, the
orchestrator removes its worktree (`git worktree remove`, then `git worktree prune`, then
`git branch -D` — `delete_branch_on_merge` already deletes the remote branch). Add
`cargo xtask worktrees` that lists worktrees whose branch is merged (`git branch --merged origin/main`
or PR state via `gh`), with `--remove` to delete them, and that reports orphan directories. The
first run should reclaim most of the disk.

**Card.** `O-?? feat(xtask): list and remove worktrees of merged branches`

### C5. Hotspot files are not visible when cards are planned — P2, S

**Observation.** The rule "two Claude tasks run in parallel only when their file lists do not
overlap" (`task_list/README.md`, Roles) is applied by eye. The hotspots are measurable:
`architecture.md` 26, `task_list/README.md` 16, `phase-3.md` 15, `accessibility_tests.rs` 14,
`locales/*/board.toml` 14, `settings/tests.rs` 12, `app.rs` 10 of 60 PRs.

**Recommendation.** Add a "Shared files" line to the card template listing any of the hotspot files
the card edits; the orchestrator serializes cards that share one, or merges the first before
dispatching the second. After B5, B6 and B7 the list shrinks to `app.rs`, `settings.rs`,
`input.rs` and the en-US/pl-PL locale files.

**Card.** `O-?? docs(plan): shared-files field in task cards and a hotspot list`

### C6. CODEOWNERS and required reviews — not recommended

With one human, `required_approving_review_count: 0` and the orchestrator merging, CODEOWNERS adds
a notification and no protection. Skip it.

### C7. Site updates wait for a release — P3, S

**Observation.** The `site` job runs only after `publish` (`release.yml:237-238`), so a typo fix on
the player site needs a full release.

**Recommendation.** Allow `workflow_dispatch` of a site-only path that rebuilds from `release` with
the tag of the latest published release (`gh release view --json tagName`).

**Card.** `O-?? ci(site): redeploy the player site without a new release`

## Faster

CI is already fast (PR median 1.5 min). The time goes into local builds on Windows across many
fresh worktrees, so most items target that.

### F1. Every worktree starts with a cold build — P1, S

**Observation.** The local cold `--quick` took 420 s: `cargo xtask` itself 1 m 48 s (it builds
`oxysol-tui`, `ratatui`, `resvg`, `syn`, because `xtask/Cargo.toml` depends on them), Clippy
1 m 34 s, first test build 44 s, second test build 1 m 04 s (B1), tests ≈ 90 s. Each of the 98
worktrees compiles all dependencies from scratch into its own `target/`.

**Recommendation.** Install `sccache` once on the owner's machine and set `RUSTC_WRAPPER=sccache`
in the user environment (not in the repo, so CI is unaffected). Dependencies then compile once
across all worktrees. Add a Windows Defender exclusion for `~/.t3/worktrees` and
`%LOCALAPPDATA%\Mozilla\sccache` — real-time scanning of `target/` is a known large slowdown on
Windows. Document both in `workflow.md` ("Local setup on Windows"). Expected: cold worktree
checks drop to roughly the workspace-crate compile time.

**Card.** `O-?? docs(workflow): shared sccache and Defender exclusions for worktrees on Windows`

### F2. The git-derived version recompiles `oxysol-tui` after each commit — P2, M

**Observation.** `crates/tui/build.rs` exports `OXYSOL_VERSION` for the menu label and declares
`rerun-if-changed` on `HEAD`, the current branch ref and `packed-refs`
(`version_scheme.rs:177-190`). On a feature branch every commit adds one first-parent commit, so the
value changes and cargo recompiles `oxysol-tui` — the largest crate (976 unit tests) — and
everything above it, on every commit and after every rebase. Measured in this worktree: after
committing only this Markdown file, the next `--quick` (144 s warm) printed
`Compiling oxysol-tui` in all four cargo steps (xtask build, Clippy, both test builds).

**Recommendation.** Compute the version only in the root `build.rs` and pass it to the TUI at
runtime (a `&'static str` in the TUI's dependency bundle, set by `oxysol`'s `main`). Then only the
thin `oxysol` crate recompiles. Delete `crates/tui/build.rs`. This is a contract change in
`architecture.md` (TUI deps).

**Card.** `O-?? refactor(tui): take the version from the binary instead of a TUI build script`

### F3. Solver tests dominate test time in debug builds — P2, S

**Observation.** In the local run, `oxysol-core` unit tests took 33.4 s and `tests/playout.rs` (2
tests) 29.1 s, against 20.9 s for all 976 TUI tests. They run unoptimized.

**Recommendation.** Add `[profile.dev.package.oxysol-core] opt-level = 2` (and `[profile.test]`
inherits dev) so the engine is optimized in tests while workspace crates above it stay fast to
compile; measure before/after on Windows and on CI. If gains are small, mark the playout tests
`#[ignore]` and run them with `--include-ignored` in the full check only.

**Card.** `O-?? perf(test): optimize the engine in test builds`

### F4. `cargo xtask` compiles the TUI before the first gate runs — P3, M

**Observation.** `xtask/Cargo.toml` depends on `oxysol-core`, `oxysol-tui`, `ratatui` and `resvg`
for `screenshots`, `fonts` and `icon`, so `cargo xtask check` first spends 1 m 48 s (cold) building
them before `fmt` even starts.

**Recommendation.** Move the asset commands (`screenshots`, `fonts`, `icon`, `midi`) into a second
binary target or package (`xtask-assets`), invoked by `cargo xtask` only for those commands; keep
`check`, `lint`, `affected`, `version`, `sync-metadata` with light dependencies. `icon --check` then
runs through the assets binary in the full check.

**Card.** `O-?? refactor(xtask): split asset commands from the check runner`

### F5. Small CI trims — P3, S

**Observation.** `Install the ALSA development package` takes 8–26 s of a 50–75 s PR check step
(runs 44700840121, 44700494421) and runs `apt-get update` each time. Release `check` repeats the
three-OS full check (2.3 min of 7) on a commit `main` already checked.

**Recommendation.** Cache the apt package (or install with `--no-install-recommends` and skip the
update when the package index is fresh). Keep the release re-check: it is cheap insurance on the
publish path.

**Card.** `O-?? ci(pr): cache the ALSA development package`

## Roadmap

Ordered by value per effort. P1 items first; the first four remove most merge breakage.

| # | Item | Card title | Effort | Priority |
|---|---|---|---|---|
| 1 | S1 | chore(repo): require up-to-date branches and enable auto-merge on main | S | P1 |
| 2 | B1 | fix(xtask): check --quick runs every test target in one cargo call | S | P1 |
| 3 | B2 | feat(xtask): lint rule against conflict markers in every tracked file | S | P1 |
| 4 | B3 | feat(xtask): decision ID uniqueness and reference lint | S | P1 |
| 5 | B5 | docs(architecture): drop the status changelog line that every PR edits | S | P1 |
| 6 | C1 | feat(xtask): check writes a result file and report prints it for the PR | S | P1 |
| 7 | F1 | docs(workflow): shared sccache and Defender exclusions for worktrees on Windows | S | P1 |
| 8 | S3 | ci(workflows): pin actions by SHA and keep them current with Dependabot | S | P1 |
| 9 | S2 | ci(deps): cargo-deny for licenses, advisories and sources on pull requests | M | P1 |
| 10 | B6 | refactor(i18n): English fallback for incomplete locales and a locale-diff command | M | P1 |
| 11 | S6 | ci(pr): check pull requests on Windows as well | M | P2 |
| 12 | B7 | test(tui): derive row counts instead of hard-coding them | S | P2 |
| 13 | C4 | feat(xtask): list and remove worktrees of merged branches | S | P2 |
| 14 | B4 | ci(main): re-run jobs that never got a runner before failing main | S | P2 |
| 15 | B8 | ci(site): build the site on pull requests and check screenshots are current | S | P2 |
| 16 | F2 | refactor(tui): take the version from the binary instead of a TUI build script | M | P2 |
| 17 | C2, C3, C5 | preflight command; CI mode locally; shared-files field in cards | S–M | P2 |
| 18 | S4, S5 | provenance, SBOM, immutable releases; pinned host key | S | P2 |
| 19 | F3 | perf(test): optimize the engine in test builds | S | P2 |
| 20 | B9, C7, F4, F5 | pycache, site redeploy, xtask split, apt cache | S–M | P3 |

Items 1, 7 and 8 are repository or machine settings the owner applies; items 2–6 and 9 fit Haiku
cards; B6 and F2 change contracts and suit Sonnet. Items 3–5 and 12 touch different files and can
run in parallel; item 1 should land first so every later PR is tested against the latest `main`.
