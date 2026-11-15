//! A small polyphonic synthesizer that plays a [`Sequence`]; mono, `f32` samples in `[-1, 1]`.

mod voice;

use std::sync::Arc;

use thiserror::Error;

use super::sequence::{Event, EventKind, Sequence};
use super::style::Programs;
use voice::{NoteStart, Voice};

/// The number of simultaneous voices a synthesizer has unless told otherwise.
pub const DEFAULT_VOICE_LIMIT: usize = 16;
const MASTER_GAIN: f32 = 0.5;

/// Failure to start a sequence.
#[derive(Debug, Error)]
pub enum SynthError {
    #[error("the sequence was loaded for {sequence} Hz but the synthesizer runs at {synth} Hz")]
    SampleRateMismatch { synth: u32, sequence: u32 },
}

struct Playback {
    sequence: Arc<Sequence>,
    cursor: usize,
    position: u64,
    looping: bool,
}

/// Plays one sequence at a time through a fixed pool of voices. `fill` never allocates.
pub struct Synth {
    sample_rate: u32,
    programs: Programs,
    voices: Vec<Voice>,
    playback: Option<Playback>,
    started_notes: u64,
}

impl Synth {
    pub fn new(sample_rate: u32, programs: Programs) -> Self {
        Self::with_voice_limit(sample_rate, DEFAULT_VOICE_LIMIT, programs)
    }

    /// A synthesizer with `voice_limit` voices (at least one); the oldest voice is stolen when
    /// all are busy.
    pub fn with_voice_limit(sample_rate: u32, voice_limit: usize, programs: Programs) -> Self {
        Self {
            sample_rate,
            programs,
            voices: vec![Voice::idle(); voice_limit.max(1)],
            playback: None,
            started_notes: 0,
        }
    }

    /// Replaces whatever is playing. Voices of the previous sequence are cut off.
    pub fn start(&mut self, sequence: Arc<Sequence>, looping: bool) -> Result<(), SynthError> {
        if sequence.sample_rate() != self.sample_rate {
            return Err(SynthError::SampleRateMismatch {
                synth: self.sample_rate,
                sequence: sequence.sample_rate(),
            });
        }
        self.voices.iter_mut().for_each(Voice::silence);
        self.playback = (sequence.length() > 0).then_some(Playback {
            sequence,
            cursor: 0,
            position: 0,
            looping,
        });
        Ok(())
    }

    /// Stops the sequence; sounding notes ring out through their release.
    pub fn stop(&mut self) {
        self.playback = None;
        self.release_all();
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_active()).count()
    }

    /// True when no sequence is playing and every voice has gone quiet.
    pub fn is_finished(&self) -> bool {
        self.playback.is_none() && self.active_voices() == 0
    }

    /// Overwrites `out` with the next samples.
    pub fn fill(&mut self, out: &mut [f32]) {
        for slot in out.iter_mut() {
            *slot = self.next_sample();
        }
    }

    fn next_sample(&mut self) -> f32 {
        while let Some(event) = self.take_due_event() {
            self.apply(event);
        }
        let mixed: f32 = self.voices.iter_mut().map(Voice::next_sample).sum();
        self.advance_position();
        (mixed * MASTER_GAIN).tanh()
    }

    fn take_due_event(&mut self) -> Option<Event> {
        let playback = self.playback.as_mut()?;
        let event = *playback.sequence.events().get(playback.cursor)?;
        if event.sample > playback.position {
            return None;
        }
        playback.cursor += 1;
        Some(event)
    }

    fn advance_position(&mut self) {
        let Some(playback) = self.playback.as_mut() else {
            return;
        };
        playback.position += 1;
        if playback.position < playback.sequence.length() {
            return;
        }
        if playback.looping {
            playback.position = 0;
            playback.cursor = 0;
        } else {
            self.playback = None;
        }
        self.release_all();
    }

    fn apply(&mut self, event: Event) {
        match event.kind {
            EventKind::NoteOn {
                channel,
                key,
                velocity,
                program,
            } => self.note_on(
                NoteStart {
                    channel,
                    key,
                    velocity,
                },
                program,
            ),
            EventKind::NoteOff { channel, key } => self.note_off(channel, key),
        }
    }

    fn note_on(&mut self, note: NoteStart, program: u8) {
        let timbre = self.programs.get(program);
        self.started_notes += 1;
        let order = self.started_notes;
        let sample_rate = self.sample_rate as f32;
        if let Some(voice) = self.free_or_oldest_voice() {
            voice.start(note, &timbre, sample_rate, order);
        }
    }

    fn free_or_oldest_voice(&mut self) -> Option<&mut Voice> {
        let index = self
            .voices
            .iter()
            .position(|voice| !voice.is_active())
            .or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, voice)| voice.order())
                    .map(|(index, _)| index)
            })?;
        self.voices.get_mut(index)
    }

    fn note_off(&mut self, channel: u8, key: u8) {
        let held = self
            .voices
            .iter_mut()
            .filter(|voice| voice.is_held(channel, key))
            .min_by_key(|voice| voice.order());
        if let Some(voice) = held {
            voice.release();
        }
    }

    fn release_all(&mut self) {
        self.voices.iter_mut().for_each(Voice::release);
    }
}

/// Renders a whole sequence, including the release tails, into a buffer. For tests and tools.
pub fn render(sequence: Arc<Sequence>, programs: Programs) -> Vec<f32> {
    let mut synth = Synth::new(sequence.sample_rate(), programs);
    let mut out = Vec::new();
    if synth.start(sequence, false).is_err() {
        return out;
    }
    while !synth.is_finished() {
        out.push(synth.next_sample());
    }
    out
}

#[cfg(test)]
mod tests;
