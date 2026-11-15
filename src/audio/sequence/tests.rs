use std::error::Error;

use midly::num::{u4, u7, u15, u28};
use midly::{Format, MidiMessage, Timing, TrackEvent, TrackEventKind};

use super::*;
use crate::audio::test_support::{Item, RATE, TICKS, smf_bytes, track, write};

fn on(sample: u64, key: u8, program: u8) -> Event {
    Event {
        sample,
        kind: EventKind::NoteOn {
            channel: 0,
            key,
            velocity: 100,
            program,
        },
    }
}

fn off(sample: u64, key: u8) -> Event {
    Event {
        sample,
        kind: EventKind::NoteOff { channel: 0, key },
    }
}

#[test]
fn default_tempo_is_120_bpm() {
    let bytes = smf_bytes(&[(0, Item::On(60)), (480, Item::Off(60)), (0, Item::End)]);
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.events(), &[on(0, 60, 0), off(22_050, 60)]);
    assert_eq!(sequence.length(), 22_050);
}

#[test]
fn tempo_changes_move_later_events() {
    let bytes = smf_bytes(&[
        (0, Item::Tempo(500_000)),
        (480, Item::On(60)),
        (480, Item::Tempo(1_000_000)),
        (480, Item::On(62)),
        (0, Item::End),
    ]);
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.events(), &[on(22_050, 60, 0), on(88_200, 62, 0)]);
}

#[test]
fn trailing_rest_counts_towards_the_length() {
    let bytes = smf_bytes(&[(0, Item::On(60)), (240, Item::Off(60)), (240, Item::End)]);
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.length(), 22_050);
}

#[test]
fn program_changes_apply_to_following_notes() {
    let bytes = smf_bytes(&[
        (0, Item::On(60)),
        (0, Item::Program(10)),
        (0, Item::On(62)),
        (0, Item::End),
    ]);
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.events(), &[on(0, 60, 0), on(0, 62, 10)]);
}

#[test]
fn note_on_with_zero_velocity_is_a_note_off() {
    let silent_on = TrackEvent {
        delta: u28::from(480),
        kind: TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::NoteOn {
                key: u7::from(60),
                vel: u7::from(0),
            },
        },
    };
    let mut events = track(&[(0, Item::On(60))]);
    events.push(silent_on);
    let bytes = write(
        Format::SingleTrack,
        Timing::Metrical(u15::from(TICKS)),
        vec![events],
    );
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.events(), &[on(0, 60, 0), off(22_050, 60)]);
}

#[test]
fn parallel_tracks_are_merged_on_one_timeline() {
    let tempo_track = track(&[(0, Item::Tempo(1_000_000)), (0, Item::End)]);
    let notes = track(&[(480, Item::On(60)), (0, Item::End)]);
    let bytes = write(
        Format::Parallel,
        Timing::Metrical(u15::from(TICKS)),
        vec![tempo_track, notes],
    );
    let sequence = Sequence::from_midi(&bytes, RATE).unwrap();
    assert_eq!(sequence.events(), &[on(44_100, 60, 0)]);
}

#[test]
fn a_file_without_notes_is_empty() {
    let sequence = Sequence::from_midi(&smf_bytes(&[(0, Item::End)]), RATE).unwrap();
    assert!(sequence.events().is_empty());
    assert_eq!(sequence.length(), 0);
    assert_eq!(sequence, Sequence::empty(RATE));
}

#[test]
fn garbage_is_an_error_that_keeps_its_source() {
    let error = Sequence::from_midi(b"definitely not MIDI", RATE).unwrap_err();
    assert!(matches!(error, SequenceError::Parse(_)));
    assert!(error.source().is_some());
    assert!(Sequence::from_midi(&[], RATE).is_err());
}

#[test]
fn a_truncated_file_never_panics() {
    let bytes = smf_bytes(&[(0, Item::On(60)), (480, Item::Off(60)), (0, Item::End)]);
    for cut in 0..bytes.len() {
        let events = Sequence::from_midi(&bytes[..cut], RATE).map_or(0, |s| s.events().len());
        assert!(events <= 2);
    }
}

#[test]
fn unsupported_files_are_errors() {
    let sequential = write(
        Format::Sequential,
        Timing::Metrical(u15::from(TICKS)),
        vec![track(&[(0, Item::End)])],
    );
    assert!(matches!(
        Sequence::from_midi(&sequential, RATE),
        Err(SequenceError::UnsupportedFormat)
    ));
    let zero_ticks = write(
        Format::SingleTrack,
        Timing::Metrical(u15::from(0)),
        vec![track(&[(0, Item::End)])],
    );
    assert!(matches!(
        Sequence::from_midi(&zero_ticks, RATE),
        Err(SequenceError::ZeroTicksPerBeat)
    ));
}

#[test]
fn zero_sample_rate_is_an_error() {
    let bytes = smf_bytes(&[(0, Item::End)]);
    assert!(matches!(
        Sequence::from_midi(&bytes, 0),
        Err(SequenceError::ZeroSampleRate)
    ));
}
