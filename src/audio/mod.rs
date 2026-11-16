//! Sound for the TUI: sequencing and synthesis as pure code (`sequence`, `style`, `synth`), the embedded
//! sound styles (`bank`), the mixer, and the device output that implements the TUI's `Audio` trait.
//! Without the `audio` feature, on a remote session or without a device, [`NullAudio`] keeps the game
//! silent and reports why in its `AudioStatus`.

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
use oxysol_tui::{Audio, AudioStatus, NullAudio};

#[cfg(feature = "audio")]
use bank::Bank;
#[cfg(feature = "audio")]
use mixer::Mixer;
#[cfg(feature = "audio")]
use oxysol_tui::{AudioSettings, SoundCue};

/// The audio for this session. A remote session never opens a device (T-17), and a build without the
/// `audio` feature has no device to open.
pub fn create(caps: &TerminalCaps) -> Box<dyn Audio> {
    if caps.remote {
        return silent(AudioStatus::RemoteSession);
    }
    open_device()
}

fn silent(status: AudioStatus) -> Box<dyn Audio> {
    Box::new(NullAudio::new(status))
}

#[cfg(feature = "audio")]
fn open_device() -> Box<dyn Audio> {
    device_or_silence(output::Output::open())
}

#[cfg(not(feature = "audio"))]
fn open_device() -> Box<dyn Audio> {
    silent(AudioStatus::NotBuilt)
}

/// Device audio, or silence when the device could not be opened. The failure is logged here, once.
#[cfg(feature = "audio")]
pub fn device_or_silence(opened: Result<output::Output, output::OutputError>) -> Box<dyn Audio> {
    match opened {
        Ok(output) => Box::new(DeviceAudio::new(output)),
        Err(error) => {
            tracing::warn!(%error, "no sound output; the game runs silent");
            silent(AudioStatus::NoDevice)
        }
    }
}

/// The status of a device output. A stopped stream means no working device, which takes precedence over
/// a style that failed to load; a working device with a failed style is `StyleFailed`.
#[cfg(feature = "audio")]
const fn device_status(device_running: bool, style_ready: bool) -> AudioStatus {
    match (device_running, style_ready) {
        (false, _) => AudioStatus::NoDevice,
        (true, false) => AudioStatus::StyleFailed,
        (true, true) => AudioStatus::Ready,
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

    /// Loads `style` and swaps it into the output. The new mixer is built before the output's lock is
    /// taken, and the one it replaces is dropped after the lock is released.
    fn select_style(&mut self, style: &str) {
        let sample_rate = self.output.sample_rate();
        let loaded = load_bank(style, sample_rate);
        self.style_ready = loaded.is_some();
        let mixer = Mixer::new(sample_rate, loaded.unwrap_or_default());
        self.output.replace_mixer(mixer);
        self.style = Some(style.to_owned());
    }
}

#[cfg(feature = "audio")]
impl Audio for DeviceAudio {
    fn status(&self) -> AudioStatus {
        device_status(self.output.is_running(), self.style_ready)
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
    fn a_remote_session_is_silent_and_says_so() {
        let mut audio = create(&REMOTE);
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert_eq!(audio.status(), AudioStatus::RemoteSession);
        assert!(!audio.available());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn a_failing_device_is_silent_and_reports_no_device() {
        let mut audio = device_or_silence(Err(output::OutputError::NoDevice));
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert_eq!(audio.status(), AudioStatus::NoDevice);
        assert!(!audio.available());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn the_device_reasons_are_checked_in_order() {
        assert_eq!(device_status(false, false), AudioStatus::NoDevice);
        assert_eq!(device_status(false, true), AudioStatus::NoDevice);
        assert_eq!(device_status(true, false), AudioStatus::StyleFailed);
        assert_eq!(device_status(true, true), AudioStatus::Ready);
    }

    #[cfg(not(feature = "audio"))]
    #[test]
    fn without_the_feature_sound_is_not_built() {
        let local = TerminalCaps {
            remote: false,
            ..REMOTE
        };
        let mut audio = create(&local);
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert_eq!(audio.status(), AudioStatus::NotBuilt);
        assert!(!audio.available());
    }

    #[cfg(not(feature = "audio"))]
    #[test]
    fn remote_is_reported_before_not_built() {
        assert_eq!(create(&REMOTE).status(), AudioStatus::RemoteSession);
    }
}
