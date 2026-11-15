//! Mixes the cues and the music of a [`Bank`] into one mono signal, through one synthesizer per cue voice
//! and one for the music loop. Pure: no device and no clock. `fill` never allocates.

use oxysol_tui::SoundCue;

use super::bank::Bank;
use super::synth::{Synth, SynthError};

/// Cues that can sound at the same time; a new cue takes over the oldest voice when all are busy.
pub const CUE_VOICES: usize = 8;
/// Frames rendered per step of [`Mixer::fill`]; the scratch buffer holds one step.
pub const BLOCK_FRAMES: usize = 512;
const MUSIC_GAIN: f32 = 0.5;
const PERCENT: f32 = 100.0;

pub struct Mixer {
    bank: Bank,
    cues: Vec<Synth>,
    music: Synth,
    gain: f32,
    cues_on: bool,
    music_on: bool,
    next_steal: usize,
    scratch: Vec<f32>,
}

impl Mixer {
    /// A silent mixer for `bank` at `sample_rate`. Call [`Mixer::configure`] before the first `fill`.
    pub fn new(sample_rate: u32, bank: Bank) -> Self {
        let cues = (0..CUE_VOICES)
            .map(|_| Synth::new(sample_rate, bank.programs().clone()))
            .collect();
        let music = Synth::new(sample_rate, bank.programs().clone());
        Self {
            bank,
            cues,
            music,
            gain: 0.0,
            cues_on: false,
            music_on: false,
            next_steal: 0,
            scratch: vec![0.0; BLOCK_FRAMES],
        }
    }

    /// Applies the volume (percent, clamped to 100) and the two switches. Turning the music on starts
    /// the loop; turning it off stops it; turning the cues off stops the cues that sound.
    pub fn configure(&mut self, volume: u8, cues: bool, music: bool) -> Result<(), SynthError> {
        self.gain = f32::from(volume.min(100)) / PERCENT;
        self.cues_on = cues;
        if !cues {
            self.cues.iter_mut().for_each(Synth::stop);
        }
        if music == self.music_on {
            return Ok(());
        }
        self.music_on = music;
        match (music, self.bank.music()) {
            (true, Some(sequence)) => self.music.start(sequence, true),
            (true, None) => Ok(()),
            (false, _) => {
                self.music.stop();
                Ok(())
            }
        }
    }

    /// Starts `cue` on a free voice, or on the oldest one when all are busy. Does nothing while the
    /// cues are off or the style has no sound for the cue.
    pub fn play(&mut self, cue: SoundCue) -> Result<(), SynthError> {
        if !self.cues_on {
            return Ok(());
        }
        let Some(sequence) = self.bank.cue(cue) else {
            return Ok(());
        };
        let index = self.cue_voice();
        match self.cues.get_mut(index) {
            Some(synth) => synth.start(sequence, false),
            None => Ok(()),
        }
    }

    /// Overwrites `out` with the mixed signal in `[-1, 1]`.
    pub fn fill(&mut self, out: &mut [f32]) {
        for block in out.chunks_mut(BLOCK_FRAMES) {
            self.render(block);
        }
    }

    fn render(&mut self, block: &mut [f32]) {
        block.fill(0.0);
        for synth in &mut self.cues {
            mix_into(synth, &mut self.scratch, block, 1.0);
        }
        mix_into(&mut self.music, &mut self.scratch, block, MUSIC_GAIN);
        let gain = self.gain;
        for sample in block.iter_mut() {
            *sample = (*sample * gain).tanh();
        }
    }

    fn cue_voice(&mut self) -> usize {
        if let Some(index) = self.cues.iter().position(Synth::is_finished) {
            return index;
        }
        let index = self.next_steal;
        self.next_steal = (index + 1) % CUE_VOICES;
        index
    }
}

fn mix_into(synth: &mut Synth, scratch: &mut [f32], block: &mut [f32], gain: f32) {
    if synth.is_finished() {
        return;
    }
    let count = block.len().min(scratch.len());
    let Some(source) = scratch.get_mut(..count) else {
        return;
    };
    synth.fill(source);
    for (out, sample) in block.iter_mut().zip(source.iter()) {
        *out += sample * gain;
    }
}

#[cfg(test)]
mod tests;
