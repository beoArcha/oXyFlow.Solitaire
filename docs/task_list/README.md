# oXySol task list (MVP)

Started 2026-11-10. Revised 2026-11-15 for R-14…R-19 (roles, CI, seeds, SQLite).
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
| P3-01 | TUI foundation | Sonnet | P1-02 | TODO |
| P3-02 | Board geometry and rendering | Sonnet | P3-01, P2-01 | TODO |
| P3-03 | Board controller | Sonnet | P3-02 | TODO |
| P3-04 | Menus and game selection | Haiku | P3-01 | TODO |
| P3-05 | Settings screen | Haiku | P3-01 | TODO |
| P3-06 | Mouse | Haiku | P3-03 | TODO |
| P3-07 | Overlays: help, pause, win, dialogs | Haiku | P3-03 | TODO |
| P3-08 | Accessibility: announcer and tests | Haiku | P3-03 | TODO |
| P3-09 | Gate: TUI | Sonnet | P3-04…P3-08, P4-03 | TODO |
| P4-01 | SQLite persistence | Sonnet | P3-01 | TODO |
| P4-02 | Logging, panic hook, entry point | Haiku | P3-01, M-18 | TODO |
| P4-03 | Autosave and resume | Sonnet | P3-03, P4-01 | TODO |
| X-01 | Spike: card font in real terminals | Sonnet + User | P0-01 | TODO |
| F-02 | Checkpoint: graphics delivery | Opus | X-01 | TODO |
| P5-01 | Font builder and preview sheet | Haiku | F-02 | TODO |
| P5-02 | Style-driven cards and graphics check | Haiku | P5-01, P3-05 | TODO |
| U-02 | Artwork review | User | P5-02, M-19…M-24 | TODO |
| P6-01 | MIDI sequencer and synthesizer | Sonnet | P3-01 | TODO |
| P6-02 | Audio output and settings | Haiku | P6-01, P3-05 | TODO |
| P6-03 | Score-to-MIDI tool | Haiku | P0-01 | TODO |
| P7-01 | E2E harness and core scenarios | Haiku | P3-09, P4-02 | TODO |
| P7-02 | Player site and stable release procedure | Haiku | P0-04, P5-01, M-30…M-33 | TODO |
| P7-03 | Technical documentation and README | Haiku | P3-09 | TODO |
| U-03 | Terminal matrix | User | P7-01 | TODO |
| P8-01 | Fixes and visual polish | Sonnet | U-03 | TODO |
| P8-02 | Final review | Sonnet | P8-01 | TODO |
| F-03 | Checkpoint: release sign-off | Opus | P8-02 | TODO |
| P8-03 | Release 0.1.0 | Haiku + User | F-03, P7-02 | TODO |

Micro tasks M-01…M-34: see `micro.md`.

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
