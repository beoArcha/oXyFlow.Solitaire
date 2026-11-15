//! The audio contract. The application (`oxysol`) implements it; the TUI only emits cues.

/// A semantic sound event. Cues duplicate visible feedback and never replace it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCue {
    CardPick,
    CardPlace,
    CardFlip,
    StockDeal,
    Illegal,
    Undo,
    Hint,
    Win,
    MenuMove,
    MenuSelect,
}

/// The slice of the settings that audio output depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSettings {
    pub style: String,
    /// Volume in percent, `0..=100`.
    pub volume: u8,
    pub cues: bool,
    pub music: bool,
}

/// Why sound is or is not available. `Ready` is the only status in which sound can be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioStatus {
    Ready,
    /// The session is remote (an SSH session), where no device is opened.
    RemoteSession,
    /// The build has no sound output: the `audio` feature is off, as in the Linux release builds.
    NotBuilt,
    /// No sound output device exists or the device stopped working.
    NoDevice,
    /// The device works, but the selected sound style cannot be loaded.
    StyleFailed,
}

impl AudioStatus {
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// A sound style the settings offer: its folder id and the name its `style.toml` gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleInfo {
    pub id: String,
    pub name: String,
}

/// Sound output. Implementations must never block and never fail the game.
pub trait Audio {
    /// Why sound is or is not available right now.
    fn status(&self) -> AudioStatus;
    /// Whether sound can be produced now: the status is [`AudioStatus::Ready`].
    fn available(&self) -> bool {
        self.status().is_ready()
    }
    /// Applies new settings; called at start-up and after every change.
    fn configure(&mut self, settings: &AudioSettings);
    /// Plays one cue.
    fn play(&mut self, cue: SoundCue);
}

/// Silent audio that reports the reason it is silent. Used when sound is unavailable or disabled at
/// build time; it never reports `Ready`.
#[derive(Debug, Clone, Copy)]
pub struct NullAudio {
    status: AudioStatus,
}

impl NullAudio {
    pub const fn new(status: AudioStatus) -> Self {
        Self { status }
    }
}

impl Audio for NullAudio {
    fn status(&self) -> AudioStatus {
        self.status
    }

    fn configure(&mut self, _settings: &AudioSettings) {}

    fn play(&mut self, _cue: SoundCue) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> AudioSettings {
        AudioSettings {
            style: "default".to_owned(),
            volume: 50,
            cues: true,
            music: true,
        }
    }

    #[test]
    fn null_audio_is_unavailable_with_the_status_it_was_given_and_accepts_everything() {
        let mut audio = NullAudio::new(AudioStatus::NotBuilt);
        audio.configure(&settings());
        audio.play(SoundCue::Win);
        assert_eq!(audio.status(), AudioStatus::NotBuilt);
        assert!(!audio.available());
    }

    #[test]
    fn only_the_ready_status_is_available() {
        assert!(AudioStatus::Ready.is_ready());
        assert!(!AudioStatus::RemoteSession.is_ready());
        assert!(!AudioStatus::NotBuilt.is_ready());
        assert!(!AudioStatus::NoDevice.is_ready());
        assert!(!AudioStatus::StyleFailed.is_ready());
    }
}
