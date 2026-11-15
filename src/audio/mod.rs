//! Sound for the TUI: sequencing and synthesis as pure code (`sequence`, `style`, `synth`), the embedded
//! sound styles (`bank`), the mixer, and the device output that implements the TUI's `Audio` trait.
//! Without the `audio` feature, on a remote session or without a device, [`NullAudio`] keeps the game
//! silent and the sound options unavailable.

pub mod bank;
pub mod mixer;
pub mod sequence;
pub mod style;
pub mod synth;

#[cfg(feature = "audio")]
pub mod output;

#[cfg(test)]
mod test_support;

use oxysol_tui::caps::TerminalCaps;
use oxysol_tui::{Audio, NullAudio};

#[cfg(feature = "audio")]
use bank::Bank;
#[cfg(feature = "audio")]
use mixer::Mixer;
#[cfg(feature = "audio")]
use oxysol_tui::{AudioSettings, SoundCue};

/// The audio for this session. A remote session never opens a device (T-17).
pub fn create(caps: &TerminalCaps) -> Box<dyn Audio> {
    if caps.remote {
        return Box::new(NullAudio);
    }
    open_device()
}

#[cfg(feature = "audio")]
fn open_device() -> Box<dyn Audio> {
    device_or_silence(output::Output::open())
}

#[cfg(not(feature = "audio"))]
fn open_device() -> Box<dyn Audio> {
    Box::new(NullAudio)
}

/// Device audio, or silence when the device could not be opened. The failure is logged here, once.
#[cfg(feature = "audio")]
pub fn device_or_silence(opened: Result<output::Output, output::OutputError>) -> Box<dyn Audio> {
    match opened {
        Ok(output) => Box::new(DeviceAudio::new(output)),
        Err(error) => {
            tracing::warn!(%error, "no sound output; the game runs silent");
            Box::new(NullAudio)
        }
    }
}

#[cfg(feature = "audio")]
struct DeviceAudio {
    output: output::Output,
    style: Option<String>,
    style_ready: bool,
}

#[cfg(feature = "audio")]
impl DeviceAudio {
    fn new(output: output::Output) -> Self {
        Self {
            output,
            style: None,
            style_ready: false,
        }
    }

    fn select_style(&mut self, style: &str) {
        let sample_rate = self.output.sample_rate();
        let loaded = load_bank(style, sample_rate);
        self.style_ready = loaded.is_some();
        let bank = loaded.unwrap_or_default();
        self.output
            .with_mixer(|mixer| *mixer = Mixer::new(sample_rate, bank));
        self.style = Some(style.to_owned());
    }
}

#[cfg(feature = "audio")]
impl Audio for DeviceAudio {
    fn available(&self) -> bool {
        self.output.is_running() && self.style_ready
    }

    fn configure(&mut self, settings: &AudioSettings) {
        if self.style.as_deref() != Some(settings.style.as_str()) {
            self.select_style(&settings.style);
        }
        let result = self
            .output
            .with_mixer(|mixer| mixer.configure(settings.volume, settings.cues, settings.music));
        if let Err(error) = result {
            tracing::error!(%error, "the music cannot start");
        }
    }

    fn play(&mut self, cue: SoundCue) {
        if let Err(error) = self.output.try_play(cue) {
            tracing::error!(%error, "a sound cue cannot start");
        }
    }
}

#[cfg(feature = "audio")]
fn load_bank(style: &str, sample_rate: u32) -> Option<Bank> {
    match Bank::load(style, sample_rate) {
        Ok(bank) => Some(bank),
        Err(error) => {
            tracing::warn!(%error, "the sound style cannot be used; the game runs silent");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use oxysol_tui::caps::{ColorDepth, TerminalCaps};
    use oxysol_tui::{AudioSettings, SoundCue};

    use super::*;

    const REMOTE: TerminalCaps = TerminalCaps {
        color: ColorDepth::Ansi16,
        unicode: false,
        remote: true,
    };

    fn settings() -> AudioSettings {
        AudioSettings {
            style: "default".to_owned(),
            volume: 70,
            cues: true,
            music: true,
        }
    }

    #[test]
    fn a_remote_session_is_silent_and_unavailable() {
        let mut audio = create(&REMOTE);
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert!(!audio.available());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn a_failing_device_is_silent_and_unavailable() {
        let mut audio = device_or_silence(Err(output::OutputError::NoDevice));
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert!(!audio.available());
    }

    #[cfg(not(feature = "audio"))]
    #[test]
    fn without_the_feature_sound_is_unavailable() {
        let mut audio = create(&TerminalCaps {
            remote: false,
            ..REMOTE
        });
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert!(!audio.available());
    }
}
