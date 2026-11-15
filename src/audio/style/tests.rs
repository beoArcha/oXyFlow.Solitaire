use std::error::Error;

use super::*;

const FULL: &str = r#"
name = "Default"

[cues]
card_pick = "card_pick.mid"
win = "win.mid"

[music]
file = "music.mid"

[programs.0]
waveform = "square"
attack = 0.01
decay = 0.1
sustain = 0.5
release = 0.2
gain = 0.8

[programs.10]
waveform = "triangle"
"#;

#[test]
fn parses_the_full_schema() {
    let style = Style::parse(FULL).unwrap();
    assert_eq!(
        style.cues.get(&SoundCue::CardPick).map(String::as_str),
        Some("card_pick.mid")
    );
    assert_eq!(
        style.cues.get(&SoundCue::Win).map(String::as_str),
        Some("win.mid")
    );
    assert_eq!(style.cues.len(), 2);
    assert_eq!(style.music.as_deref(), Some("music.mid"));
    let piano = style.programs.get(0);
    assert_eq!(piano.waveform, Waveform::Square);
    assert_eq!(piano.sustain, 0.5);
    assert_eq!(style.programs.get(10).waveform, Waveform::Triangle);
    assert_eq!(style.programs.get(10).attack, Timbre::default().attack);
}

#[test]
fn an_empty_file_is_an_empty_style_with_default_timbres() {
    let style = Style::parse("").unwrap();
    assert!(style.cues.is_empty());
    assert_eq!(style.music, None);
    assert_eq!(style.programs.get(42), Timbre::default());
}

#[test]
fn integers_are_accepted_for_numeric_fields() {
    let style = Style::parse("[programs.1]\nsustain = 1\nrelease = 0\n").unwrap();
    assert_eq!(style.programs.get(1).sustain, 1.0);
}

#[test]
fn every_cue_key_round_trips() {
    for (key, cue) in CUE_KEYS {
        assert_eq!(cue_key(cue), key);
        let text = format!("[cues]\n{key} = \"a.mid\"\n");
        assert_eq!(Style::parse(&text).unwrap().cues.len(), 1);
    }
}

#[test]
fn unknown_cue_is_an_error() {
    let result = Style::parse("[cues]\nexplosion = \"a.mid\"\n");
    assert!(matches!(result, Err(StyleError::UnknownCue(key)) if key == "explosion"));
}

#[test]
fn invalid_program_keys_are_errors() {
    for key in ["128", "-1", "piano"] {
        let result = Style::parse(&format!("[programs.\"{key}\"]\n"));
        assert!(
            matches!(result, Err(StyleError::InvalidProgram(_))),
            "{key}"
        );
    }
}

#[test]
fn out_of_range_timbre_values_are_errors() {
    for line in [
        "sustain = 1.5",
        "gain = -0.1",
        "attack = -1.0",
        "release = 99.0",
    ] {
        let result = Style::parse(&format!("[programs.3]\n{line}\n"));
        assert!(
            matches!(result, Err(StyleError::InvalidTimbre { program: 3, .. })),
            "{line}"
        );
    }
}

#[test]
fn empty_file_names_are_errors() {
    assert!(matches!(
        Style::parse("[cues]\nundo = \"\"\n"),
        Err(StyleError::EmptyFileName { .. })
    ));
    assert!(matches!(
        Style::parse("[music]\nfile = \" \"\n"),
        Err(StyleError::EmptyFileName { .. })
    ));
}

#[test]
fn malformed_toml_keeps_its_source() {
    let error = Style::parse("[cues\n").unwrap_err();
    assert!(matches!(error, StyleError::Parse(_)));
    assert!(error.source().is_some());
}

#[test]
fn unknown_timbre_fields_and_waveforms_are_errors() {
    assert!(Style::parse("[programs.1]\nvolume = 1\n").is_err());
    assert!(Style::parse("[programs.1]\nwaveform = \"noise\"\n").is_err());
}
