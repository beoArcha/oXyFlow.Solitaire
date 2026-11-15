//! Sound at the application boundary. A session without sound, or with a device that fails to open, is
//! still a working game; the sound options report themselves as unavailable.

use std::time::Instant;

use oxysol::audio;
#[cfg(feature = "audio")]
#[cfg(feature = "audio")]
use oxysol::audio::output::OutputError;
use oxysol_tui::caps::{ColorDepth, TerminalCaps};
use oxysol_tui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use oxysol_tui::storage::memory::MemoryStorage;
use oxysol_tui::{App, Audio, Deps, Input};
#[cfg(feature = "audio")]
use oxysol_tui::{AudioSettings, SoundCue};

const LOCAL: TerminalCaps = TerminalCaps {
    color: ColorDepth::TrueColor,
    unicode: true,
    remote: false,
};
const REMOTE: TerminalCaps = TerminalCaps {
    remote: true,
    ..LOCAL
};

fn settings() -> AudioSettings {
    AudioSettings {
        style: "default".to_owned(),
        volume: 70,
        cues: true,
        music: false,
    }
}

fn app_with(audio: Box<dyn Audio>) -> App {
    App::new(Deps {
        storage: Box::new(MemoryStorage::new()),
        audio,
        caps: LOCAL,
        seeds: Box::new(|| 7),
        start_seed: None,
    })
}

fn press(app: &mut App, code: KeyCode) {
    app.handle(
        Input::Key(KeyEvent::new(code, KeyModifiers::NONE)),
        Instant::now(),
    );
}

fn play_menu_and_quit(app: &mut App) {
    press(app, KeyCode::Down);
    press(app, KeyCode::Up);
    press(app, KeyCode::Char('q'));
    assert!(app.should_quit());
}

#[test]
fn a_remote_session_runs_without_sound() {
    let audio = audio::create(&REMOTE);
    assert!(!audio.available());
    let mut app = app_with(audio);
    play_menu_and_quit(&mut app);
}

#[cfg(feature = "audio")]
#[test]
fn a_failing_device_runs_without_sound() {
    let audio = audio::device_or_silence(Err(OutputError::NoDevice));
    assert!(!audio.available());
    let mut app = app_with(audio);
    play_menu_and_quit(&mut app);
}

#[cfg(feature = "audio")]
#[test]
fn a_failing_device_accepts_settings_and_cues() {
    let mut audio = audio::device_or_silence(Err(OutputError::NoDevice));
    audio.configure(&settings());
    audio.play(SoundCue::CardPick);
    assert!(!audio.available());
}

#[cfg(feature = "audio")]
#[test]
#[ignore = "plays the default sound style on the default sound device; run by hand with --ignored"]
fn the_default_style_plays_on_the_default_device() {
    let mut audio = audio::create(&LOCAL);
    audio.configure(&settings());
    assert!(audio.available());
    audio.play(SoundCue::CardPick);
    std::thread::sleep(std::time::Duration::from_millis(800));
    audio.play(SoundCue::Win);
    std::thread::sleep(std::time::Duration::from_millis(2_000));
    assert!(audio.available());
}
