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
