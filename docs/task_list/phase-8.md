# Phase 8 (0.8) — Player documentation, Russian cards and Canfield

Goal: the player documentation (the wiki pages and the player site) and the stable release procedure are written against the features built so far; the Russian card style and the Canfield game are added. Version 0.8.x.

Exit criteria:

- P7-02 and P7-03 are DONE, and the micro cards M-30, M-31, M-32 and M-33 are DONE (M-32 covers the card styles);
- P8-04 and P8-05 are DONE;
- the layout linter and the MCP wrapper P8-06, P8-07 and P8-08 are DONE (decision T-44).

Order of work: the documentation cards first, because they describe what the game does; the micro cards M-30…M-33 (`micro.md`) run alongside them. The card style and the game last.

Micro cards in this phase (micro.md): M-30, M-31 and M-33 (READY), M-32 (DRAFT until P5-02 is merged, which phase 4 does).

### P7-02 — Player site and stable release procedure

- **Owner:** Small
- **Depends on:** P0-04, P5-01, M-30…M-33
- **Files:** `docs/site/**`, `.github/workflows/release.yml`, `docs/tech/workflow.md`
- **Scope:** the build, tag and release pipeline already exists (P0-04). Grow `docs/site/` into the
  player wiki (rules from `docs/wiki/rules/`, installation, controls, decks, accessibility, FAQ),
  add the card fonts to the release assets, and describe the stable release procedure in
  `workflow.md`: version in `metadata.toml` → `main` → promotion → `release.yml` with `stable: true`.
- **Acceptance:** the site builds locally with mdBook; the font files appear in a pre-release made
  from the PR's merge; state plainly what was observed on GitHub.
- **Out of scope:** a browser version of the game.

### P7-03 — Technical documentation and README

- **Owner:** Small
- **Depends on:** P3-09
- **Files:** `docs/tech/**` (except `master_prompt.md` and `spikes/`), `README.md`
- **Scope:** reconcile `architecture.md` with the code (signatures, layout). Add `conventions.md`
  (code rules, error handling, localization, themes, accessibility rules, lint exceptions) and
  `testing.md` (test categories, what runs on a PR and what on `main`, how to run each locally).
  Keep `decisions.md` current. `README`: description, installation, running, key summary, links to
  the wiki and releases.
- **Acceptance:** every signature and path in the documents exists in the code; nothing is stated
  twice across files; `README` promises nothing outside MVP.

### P8-04 — Card style: Russian suits (Unicode)

- **Owner:** Small
- **Depends on:** P7-04
- **Files:** `assets/cards/fronts/russian/style.toml` (new), `locales/*/cards.toml` (the style name key)
- **Shared files:** `locales/en-US/cards.toml`, `locales/pl-PL/cards.toml`
- **Scope:**
  - The fifth new card style: the Russian suit names Крести, Пики, Черви and Бубны, with the court labels Валет, Дама and
    Король. The names and labels are checked against a cited source named in the PR. The suit symbols may be the French
    symbols when the symbols are the same; the style is then distinct by its names and labels, which is the point of it.
  - The same rules as P4-08 (ASCII letters, colour tokens, no font glyphs needed, colour never the only signal).
- **Acceptance:** the same tests as P4-08; the cited source is in the PR description; both locales carry the style name.

### P8-05 — Game: Canfield

- **Owner:** Medium
- **Status:** DRAFT (the rules text is confirmed by Opus before dispatch, from a public-domain rules source the owner names)
- **Depends on:** P7-05
- **Files:** `crates/core/src/games/mod.rs`, `crates/core/src/games/canfield/**`, `crates/core/src/solver/canfield*` (only if
  the solver is in this card), the variant list of `oxysol-core` where the games are matched,
  `crates/tui/src/screens/main_menu/**` (the entry only), `locales/*/rules.toml`, `locales/*/menu.toml`,
  `docs/wiki/rules/canfield.md` (new), `docs/tech/architecture.md` (the games list)
- **Shared files:** `locales/en-US/menu.toml`, `locales/pl-PL/menu.toml`
- **Scope:** Canfield as a module of `oxysol-core` on the game contract, with its reserve pile, its stock and the
  foundation rule as in the confirmed rules text. A solver only if it stays within the budget of T-29; otherwise the
  winnable option stays off for Canfield and the PR says so. Entry, rules tab and rules page as in P4-10.
- **Acceptance:** engine tests for the reserve, the stock and the foundations; the same seed gives the same deal; the
  variant is stored and read back; both locales.

## Developer testing infrastructure: layout linter and a thin MCP wrapper

Owner request 2026-11-21, built on the dev bridge (phase 6, P6-07 to P6-09) and the screenshots (phase 7, P7-06 to
P7-09); decision T-44. Every dev frame is checked by structural lint rules that target the regression classes this
project has actually shipped. O-69 (the window height jumped between tabs), O-70 (a tooltip stayed under a window),
O-73 (two panes both looked focused) and O-84 (footer text ran onto the `Now:` row) each become a rule. A small,
separate MCP server lets AI agents drive play sessions through `--dev-bridge`.

The linter belongs to the dev-gated interface layer, `oxysol-tui` behind the `dev` feature, not to `oxysol-core`.
The engine knows nothing about the screen (AGENTS.md, architecture boundaries). The MCP server is its own package, and
the game binary never depends on it.

### P8-06 — Layout linter: frame trace and the six rules

- **Owner:** Medium
- **Depends on:** P6-09
- **Files:**
  - `crates/tui/src/headless/lint/**` (new, feature `dev`);
  - `crates/tui/src/headless/trace.rs` (new, feature `dev`);
  - the draw calls that record into the trace: footer, status line (`Now:` row), card columns, overlays, docked panes
    and the language tooltip, each with `#[cfg(feature = "dev")]` and nothing else changed;
  - `src/dev_bridge/**` (the `warnings` field);
  - `docs/tech/architecture.md`;
  - tests.
- **Scope:**
  - A dev-only `LayoutTrace` is filled during each draw. Each widget records its assigned `Rect`, the width of the text
    it wanted to draw, whether it shows a primary focus highlight, and its role (footer, status, column, modal,
    tooltip, pane). With the feature off the calls compile to nothing.
  - After each step the linter reads four inputs: the trace, the `UiSnapshot`, the buffer and the previous frame's
    snapshot. From these it returns warnings, each shaped as `{code, severity, message, rect, frame_id}`.
  - The rules:
    - `OVERLAY_TOOLTIP_LEAK`: a tooltip or hover element of the background is visible while an overlay is active.
    - `DUAL_ACTIVE_FOCUS`: two independent panes, such as the main view and docked Help, both draw a primary focus
      highlight.
    - `RECT_CLIP_OR_OVERFLOW`: a widget's text or cells do not fit its assigned `Rect` or the viewport. Typical
      offenders are the footer, the status line, a card column and a modal.
    - `MODAL_GEOMETRY_JUMP`: switching tabs inside the same modal changes its outer `Rect`.
    - `STICKY_HOVER_AFTER_KEY`: a mouse hover highlight is still drawn after a later keyboard navigation step.
    - `STATUS_FOOTER_COLLISION`: status or error text and footer shortcuts overlap, or bleed onto the wrong row.
  - Severity is `error` or `warning`, fixed per rule and documented. Warnings appear in every bridge response, in the
    `--dev-log` and, for batch runs, in `manifest.json`.
- **Acceptance:**
  - For each rule, a test feeds a crafted trace and snapshot and finds exactly that code. Where it is reproducible
    through the bridge, a scenario gives the same result. The pre-fix behaviour of O-69, O-70, O-73 and O-84 serves as
    the reference cases.
  - The baseline scenario pack (P7-09) runs with no `error`-level warnings, or each remaining one is listed in the PR
    as a new visual-fix card.
  - A default build contains no trace code.

### P8-07 — Layout linter: `--dev-strict` gate

- **Owner:** Small
- **Depends on:** P8-06, P7-08
- **Files:**
  - `src/cli.rs`;
  - `src/dev_bridge/**`;
  - `xtask/src/scenarios.rs` (a `--strict` switch that passes the flag through);
  - `docs/tech/workflow.md`;
  - `tests/integration/dev_bridge.rs`;
  - tests.
- **Scope:**
  - `--dev-strict` (dev builds only, with `--dev-script`) makes the run exit with a non-zero code when any `error`- or
    `warning`-level lint issue occurs.
  - The run still writes all frames, the log and the manifest. The manifest lists the issues and the exit reason.
- **Acceptance:**
  - With `--dev-strict`, a scenario that triggers a rule exits non-zero and its manifest names the code.
  - With `--dev-strict`, a clean scenario exits zero.
  - Without the flag, a run exits zero and still reports the warnings.

### P8-08 — Thin MCP wrapper over `--dev-bridge`

- **Owner:** Medium
- **Depends on:** P8-06, P7-07
- **Files:**
  - `dev/mcp-server/**` (new workspace package `oxysol-mcp`, `publish = false`);
  - the root `Cargo.toml` (the workspace member and the new workspace dependencies `rmcp`, `tokio`, `base64`);
  - `deny.toml` (only if a new licence must be allowed; say which);
  - `dev/mcp-server/README.md`;
  - `docs/tech/architecture.md` (package list and dependency direction);
  - `docs/tech/workflow.md`;
  - tests.
- **Scope:**
  - A minimal MCP server over stdio, built on the official Rust MCP SDK (`rmcp`, with `tokio`). It spawns and manages
    `oxysol --dev-bridge` child processes, one per session, built with the dev feature. The game packages never
    depend on this package.
  - Tools:
    - `start_session(cols, rows, game, seed, theme, lang, ascii)` returns a `session_id`, the initial state and the
      text frame.
    - `step(session_id, actions[])` runs one action or a short batch of `key`, `mouse`, `resize` and `tick` actions.
      It returns the latest state, the `warnings` and, optionally, the text frame.
    - `capture_screenshot(session_id, format = "png")` takes a bridge snapshot to a temporary file. It returns the
      path, the image as MCP image content (base64) and the current state with its warnings.
    - `get_logs(session_id, tail = 50, warnings_only = false)` returns recent entries of the session's `--dev-log`.
    - `close_session(session_id)` sends `quit`, waits for the child with a timeout and kills it after the timeout. It
      also removes the session's temporary files.
  - Every child is killed when the server exits, including on end of input and on error, so no `oxysol` process is
    left behind.
  - Dependencies follow the industry standard (owner 2026-11-21): `rmcp` for the protocol, `tokio` for the process
    and stdio handling, `base64` for image content, `serde` and `serde_json` from the workspace. None of them is
    rewritten by hand. They are added to `[workspace.dependencies]` with exact versions and must pass `cargo deny`.
  - The README shows how to register the server in local agent and IDE configurations: VS Code, Antigravity, Claude
    Code and Gemini CLI.
- **Acceptance:**
  - An integration test starts the server and runs a multi-step session through it: start, keys, a mouse click, a
    resize and a tick. It then captures a PNG and checks the image content's header. It reads the warnings and the
    logs, closes the session, and finds no `oxysol` child still running.
  - `cargo build -p oxysol` and the release build do not build `oxysol-mcp` or pull in its dependencies (shown in the
    PR with `cargo tree -p oxysol`).

