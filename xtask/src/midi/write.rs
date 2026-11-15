//! MIDI SMF file generation using midly.

use std::fs;
use std::path::Path;

use midly::num::{u4, u7, u15, u24, u28};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use super::MidiError;
use super::parse::{Score, ScoreEvent, TICKS_PER_QUARTER};

const NOTE_ON_VELOCITY: u8 = 96;
const NOTE_OFF_VELOCITY: u8 = 0;
const CHANNEL_ZERO: u8 = 0;
const MICROSECONDS_PER_MINUTE: u32 = 60_000_000;

/// Compiles a parsed `Score` into an in-memory format-0 MIDI SMF.
pub fn compile_score_to_smf(score: &Score) -> Smf<'static> {
    let header = Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(u15::from(TICKS_PER_QUARTER as u16)),
    };

    let mpqn = MICROSECONDS_PER_MINUTE / score.tempo;
    let mut track = Vec::new();

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(mpqn))),
    });

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Midi {
            channel: u4::from(CHANNEL_ZERO),
            message: MidiMessage::ProgramChange {
                program: u7::from(score.program),
            },
        },
    });

    let mut pending_delta: u32 = 0;

    for event in &score.events {
        match event {
            ScoreEvent::Rest { duration_ticks } => {
                pending_delta += duration_ticks;
            }
            ScoreEvent::Note {
                pitch,
                duration_ticks,
            } => {
                track.push(TrackEvent {
                    delta: u28::from(pending_delta),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(CHANNEL_ZERO),
                        message: MidiMessage::NoteOn {
                            key: u7::from(*pitch),
                            vel: u7::from(NOTE_ON_VELOCITY),
                        },
                    },
                });
                track.push(TrackEvent {
                    delta: u28::from(*duration_ticks),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(CHANNEL_ZERO),
                        message: MidiMessage::NoteOff {
                            key: u7::from(*pitch),
                            vel: u7::from(NOTE_OFF_VELOCITY),
                        },
                    },
                });
                pending_delta = 0;
            }
            ScoreEvent::Chord {
                pitches,
                duration_ticks,
            } => {
                for (idx, pitch) in pitches.iter().enumerate() {
                    let delta = if idx == 0 { pending_delta } else { 0 };
                    track.push(TrackEvent {
                        delta: u28::from(delta),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(CHANNEL_ZERO),
                            message: MidiMessage::NoteOn {
                                key: u7::from(*pitch),
                                vel: u7::from(NOTE_ON_VELOCITY),
                            },
                        },
                    });
                }
                for (idx, pitch) in pitches.iter().enumerate() {
                    let delta = if idx == 0 { *duration_ticks } else { 0 };
                    track.push(TrackEvent {
                        delta: u28::from(delta),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(CHANNEL_ZERO),
                            message: MidiMessage::NoteOff {
                                key: u7::from(*pitch),
                                vel: u7::from(NOTE_OFF_VELOCITY),
                            },
                        },
                    });
                }
                pending_delta = 0;
            }
        }
    }

    track.push(TrackEvent {
        delta: u28::from(pending_delta),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Smf {
        header,
        tracks: vec![track],
    }
}

/// Compiles a parsed score and saves it to the given output path.
pub fn write_midi(score: &Score, out_path: &Path) -> Result<(), MidiError> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|source| MidiError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let smf = compile_score_to_smf(score);
    smf.save(out_path).map_err(|source| MidiError::Write {
        path: out_path.to_path_buf(),
        source,
    })?;
    Ok(())
}
