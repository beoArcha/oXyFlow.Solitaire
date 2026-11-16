//! The sound device: a `cpal` output stream that pulls mono samples from a shared [`Mixer`] and writes
//! them to every channel. The audio callback only tries to lock the mixer and writes silence when the
//! lock is busy, so it never waits and never allocates.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError, TryLockError};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};
use oxysol_tui::SoundCue;
use thiserror::Error;

use super::bank::Bank;
use super::mixer::{BLOCK_FRAMES, Mixer};
use super::synth::SynthError;

/// Failure to open the default sound output.
#[derive(Debug, Error)]
pub enum OutputError {
    #[error("there is no default sound output device")]
    NoDevice,
    #[error("the sound output device has no usable configuration")]
    Config(#[source] cpal::Error),
    #[error("the sound output device uses the unsupported sample format {0:?}")]
    Format(SampleFormat),
    #[error("the sound output stream cannot be built")]
    Build(#[source] cpal::Error),
    #[error("the sound output stream cannot be started")]
    Start(#[source] cpal::Error),
}

/// A running output stream. Dropping it stops the sound.
pub struct Output {
    _stream: cpal::Stream,
    mixer: Arc<Mutex<Mixer>>,
    failed: Arc<AtomicBool>,
    sample_rate: u32,
}

impl Output {
    /// Opens the default output device with its default configuration and starts the stream.
    pub fn open() -> Result<Self, OutputError> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(OutputError::NoDevice)?;
        let supported = device
            .default_output_config()
            .map_err(OutputError::Config)?;
        let format = supported.sample_format();
        let config = supported.config();
        let mixer = Arc::new(Mutex::new(Mixer::new(config.sample_rate, Bank::default())));
        let failed = Arc::new(AtomicBool::new(false));
        let stream = build_stream(&device, config, format, &mixer, &failed)?;
        stream.play().map_err(OutputError::Start)?;
        Ok(Self {
            _stream: stream,
            mixer,
            failed,
            sample_rate: config.sample_rate,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// False once the stream has reported an error; the error is logged once when it happens.
    pub fn is_running(&self) -> bool {
        !self.failed.load(Ordering::Relaxed)
    }

    /// Runs `update` with the mixer. Waits for the audio callback to release the mixer, which takes
    /// the length of one block at most; used for settings, not for every cue.
    pub fn with_mixer<R>(&self, update: impl FnOnce(&mut Mixer) -> R) -> R {
        let mut mixer = self.mixer.lock().unwrap_or_else(PoisonError::into_inner);
        update(&mut mixer)
    }

    /// Puts `mixer` in place of the current one and returns the current one. The lock is held only for
    /// the swap, so the caller drops the returned mixer, and any allocation it implies, after the lock
    /// is released.
    pub fn replace_mixer(&self, mixer: Mixer) -> Mixer {
        let mut current = self.mixer.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::replace(&mut *current, mixer)
    }

    /// Starts a cue without waiting. If the audio callback holds the mixer at that moment the cue is
    /// dropped, which is inaudible in practice.
    pub fn try_play(&self, cue: SoundCue) -> Result<(), SynthError> {
        match self.mixer.try_lock() {
            Ok(mut mixer) => mixer.play(cue),
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner().play(cue),
            Err(TryLockError::WouldBlock) => Ok(()),
        }
    }
}

fn build_stream(
    device: &cpal::Device,
    config: StreamConfig,
    format: SampleFormat,
    mixer: &Arc<Mutex<Mixer>>,
    failed: &Arc<AtomicBool>,
) -> Result<cpal::Stream, OutputError> {
    match format {
        SampleFormat::F32 => build::<f32>(device, config, mixer, failed),
        SampleFormat::I16 => build::<i16>(device, config, mixer, failed),
        SampleFormat::U16 => build::<u16>(device, config, mixer, failed),
        SampleFormat::I32 => build::<i32>(device, config, mixer, failed),
        other => Err(OutputError::Format(other)),
    }
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    mixer: &Arc<Mutex<Mixer>>,
    failed: &Arc<AtomicBool>,
) -> Result<cpal::Stream, OutputError>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels).max(1);
    let shared = Arc::clone(mixer);
    let mut mono = vec![0.0; BLOCK_FRAMES];
    let reported = Arc::clone(failed);
    device
        .build_output_stream::<T, _, _>(
            config,
            move |data: &mut [T], _| write_frames(data, channels, &shared, &mut mono),
            move |error| report_failure(&reported, &error),
            None,
        )
        .map_err(OutputError::Build)
}

fn report_failure(failed: &AtomicBool, error: &cpal::Error) {
    if !failed.swap(true, Ordering::Relaxed) {
        tracing::error!(%error, "the sound output stopped; the game runs silent");
    }
}

fn write_frames<T>(data: &mut [T], channels: usize, shared: &Mutex<Mixer>, mono: &mut [f32])
where
    T: SizedSample + FromSample<f32>,
{
    let Ok(mut mixer) = shared.try_lock() else {
        data.fill(T::EQUILIBRIUM);
        return;
    };
    for frames in data.chunks_mut(channels * mono.len()) {
        let count = frames.len() / channels;
        let Some(samples) = mono.get_mut(..count) else {
            return;
        };
        mixer.fill(samples);
        for (frame, sample) in frames.chunks_mut(channels).zip(samples.iter()) {
            frame.fill(T::from_sample(*sample));
        }
    }
}
