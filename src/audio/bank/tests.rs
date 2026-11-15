use std::collections::HashMap;

use oxysol_tui::SoundCue;

use super::*;
use crate::audio::test_support::{Item, RATE, smf_bytes};

const MANIFEST_TEXT: &str = "[cues]\ncard_pick = \"pick.mid\"\n\n[music]\nfile = \"music.mid\"\n\n[programs.0]\nwaveform = \"square\"\n";

fn note_file() -> Vec<u8> {
    smf_bytes(&[
        (0, Item::Program(0)),
        (0, Item::On(60)),
        (480, Item::Off(60)),
        (0, Item::End),
    ])
}

fn complete_style() -> HashMap<&'static str, Vec<u8>> {
    HashMap::from([
        ("style.toml", MANIFEST_TEXT.as_bytes().to_vec()),
        ("pick.mid", note_file()),
        ("music.mid", note_file()),
    ])
}

fn build(files: &HashMap<&'static str, Vec<u8>>) -> Result<Bank, BankError> {
    Bank::build("test", |name| files.get(name).map(Vec::as_slice), RATE)
}

#[test]
fn a_complete_style_builds_cues_and_music_at_the_output_rate() {
    let bank = build(&complete_style()).expect("the style is complete");
    let pick = bank.cue(SoundCue::CardPick).expect("card_pick is mapped");
    assert_eq!(pick.sample_rate(), RATE);
    assert!(bank.cue(SoundCue::Win).is_none());
    assert!(bank.music().is_some_and(|music| music.length() > 0));
    assert!(bank.programs().get(0).waveform != Default::default());
}

#[test]
fn an_unknown_style_is_an_error() {
    let result = Bank::load("no-such-style", RATE);
    assert!(
        matches!(result, Err(BankError::UnknownStyle { ref style }) if style == "no-such-style")
    );
}

#[test]
fn a_style_without_manifest_names_style_toml() {
    let result = Bank::build("test", |_| None, RATE);
    assert!(matches!(result, Err(BankError::MissingFile { ref file, .. }) if file == MANIFEST));
}

#[test]
fn a_mapped_file_that_is_not_in_the_folder_is_named() {
    let mut files = complete_style();
    files.remove("pick.mid");
    let result = build(&files);
    assert!(matches!(result, Err(BankError::MissingFile { ref file, .. }) if file == "pick.mid"));
}

#[test]
fn a_malformed_midi_file_is_an_error_not_a_panic() {
    let mut files = complete_style();
    files.insert("music.mid", b"definitely not midi".to_vec());
    let result = build(&files);
    assert!(matches!(result, Err(BankError::Midi { ref file, .. }) if file == "music.mid"));
}

#[test]
fn an_invalid_manifest_is_an_error() {
    let mut files = complete_style();
    files.insert(
        "style.toml",
        b"[cues]\nunknown_cue = \"pick.mid\"\n".to_vec(),
    );
    let result = build(&files);
    assert!(matches!(result, Err(BankError::Manifest { .. })));
}

#[test]
fn a_manifest_that_is_not_utf8_is_an_error() {
    let mut files = complete_style();
    files.insert("style.toml", vec![0xff, 0xfe, 0x00]);
    let result = build(&files);
    assert!(matches!(result, Err(BankError::NotUtf8 { .. })));
}

#[test]
fn every_embedded_style_loads_at_the_output_rate() {
    for style in EMBEDDED_STYLES {
        let result = Bank::load(style.id, RATE);
        assert!(result.is_ok(), "sound style {} does not load", style.id);
    }
}
