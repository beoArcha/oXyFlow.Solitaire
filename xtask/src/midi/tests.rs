//! Tests for score parsing and MIDI file generation.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use midly::num::{u4, u7, u15, u24};
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

use super::parse::{ScoreEvent, parse_duration_ticks, parse_note, parse_score};
use super::write::compile_score_to_smf;
use super::{MidiError, run};

#[test]
fn test_parse_note_pitches() {
    assert_eq!(parse_note("C-1").unwrap(), 0);
    assert_eq!(parse_note("C#-1").unwrap(), 1);
    assert_eq!(parse_note("Db-1").unwrap(), 1);
    assert_eq!(parse_note("C4").unwrap(), 60);
    assert_eq!(parse_note("A4").unwrap(), 69);
    assert_eq!(parse_note("F#3").unwrap(), 54);
    assert_eq!(parse_note("Bb-1").unwrap(), 10);
    assert_eq!(parse_note("G9").unwrap(), 127);

    assert!(parse_note("Cb-1").is_err());
    assert!(parse_note("A9").is_err());
    assert!(parse_note("C10").is_err());
    assert!(parse_note("X4").is_err());
}

#[test]
fn test_parse_duration_ticks() {
    assert_eq!(parse_duration_ticks("1").unwrap(), 1920);
    assert_eq!(parse_duration_ticks("2").unwrap(), 960);
    assert_eq!(parse_duration_ticks("4").unwrap(), 480);
    assert_eq!(parse_duration_ticks("8").unwrap(), 240);
    assert_eq!(parse_duration_ticks("16").unwrap(), 120);
    assert_eq!(parse_duration_ticks("32").unwrap(), 60);

    assert_eq!(parse_duration_ticks("1.").unwrap(), 2880);
    assert_eq!(parse_duration_ticks("2.").unwrap(), 1440);
    assert_eq!(parse_duration_ticks("4.").unwrap(), 720);
    assert_eq!(parse_duration_ticks("8.").unwrap(), 360);
    assert_eq!(parse_duration_ticks("16.").unwrap(), 180);
    assert_eq!(parse_duration_ticks("32.").unwrap(), 90);

    assert!(parse_duration_ticks("3").is_err());
    assert!(parse_duration_ticks("4..").is_err());
}

#[test]
fn test_known_score_compiles_and_roundtrips() {
    let text = "# Test score\ntempo 120\nprogram 0\nC4:4 E4:8 G4:16\n";
    let path = Path::new("test.txt");
    let score = parse_score(text, path).unwrap();

    assert_eq!(score.tempo, 120);
    assert_eq!(score.program, 0);
    assert_eq!(score.events.len(), 3);

    let smf = compile_score_to_smf(&score);
    assert_eq!(smf.header.format, Format::SingleTrack);
    assert_eq!(smf.header.timing, Timing::Metrical(u15::from(480)));

    let mut bytes = Vec::new();
    smf.write(&mut bytes).unwrap();
    let parsed = Smf::parse(&bytes).unwrap();

    assert_eq!(parsed.tracks.len(), 1);
    let events = &parsed.tracks[0];
    assert_eq!(
        events[0].kind,
        TrackEventKind::Meta(MetaMessage::Tempo(u24::from(500_000)))
    );
    assert_eq!(
        events[1].kind,
        TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::ProgramChange {
                program: u7::from(0)
            }
        }
    );
    assert_eq!(events[2].delta.as_int(), 0);
    assert_eq!(
        events[2].kind,
        TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::NoteOn {
                key: u7::from(60),
                vel: u7::from(96)
            }
        }
    );
    assert_eq!(events[3].delta.as_int(), 480);
    assert_eq!(
        events[3].kind,
        TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::NoteOff {
                key: u7::from(60),
                vel: u7::from(0)
            }
        }
    );
}

#[test]
fn test_chords_produce_simultaneous_note_ons() {
    let text = "tempo 100\nprogram 1\nC4+E4+G4:2\n";
    let score = parse_score(text, Path::new("chord.txt")).unwrap();
    let smf = compile_score_to_smf(&score);
    let track = &smf.tracks[0];

    assert_eq!(track[2].delta.as_int(), 0);
    assert_eq!(track[3].delta.as_int(), 0);
    assert_eq!(track[4].delta.as_int(), 0);

    assert_eq!(track[5].delta.as_int(), 960);
    assert_eq!(track[6].delta.as_int(), 0);
    assert_eq!(track[7].delta.as_int(), 0);
}

#[test]
fn test_rests_advance_time() {
    let text = "tempo 120\nprogram 0\nC4:4 R:4 E4:4\n";
    let score = parse_score(text, Path::new("rest.txt")).unwrap();
    let smf = compile_score_to_smf(&score);
    let track = &smf.tracks[0];

    assert_eq!(track[3].delta.as_int(), 480);
    assert_eq!(track[4].delta.as_int(), 480);
    assert_eq!(
        track[4].kind,
        TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::NoteOn {
                key: u7::from(64),
                vel: u7::from(96)
            }
        }
    );
}

#[test]
fn test_dotted_lengths() {
    let text = "tempo 120\nprogram 0\nC4:4. D4:8.\n";
    let score = parse_score(text, Path::new("dotted.txt")).unwrap();
    assert_eq!(
        score.events[0],
        ScoreEvent::Note {
            pitch: 60,
            duration_ticks: 720
        }
    );
    assert_eq!(
        score.events[1],
        ScoreEvent::Note {
            pitch: 62,
            duration_ticks: 360
        }
    );
}

#[test]
fn test_syntax_errors_report_file_line_token() {
    let path = PathBuf::from("scores/bad.txt");
    let text = "tempo 120\nprogram 0\nBADTOKEN:4\n";
    let err = parse_score(text, &path).unwrap_err();

    match err {
        MidiError::Syntax {
            path: err_path,
            line,
            token,
            ..
        } => {
            assert_eq!(err_path, path);
            assert_eq!(line, 3);
            assert_eq!(token, "BADTOKEN:4");
        }
        _ => panic!("expected syntax error"),
    }
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("oxysol_midi_{prefix}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_full_pipeline_in_temp_dir() {
    let root = temp_test_dir("pipeline");
    let style_dir = root.join("assets").join("audio").join("classic");
    let scores_dir = style_dir.join("scores");
    fs::create_dir_all(&scores_dir).unwrap();

    let score_file = scores_dir.join("pick.txt");
    fs::write(&score_file, "tempo 140\nprogram 10\nC4:8 G4:8\n").unwrap();

    run(&root, "classic").unwrap();

    let out_file = style_dir.join("pick.mid");
    assert!(out_file.is_file());
    let bytes = fs::read(&out_file).unwrap();
    let parsed = Smf::parse(&bytes).unwrap();
    assert_eq!(parsed.tracks.len(), 1);

    let err = run(&root, "nonexistent").unwrap_err();
    assert!(matches!(err, MidiError::MissingStyleDir { .. }));

    let empty_style = root.join("assets").join("audio").join("empty_style");
    fs::create_dir_all(empty_style.join("scores")).unwrap();
    let empty_err = run(&root, "empty_style").unwrap_err();
    assert!(matches!(empty_err, MidiError::EmptyScoresDir { .. }));

    fs::remove_dir_all(&root).unwrap();
}
