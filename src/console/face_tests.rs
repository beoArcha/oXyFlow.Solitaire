//! The font face and size the player chooses in Settings, and how they share the one console font
//! with the oXySol card font: one owner, and the font the game found comes back at exit.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use oxysol_tui::app::{CardFontState, ConsoleFont};

use super::card_font::{CARD_FACE, face_name, face_units};
use super::font::{Console, FontInfo, FontZoom};

const FOUND: &str = "Raster Fonts";

fn info(face: &str, height: i16) -> FontInfo {
    FontInfo {
        index: 3,
        width: 8,
        height,
        family: 54,
        weight: 400,
        face: face_units(face),
    }
}

#[derive(Default)]
struct State {
    font: Option<FontInfo>,
    sets: Vec<FontInfo>,
    missing: Vec<&'static str>,
    refuse: bool,
}

struct Fake(Rc<RefCell<State>>);

impl Console for Fake {
    fn font(&self) -> io::Result<FontInfo> {
        self.0
            .borrow()
            .font
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn set_font(&mut self, font: &FontInfo) -> io::Result<()> {
        let mut state = self.0.borrow_mut();
        if state.refuse {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        state.sets.push(*font);
        let name = face_name(&font.face);
        if !state.missing.iter().any(|missing| *missing == name) {
            state.font = Some(*font);
        }
        Ok(())
    }
}

fn zoom() -> (Rc<RefCell<State>>, FontZoom) {
    let state = Rc::new(RefCell::new(State {
        font: Some(info(FOUND, 16)),
        missing: Vec::new(),
        ..State::default()
    }));
    let zoom = FontZoom::new(Box::new(Fake(Rc::clone(&state))));
    let Some(zoom) = zoom else {
        panic!("the fake console can be read");
    };
    (state, zoom)
}

fn shown(state: &Rc<RefCell<State>>) -> (String, i16) {
    let font = state.borrow().font.unwrap_or_else(|| info("", 0));
    (face_name(&font.face), font.height)
}

#[test]
fn a_face_is_set_read_back_and_the_old_one_comes_back_on_default() {
    let (state, mut zoom) = zoom();
    assert_eq!(zoom.use_face(Some("Consolas")), CardFontState::Active);
    assert_eq!(shown(&state), ("Consolas".to_owned(), 16));
    assert_eq!(zoom.use_face(None), CardFontState::Off);
    assert_eq!(shown(&state), (FOUND.to_owned(), 16));
}

#[test]
fn a_face_the_machine_does_not_have_is_reported_and_the_font_stays() {
    let (state, mut zoom) = zoom();
    state.borrow_mut().missing.push("Cascadia Mono");
    assert_eq!(
        zoom.use_face(Some("Cascadia Mono")),
        CardFontState::NotRegistered
    );
    assert_eq!(shown(&state), (FOUND.to_owned(), 16));
}

#[test]
fn a_console_that_refuses_the_face_is_reported() {
    let (state, mut zoom) = zoom();
    state.borrow_mut().refuse = true;
    assert_eq!(zoom.use_face(Some("Consolas")), CardFontState::Refused);
}

#[test]
fn the_size_chosen_stays_when_the_face_goes_back() {
    let (state, mut zoom) = zoom();
    zoom.use_face(Some("Consolas"));
    assert!(zoom.use_size(Some(24)));
    assert_eq!(shown(&state), ("Consolas".to_owned(), 24));
    zoom.use_face(None);
    assert_eq!(shown(&state), (FOUND.to_owned(), 24));
}

#[test]
fn a_size_is_set_and_default_returns_the_size_the_game_started_with() {
    let (state, mut zoom) = zoom();
    assert!(zoom.use_size(Some(28)));
    assert_eq!(shown(&state), (FOUND.to_owned(), 28));
    assert_eq!(state.borrow().font.map(|font| font.width), Some(0));
    assert!(zoom.use_size(None));
    assert_eq!(shown(&state), (FOUND.to_owned(), 16));
    assert_eq!(state.borrow().font.map(|font| font.width), Some(8));
}

#[test]
fn a_console_that_refuses_the_size_says_so() {
    let (state, mut zoom) = zoom();
    state.borrow_mut().refuse = true;
    assert!(!zoom.use_size(Some(20)));
}

#[test]
fn the_card_font_holds_the_face_and_gives_it_back_to_the_chosen_one() {
    let (state, mut zoom) = zoom();
    zoom.use_face(Some("Consolas"));
    assert_eq!(zoom.use_card_font(true), CardFontState::Active);
    let sets = state.borrow().sets.len();
    assert_eq!(zoom.use_face(Some("Lucida Console")), CardFontState::Off);
    assert_eq!(state.borrow().sets.len(), sets);
    assert_eq!(shown(&state).0, CARD_FACE);
    assert_eq!(zoom.use_card_font(false), CardFontState::Off);
    assert_eq!(shown(&state).0, FOUND);
    assert_eq!(zoom.use_face(Some("Lucida Console")), CardFontState::Active);
    assert_eq!(shown(&state).0, "Lucida Console");
}

#[test]
fn the_font_the_game_found_comes_back_at_exit_whatever_was_changed() {
    let (state, mut zoom) = zoom();
    zoom.use_face(Some("Consolas"));
    zoom.use_size(Some(32));
    zoom.use_card_font(true);
    drop(zoom);
    assert_eq!(shown(&state), (FOUND.to_owned(), 16));
}

#[test]
fn nothing_is_touched_at_exit_when_the_game_changed_nothing() {
    let (state, zoom) = zoom();
    drop(zoom);
    assert!(state.borrow().sets.is_empty());
}
