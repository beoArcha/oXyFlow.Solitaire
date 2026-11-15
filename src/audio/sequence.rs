//! Loads a Standard MIDI File into note events with sample timestamps.

use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use thiserror::Error;

const DEFAULT_TEMPO_MICROS: u64 = 500_000;
const MICROS_PER_SECOND: u128 = 1_000_000;
const CHANNELS: usize = 16;

/// Failure to turn bytes into a [`Sequence`].
#[derive(Debug, Error)]
pub enum SequenceError {
    #[error("not a valid Standard MIDI File")]
    Parse(#[from] midly::Error),
    #[error("sequential-track (format 2) MIDI files are not supported")]
    UnsupportedFormat,
    #[error("SMPTE timecode timing is not supported; use ticks per quarter note")]
    UnsupportedTiming,
    #[error("the file declares zero ticks per quarter note")]
    ZeroTicksPerBeat,
    #[error("the sample rate must be positive")]
    ZeroSampleRate,
}

/// What happens to a voice. The program is the one in effect on the channel at note start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    NoteOn {
        channel: u8,
        key: u8,
        velocity: u8,
        program: u8,
    },
    NoteOff {
        channel: u8,
        key: u8,
    },
}

/// A note event at a position in samples from the start of the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub sample: u64,
    pub kind: EventKind,
}

/// Note events ordered by time, plus the total length (up to the end-of-track marker).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    sample_rate: u32,
    events: Vec<Event>,
    length: u64,
}

impl Sequence {
    /// A sequence without events and with zero length.
    pub fn empty(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            events: Vec::new(),
            length: 0,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Length in samples, including trailing rests up to the end-of-track marker.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Parses `bytes` as a Standard MIDI File, converting ticks to samples at `sample_rate`
    /// with every tempo change honored. Tracks of a format 1 file are merged.
    pub fn from_midi(bytes: &[u8], sample_rate: u32) -> Result<Self, SequenceError> {
        if sample_rate == 0 {
            return Err(SequenceError::ZeroSampleRate);
        }
        let smf = Smf::parse(bytes)?;
        let ticks_per_beat = ticks_per_beat(&smf.header)?;
        let mut clock = Clock::new(ticks_per_beat, sample_rate);
        let mut programs = [0u8; CHANNELS];
        let mut events = Vec::new();
        let mut length = 0;
        for item in merged_timeline(&smf) {
            let sample = clock.sample_at(item.tick);
            length = length.max(sample);
            match item.kind {
                TimelineKind::Tempo(micros) => clock.set_tempo(micros),
                TimelineKind::Program { channel, program } => {
                    if let Some(slot) = programs.get_mut(usize::from(channel)) {
                        *slot = program;
                    }
                }
                TimelineKind::Note {
                    channel,
                    key,
                    velocity,
                } => {
                    let program = programs.get(usize::from(channel)).copied().unwrap_or(0);
                    events.push(Event {
                        sample,
                        kind: note_kind(channel, key, velocity, program),
                    });
                }
                TimelineKind::End => {}
            }
        }
        Ok(Self {
            sample_rate,
            events,
            length,
        })
    }
}

fn note_kind(channel: u8, key: u8, velocity: u8, program: u8) -> EventKind {
    if velocity == 0 {
        EventKind::NoteOff { channel, key }
    } else {
        EventKind::NoteOn {
            channel,
            key,
            velocity,
            program,
        }
    }
}

fn ticks_per_beat(header: &Header) -> Result<u64, SequenceError> {
    if header.format == Format::Sequential {
        return Err(SequenceError::UnsupportedFormat);
    }
    match header.timing {
        Timing::Metrical(ticks) if ticks.as_int() == 0 => Err(SequenceError::ZeroTicksPerBeat),
        Timing::Metrical(ticks) => Ok(u64::from(ticks.as_int())),
        Timing::Timecode(..) => Err(SequenceError::UnsupportedTiming),
    }
}

#[derive(Debug, Clone, Copy)]
enum TimelineKind {
    Tempo(u64),
    Program { channel: u8, program: u8 },
    Note { channel: u8, key: u8, velocity: u8 },
    End,
}

#[derive(Debug, Clone, Copy)]
struct TimelineItem {
    tick: u64,
    kind: TimelineKind,
}

fn merged_timeline(smf: &Smf<'_>) -> Vec<TimelineItem> {
    let mut items = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += u64::from(event.delta.as_int());
            if let Some(kind) = timeline_kind(&event.kind) {
                items.push(TimelineItem { tick, kind });
            }
        }
    }
    items.sort_by_key(|item| item.tick);
    items
}

fn timeline_kind(kind: &TrackEventKind<'_>) -> Option<TimelineKind> {
    match kind {
        TrackEventKind::Meta(MetaMessage::Tempo(micros)) => {
            Some(TimelineKind::Tempo(u64::from(micros.as_int())))
        }
        TrackEventKind::Meta(MetaMessage::EndOfTrack) => Some(TimelineKind::End),
        TrackEventKind::Midi { channel, message } => midi_kind(channel.as_int(), message),
        _ => None,
    }
}

fn midi_kind(channel: u8, message: &MidiMessage) -> Option<TimelineKind> {
    match message {
        MidiMessage::NoteOn { key, vel } => Some(TimelineKind::Note {
            channel,
            key: key.as_int(),
            velocity: vel.as_int(),
        }),
        MidiMessage::NoteOff { key, .. } => Some(TimelineKind::Note {
            channel,
            key: key.as_int(),
            velocity: 0,
        }),
        MidiMessage::ProgramChange { program } => Some(TimelineKind::Program {
            channel,
            program: program.as_int(),
        }),
        _ => None,
    }
}

struct Clock {
    ticks_per_beat: u128,
    sample_rate: u128,
    tempo_micros: u64,
    last_tick: u64,
    elapsed_micro_beats: u128,
}

impl Clock {
    fn new(ticks_per_beat: u64, sample_rate: u32) -> Self {
        Self {
            ticks_per_beat: u128::from(ticks_per_beat),
            sample_rate: u128::from(sample_rate),
            tempo_micros: DEFAULT_TEMPO_MICROS,
            last_tick: 0,
            elapsed_micro_beats: 0,
        }
    }

    fn sample_at(&mut self, tick: u64) -> u64 {
        let delta = tick.saturating_sub(self.last_tick);
        self.last_tick = self.last_tick.max(tick);
        self.elapsed_micro_beats += u128::from(delta) * u128::from(self.tempo_micros);
        let divisor = self.ticks_per_beat * MICROS_PER_SECOND;
        let samples = (self.elapsed_micro_beats * self.sample_rate + divisor / 2) / divisor;
        u64::try_from(samples).unwrap_or(u64::MAX)
    }

    fn set_tempo(&mut self, micros: u64) {
        self.tempo_micros = micros;
    }
}

#[cfg(test)]
mod tests;
