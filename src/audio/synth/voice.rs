//! One oscillator with an ADSR envelope.

use std::f32::consts::TAU;

use crate::audio::style::{Timbre, Waveform};

const A4_KEY: f32 = 69.0;
const A4_HZ: f32 = 440.0;
const MAX_VELOCITY: f32 = 127.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NoteStart {
    pub channel: u8,
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Voice {
    stage: Stage,
    channel: u8,
    key: u8,
    order: u64,
    waveform: Waveform,
    phase: f32,
    increment: f32,
    level: f32,
    gain: f32,
    sustain: f32,
    attack_step: f32,
    decay_step: f32,
    release_samples: f32,
    release_step: f32,
}

impl Voice {
    pub const fn idle() -> Self {
        Self {
            stage: Stage::Idle,
            channel: 0,
            key: 0,
            order: 0,
            waveform: Waveform::Sine,
            phase: 0.0,
            increment: 0.0,
            level: 0.0,
            gain: 0.0,
            sustain: 0.0,
            attack_step: 0.0,
            decay_step: 0.0,
            release_samples: 1.0,
            release_step: 0.0,
        }
    }

    pub fn start(&mut self, note: NoteStart, timbre: &Timbre, sample_rate: f32, order: u64) {
        let attack = steps(timbre.attack, sample_rate);
        let decay = steps(timbre.decay, sample_rate);
        *self = Self {
            stage: Stage::Attack,
            channel: note.channel,
            key: note.key,
            order,
            waveform: timbre.waveform,
            phase: 0.0,
            increment: frequency(note.key) / sample_rate,
            level: 0.0,
            gain: timbre.gain * f32::from(note.velocity) / MAX_VELOCITY,
            sustain: timbre.sustain,
            attack_step: 1.0 / attack,
            decay_step: (1.0 - timbre.sustain) / decay,
            release_samples: steps(timbre.release, sample_rate),
            release_step: 0.0,
        };
    }

    pub fn release(&mut self) {
        if self.is_active() && self.stage != Stage::Release {
            self.release_step = self.level / self.release_samples;
            self.stage = Stage::Release;
        }
    }

    pub fn silence(&mut self) {
        *self = Self::idle();
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }

    pub fn is_held(&self, channel: u8, key: u8) -> bool {
        self.is_active()
            && self.stage != Stage::Release
            && self.channel == channel
            && self.key == key
    }

    pub fn order(&self) -> u64 {
        self.order
    }

    pub fn next_sample(&mut self) -> f32 {
        if !self.advance_envelope() {
            return 0.0;
        }
        let value = oscillator(self.waveform, self.phase) * self.level * self.gain;
        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        value
    }

    fn advance_envelope(&mut self) -> bool {
        match self.stage {
            Stage::Idle => return false,
            Stage::Attack => {
                self.level += self.attack_step;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.level -= self.decay_step;
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {}
            Stage::Release => {
                self.level -= self.release_step;
                if self.level <= 0.0 {
                    self.silence();
                    return false;
                }
            }
        }
        true
    }
}

fn steps(seconds: f32, sample_rate: f32) -> f32 {
    (seconds * sample_rate).max(1.0)
}

fn frequency(key: u8) -> f32 {
    A4_HZ * ((f32::from(key) - A4_KEY) / 12.0).exp2()
}

fn oscillator(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * TAU).sin(),
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Saw => 2.0 * phase - 1.0,
        Waveform::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
    }
}
