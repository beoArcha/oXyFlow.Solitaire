# Audio and Sound Styles

Audio in oXySol is organized into sound styles under `assets/audio/<style>/`. Each style folder contains:
- `style.toml`: style metadata, sound cue mappings, and instrument timbres.
- `scores/*.txt`: text score source files for sound cues.
- `*.mid`: compiled Standard MIDI Files (SMF format 0) generated next to `style.toml`.

External or hand-composed `.mid` files remain valid; the score compiler is an authoring aid.

## Score-to-MIDI Compiler

To compile all scores for a style:

```bash
cargo xtask midi <style>
```

This reads every `assets/audio/<style>/scores/<name>.txt` and writes `assets/audio/<style>/<name>.mid`.

## Score Notation

Each score is a UTF-8 text file consisting of header lines followed by musical tokens.

### Comments and Whitespace

- Lines starting with `#` are comments.
- Empty lines and leading/trailing whitespace are ignored.

### Headers

Every score must begin with two header lines (in either order) before any notes:

- `tempo <bpm>`: playback tempo in beats per minute (positive integer, e.g. `tempo 120`).
- `program <n>`: General MIDI instrument program number from 0 to 127 (e.g. `program 0` for Acoustic Grand Piano).

### Tokens

After the headers, musical events are written as whitespace-separated tokens in the format:

```text
<target>:<length>
```

#### Targets

1. **Single Note**: `<note><octave>` or `<note><accidental><octave>`
   - Note letters: `C`, `D`, `E`, `F`, `G`, `A`, `B`
   - Optional accidental: `#` (sharp) or `b` (flat)
   - Octave: integer from `-1` to `9`
   - Middle C is `C4` (MIDI pitch 60). Pitches must resolve within the standard MIDI range of 0 to 127.
   - Examples: `C4:4`, `F#3:8`, `Bb-1:16`, `D#9:32`

2. **Rest**: `R`
   - Advances time without playing any note.
   - Example: `R:4` (one quarter note rest)

3. **Chord**: notes joined with `+`
   - Plays all notes simultaneously for the specified duration.
   - Example: `C4+E4+G4:2` (C major chord for a half note)

#### Lengths

Lengths represent standard note subdivisions:

- `1`: whole note (1920 ticks)
- `2`: half note (960 ticks)
- `4`: quarter note (480 ticks)
- `8`: eighth note (240 ticks)
- `16`: sixteenth note (120 ticks)
- `32`: thirty-second note (60 ticks)

An optional trailing dot (`.`) indicates a dotted note, increasing duration by 50%:
- `4.`: dotted quarter (720 ticks)
- `8.`: dotted eighth (360 ticks)

### MIDI Output Specification

The compiler outputs Standard MIDI Files with:
- Format: Format 0 (single track containing all events)
- Timing: 480 ticks per quarter note (PPQN)
- Initial events at delta 0: Tempo meta event (`60_000_000 / bpm` microseconds per quarter note) and Program Change event on channel 0
- Note On / Note Off pairs with velocity 96 / 0 on channel 0
- End of Track meta event

### Example Score

```text
# Fanfare cue
tempo 140
program 10

C4:8 E4:8 G4:8 C5:4.
R:8
C4+E4+G4+C5:2
```

## `style.toml` Schema

`style.toml` is parsed by `src/audio/style.rs`. Unknown top-level keys (for example `name`) are ignored;
unknown keys inside `[music]` and `[programs.<n>]` are errors. File names are relative to the style folder.

```toml
name = "Default"

[cues]
card_pick = "card_pick.mid"
win = "win.mid"

[music]
file = "music.mid"

[programs.0]
waveform = "square"
attack = 0.005
decay = 0.05
sustain = 0.7
release = 0.05
gain = 0.5
```

### `[cues]`

Maps a sound cue to a `.mid` file. A cue that is not listed is silent. The keys are:
`card_pick`, `card_place`, `card_flip`, `stock_deal`, `illegal`, `undo`, `hint`, `win`, `menu_move`,
`menu_select`. Any other key is an error.

### `[music]` (optional)

`file`: one `.mid` file that loops while music is enabled. Without the table there is no music.

### `[programs.<n>]`

Defines the timbre of MIDI program `<n>` (0 to 127). A note plays with the timbre of the program in effect on
its channel; a program that is not defined uses the defaults below.

| Key | Meaning | Range | Default |
|---|---|---|---|
| `waveform` | oscillator: `sine`, `square`, `saw` or `triangle` | — | `sine` |
| `attack` | seconds from silence to full level | 0 to 10 | 0.005 |
| `decay` | seconds from full level to `sustain` | 0 to 10 | 0.05 |
| `sustain` | level held while the note is down | 0 to 1 | 0.7 |
| `release` | seconds from note off to silence | 0 to 10 | 0.05 |
| `gain` | volume of the program | 0 to 1 | 0.5 |

## Playback Model

`src/audio/sequence.rs` reads a Standard MIDI File (format 0 or 1, ticks per quarter note timing) into note
events positioned in samples, honoring every tempo change. Format 2 files and SMPTE timecode timing are
rejected. Note On with velocity 0 is a Note Off. All channels are treated alike. `src/audio/synth.rs` plays a
sequence through a pool of voices (16 by default; the oldest voice is stolen when all are busy) and a soft
limiter, so output always stays within -1 and 1. Output is mono; the device layer spreads it over channels.
