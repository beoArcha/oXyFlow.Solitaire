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

/// Sound output. Implementations must never block and never fail the game.
pub trait Audio {
    /// Whether sound can be produced at all (a device exists, the session is local).
    fn available(&self) -> bool;
    /// Applies new settings; called at start-up and after every change.
    fn configure(&mut self, settings: &AudioSettings);
    /// Plays one cue.
    fn play(&mut self, cue: SoundCue);
}

/// Silent audio: used when sound is unavailable or disabled at build time.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullAudio;

impl Audio for NullAudio {
    fn available(&self) -> bool {
        false
    }

    fn configure(&mut self, _settings: &AudioSettings) {}

    fn play(&mut self, _cue: SoundCue) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_audio_is_unavailable_and_accepts_everything() {
        let mut audio = NullAudio;
        audio.configure(&AudioSettings {
            style: "default".to_owned(),
            volume: 50,
            cues: true,
            music: true,
        });
        audio.play(SoundCue::Win);
        assert!(!audio.available());
    }
}
