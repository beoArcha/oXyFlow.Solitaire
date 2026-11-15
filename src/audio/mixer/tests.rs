use std::collections::HashMap;

use super::*;
use crate::audio::test_support::{Item, RATE, smf_bytes};

const MANIFEST_TEXT: &str = "[cues]\ncard_pick = \"pick.mid\"\n\n[music]\nfile = \"music.mid\"\n";
const QUARTER_SAMPLES: usize = 22_050;
const SECOND: usize = RATE as usize;

fn bank() -> Bank {
    let pick = smf_bytes(&[(0, Item::On(72)), (240, Item::Off(72)), (0, Item::End)]);
    let music = smf_bytes(&[(0, Item::On(48)), (480, Item::Off(48)), (0, Item::End)]);
    let files: HashMap<&str, Vec<u8>> = HashMap::from([
        ("style.toml", MANIFEST_TEXT.as_bytes().to_vec()),
        ("pick.mid", pick),
        ("music.mid", music),
    ]);
    Bank::build("test", |name| files.get(name).map(Vec::as_slice), RATE)
        .expect("the test style is complete")
}

fn mixer(volume: u8, cues: bool, music: bool) -> Mixer {
    let mut mixer = Mixer::new(RATE, bank());
    mixer
        .configure(volume, cues, music)
        .expect("the test style starts");
    mixer
}

fn render(mixer: &mut Mixer, samples: usize) -> Vec<f32> {
    let mut out = vec![0.0; samples];
    mixer.fill(&mut out);
    out
}

fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0, |peak, sample| peak.max(sample.abs()))
}

#[test]
fn volume_zero_is_silence() {
    let mut mixer = mixer(0, true, true);
    mixer.play(SoundCue::CardPick).expect("the cue starts");
    let out = render(&mut mixer, SECOND);
    assert!(out.iter().all(|sample| *sample == 0.0));
}

#[test]
fn a_cue_is_audible_and_stops_after_its_note() {
    let mut mixer = mixer(100, true, false);
    mixer.play(SoundCue::CardPick).expect("the cue starts");
    let sounding = render(&mut mixer, SECOND / 4);
    assert!(peak(&sounding) > 0.05);
    let after = render(&mut mixer, SECOND);
    assert!(peak(&after[SECOND / 2..]) == 0.0);
}

#[test]
fn cues_switched_off_stay_silent() {
    let mut mixer = mixer(100, false, false);
    mixer.play(SoundCue::CardPick).expect("nothing to start");
    let out = render(&mut mixer, SECOND / 2);
    assert!(out.iter().all(|sample| *sample == 0.0));
}

#[test]
fn music_loops() {
    let mut mixer = mixer(100, false, true);
    let out = render(&mut mixer, 2 * QUARTER_SAMPLES + SECOND / 10);
    let first_loop = &out[..QUARTER_SAMPLES];
    let second_loop = &out[QUARTER_SAMPLES + 1_000..QUARTER_SAMPLES + 3_000];
    assert!(peak(first_loop) > 0.01);
    assert!(peak(second_loop) > 0.01);
}

#[test]
fn music_stops_when_switched_off() {
    let mut mixer = mixer(100, false, true);
    render(&mut mixer, SECOND / 10);
    mixer.configure(100, false, false).expect("the music stops");
    let out = render(&mut mixer, 2 * SECOND);
    assert!(out[SECOND..].iter().all(|sample| *sample == 0.0));
}

#[test]
fn cues_and_music_together_stay_in_the_unit_range() {
    let mut mixer = mixer(100, true, true);
    for _ in 0..(CUE_VOICES + 3) {
        mixer.play(SoundCue::CardPick).expect("the cue starts");
    }
    let out = render(&mut mixer, 2 * SECOND);
    assert!(out.iter().all(|sample| sample.is_finite()));
    assert!(out.iter().all(|sample| sample.abs() <= 1.0));
    assert!(peak(&out) > 0.05);
}

#[test]
fn a_style_without_the_cue_is_silent_for_it() {
    let mut mixer = mixer(100, true, false);
    mixer.play(SoundCue::Win).expect("nothing to start");
    let out = render(&mut mixer, SECOND / 4);
    assert!(out.iter().all(|sample| *sample == 0.0));
}

#[test]
fn fill_accepts_any_length() {
    let mut mixer = mixer(100, true, true);
    mixer.play(SoundCue::CardPick).expect("the cue starts");
    let out = render(&mut mixer, BLOCK_FRAMES * 3 + 7);
    assert_eq!(out.len(), BLOCK_FRAMES * 3 + 7);
    assert!(out.iter().all(|sample| sample.is_finite()));
}
