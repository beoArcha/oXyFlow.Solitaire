# Decisions

Updated 2026-11-15. This file overrides `master_prompt.md` where they differ.

## Open — owner: user

### D-06 Card graphics delivery — confirmed after spike X-01

Proposed in T-16: vector art reaches the terminal as a font. Consequence to accept: a player sees the
custom artwork only after installing the oXySol card font and enabling it in the terminal; everyone
else gets the Unicode or ASCII rendering. The app cannot install or select a font.

### Assumed defaults (object if wrong)

- MVP front styles: International (default) plus the skins French, German and Italian. MVP back styles: two.
- MVP sound: event cues plus one optional looping music track per sound style; two sound styles.
- Klondike: unlimited passes through the stock.
- No scoring, move counter or auto-finish in MVP.
- FreeCell does not reproduce Microsoft deal numbers.
- `NO_COLOR` is not handled in MVP.

## Resolved by the user (2026-11-10) — amendments to the master prompt

| ID | Decision | Amends |
|---|---|---|
| R-01 | Everything is produced in English: code, docs, replies. Other languages are translations; `en-US` is the source locale. | §13, §19 |
| R-02 | A deck style is graphics only. Every MVP style has 52 cards (4 suits × 13 ranks). Fronts and backs are separate, independently selectable styles. | §9 |
| R-03 | Graphics sources are vector only. "SVG" means vector artwork, delivered to the terminal as fonts (preferred) or, later, rasterized. | §6 |
| R-04 | Sound is in MVP, as MIDI. Each sound style lives in its own folder. Streaming-service integration (Spotify, Apple Music, YouTube Music) is post-MVP. | §10 |
| R-05 | Accessibility ranks with localization. The TUI is fully usable by keyboard and by mouse. | §13 |
| R-06 | Platforms: Windows 10/11, macOS, Linux. iOS and Android are post-MVP. | §5 |
| R-07 | Two protected branches, `main` and `release`. Every change goes through its own `feat`/`fix`/`refactor`/`chore`/… branch and a squash-merged PR. `main` runs the full test cycle and, when green, promotes automatically to `release`. Deployment from `release` is a manual click that builds, publishes artifacts and updates the web page. | §18 |
| R-08 | CI budget: pull requests run formatting, lints, unit and integration tests only. Full E2E and architecture tests run on `main`. | §18 |
| R-09 | Roles: Fable is architect and manager and does not do routine writing. Sonnet and Haiku implement. Antigravity (Gemini 3.8 Flash) instances get only small, self-contained tasks because they lose context and are slow. | — |
| R-10 | License: MIT, for code and for artwork, fonts and sound built from our own sources. A patched third-party base font (T-16) would keep its own license. | §20 |
| R-11 | Pyramid ships as two games: Relaxed (3 passes through the stock, win when the pyramid is cleared) and Classic (1 pass, win when all 52 cards are removed). Configurable per-game options are post-MVP. | §8 |
| R-12 | The repository is public and lives at `github.com/beoArcha/oXyFlow.Solitare`. The name `oXyFlow` is taken on GitHub by an unrelated personal account, so no organization of that name is used; `oXyFlow` remains the project group name. | §2, §18 |
| R-13 | The default deck style is International (♣ ♦ ♥ ♠ with A, K, Q, J). French, German, Italian and any later style are skins of the same 52 cards. | §9 |

## Resolved by the user (2026-11-15) — process, CI, seeds, storage

| ID | Decision | Supersedes |
|---|---|---|
| R-14 | Roles: Claude Opus 5.5 is orchestrator and reviewer — it splits work, reviews every PR, runs the checkpoints F-01…F-03 and the phase gates. Sonnet 5.5 and Haiku 5.5 implement. Fable only by exception (a contract question Opus cannot settle). Antigravity Beo, Oxy, Son, Dad keep the micro tasks. | R-09 (roles) |
| R-15 | Pull requests are small: one logical piece of functionality, always compiling, always leaving a working program, with its tests (unit, integration, E2E, architecture — whichever apply) in the same PR. | — |
| R-16 | A pull request is checked when it is opened, reopened and on every new commit: formatting, Clippy, project lint, the full architecture and E2E suites, and unit and integration tests **incrementally** — only for the packages the diff touches and the packages that depend on them. | R-08 |
| R-17 | **Revised by R-20.** `main` runs the full check on Windows, Linux and macOS. When it is green and `release` differs, `release` is fast-forwarded to it automatically. `release` runs the full check on every update and on manual dispatch; that run ends by building the game for Windows, Linux and macOS, tagging the version, publishing a GitHub Release with the artifacts and updating the documentation site. | R-07 (manual deploy) |
| R-18 | Every deal has a seed (`u64`). The same seed gives the same deal on every platform and app version. The seed is shown to the player, a game can be started from a typed seed (menu and `--seed`), and every dealt seed is recorded in the database with variant, start and end time and outcome. | — |
| R-19 | Persistence uses SQLite: one database file holds settings, the saved game and the game history. Diagnostics go to a full plain-text log file. | T-02 |
| R-20 | (2026-11-15) Building and publishing a release is started only by the user: `release.yml` runs on manual dispatch, never on a push. `main` still fast-forwards `release` automatically after a green full check, so `release` always holds the latest tested commit ready to be released. | R-17 (automatic release run) |
| R-21 | Every release platform also ships a **portable** archive. A portable copy keeps all its files (`oxysol.db`, `oxysol.log`) in a `data` folder next to the executable and writes nothing to system folders. A copy is portable when a file named `oxysol.portable` exists in the same directory as the executable. | — |

## Technical decisions — owner: architect

| ID | Decision | Rationale |
|---|---|---|
| T-01 | Workspace: `oxysol-core` (engine plus four games as modules), `oxysol-tui`, root package `oxysol` (lib + bin), tooling crate `xtask`. No separate `games` crate. | Cargo enforces the engine/TUI boundary at compile time. A games crate adds nothing over `pub(crate)` modules. |
| T-02 | **Revised by R-19.** SQLite through `rusqlite` with feature `bundled` (compiled from source, no system library). One file `oxysol.db` in the data directory; schema version in `PRAGMA user_version` with forward-only migrations in code; every write in a transaction. Structured values (`Settings`, `SessionRecord`) are stored as JSON text (`serde_json`) in columns. | Game history and seeds need queries; the bundled build works on all CI runners, musl included (`musl-tools`). Transactions replace temp-file-and-rename. |
| T-03 | A saved game is variant + seed + move log + history cursor + elapsed time. State is rebuilt by replaying the log; the same mechanism implements undo, redo and resume. | One mechanism instead of three, no inverse moves, tiny files. Cost: a change to rules or shuffling invalidates old saves — guarded by golden-deal tests and the `format` field. |
| T-04 | Own SplitMix64 generator and Fisher–Yates shuffle in `core::rng`. | A seed must give the same deal on every platform and app version (needed by T-03). About 30 lines, no dependency. |
| T-05 | A shared command vocabulary (`Move`, `PileRef`) and a small `Game` trait; rules of each game in its own module. | The TUI drives all games with one interaction code path while mechanics stay separate. |
| T-06 | Own minimal localization: TOML files `locales/<lang>/<area>.toml`, embedded, string keys, fallback to `en-US`. Messages are worded to avoid plural forms. | A few dozen strings do not justify a framework. Per-area files avoid merge conflicts. |
| T-07 | Styles are data folders: `assets/cards/fronts/<style>/`, `assets/cards/backs/<style>/`, `assets/audio/<style>/`, each with a `style.toml` manifest. A build script embeds every folder it finds. | Adding a style is adding a folder; no code changes, no shared file to conflict on. |
| T-08 | Tooling as `cargo xtask`, not shell scripts. The custom checker covers only project rules (LOC, nesting, comments in functions, error policy); everything else is Clippy and compiler lints. | One tooling codebase for three systems. |
| T-09 | Error types via `thiserror`, no `anyhow`. CRITICAL = `tracing::error!` with target `oxysol::critical`. | Precise types at every boundary; `tracing` has no CRITICAL level. |
| T-10 | Windows: default console subsystem, no `AttachConsole`/`AllocConsole`, no `windows_subsystem`. | A console app started from a terminal uses that console; started by double-click it gets its own window from the OS. No code, no `unsafe`. Verified manually in U-03. Exception (O-05): after an error exit, a process whose console it alone owns (a double-click) waits for Enter before the window closes, via `console::owns_window` (one `GetConsoleProcessList` call). |
| T-11 | Minimum grid 100 × 30 cells; below that a notice screen. | 854 × 480 px at an 8 × 16 px cell is 106 × 30; 100 columns leaves margin for other fonts. |
| T-12 | E2E tests run in-process: `App` + `TestBackend` + real `SqliteStorage` in a temp dir, plus a binary smoke test. | Pseudo-terminal tests differ per OS and are flaky. |
| T-13 | Scaling means card size, spacing and margins in cells. | The user's terminal controls the font size. |
| T-14 | Clock and seed generation live outside the engine. | The engine stays deterministic and free of I/O. |
| T-15 | Dependencies are added only by phase tasks on the Claude lane. Micro tasks never touch `Cargo.toml` or `Cargo.lock`. | No merge conflicts, controlled dependency list. |
| T-16 | **Graphics (proposed, pending X-01):** SVG sources are compiled into a card font whose glyphs sit in the Unicode Private Use Area. Three render modes: Font, Unicode, ASCII. The font cannot be detected, so Font mode is enabled by the player in a graphics-check screen; the default is Unicode. Single-cell glyphs (suits, emblems, back tiles) are the MVP scope; multi-cell card illustrations and a patched full monospace font for terminals without font fallback depend on the spike. Raster protocols (Kitty, Sixel, iTerm2) are post-MVP and would use the same SVGs. | A standard terminal draws only font glyphs. Fonts stay sharp at any size and work over SSH because rendering happens on the player's machine. |
| T-17 | **Audio:** `.mid` files per sound style, parsed with `midly`, played by a small built-in polyphonic synthesizer (oscillators and envelopes; timbres defined in `style.toml`), output through `cpal`, behind cargo feature `audio`. No audio device, or an SSH session, means audio is off and the game is unaffected. No SoundFont in MVP. The TUI `Audio` trait keeps cues and music separate so an external music source can replace music later. | A SoundFont adds megabytes and licensing review; a built-in synth keeps sound styles small, text-defined and authorable by agents. The sound is deliberately simple. |
| T-18 | Linux ships two artifacts: a glibc build with audio and a static musl build without audio. | Servers reached over SSH often lack ALSA; a binary linked to it would not start there. |
| T-19 | **Revised by R-16, R-17.** `pr.yml` (Linux) runs `cargo xtask check --pr <base>`: fmt, Clippy, lint, the architecture and E2E suites, and unit/integration tests of affected packages only. `main.yml` runs `cargo xtask check` on Linux, Windows and macOS, then a `promote` job fast-forwards `release` and dispatches `release.yml`. `release.yml` (push to `release`, `workflow_dispatch`) runs the full check on three systems, builds, tags, publishes the GitHub Release and deploys GitHub Pages. | The repository is public, so Actions minutes on standard runners are free and PRs can afford the E2E and architecture suites. |
| T-21 | Release versions: a `release.yml` run publishes a pre-release `v<version>-dev.<n>`, `n` = `git rev-list --count HEAD`; with `stable: true` it publishes `v<version>` and fails if that tag exists. `<version>` comes from `metadata.toml` and names the next stable version. | Every requested build is downloadable and correctly ordered without a version-bump PR; stable releases stay deliberate. |
| T-22 | Affected packages: `cargo xtask affected --base <ref>` maps changed paths to workspace packages and adds reverse dependents (`oxysol-core` → `oxysol-tui` → `oxysol`; `xtask` alone). A change to `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, `.cargo/`, `.github/`, `locales/` or `assets/` selects every package; a change only under `docs/` or to `*.md` selects none. | Deterministic, no extra tool, testable as a pure function. |
| T-23 | Promotion credential: the `promote` job pushes to `release` with a write deploy key (secret `RELEASE_DEPLOY_KEY`), the only bypass actor of the `release` ruleset. It starts no release run (R-20). | `GITHUB_TOKEN` cannot bypass a ruleset on a personal repository, and its pushes start no workflows. |
| T-24 | Logging: `tracing` to `oxysol.log`, default level `info`, `OXYSOL_LOG` overrides. Every deal (variant, seed), resume, outcome and storage error is logged; moves at `debug`. | "Full" text logs that reproduce a session together with the seed. |
| T-20 | Task status lives in the index of `docs/task_list/README.md` and is updated by phase gates, not by executors. | Fewer steps for small models and no status-only merge conflicts. |
| T-25 | Release targets (2026-11-15, owner): x86_64-pc-windows-msvc, x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin; each as a normal and a `-portable` archive (marker `oxysol.portable`, R-21). | Static musl runs on any distro; Apple Silicon only. |
