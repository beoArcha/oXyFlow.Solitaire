//! Helpers that build Standard MIDI File bytes for the audio tests.

use midly::num::{u4, u7, u15, u24, u28};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

pub const RATE: u32 = 44_100;
pub const TICKS: u16 = 480;

#[derive(Debug, Clone, Copy)]
pub enum Item {
    Tempo(u32),
    Program(u8),
    On(u8),
    Off(u8),
    End,
}

fn track_event(delta: u32, item: Item) -> TrackEvent<'static> {
    let kind = match item {
        Item::Tempo(micros) => TrackEventKind::Meta(MetaMessage::Tempo(u24::from(micros))),
        Item::End => TrackEventKind::Meta(MetaMessage::EndOfTrack),
        Item::Program(program) => midi(MidiMessage::ProgramChange {
            program: u7::from(program),
        }),
        Item::On(key) => midi(MidiMessage::NoteOn {
            key: u7::from(key),
            vel: u7::from(100),
        }),
        Item::Off(key) => midi(MidiMessage::NoteOff {
            key: u7::from(key),
            vel: u7::from(0),
        }),
    };
    TrackEvent {
        delta: u28::from(delta),
        kind,
    }
}

fn midi(message: MidiMessage) -> TrackEventKind<'static> {
    TrackEventKind::Midi {
        channel: u4::from(0),
        message,
    }
}

pub fn track(events: &[(u32, Item)]) -> Vec<TrackEvent<'static>> {
    events
        .iter()
        .map(|(delta, item)| track_event(*delta, *item))
        .collect()
}

pub fn write(format: Format, timing: Timing, tracks: Vec<Vec<TrackEvent<'static>>>) -> Vec<u8> {
    let smf = Smf {
        header: Header { format, timing },
        tracks,
    };
    let mut bytes = Vec::new();
    smf.write_std(&mut bytes).unwrap();
    bytes
}

pub fn smf_bytes(events: &[(u32, Item)]) -> Vec<u8> {
    write(
        Format::SingleTrack,
        Timing::Metrical(u15::from(TICKS)),
        vec![track(events)],
    )
}
