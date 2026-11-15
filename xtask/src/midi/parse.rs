//! Score notation parser.

use std::path::Path;

use super::MidiError;

/// Ticks per quarter note for format-0 MIDI output.
pub const TICKS_PER_QUARTER: u32 = 480;

/// Parsed score file representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    pub tempo: u32,
    pub program: u8,
    pub events: Vec<ScoreEvent>,
}

/// An individual musical event in a score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreEvent {
    Note {
        pitch: u8,
        duration_ticks: u32,
    },
    Chord {
        pitches: Vec<u8>,
        duration_ticks: u32,
    },
    Rest {
        duration_ticks: u32,
    },
}

/// Parses a note token like `C4`, `F#3`, `Bb-1`, `D#9` into a MIDI pitch (0..=127).
pub(crate) fn parse_note(token: &str) -> Result<u8, &'static str> {
    let mut chars = token.chars();
    let letter = chars.next().ok_or("empty note")?;
    let base_semitone = match letter {
        'C' | 'c' => 0,
        'D' | 'd' => 2,
        'E' | 'e' => 4,
        'F' | 'f' => 5,
        'G' | 'g' => 7,
        'A' | 'a' => 9,
        'B' | 'b' => 11,
        _ => return Err("invalid note name (expected C, D, E, F, G, A, B)"),
    };

    let remaining: String = chars.collect();
    let (accidental, octave_str) = if let Some(stripped) = remaining.strip_prefix('#') {
        (1, stripped)
    } else if let Some(stripped) = remaining
        .strip_prefix('b')
        .or_else(|| remaining.strip_prefix('B'))
    {
        (-1, stripped)
    } else {
        (0, remaining.as_str())
    };

    let octave: i32 = octave_str.parse().map_err(|_| "invalid octave in note")?;

    if !(-1..=9).contains(&octave) {
        return Err("octave out of range (-1..9)");
    }

    let pitch = (octave + 1) * 12 + base_semitone + accidental;
    if !(0..=127).contains(&pitch) {
        return Err("pitch outside MIDI range (0..127)");
    }

    Ok(pitch as u8)
}

/// Parses note length (1, 2, 4, 8, 16, 32 with optional trailing `.`) into tick duration.
pub(crate) fn parse_duration_ticks(len_str: &str) -> Result<u32, &'static str> {
    let (is_dotted, base_str) = if let Some(stripped) = len_str.strip_suffix('.') {
        (true, stripped)
    } else {
        (false, len_str)
    };

    let base_ticks = match base_str {
        "1" => TICKS_PER_QUARTER * 4,
        "2" => TICKS_PER_QUARTER * 2,
        "4" => TICKS_PER_QUARTER,
        "8" => TICKS_PER_QUARTER / 2,
        "16" => TICKS_PER_QUARTER / 4,
        "32" => TICKS_PER_QUARTER / 8,
        _ => return Err("invalid note length; expected 1, 2, 4, 8, 16, or 32"),
    };

    if is_dotted {
        Ok(base_ticks + (base_ticks / 2))
    } else {
        Ok(base_ticks)
    }
}

/// Parses a chord string like `C4+E4+G4` into pitch values.
fn parse_chord(token: &str) -> Result<Vec<u8>, &'static str> {
    let parts: Vec<&str> = token.split('+').collect();
    if parts.len() < 2 {
        return Err("chord must contain at least two notes separated by '+'");
    }
    let mut pitches = Vec::with_capacity(parts.len());
    for part in parts {
        if part.is_empty() {
            return Err("empty note in chord");
        }
        pitches.push(parse_note(part)?);
    }
    Ok(pitches)
}

/// Parses a single token into a ScoreEvent.
fn parse_token(token: &str) -> Result<ScoreEvent, &'static str> {
    let (target, len_str) = token.split_once(':').ok_or("missing ':' in score token")?;

    if target.is_empty() || len_str.is_empty() || len_str.contains(':') {
        return Err("malformed token; expected <target>:<length>");
    }

    let duration_ticks = parse_duration_ticks(len_str)?;

    if target.eq_ignore_ascii_case("r") {
        Ok(ScoreEvent::Rest { duration_ticks })
    } else if target.contains('+') {
        let pitches = parse_chord(target)?;
        Ok(ScoreEvent::Chord {
            pitches,
            duration_ticks,
        })
    } else {
        let pitch = parse_note(target)?;
        Ok(ScoreEvent::Note {
            pitch,
            duration_ticks,
        })
    }
}

/// Parses a score file text into a `Score`.
pub fn parse_score(input: &str, file_path: &Path) -> Result<Score, MidiError> {
    let mut tempo: Option<u32> = None;
    let mut program: Option<u8> = None;
    let mut events = Vec::new();

    for (line_idx, raw_line) in input.lines().enumerate() {
        let line = line_idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        match words.as_slice() {
            ["tempo", bpm_str] => {
                if tempo.is_some() {
                    return Err(MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: trimmed.to_owned(),
                        message: "duplicate `tempo` header".to_owned(),
                    });
                }
                let bpm: u32 = bpm_str.parse().map_err(|_| MidiError::Syntax {
                    path: file_path.to_path_buf(),
                    line,
                    token: (*bpm_str).to_owned(),
                    message: "tempo must be a positive integer".to_owned(),
                })?;
                if bpm == 0 {
                    return Err(MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: (*bpm_str).to_owned(),
                        message: "tempo must be greater than zero".to_owned(),
                    });
                }
                tempo = Some(bpm);
            }
            ["tempo", ..] => {
                return Err(MidiError::Syntax {
                    path: file_path.to_path_buf(),
                    line,
                    token: trimmed.to_owned(),
                    message: "invalid `tempo <bpm>` header syntax".to_owned(),
                });
            }
            ["program", prog_str] => {
                if program.is_some() {
                    return Err(MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: trimmed.to_owned(),
                        message: "duplicate `program` header".to_owned(),
                    });
                }
                let prog: u8 = prog_str.parse().map_err(|_| MidiError::Syntax {
                    path: file_path.to_path_buf(),
                    line,
                    token: (*prog_str).to_owned(),
                    message: "program must be an integer between 0 and 127".to_owned(),
                })?;
                if prog > 127 {
                    return Err(MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: (*prog_str).to_owned(),
                        message: "program out of range (0..127)".to_owned(),
                    });
                }
                program = Some(prog);
            }
            ["program", ..] => {
                return Err(MidiError::Syntax {
                    path: file_path.to_path_buf(),
                    line,
                    token: trimmed.to_owned(),
                    message: "invalid `program <n>` header syntax".to_owned(),
                });
            }
            _ => {
                if tempo.is_none() || program.is_none() {
                    let missing = if tempo.is_none() { "tempo" } else { "program" };
                    return Err(MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: words.first().copied().unwrap_or(trimmed).to_owned(),
                        message: format!("header `{missing}` must precede score tokens"),
                    });
                }
                for token in words {
                    let ev = parse_token(token).map_err(|msg| MidiError::Syntax {
                        path: file_path.to_path_buf(),
                        line,
                        token: token.to_owned(),
                        message: msg.to_owned(),
                    })?;
                    events.push(ev);
                }
            }
        }
    }

    let tempo = tempo.ok_or_else(|| MidiError::Syntax {
        path: file_path.to_path_buf(),
        line: 1,
        token: "tempo".to_owned(),
        message: "missing `tempo <bpm>` header".to_owned(),
    })?;

    let program = program.ok_or_else(|| MidiError::Syntax {
        path: file_path.to_path_buf(),
        line: 1,
        token: "program".to_owned(),
        message: "missing `program <n>` header".to_owned(),
    })?;

    Ok(Score {
        tempo,
        program,
        events,
    })
}
