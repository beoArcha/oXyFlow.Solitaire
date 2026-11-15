use std::collections::BTreeMap;

use super::*;
use crate::audio::style::{Timbre, Waveform};
use crate::audio::test_support::{Item, RATE, smf_bytes};

fn gate() -> Timbre {
    Timbre {
        waveform: Waveform::Square,
        attack: 0.0,
        decay: 0.0,
        sustain: 1.0,
        release: 0.0,
        gain: 1.0,
    }
}

fn programs(timbre: Timbre) -> Programs {
    Programs::new(BTreeMap::from([(0, timbre)]))
}

fn load(events: &[(u32, Item)]) -> Arc<Sequence> {
    Arc::new(Sequence::from_midi(&smf_bytes(events), RATE).unwrap())
}

fn melody() -> Arc<Sequence> {
    load(&[
        (0, Item::On(60)),
        (240, Item::Off(60)),
        (0, Item::On(64)),
        (0, Item::On(67)),
        (480, Item::Off(64)),
        (0, Item::Off(67)),
        (480, Item::End),
    ])
}

fn first_sound(samples: &[f32]) -> Option<usize> {
    samples.iter().position(|sample| *sample != 0.0)
}

#[test]
fn output_is_deterministic() {
    let rich = Timbre {
        waveform: Waveform::Saw,
        attack: 0.01,
        decay: 0.05,
        sustain: 0.4,
        release: 0.1,
        gain: 0.8,
    };
    let first = render(melody(), programs(rich));
    let second = render(melody(), programs(rich));
    assert!(!first.is_empty());
    assert!(first.iter().any(|sample| *sample != 0.0));
    assert_eq!(first, second);
}

#[test]
fn an_empty_sequence_is_silent() {
    let mut synth = Synth::new(RATE, programs(gate()));
    synth.start(Arc::new(Sequence::empty(RATE)), false).unwrap();
    let mut buffer = [1.0f32; 512];
    synth.fill(&mut buffer);
    assert!(buffer.iter().all(|sample| *sample == 0.0));
    assert!(synth.is_finished());
    assert!(render(Arc::new(Sequence::empty(RATE)), programs(gate())).is_empty());
}

#[test]
fn a_sequence_without_notes_renders_silence_of_its_length() {
    let silent = load(&[(480, Item::End)]);
    let out = render(silent, programs(gate()));
    assert_eq!(out.len(), 22_050);
    assert!(out.iter().all(|sample| *sample == 0.0));
}

#[test]
fn an_idle_synth_fills_silence() {
    let mut synth = Synth::new(RATE, programs(gate()));
    let mut buffer = [1.0f32; 64];
    synth.fill(&mut buffer);
    assert!(buffer.iter().all(|sample| *sample == 0.0));
}

#[test]
fn notes_start_and_stop_on_the_expected_samples() {
    let sequence = load(&[(480, Item::On(60)), (240, Item::Off(60)), (0, Item::End)]);
    let out = render(sequence, programs(gate()));
    assert_eq!(first_sound(&out), Some(22_050));
    assert!((33_075..=33_076).contains(&out.len()));
    assert!(out[22_050..33_075].iter().all(|sample| *sample != 0.0));
}

#[test]
fn onsets_after_a_tempo_change_are_at_the_expected_samples() {
    let sequence = load(&[
        (480, Item::On(60)),
        (240, Item::Off(60)),
        (240, Item::Tempo(1_000_000)),
        (480, Item::On(62)),
        (480, Item::Off(62)),
        (0, Item::End),
    ]);
    let out = render(sequence, programs(gate()));
    assert_eq!(out[22_049], 0.0);
    assert_ne!(out[22_050], 0.0);
    assert_eq!(out[33_075], 0.0);
    assert_eq!(out[88_199], 0.0);
    assert_ne!(out[88_200], 0.0);
    assert!((132_300..=132_301).contains(&out.len()));
}

#[test]
fn the_voice_limit_holds() {
    let chord: Vec<(u32, Item)> = (0..10u8)
        .map(|key| (0, Item::On(60 + key)))
        .chain([(4800, Item::End)])
        .collect();
    let mut synth = Synth::with_voice_limit(RATE, 4, programs(gate()));
    synth.start(load(&chord), false).unwrap();
    let mut buffer = [0.0f32; 64];
    let mut peak = 0;
    for _ in 0..100 {
        synth.fill(&mut buffer);
        peak = peak.max(synth.active_voices());
        assert!(synth.active_voices() <= 4);
    }
    assert_eq!(peak, 4);
}

#[test]
fn stealing_takes_the_oldest_voice() {
    let sequence = load(&[
        (0, Item::On(60)),
        (0, Item::On(62)),
        (0, Item::On(64)),
        (480, Item::Off(60)),
        (480, Item::End),
    ]);
    let mut synth = Synth::with_voice_limit(RATE, 2, programs(gate()));
    synth.start(sequence, false).unwrap();
    let mut buffer = [0.0f32; 128];
    synth.fill(&mut buffer);
    assert_eq!(synth.active_voices(), 2);
}

#[test]
fn output_stays_in_range_and_finite() {
    let chord: Vec<(u32, Item)> = (0..32u8)
        .map(|key| (0, Item::On(30 + key * 3)))
        .chain([(480, Item::End)])
        .collect();
    for waveform in [
        Waveform::Sine,
        Waveform::Square,
        Waveform::Saw,
        Waveform::Triangle,
    ] {
        let timbre = Timbre {
            waveform,
            gain: 1.0,
            ..gate()
        };
        let out = render(load(&chord), programs(timbre));
        assert!(!out.is_empty());
        assert!(
            out.iter()
                .all(|sample| sample.is_finite() && sample.abs() <= 1.0)
        );
    }
}

#[test]
fn looping_repeats_the_sequence_exactly() {
    let sequence = load(&[(0, Item::On(60)), (240, Item::Off(60)), (240, Item::End)]);
    let length = usize::try_from(sequence.length()).unwrap();
    let mut synth = Synth::new(RATE, programs(gate()));
    synth.start(sequence, true).unwrap();
    let mut out = vec![0.0f32; length * 3];
    synth.fill(&mut out);
    assert_eq!(out[..length], out[length..length * 2]);
    assert_eq!(out[..length], out[length * 2..]);
    assert!(!synth.is_finished());
}

#[test]
fn a_note_that_never_ends_is_released_with_the_sequence() {
    let out = render(
        load(&[(0, Item::On(60)), (480, Item::End)]),
        programs(gate()),
    );
    assert!((22_050..=22_051).contains(&out.len()));
}

#[test]
fn release_rings_out_after_the_sequence_ends() {
    let slow = Timbre {
        release: 0.1,
        ..gate()
    };
    let out = render(
        load(&[(0, Item::On(60)), (480, Item::Off(60)), (0, Item::End)]),
        programs(slow),
    );
    assert!((26_460..=26_462).contains(&out.len()));
    assert_ne!(out[22_050], 0.0);
    assert_eq!(out[out.len() - 1], 0.0);
}

#[test]
fn starting_with_a_different_sample_rate_is_an_error() {
    let mut synth = Synth::new(48_000, programs(gate()));
    let result = synth.start(melody(), false);
    assert!(matches!(
        result,
        Err(SynthError::SampleRateMismatch {
            synth: 48_000,
            sequence: RATE
        })
    ));
}

#[test]
fn stop_silences_through_the_release() {
    let mut synth = Synth::new(RATE, programs(gate()));
    synth
        .start(load(&[(0, Item::On(60)), (4800, Item::End)]), false)
        .unwrap();
    let mut buffer = [0.0f32; 32];
    synth.fill(&mut buffer);
    assert_eq!(synth.active_voices(), 1);
    synth.stop();
    synth.fill(&mut buffer);
    assert!(synth.is_finished());
}
