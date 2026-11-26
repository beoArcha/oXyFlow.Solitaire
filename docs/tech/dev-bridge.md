# Driving the Dev Bridge (`--dev-bridge`)

The developer testing bridge (decision T-42) runs the real `App` synchronously over `TestBackend` and `VirtualClock` using one JSON command per line on `stdin` and one JSON response per line on `stdout`.

Default and release builds exclude all bridge and dev-log code (`--dev-bridge` and `--dev-log` exit with code 2 and are omitted from `--help`).

## Building and starting

Build or run with the `dev-bridge` feature enabled:

```sh
cargo run --features dev-bridge -- --dev-bridge
```

Optional flags valid alongside `--dev-bridge`:

- `--seed <u64>`: default starting seed used by `start` when `seed` is omitted in JSON (and initial seed counter for games dealt from the menu).
- `--dev-log <path>`: append-only JSON-lines trace file recording every command, state transition, focus change, move result, and engine/protocol error with `frame_id`. Fails immediately at start-up with a non-zero exit code if `<path>` cannot be opened. Passing `--dev-log` without `--dev-bridge` exits with code 2.

## Protocol commands (`stdin`)

Send one compact JSON object per line. Blank lines are ignored; EOF or `{"cmd":"quit"}` ends the session cleanly.

| `cmd` | Parameters (`defaults`) | Description |
|---|---|---|
| `start` | `cols` (`120`), `rows` (`40`), `game` (`"menu"` or variant slug), `seed`, `ascii` (`false`), `colors` (`"truecolor"`), `lang` (`"en-US"`), `theme` (`"dark"`), `include_text` (`false`), `include_state` (`true`) | Creates or replaces the headless session at `frame_id: 0`. Game slugs include `klondike`, `klondike_three`, `spider`, `spider_two`, `spider_four`, `freecell`, `pyramid`, `pyramid_classic`, `yukon`, `yukon_russian`, `yukon_alaska`, `yukon_moosehide`, `yukon_australian`, `golf`, `tripeaks`. |
| `key` | `key`, `modifiers` (`[]`: `"shift"`, `"ctrl"`, `"alt"`), `include_text` (`false`), `include_state` (`true`) | Sends a single key event and steps one frame. Named keys: `Enter`, `Esc`, `Tab`, `BackTab`, `Space`, `Backspace`, `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`, `Delete`, `Insert`, `F1`–`F12`, or any single character. |
| `mouse` | `action` (`"move"`, `"down"`, `"up"`, `"click"`, `"double_click"`, `"right_click"`), `x`, `y`, `modifiers` (`[]`), `include_text` (`false`), `include_state` (`true`) | Dispatches mouse events in a single command without wall-clock waits. |
| `resize` | `cols`, `rows`, `include_text` (`false`), `include_state` (`true`) | Resizes `TestBackend` and emits a `resized` event. |
| `tick` | `ms` (`0`), `include_text` (`false`), `include_state` (`true`) | Advances `VirtualClock` deterministically and steps one tick. |
| `snapshot` | `include_text` (`true`), `include_state` (`true`) | Returns the current `frame_id`, optional rendered screen text, and `UiSnapshot` without advancing state. |
| `quit` | — | Emits a `quit` event and terminates the bridge loop cleanly. |

## Responses (`stdout`) and `--dev-log`

Every command returns one flushed JSON line:

- `frame_id`: `u64` or `null`
- `ok`: `true` or `false` (`error` explains malformed JSON, unknown commands, or invalid parameters; the loop stays alive)
- `state`: `UiSnapshot` when `include_state` is `true`
- `text_frame`: newline-joined screen buffer when `include_text` is `true`
- `events`: semantic diffs from the step (`screen_changed`, `mode_changed`, `overlay_changed`, `focus_changed`, `hover_changed`, `move_applied`, `move_rejected`, `status_changed`, `game_won`, `resized`, `quit`)

When `--dev-log <path>` is active, each processed step appends JSON lines tagged by `"kind"`:

- `command`: `{"kind":"command","frame_id":...,"cmd":...,"raw":...}`
- `transition`: screen/mode/overlay/hover/status/resize/win/quit transitions (`"event"`, optional `"from"`, `"to"`, `"cols"`, `"rows"`)
- `focus`: keyboard/UI focus changes (`"from"`, `"to"`)
- `move`: accepted or rejected move outcomes (`"accepted"`, optional `"moves"`, optional `"message"`)
- `error`: `"source": "engine"` on move/UI errors and `"source": "protocol"` on invalid commands
