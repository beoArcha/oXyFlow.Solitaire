# oXySol task list (MVP)

Started 2026-11-10. Revised 2026-11-20 for R-25: phases 4 to 9 are replanned, phases 0 to 3 are done.
Statuses synced with `main` on 2026-11-26.
Contract behind the cards: `docs/tech/architecture.md`. Decisions: `docs/tech/decisions.md`.

## Phases

Phase N is worked on during version 0.N.x (decision R-25). The plan ends when phase 9 is done, at 0.9.x; 1.0 is a later
owner decision. Phase cards live in `docs/task_list/phase-N.md`. DONE and TODO count the cards of each file (TODO includes
DRAFT). Micro cards in `micro.md` are not counted here; the Index lists the open ones.

| Phase | Version | Name | Goal | Status | DONE | TODO |
|---|---|---|---|---|---|---|
| 0 | 0.0.x | Bootstrap | Repository, CI, release workflow and the first decisions | done | 21 | 0 |
| 1 | 0.1.x | Engine | Domain model and the game contract | done | 4 | 0 |
| 2 | 0.2.x | Games | Klondike, Spider, FreeCell, Pyramid, stock counters and the solver | done | 9 | 0 |
| 3 | 0.3.x | Terminal interface | Menus, board, settings, mouse, overlays, accessibility and polish | done | 51 | 0 |
| 4 | 0.4.x | Workflow, fixes, first card style and game | Harden the workflow first, close the fixes, add the card-style setting, the game menu, Swiss cards and Yukon | done | 51 | 0 |
| 5 | 0.5.x | Themes, colour vision, Spanish cards, Golf | Table and card-face settings, low vision, colour-vision modes, Spanish cards, Golf, Spider solver | done | 59 | 0 |
| 6 | 0.6.x | Themes as data, four-colour French, TriPeaks | Themes as data and the interface size scale, four-colour French cards, TriPeaks, game options, developer bridge | in progress (now) | 5 | 18 |
| 7 | 0.7.x | Hungarian cards, Scorpion | Hungarian suits and the Scorpion game, dev screenshots and batch runner | planned | 1 | 6 |
| 8 | 0.8.x | Player documentation, Russian cards, Canfield | Wiki and player site, release procedure, Russian suits, Canfield, dev layout linter and MCP wrapper | planned | 0 | 7 |
| 9 | 0.9.x | Final tests and the stable release | UI/UX test plan and journeys, fit and accessibility audits, terminal matrix, Portuguese cards, La Belle Lucie, new themes, polish, review and the stable 0.9 release that ends the plan | planned | 0 | 13 |

## Roles

Roles name a model tier, never a model (decision R-26). The orchestrator picks a concrete model of the tier at dispatch.

| Tier | Examples |
|---|---|
| Huge (very large) | Fable, Astra, Carbon |
| Large | Opus, Son Neo |
| Medium | Sonnet, Terra, Hydrogen |
| Small | Haiku, Luna |
| Very small | local models of 10B to 60B parameters |
| Embedded | local models below 10B parameters |

| Role | Tier | Used for |
|---|---|---|
| Orchestrator / reviewer | Large | splits work, dispatches, reviews every PR and turns on its auto-merge, checkpoints F-01…F-03, phase gates, plan upkeep, GitHub settings (R-14) |
| Architect on escalation | Huge | only a contract question the orchestrator cannot settle |
| Implementer | Medium | contracts, keystone code, cross-cutting integration |
| Implementer | Small | fully specified work: scaffolding, screens, storage, workflows, documentation |
| Micro tasks | Small or very small | one small file, no project context, verified by a test or by eye |
| User | — | decisions, checks on real terminals, deployment |

The orchestrator dispatches medium and small implementers, each in its own worktree; two tasks run in parallel only when
their file lists do not overlap. Micro tasks run in parallel, and nothing may wait on a micro task.

Assignments are a starting heuristic, not a measurement. A micro task that fails its check twice goes to a small model; a
small-model task that fails its gate goes to a medium model; a medium-model task that fails twice goes back to the user,
who may call in a huge model.

## How a task flows

1. Branch from current `main`: `<type>/<ID>-<slug>`.
2. Do the card; change only its files; run `cargo xtask check --quick`.
3. Open a PR to `main` with the title `<type>(<scope>): <summary> [<ID>]`.
4. The orchestrator reviews the PR and turns on auto-merge (`gh pr merge <n> --squash --auto`). GitHub squash-merges it once
   the branch is up to date with `main` and the required checks (`check`, `check-windows`, `deny`, `site`) pass.
5. `main.yml` runs the full check on three systems and promotes the commit to `release`.
6. `release.yml` runs the full check again, builds Windows, Linux and macOS binaries, tags a
   pre-release and updates the site (R-17, T-21).
7. The phase gate reviews the merged diff and updates the Status column here.

Cards name their files on a **Files** line and, when they touch a hotspot file listed in
`docs/tech/workflow.md` ("Parallel work and shared files"), on a **Shared files** line; cards that share
one are serialized. Decision IDs a card needs are assigned in the card by the orchestrator.

Micro tasks add three rules: a card is workable only when its status is `READY` (the owning phase
task fills in exact signatures and flips `DRAFT` to `READY`); the agent reads only `AGENTS.md` and
its card; the work must fit one short session.

Statuses: `DRAFT`, `READY`, `TODO`, `IN PROGRESS`, `DONE`, `BLOCKED (reason)`.

## Implementer-lane order

Cards that share a file (their **Shared files** line, or the same **Files** entry) run one after another; the rest may run in
parallel. Each phase ends with its gate; the next phase starts when the gate is green.

1. Phase 4 (0.4), workflow first: O-50 (user setting) and O-51 … O-57 (P1), then O-58 … O-66 (P2), O-67 (P3). Small and Medium
   in parallel where the files differ.
2. Phase 4, fixes: O-37, O-38, O-39 (Small, micro), O-35 (Small).
3. Phase 4, card style: P5-02 (Small) → O-34 (User + Medium) and U-02 (User; reviews M-19…M-24).
4. Phase 4, the rest: P4-07 (Medium) and P3-29 (Medium) → P4-06 (Medium) → P4-08 (Small) → P4-09 (Medium) → P4-10 (Medium, after its
   DRAFT is finalized by the orchestrator). **Milestone: 0.4 closed** (workflow closed, card style and game menu in place, Yukon).
5. Phase 5 (0.5): P3-32 and P3-33 (Small) → P3-34 and P3-35 (Medium) → P5-03 (Small); P5-04 Golf (Medium) and P5-05 Spider solver
   (Medium) in parallel. **Milestone: 0.5.**
6. Phase 6 (0.6, now): P3-36 (Medium) → P6-18 (Medium) and P6-05 (Medium); P6-04 is covered by P3-35; P6-06 after the orchestrator finalizes its scope; the dev bridge
   P6-07 → P6-08 → P6-09 (Medium) in parallel, one PR at the end. **Milestone: 0.6.**
   Solver and hint cards, in this order (phase-6.md, "Order of the solver and hint cards"): P6-12 → P6-13; P6-14 → P6-15 →
   P6-16 → P6-17; P6-11 → O-115; P6-10.
7. Phase 7 (0.7): P7-04 (Small) → P7-05 (Medium, after its DRAFT is final); the dev screenshots P7-06 → P7-07 →
   P7-08 → P7-09 (Medium). **Milestone: 0.7.**
8. Phase 8 (0.8): P7-02 and P7-03 (Small) with the micro cards M-30…M-33 alongside; P8-04 (Small) → P8-05 (Medium, after its DRAFT is
   final); the layout linter P8-06 (Medium) → P8-07 (Small), and the MCP wrapper P8-08 (Medium). **Milestone: 0.8.**
9. Phase 9 (0.9): P9-01 test plan (Medium) → P7-01 (Small) → P9-02 (Medium) with P9-03 (Small) and P9-04 (Medium) → U-03 (User) →
   P9-05 (Small), P9-06 (Medium) and P9-07 (Small), each after its DRAFT is final → P8-01 (Medium) → P8-02 (Medium) → F-03 (orchestrator) →
   P8-03 (Small + User). **Milestone: the stable 0.9 release; the plan ends.**

Micro tasks run alongside; `micro.md` lists what each one waits for. Fifteen of them need only P0-01.

## Index

Every card has exactly one row here and one heading in `docs/task_list/phase-N.md`. Cards moved to a new phase keep their ID.
Status: DONE, TODO, or DRAFT (a card the orchestrator finalizes before dispatch).

### Phase 0 (0.0.x) — Bootstrap

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| U-01 | Decisions, master prompt, repository | User | — | DONE |
| P0-01 | Repository skeleton and workspace | Haiku | U-01 | DONE |
| P0-02 | Master prompt in English with amendments | Haiku | P0-01 (the Polish original must be in git history first) | DONE |
| P0-03 | CI: pull requests, main, promotion | Haiku | P0-01 | DONE |
| P0-04 | Release workflow: build, tag, publish, site | Haiku | P0-03 | DONE |
| O-01 | Record roles, CI, seed and SQLite decisions (R-14…R-19) | not recorded (follow-up) | U-01 | DONE |
| O-03 | CI actions on Node 24 | not recorded (follow-up) | P0-03 | DONE |
| O-04 | Release builds only by manual dispatch | not recorded (follow-up) | P0-04 | DONE |
| O-08 | Release targets: Windows x64, Linux x64/arm64 musl, macOS arm64 | not recorded (follow-up) | O-04 | DONE |
| O-22 | No-sound CI build, cache tuning, author and third-party licences | not recorded (follow-up) | — | DONE |
| O-36 | Version scheme: patch counts merges, minor counts phases, the plan ends at 0.9 | Sonnet | — | DONE |
| O-40 | Player site in nine languages and a refreshed README | Opus | O-30 | DONE |
| O-41 | Seven more interface languages in code, offered once translated | Opus | P3-37 | DONE |
| O-42 | Interface translation: German (de-DE) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-43 | Interface translation: French (fr-FR) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-44 | Interface translation: Italian (it-IT) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-45 | Interface translation: Brazilian Portuguese (pt-BR) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-46 | Interface translation: Ukrainian (uk-UA) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-47 | Interface translation: Dutch (nl-NL) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-48 | Interface translation: Spanish (es-ES) | Antigravity (text only), checked by Opus | O-41 | DONE |
| O-49 | Workflow review: CI, release and agent workflows | Opus | — | DONE |

### Phase 1 (0.1.x) — Engine

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P1-01 | Engine domain model | Sonnet | P0-01 | DONE |
| P1-02 | Game contract and session | Sonnet | P1-01 | DONE |
| F-01 | Checkpoint: engine contract | Opus (Fable only if Opus cannot settle a contract question — R-14) | P1-02 | DONE |
| P1-03 | Comment lint and phase 1 gate | Haiku | P1-02, P0-03, M-01…M-06 | DONE |

### Phase 2 (0.2.x) — Games

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P2-01 | Klondike (Draw 1, Draw 3) | Sonnet | F-01 | DONE |
| P2-02 | Spider (1, 2, 4 suits) | Sonnet | P2-01 | DONE |
| P2-03 | FreeCell | Haiku | P2-01 | DONE |
| P2-04 | Pyramid (Relaxed, Classic) | Haiku | P2-01 | DONE |
| P2-05 | Gate: game rules and flow tests | Sonnet | P2-01…P2-04 | DONE |
| P2-07 | Forced-win detection and the finishing moves | Sonnet | P2-06 | DONE |
| P2-08 | Solver for winnable deals | Sonnet (Fable may be consulted for the search design) | P2-07 | DONE |
| P2-06 | Stock pass and deal counters in the engine | not recorded (follow-up) | P2-05 | DONE |
| O-02 | Phase 2 gate follow-ups | not recorded (follow-up) | P2-05 | DONE |

### Phase 3 (0.3.x) — Terminal interface

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P3-01 | TUI foundation | Sonnet | P1-02 | DONE |
| P3-02 | Board geometry and rendering | Sonnet | P3-01, P2-01 | DONE |
| P3-03 | Board controller | Sonnet | P3-02 | DONE |
| P3-04 | Menus and game selection | Haiku | P3-01 | DONE |
| P3-05 | Settings screen | Haiku | P3-01 | DONE |
| P3-06 | Mouse on the board | Haiku | P3-03 | DONE |
| P3-07 | Overlays: help, pause, win, dialogs | Haiku | P3-03 | DONE |
| P3-08 | Accessibility: announcer and tests | Haiku | P3-03 | DONE |
| P3-09 | Gate: TUI | Sonnet | P3-04…P3-08, P4-03 | DONE |
| P3-11 | Board guidance: focus aura, action line and guide panel | Sonnet | P3-06, P3-08, P3-10 | DONE |
| P3-12 | Board counters, the no-undo mode and copying the seed | Sonnet | P3-11, P4-04, P2-06 | DONE |
| U-04 | Interface review: usability and readability | Fable with Opus (owner request, 2026-11-16) | P3-12, P3-13, O-14 | DONE |
| P3-14 | Readable card faces | Sonnet | U-04 | DONE |
| P3-15 | Screen layout: game information apart from controls | Sonnet | U-04, P3-14 | DONE |
| P3-16 | Zoom: card size by keyboard and console font | Sonnet | U-04, P3-15 | DONE |
| P3-17 | Tab moves the focus | Haiku | U-04 | DONE |
| O-16 | Full-screen option | Sonnet | U-04, P3-16 | DONE |
| P3-18 | Guide panel on screens slightly too small | Sonnet | U-04, P3-15 | DONE |
| P3-19 | Auto-finish a won game | Haiku | P2-07, P3-15 | DONE |
| P3-20 | Winnable deals only | Sonnet | P2-08 | DONE |
| P3-21 | Aura options and the stock recycle button | Haiku | U-04, P3-14 | DONE |
| P3-22 | Localization templates and localized key names | Haiku | P3-15 | DONE |
| P3-23 | Colors: 16-color swatches, mark contrast, High Contrast board edge | Haiku | U-04 | DONE |
| P3-24 | Menus, settings and statistics polish | Haiku | P3-17 | DONE |
| P3-25 | Footer keys as mouse buttons | Haiku | P3-15 | DONE |
| P3-26 | Possible moves panel | Sonnet | P3-15, P3-18 | DONE |
| P3-27 | Quick options during the game and defaults in Settings | Sonnet | P3-19, P3-21, P3-26 | DONE |
| O-23 | Modal fixes: scroll bar, Close shortcut, focus between selector rows | Sonnet | P3-18, P3-25 | DONE |
| P3-28 | Docking modals to a side of the screen | Sonnet | O-23 | DONE |
| P3-30 | Themes Monokai and Dracula | Haiku | P3-23 | DONE |
| P3-31 | Card back selection | Haiku | P3-30 | DONE |
| O-26 | Contrast fixes found by the UI inventory | Haiku | O-25 | DONE |
| P3-10 | How-to-play overlay and first-steps hint | not recorded (follow-up) | P3-07 | DONE |
| P3-13 | Large card faces: big suit and rank on medium and large cards | not recorded (follow-up) | P3-11 | DONE |
| O-06 | Unicode glyphs within WGL4 for the Windows console | not recorded (follow-up) | P3-02 | DONE |
| O-10 | Remove unused common locale keys | not recorded (follow-up) | M-17 | DONE |
| O-13 | Sync task list statuses with `main` | not recorded (follow-up) | — | DONE |
| O-14 | Game rules as a tab of the help window | not recorded (follow-up) | O-13, P3-10 | DONE |
| O-17 | Task cards for the owner's interface feedback of 2026-11-16 | not recorded (follow-up) | — | DONE |
| O-18 | Task cards for the owner's feedback of 2026-11-16 (second round) | not recorded (follow-up) | — | DONE |
| O-20 | Sync follow-up task statuses | not recorded (follow-up) | — | DONE |
| O-21 | Task cards for the owner's feedback of 2026-11-19 | not recorded (follow-up) | — | DONE |
| O-25 | UI inventory of colors, sizes and fonts | not recorded (follow-up) | — | DONE |
| O-27 | Sync the task list with `main` | Haiku | — | DONE |
| P3-37 | Language selector with flags in the main menu corner | Sonnet | P3-24 | DONE |
| P3-38 | Run selection wraps around: Down or `-` on the top card selects the whole run | Haiku | P3-26 | DONE |
| O-28 | Focused card drawn with a gap of felt inside its face | Haiku | — | DONE |
| O-29 | Screen-specific labels for Select and Back | Haiku | P3-26 | DONE |
| O-31 | Widen a console window the game owns to 130 columns | Antigravity (Beo) | — | DONE |
| O-32 | Easy restart from the beginning, confirmed like Quit | Antigravity (Dad) | P3-27 | DONE |
| O-33 | Statistics: scroll bars and a winnable mark per game | Antigravity (Oxy) | O-23, P3-24 | DONE |

### Phase 4 (0.4.x) — Workflow, fixes, first card style and game

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| O-50 | Require up-to-date branches and auto-merge on main (repository setting) | Owner | O-49 | DONE |
| O-51 | `check --quick` runs every test target in one cargo call | Haiku | O-49 | DONE |
| O-52 | Lint: conflict markers in every tracked file | Haiku | O-49 | DONE |
| O-53 | Lint: decision ID uniqueness and references | Haiku | O-49 | DONE |
| O-54 | `check` writes a result file; `cargo xtask report`; PR template | Haiku | O-49 | DONE |
| O-55 | Pin GitHub Actions by SHA and add Dependabot | Haiku | O-49 | DONE |
| O-56 | cargo-deny on pull requests | Sonnet | O-49 | DONE |
| O-57 | English fallback for incomplete locales and `cargo xtask locale-diff` | Sonnet | O-49, O-41 | DONE |
| O-58 | Check pull requests on Windows as well | Haiku | O-49 | DONE |
| O-59 | Tests derive row counts instead of hard-coding them | Haiku | O-49 | DONE |
| O-60 | `cargo xtask worktrees`: list and remove worktrees of merged branches | Haiku | O-49 | DONE |
| O-61 | main: re-run jobs that never got a runner before failing | Haiku | O-49 | DONE |
| O-62 | Build the site on pull requests; check screenshots are current | Haiku | O-49 | DONE |
| O-63 | Take the version from the binary instead of a TUI build script | Sonnet | O-49 | DONE |
| O-64 | `cargo xtask preflight` and a local CI mode | Haiku | O-49 | DONE |
| O-65 | Release provenance, SBOM, immutable releases; pinned host key | Sonnet | O-49 | DONE |
| O-66 | Optimize the engine in test builds | Haiku | O-49 | DONE |
| O-67 | Small cleanups: pycache, site redeploy, xtask split, apt cache | Antigravity / Haiku | O-49 | DONE |
| O-37 | Mouse path for Pause and Possible moves on every screen width | Haiku | P3-16, P3-28 | DONE |
| O-38 | Settings save failure: alert and log once per session | Haiku | — | DONE |
| O-39 | Split files over 300 LOC found by the P3-09 gate | Antigravity | — | DONE |
| O-35 | Release archives carry the font licence (OFL) | Haiku | P5-01 (the font, `assets/fonts/OFL.txt` and `assets/fonts/NOTICE.md` exist) | DONE |
| P5-02 | Style-driven cards and graphics check | Haiku | P5-01, P3-05; the `Console` trait of P3-16 in `oxysol` (`src/`), which reads and sets the | DONE |
| O-34 | Windows Terminal: test the card font on a real machine | User + Sonnet | P5-02 | DONE |
| U-02 | Artwork review | User | P5-02, M-19…M-24 | DONE |
| P4-07 | Site: language landing page | Sonnet | O-40, O-62 | DONE |
| P3-29 | Settings: an Interface section (colors, aura, size, font) | Sonnet | P3-16, P3-21, P3-24, P3-27 | DONE |
| P4-06 | Automatic moves in statistics: failed attempts and streaks | Sonnet | P4-05 | DONE |
| P4-08 | Card style: Swiss suits (Unicode) | Haiku | P5-02 | DONE |
| P4-09 | Game menu: stars, favourites, families and search | Sonnet | P3-04, P3-17, O-59 | DONE |
| P4-10a | Game: Yukon (classic) | Opus | P4-09, P1-02 | DONE |
| P4-10b | Yukon variant: Russian Solitaire | Haiku | P4-10a | DONE |
| P4-10c | Yukon variant: Alaska | Haiku | P4-10b | DONE |
| P4-10d | Yukon variant: Moosehide | Haiku | P4-10c | DONE |
| P4-10e | Yukon variant: Australian Patience (stock, one pass) | Sonnet | P4-10d | DONE |
| P4-10f | Solver for the Yukon family: winnable deals and the statistics check | Opus | P4-10e, P2-08 | DONE |
| P4-01 | SQLite persistence | Sonnet | P3-01 | DONE |
| P4-02 | Logging, panic hook, entry point | Haiku | P3-01, M-18 | DONE |
| P4-03 | Autosave and resume | Sonnet | P3-03, P4-01 | DONE |
| P4-05 | Restarts counted separately in games without undo | Sonnet | P4-04, P3-12 | DONE |
| P4-04 | Statistics, game history and seed copying | not recorded (follow-up) | P4-03 | DONE |
| O-05 | Keep a double-clicked console open after a startup error | not recorded (follow-up) | P4-02 | DONE |
| O-07 | Portable mode: data next to the executable | not recorded (follow-up) | P4-01 | DONE |
| O-09 | Application icon for the Windows executable | not recorded (follow-up) | P0-01 | DONE |
| O-11 | `--seed` starts the game directly; typed keys stay in text fields | not recorded (follow-up) | P3-10 | DONE |
| O-15 | `--new-window` flag and the command-line options in Help | not recorded (follow-up) | O-05, O-14 | DONE |
| O-19 | `--new-window`: alacritty, kitty, wezterm and foot on Linux | not recorded (follow-up) | O-15 | DONE |
| P4-11 | Language selector: stray glyph, reachable with Up, Tab hint in the footer | Haiku | P3-37 | DONE |
| P4-13 | Process priority: `--priority` flag, Settings row, solver threads below the UI thread | Sonnet | P3-29 | DONE |
| P4-14 | Docs-only pull requests run a docs check instead of the full build; light `xtask` (O-67 F4) | Sonnet | O-62, O-64 | DONE |
| P4-12 | Statistics: a clear winnable column and a check for unchecked deals | Haiku | P4-04, P2-08 | DONE |
### Phase 5 (0.5.x) — Themes, colour vision, Spanish cards, Golf

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P3-32 | Table color and feel | Small | O-26, P3-29 | DONE |
| P3-33 | Card face tint | Small | O-26, P3-29 | DONE |
| P3-34 | Low-vision mode | Medium | O-26, P3-16, P3-29 | DONE |
| P3-35 | Color-blind modes | Medium | O-26, P3-29 | DONE |
| P5-03 | Card style: Spanish suits (Unicode) | Small | P4-08 | DONE |
| P5-04 | Game: Golf | Medium | P4-10a | DONE |
| P5-05 | Solver: Spider with 2 and 4 suits | Medium | P2-08 | DONE |
| X-01 | Spike: card font in real terminals | Sonnet (prototype) + User (testing) | P0-01 | DONE |
| F-02 | Checkpoint: graphics delivery | Opus (Fable only on escalation — R-14) | X-01 | DONE |
| P5-01 | Font builder and preview sheet | Haiku | F-02 | DONE |
| P5-06 | New game from seed: game first, then a seed from played games, random replay, five difficulty levels | Medium | P4-09, P2-08 | DONE |
| P5-07 | Better hints: a hint that leads toward a win | Medium | P4-10a | DONE |
| O-68 | Help window: a mouse click on the Keys and Command line tabs switches to them | Haiku | — | DONE |
| O-69 | Help window: one height for every tab; optional backdrop dimming | Haiku | — | DONE |
| O-70 | Language tooltip disappears when a window opens | Haiku | — | DONE |
| O-71 | Help window: a separator between the scrolling text and the footer | Haiku | — | DONE |
| O-72 | Docked Help: a margin between the version label and the panel | Small | — | DONE |
| O-73 | Docked Help: dim the selection in the pane without focus | Haiku | — | DONE |
| O-74 | Help window: one footer layout and scroll indicator on every tab | Medium | O-71 | DONE |
| O-75 | Windows: the active tab and the focused button look different | Medium | — | DONE |
| O-76 | Docking and pane focus keys shown where they apply | Small | — | DONE |
| O-77 | Docked Help: key and description on one line | Medium | O-74 | DONE |
| O-78 | Main menu: Resume set apart and balanced under the logo | Small | — | DONE |
| O-79 | Bottom key bar: keys in an accent colour | Small | — | DONE |
| O-80 | Language selector: quieter at rest, clearer when open | Small | O-70 | DONE |
| O-81 | Window title by context, en-US spelling and wording | Small | — | DONE |
| O-82 | Statistics footer: a space between `Copy seed` and `Esc Back` | Haiku | — | DONE |
| O-83 | Board footer: the `Do:` prefix is clipped when a side panel is open | Haiku | — | DONE |
| O-84 | Board footer and `Now:` row: no overflow into the status line; the source column kept | Sonnet | O-83 | DONE |
| O-85 | `Turned over` counter: counts what its label says | Small | — | DONE |
| O-86 | Settings: `Font face` shows its own value | Haiku | — | DONE |
| O-87 | Large cards: a spade with a clear stem | Haiku | — | DONE |
| O-88 | Stock card back: no stray marks in the centre | Haiku | — | DONE |
| O-89 | Keyboard focus skips face-down cards | Small | — | DONE |
| O-90 | Quick options: the app's selection colour, one name for Aura, real buttons | Small | O-75 | DONE |
| O-91 | Possible moves: readable unavailable entries, active ones first | Haiku | — | DONE |
| O-92 | Guide panel: `▶` means focus only | Haiku | — | DONE |
| O-93 | Footer: the `v: moves` highlight clears | Small | — | DONE |
| O-94 | Settings: unavailable rows dimmed; no reference to a missing screen | Haiku | — | DONE |
| O-95 | Statistics: a plain summary line and footer hints for the action bar | Medium | O-82 | DONE |
| O-96 | Settings: labelled Aura swatches, a Performance section, Left/Right in the footer | Small | — | DONE |
| O-97 | Statistics table: results by style, the legend's unchecked mark, local time | Small | O-95 | DONE |
| O-98 | Choose a game: en-US spelling, spacing, a key and wording for Winnable deals, full footer | Haiku | — | DONE |
| O-99 | Large cards: consider dropping the centred rank and suit line (optional) | Small | O-87 | DONE |
| O-100 | Spider and Pyramid: no `A` in slots that are not ace foundations | Small | — | DONE |
| O-101 | Yukon footer: `Enter: pick up` and `a: quick move` when a card can move | Small | — | DONE |
| O-102 | Settings: a live preview pane instead of the 4 × 1 swatch | Medium | — | DONE |
| O-103 | Settings in three tabs: Audio & Visual, Popular (default), Gameplay | Medium | O-102 | DONE |
| O-104 | FreeCell: a gap between the free cells and the foundations | Small | — | DONE |
| O-105 | Pyramid: blocked cards look different from playable ones | Small | — | DONE |
| O-106 | Covered cards: no clipped big pips (FreeCell, Yukon, Pyramid) | Medium | O-87 | DONE |
| O-107 | Yukon: column 7 never runs off the bottom | Medium | — | DONE |
| O-108 | Install scripts that check dependencies first (Linux, macOS, Windows) | Medium | — | DONE |
| O-109 | Full screen recovery and short CLI flags | Medium | — | DONE |
| O-110 | Quick move from the exposed waste pile | Medium | — | DONE |
| O-111 | Default console width and three window sizes | Medium | O-109 | DONE |
| O-112 | Roles by model tier instead of model names (R-26) | Large | — | DONE |
| O-113 | New game: described in the guide and a button on the board | Small | P3-35 | DONE |
| O-114 | Quick move can be turned off (Quick options and Settings) | Small | P3-35 | DONE |

### Phase 6 (0.6.x) — Themes as data, four-colour French, TriPeaks

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P3-36 | Themes as data and the interface size scale | Medium | P3-29, P3-30 | DONE |
| P6-18 | The interface size scale S / M / L / XL | Medium | P3-36 | DONE |
| P6-04 | Card style: four-colour French | Small | P5-03, P3-35 | DONE |
| P6-05 | Game: TriPeaks | Medium | P5-04 | DONE |
| P6-06 | More game options: Klondike passes and empty column, Spider deal, Golf wrap and King | Medium | P4-09, P3-27 | TODO |
| P6-07 | Dev bridge: headless driver, virtual clock and UI state snapshot | Medium | T-12, T-14 | TODO |
| P6-08 | Dev bridge: `--dev-bridge` JSON-lines protocol | Medium | P6-07 | TODO |
| P6-09 | Dev bridge: `--dev-log` structured log and determinism tests | Medium | P6-08 | TODO |
| O-115 | Is it winnable? A check for the deal and for the current position | Medium | P5-05 (merged) | TODO |
| P6-10 | Hint and in-play check knowledge: full, or the player's with a stock memory | Medium | P5-07, O-115, P6-11 | TODO |
| P6-11 | Hints off the redraw path, and no repeated positions | Medium | P5-07 | TODO |
| P6-12 | Solver: no heap allocation per node | Medium | — | DONE |
| P6-13 | Winnable deals: a small thread pool, same seed as one thread | Medium | — | TODO |
| P6-14 | Klondike solver: best-first, a canonical draw-1 talon, safe cards stay home | Medium | P6-12 | TODO |
| P6-15 | Yukon family and Pyramid Relaxed: best-first and complete pruning | Medium | P6-12, P6-14 | TODO |
| P6-16 | Golf solver: a proof by rank groups, a packed key, a lower budget | Medium | P6-12 | TODO |
| P6-17 | Spider 2 and 4 suits: a beam stratified by deals left | Medium | — | TODO |
| O-116 | The bottom bar: `a`, `e` and `s` by preference, only the ones that are available | Medium | O-110 | TODO |
| O-117 | A setting for `e`: off, semi-automatic (to the focused pile) or automatic | Medium | O-116 | TODO |
| O-118 | Esc on the main menu asks whether to quit | Small | — | DONE |
| O-119 | Quick options always offer New game, Restart and the winnable check | Medium | O-115, O-117 | TODO |
| O-120 | Settings: clear game history and statistics (soft delete) | Medium | — | TODO |
| O-121 | Achievements: a menu entry and the latest one in the top right corner | Medium | O-120 | DRAFT |
| O-122 | No moves left: a dialog that the game cannot be won, with Restart, New game and Menu | Medium | — | TODO |
| O-123 | Game menu: scrolling by wheel and page keys, families that fold | Medium | — | TODO |
| O-124 | Help, Keys tab: the keys of the game menu | Small | O-123 | TODO |
| P6-01 | MIDI sequencer and synthesizer | Sonnet | P3-01 | DONE |
| P6-02 | Audio output and settings | Haiku | P6-01, P3-05 | DONE |
| P6-03 | Score-to-MIDI tool | Haiku | P0-01 | DONE |
| O-12 | Selectable sound styles and clearer sound status in Settings | not recorded (follow-up) | P6-02, P3-11 | DONE |

### Phase 7 (0.7.x) — Hungarian cards, Scorpion

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P7-04 | Card style: Hungarian suits (Unicode) | Small | P6-04 | TODO |
| P7-05 | Game: Scorpion | Medium | P6-05 | DRAFT |
| P7-10 | FreeCell family: Baker's Game and a choice of 1 to 4 cells | Medium | P6-05, P6-12 | DRAFT |
| P7-11 | FreeCell family: Eight Off and Seahaven Towers | Medium | P7-10 | DRAFT |
| P7-12 | TriPeaks family: no K–A wrap, and Four Peaks | Medium | P6-05 | DRAFT |
| P7-13 | Solver for the game options of P6-06 | Medium | P6-06, P6-14 | TODO |
| P7-06 | Screenshots: one SVG renderer for the dev bridge and the player site | Medium | P6-07 | TODO |
| P7-07 | Screenshots: deterministic PNG and image paths in the bridge `snapshot` | Medium | P7-06, P6-08 | TODO |
| P7-08 | Batch runner: `--dev-script` and `--dev-out` | Medium | P7-07, P6-09 | TODO |
| P7-09 | Baseline scenario pack and the viewport runner | Medium | P7-08 | TODO |
| O-30 | Player site: screenshots, both languages, latest release, build from source | Sonnet | — | DONE |

### Phase 8 (0.8.x) — Player documentation, Russian cards, Canfield

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P7-02 | Player site and stable release procedure | Small | P0-04, P5-01, M-30…M-33 | TODO |
| P7-03 | Technical documentation and README | Small | P3-09 | TODO |
| P8-04 | Card style: Russian suits (Unicode) | Small | P7-04 | TODO |
| P8-05 | Game: Canfield | Medium | P7-05 | DRAFT |
| P8-06 | Layout linter: frame trace and the six rules | Medium | P6-09 | TODO |
| P8-07 | Layout linter: `--dev-strict` gate | Small | P8-06, P7-08 | TODO |
| P8-08 | Thin MCP wrapper over `--dev-bridge` | Medium | P8-06, P7-07 | TODO |

### Phase 9 (0.9.x) — Final tests and the stable release

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| P9-01 | UI/UX test plan | Medium | P8-04, P8-05, P7-03 | TODO |
| P7-01 | E2E harness and core scenarios | Small | P3-09, P4-02 | TODO |
| P9-02 | Keyboard and mouse journeys per screen and per game | Medium | P9-01, P7-01 | TODO |
| P9-03 | Locale fit at the minimum size, all nine languages | Small | P9-01, O-57 | TODO |
| P9-04 | Accessibility audit | Medium | P9-01, P3-34, P3-35 | TODO |
| U-03 | Terminal matrix | User | P7-01 | TODO |
| P9-05 | Card style: Portuguese suits (Unicode) | Small | P8-04 | TODO |
| P9-06 | Game: La Belle Lucie | Medium | P8-05 | DRAFT |
| P9-07 | New themes | Small | P3-36 | DRAFT |
| P8-01 | Fixes and visual polish | Medium | U-03 | TODO |
| P8-02 | Final review | Medium | P8-01 | TODO |
| F-03 | Checkpoint: release sign-off | Large (Huge only on escalation — R-14) | P8-02 | TODO |
| P8-03 | Stable 0.9 release that ends the plan | Small + User | F-03, P7-02 | TODO |

Micro cards (`micro.md`, not in this index): open ones are M-19…M-24 (phase 4, READY, artwork), M-30, M-31, M-33 (phase 8,
READY) and M-32 (phase 8, DRAFT until P5-02 is merged), M-28 and M-29 (phase 9, DRAFT until P7-01 is merged).

## Start prompts

Phase task (medium or small model):

```text
You work on oXySol. Read AGENTS.md, docs/tech/architecture.md and card <ID> in
docs/task_list/phase-<N>.md. Do only that task on branch <type>/<ID>-<slug>, run
`cargo xtask check --quick`, open a PR to main, and give the final report from AGENTS.md.
```

Micro task (small or very small model):

```text
You work on oXySol. Read AGENTS.md and card <ID> in docs/task_list/micro.md — nothing else.
Do only that card on branch <type>/<ID>-<slug>. Touch only the files it lists.
Run `cargo xtask check --quick`, open a PR to main, and give the final report from AGENTS.md.
```
