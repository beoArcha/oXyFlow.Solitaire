# oXySol task list (MVP)

Started 2026-11-10. Revised 2026-11-15 for R-14…R-19 (roles, CI, seeds, SQLite). Statuses synced with `main` on 2026-11-19.
Contract behind the cards: `docs/tech/architecture.md`. Decisions: `docs/tech/decisions.md`.

## Roles

| Role | Who | Used for |
|---|---|---|
| Orchestrator / reviewer | Claude Opus 5.5 | splits work, dispatches, reviews every PR, checkpoints F-01…F-03, phase gates, plan upkeep, GitHub settings (R-14) |
| Architect on escalation | Claude Fable | only a contract question Opus cannot settle |
| Implementer | Claude Sonnet 5.5 | contracts, keystone code, cross-cutting integration |
| Implementer | Claude Haiku 5.5 | fully specified work: scaffolding, screens, storage, workflows, documentation |
| Micro tasks | Antigravity (Gemini 3.8 Flash): **Beo, Oxy, Son, Dad** | one small file, no project context, verified by a test or by eye |
| User | — | decisions, checks on real terminals, merging PRs, deployment |

Opus dispatches Sonnet and Haiku as subagents, each in its own worktree; two Claude tasks run in
parallel only when their file lists do not overlap. Beo, Oxy and Son run micro tasks in parallel. Dad exists on one computer only: it is an
overflow lane that takes any `READY` micro card nobody has started, and nothing may wait on it.

Assignments are a starting heuristic, not a measurement. A micro task that fails its check twice
goes to Haiku; a Haiku task that fails its gate goes to Sonnet.

## How a task flows

1. Branch from current `main`: `<type>/<ID>-<slug>`.
2. Do the card; change only its files; run `cargo xtask check --quick`.
3. Open a PR to `main` with the title `<type>(<scope>): <summary> [<ID>]`.
4. `pr.yml` must be green and Opus reviews the PR. The PR is squash-merged.
5. `main.yml` runs the full check on three systems and promotes the commit to `release`.
6. `release.yml` runs the full check again, builds Windows, Linux and macOS binaries, tags a
   pre-release and updates the site (R-17, T-21).
7. The phase gate reviews the merged diff and updates the Status column here.

Micro tasks add three rules: a card is workable only when its status is `READY` (the owning phase
task fills in exact signatures and flips `DRAFT` to `READY`); the agent reads only `AGENTS.md` and
its card; the work must fit one short session.

Statuses: `DRAFT`, `READY`, `TODO`, `IN PROGRESS`, `DONE`, `BLOCKED (reason)`.

## Claude-lane order

1. P0-01, P0-02 (Haiku) — done
2. P0-03 → P0-04 (Haiku) in parallel with P1-01 → P1-02 (Sonnet) → F-01 (Opus)
3. X-01 prototype (Sonnet) — the user tests it on real terminals while work continues
4. P1-03 (Haiku)
5. P2-01, P2-02 (Sonnet) → P2-03, P2-04 (Haiku) → P2-05 (Sonnet) — **milestone: engine complete**
6. P3-01, P3-02, P3-03 (Sonnet) — **milestone: playable in the terminal**
7. P4-01, P4-02 (Haiku) → P4-03 (Sonnet)
8. P3-04 … P3-08 (Haiku) → P3-09 (Sonnet)
9. F-02 (Opus, needs X-01 results) → P5-01, P5-02 (Haiku) → U-02
10. P6-01 (Sonnet) → P6-03, P6-02 (Haiku) — **milestone: feature complete**
11. P7-01, P7-02, P7-03 (Haiku)
12. U-03 → P8-01, P8-02 (Sonnet) → F-03 (Opus) → P8-03 (Haiku + user) — **milestone: 0.1.0**

Micro tasks run alongside; `micro.md` lists what each one waits for. Fifteen of them need only P0-01.

## Index

| ID | Task | Owner | Depends on | Status |
|---|---|---|---|---|
| U-01 | Decisions, master prompt, repository | User | — | DONE |
| P0-01 | Repository skeleton and workspace | Haiku | U-01 | DONE |
| P0-02 | Master prompt in English with amendments | Haiku | P0-01 | DONE |
| P0-03 | CI: pull requests, main, promotion | Haiku | P0-01 | DONE |
| P0-04 | Release workflow: build, tag, publish, site | Haiku | P0-03 | DONE |
| P1-01 | Engine domain model | Sonnet | P0-01 | DONE |
| P1-02 | Game contract and session | Sonnet | P1-01 | DONE |
| F-01 | Checkpoint: engine contract | Opus | P1-02 | DONE |
| P1-03 | Comment lint and phase 1 gate | Haiku | P1-02, P0-03, M-01…M-06 | DONE |
| P2-01 | Klondike | Sonnet | F-01 | DONE |
| P2-02 | Spider | Sonnet | P2-01 | DONE |
| P2-03 | FreeCell | Haiku | P2-01 | DONE |
| P2-04 | Pyramid (Relaxed, Classic) | Haiku | P2-01 | DONE |
| P2-05 | Gate: game rules and flow tests | Sonnet | P2-01…P2-04 | DONE |
| P3-01 | TUI foundation | Sonnet | P1-02 | DONE |
| P3-02 | Board geometry and rendering | Sonnet | P3-01, P2-01 | DONE |
| P3-03 | Board controller | Sonnet | P3-02 | DONE |
| P3-04 | Menus and game selection | Haiku | P3-01 | DONE |
| P3-05 | Settings screen | Haiku | P3-01 | DONE |
| P3-06 | Mouse | Haiku | P3-03 | DONE |
| P3-07 | Overlays: help, pause, win, dialogs | Haiku | P3-03 | DONE |
| P3-08 | Accessibility: announcer and tests | Haiku | P3-03 | DONE |
| P3-09 | Gate: TUI | Sonnet | P3-04…P3-08, P4-03 | TODO |
| P4-01 | SQLite persistence | Sonnet | P3-01 | DONE |
| P4-02 | Logging, panic hook, entry point | Haiku | P3-01, M-18 | DONE |
| P4-03 | Autosave and resume | Sonnet | P3-03, P4-01 | DONE |
| X-01 | Spike: card font in real terminals | Sonnet + User | P0-01 | IN PROGRESS |
| F-02 | Checkpoint: graphics delivery | Opus | X-01 | TODO |
| P5-01 | Font builder and preview sheet | Haiku | F-02 | TODO |
| P5-02 | Style-driven cards and graphics check | Haiku | P5-01, P3-05 | TODO |
| U-02 | Artwork review | User | P5-02, M-19…M-24 | TODO |
| P6-01 | MIDI sequencer and synthesizer | Sonnet | P3-01 | DONE |
| P6-02 | Audio output and settings | Haiku | P6-01, P3-05 | DONE |
| P6-03 | Score-to-MIDI tool | Haiku | P0-01 | DONE |
| P7-01 | E2E harness and core scenarios | Haiku | P3-09, P4-02 | TODO |
| P7-02 | Player site and stable release procedure | Haiku | P0-04, P5-01, M-30…M-33 | TODO |
| P7-03 | Technical documentation and README | Haiku | P3-09 | TODO |
| U-03 | Terminal matrix | User | P7-01 | TODO |
| P8-01 | Fixes and visual polish | Sonnet | U-03 | TODO |
| P8-02 | Final review | Sonnet | P8-01 | TODO |
| F-03 | Checkpoint: release sign-off | Opus | P8-02 | TODO |
| P8-03 | Release 0.1.0 | Haiku + User | F-03, P7-02 | TODO |

Micro tasks M-01…M-35: see `micro.md`.

### Follow-up tasks

Added after the plan was written: owner feedback, review follow-ups and fixes. Their briefs live in
the PR descriptions; cards that exist are in the phase files.

| ID | Task | Depends on | Status |
|---|---|---|---|
| O-01 | Record roles, CI, seed and SQLite decisions (R-14…R-19) | U-01 | DONE |
| O-02 | Phase 2 gate follow-ups | P2-05 | DONE |
| O-03 | CI actions on Node 24 | P0-03 | DONE |
| O-04 | Release builds only by manual dispatch | P0-04 | DONE |
| O-05 | Keep a double-clicked console open after a startup error | P4-02 | DONE |
| O-06 | Unicode glyphs within WGL4 for the Windows console | P3-02 | DONE |
| O-07 | Portable mode: data next to the executable | P4-01 | DONE |
| O-08 | Release targets: Windows x64, Linux x64/arm64 musl, macOS arm64 | O-04 | DONE |
| O-09 | Application icon for the Windows executable | P0-01 | DONE |
| O-10 | Remove unused common locale keys | M-17 | DONE |
| O-11 | `--seed` starts the game directly; typed keys stay in text fields | P3-10 | DONE |
| O-12 | Selectable sound styles and clearer sound status in Settings | P6-02, P3-11 | DONE |
| O-13 | Sync task list statuses with `main` | — | DONE |
| O-14 | Game rules as a tab of the help window | O-13, P3-10 | DONE |
| O-17 | Task cards for the owner's interface feedback of 2026-11-16 | — | DONE |
| O-15 | `--new-window` flag and the command-line options in Help | O-05, O-14 | DONE |
| O-19 | `--new-window`: alacritty, kitty, wezterm and foot on Linux | O-15 | DONE |
| O-20 | Sync follow-up task statuses (#74) | — | DONE |
| O-21 | Task cards for the owner's feedback of 2026-11-19 (Restart, recycle button, footer buttons, moves panel, quick options) | — | DONE |
| O-22 | No-sound CI build, cache tuning, author and third-party licences (#85) | — | DONE |
| O-16 | Full-screen (borderless window) option: setting and `--fullscreen` | U-04, P3-16 | TODO |
| U-04 | Interface review for usability and readability (Fable with Opus) | P3-12, P3-13, O-14 | DONE |
| P3-14 | Readable card faces (replace the block-art rank) (#75) | U-04 | DONE |
| P3-15 | Screen layout: game information apart from controls; framed guide panel (#80) | U-04, P3-14 | DONE |
| P3-16 | Zoom: card size by keyboard, console font on Windows, terminal zoom in Help | U-04, P3-15 | TODO |
| P3-17 | Tab and Shift+Tab move the focus through menus, buttons and dialogs (#76) | U-04 | DONE |
| O-18 | Task cards for the owner's feedback of 2026-11-16 (second round) | — | DONE |
| P3-18 | Guide panel on screens slightly too small (the panel as a window below 130 columns) (#83) | U-04, P3-15 | DONE |
| P3-19 | Auto-finish a won game (option); unfinished start on branch `feat/P3-19-auto-finish-b` (#81) | P2-07, P3-15 | DONE |
| P3-20 | Winnable deals only (option) (#82) | P2-08 | DONE |
| P3-21 | Aura options (focus only) and the stock recycle button | U-04, P3-14 | DONE |
| P3-22 | Localization templates (no pile name after a verb) and localized key names | P3-15 | DONE |
| P3-23 | Colors: 16-color swatches, mark contrast test, High Contrast board edge | U-04 | DONE |
| P3-24 | Menus, settings and statistics polish (aligned values, groups, descriptions) | P3-17 | DONE |
| P3-25 | Footer keys as mouse buttons; `g` in the footer (#86) | P3-15 | DONE |
| P3-26 | Possible moves panel with active and inactive action buttons | P3-15, P3-18 | DONE |
| P3-27 | Quick options during the game and defaults in Settings (hints, auto-finish) | P3-19, P3-21, P3-26 | DONE |
| P4-05 | Restarts counted separately in games without undo | P4-04, P3-12 | DONE |
| O-23 | Modal fixes: scroll bar, Close shortcut, focus between the help selector rows (#91) | P3-18, P3-25 | DONE |
| P3-28 | Docking modals to a side of the screen | O-23 | TODO |
| P3-29 | Settings: Interface section (colors, aura color/style/size, interface size, font) | P3-16, P3-21, P3-24, P3-27 | TODO |
| P3-30 | Themes Monokai and Dracula | P3-23 | DONE |
| P3-31 | Card back selection (Theme, Monokai, Dracula) | P3-30 | TODO |
| P4-06 | Automatic moves in statistics: failed attempts and streaks, "assisted" games | P4-05 | TODO |
| O-25 | UI inventory of colors, sizes and fonts (`docs/tech/ui-inventory.md`) | — | DONE |
| O-26 | Contrast fixes found by the UI inventory (focus frame, card back edge, field edge) (#92) | O-25 | DONE |
| O-27 | Sync the task list with `main`: statuses, cards, PR numbers and deviations | — | DONE |
| O-28 | Focused card drawn with a gap of felt inside its face | — | DONE |
| O-29 | Screen-specific labels for Select and Back (P3-22 part 4) | P3-26 | DONE |
| O-30 | Player site: screenshots, both languages, latest release, build from source | — | DONE |
| O-31 | Widen a console window the game owns to 130 columns | — | IN PROGRESS |
| O-32 | Easy restart from the beginning, confirmed like Quit | P3-27 | IN PROGRESS |
| O-33 | Statistics: scroll bars and a winnable mark per game | O-23, P3-24 | IN PROGRESS |
| P3-32 | Table color and feel | O-26, P3-29 | TODO |
| P3-33 | Card face tint | O-26, P3-29 | TODO |
| P3-34 | Low-vision mode | O-26, P3-16, P3-29 | TODO |
| P3-35 | Color-blind modes (protanopia, deuteranopia, tritanopia, monochrome; four-color deck) | O-26, P3-29 | TODO |
| P3-36 | Themes as data and the interface size scale | P3-29, P3-30 | TODO |
| P3-37 | Language selector with flags in the main menu corner (tooltip, arrows, drop-down list) | P3-24 | TODO |
| P3-38 | Run selection wraps around: Down or `-` on the top card selects the whole run | P3-26 | DONE |
| P2-07 | Forced-win detection and the finishing moves | P2-06 | DONE |
| P2-08 | Solver for winnable deals (#78) | P2-07 | DONE |
| P2-06 | Stock pass and deal counters in the engine | P2-05 | DONE |
| P3-10 | How-to-play overlay and first-steps hint | P3-07 | DONE |
| P3-11 | Board guidance: focus aura, action line and guide panel | P3-06, P3-08, P3-10 | DONE |
| P3-12 | Move counter, no-undo mode, seed copy and stock counters on the board | P3-11, P4-04, P2-06 | DONE |
| P3-13 | Large card faces: big suit and rank on medium and large cards | P3-11 | DONE |
| P4-04 | Statistics, game history and seed copying | P4-03 | DONE |

## Start prompts

Phase task (Sonnet or Haiku):

```text
You work on oXySol. Read AGENTS.md, docs/tech/architecture.md and card <ID> in
docs/task_list/phase-<N>.md. Do only that task on branch <type>/<ID>-<slug>, run
`cargo xtask check --quick`, open a PR to main, and give the final report from AGENTS.md.
```

Micro task (Antigravity):

```text
You work on oXySol. Read AGENTS.md and card <ID> in docs/task_list/micro.md — nothing else.
Do only that card on branch <type>/<ID>-<slug>. Touch only the files it lists.
Run `cargo xtask check --quick`, open a PR to main, and give the final report from AGENTS.md.
```
