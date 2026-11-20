//! The oXySol font in the classic Windows console (D-06, P5-02). The game never installs the font; it
//! only asks the console to use `oXySol Mono` when the player turned the option on. The check is
//! the read-back of the face: the console accepts a face that is not registered without complaint
//! and keeps another font, so the face is set, read again and compared. A font that is not in use
//! afterwards is put back and reported as not registered. The size is never changed.

use super::font::{Console, FontInfo};
use oxysol_tui::app::CardFontState;

/// The family name of the card font.
pub const CARD_FACE: &str = "oXySol Mono";

/// The face as the 32 UTF-16 units of `CONSOLE_FONT_INFOEX`, ended by a zero unit.
pub fn face_units(name: &str) -> [u16; 32] {
    let mut face = [0_u16; 32];
    for (slot, unit) in face.iter_mut().zip(name.encode_utf16().take(31)) {
        *slot = unit;
    }
    face
}

/// The text of a face, up to its first zero unit.
pub fn face_name(face: &[u16; 32]) -> String {
    let units: Vec<u16> = face.iter().copied().take_while(|unit| *unit != 0).collect();
    String::from_utf16_lossy(&units)
}

fn card_face(font: &FontInfo) -> bool {
    face_name(&font.face) == CARD_FACE
}

fn put_back(console: &mut dyn Console, font: &FontInfo) {
    if let Err(error) = console.set_font(font) {
        tracing::info!(%error, "the previous console font could not be set back");
    }
}

/// Switches `console` to the card face and keeps the font it had in `saved`, to restore later.
/// Already showing the card face is a success with nothing to restore.
pub fn apply(console: &mut dyn Console, saved: &mut Option<FontInfo>) -> CardFontState {
    let current = match console.font() {
        Ok(font) => font,
        Err(error) => {
            tracing::info!(%error, "the console font cannot be read; the card font is not used");
            return CardFontState::Refused;
        }
    };
    if card_face(&current) {
        return CardFontState::Active;
    }
    let wanted = FontInfo {
        face: face_units(CARD_FACE),
        ..current
    };
    if let Err(error) = console.set_font(&wanted) {
        tracing::info!(%error, "the console refused the card font");
        return CardFontState::Refused;
    }
    match console.font() {
        Ok(now) if card_face(&now) => {
            saved.get_or_insert(current);
            CardFontState::Active
        }
        Ok(_) => {
            tracing::info!("the card font is not registered in this console");
            put_back(console, &current);
            CardFontState::NotRegistered
        }
        Err(error) => {
            tracing::info!(%error, "the console font cannot be read back");
            put_back(console, &current);
            CardFontState::Refused
        }
    }
}

/// Sets back the font `apply` replaced; nothing to do when it replaced none.
pub fn restore(console: &mut dyn Console, saved: &mut Option<FontInfo>) {
    if let Some(font) = saved.take() {
        put_back(console, &font);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    use oxysol_tui::app::{ConsoleFont, FontStep};

    use super::super::font::FontZoom;
    use super::*;

    fn info(face: &str) -> FontInfo {
        FontInfo {
            index: 7,
            width: 8,
            height: 20,
            family: 54,
            weight: 400,
            face: face_units(face),
        }
    }

    #[derive(Default)]
    struct State {
        font: Option<FontInfo>,
        sets: Vec<FontInfo>,
        registered: bool,
        refuse: bool,
        unreadable_after_set: bool,
    }

    struct Fake(Rc<RefCell<State>>);

    impl Console for Fake {
        fn font(&self) -> io::Result<FontInfo> {
            let state = self.0.borrow();
            if state.unreadable_after_set && !state.sets.is_empty() {
                return Err(io::ErrorKind::Other.into());
            }
            state.font.ok_or_else(|| io::ErrorKind::NotFound.into())
        }

        fn set_font(&mut self, font: &FontInfo) -> io::Result<()> {
            let mut state = self.0.borrow_mut();
            if state.refuse {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            state.sets.push(*font);
            let known = face_name(&font.face) != CARD_FACE || state.registered;
            if known {
                state.font = Some(*font);
            }
            Ok(())
        }
    }

    fn console(registered: bool) -> (Rc<RefCell<State>>, Fake) {
        let state = Rc::new(RefCell::new(State {
            font: Some(info("Consolas")),
            registered,
            ..State::default()
        }));
        (Rc::clone(&state), Fake(state))
    }

    fn current(state: &Rc<RefCell<State>>) -> Option<FontInfo> {
        state.borrow().font
    }

    #[test]
    fn a_registered_font_is_used_and_the_size_stays() {
        let (state, mut console) = console(true);
        let mut saved = None;
        assert_eq!(apply(&mut console, &mut saved), CardFontState::Active);
        let now = current(&state);
        assert_eq!(
            now.map(|font| face_name(&font.face)).as_deref(),
            Some(CARD_FACE)
        );
        assert_eq!(now.map(|font| (font.width, font.height)), Some((8, 20)));
        assert!(saved.is_some());
    }

    #[test]
    fn a_font_that_is_not_registered_is_reported_and_the_previous_font_stays() {
        let (state, mut console) = console(false);
        let mut saved = None;
        assert_eq!(
            apply(&mut console, &mut saved),
            CardFontState::NotRegistered
        );
        assert_eq!(current(&state), Some(info("Consolas")));
        assert!(saved.is_none());
        assert_eq!(state.borrow().sets.len(), 2);
    }

    #[test]
    fn a_console_that_refuses_the_face_is_reported_and_nothing_changes() {
        let (state, mut console) = console(true);
        state.borrow_mut().refuse = true;
        let mut saved = None;
        assert_eq!(apply(&mut console, &mut saved), CardFontState::Refused);
        assert_eq!(current(&state), Some(info("Consolas")));
        assert!(saved.is_none());
    }

    #[test]
    fn a_console_that_cannot_be_read_gives_refused_without_a_set() {
        let (state, mut console) = console(true);
        state.borrow_mut().font = None;
        let mut saved = None;
        assert_eq!(apply(&mut console, &mut saved), CardFontState::Refused);
        assert!(state.borrow().sets.is_empty());
    }

    #[test]
    fn a_font_that_cannot_be_read_back_is_put_back_and_refused() {
        let (state, mut console) = console(true);
        state.borrow_mut().unreadable_after_set = true;
        let mut saved = None;
        assert_eq!(apply(&mut console, &mut saved), CardFontState::Refused);
        assert_eq!(
            state
                .borrow()
                .sets
                .last()
                .map(|font| face_name(&font.face))
                .as_deref(),
            Some("Consolas")
        );
    }

    #[test]
    fn restoring_returns_the_previous_face_and_size_once() {
        let (state, mut console) = console(true);
        let mut saved = None;
        apply(&mut console, &mut saved);
        restore(&mut console, &mut saved);
        assert_eq!(current(&state), Some(info("Consolas")));
        assert!(saved.is_none());
        let sets = state.borrow().sets.len();
        restore(&mut console, &mut saved);
        assert_eq!(state.borrow().sets.len(), sets);
    }

    #[test]
    fn the_card_face_already_in_use_is_a_success_with_nothing_to_restore() {
        let (state, mut console) = console(true);
        state.borrow_mut().font = Some(info(CARD_FACE));
        let mut saved = None;
        assert_eq!(apply(&mut console, &mut saved), CardFontState::Active);
        assert!(saved.is_none());
        assert!(state.borrow().sets.is_empty());
        restore(&mut console, &mut saved);
        assert!(state.borrow().sets.is_empty());
    }

    #[test]
    fn applying_twice_keeps_the_first_font_to_restore() {
        let (state, mut console) = console(true);
        let mut saved = None;
        apply(&mut console, &mut saved);
        apply(&mut console, &mut saved);
        restore(&mut console, &mut saved);
        assert_eq!(current(&state), Some(info("Consolas")));
    }

    #[test]
    fn a_face_round_trips_through_its_units() {
        assert_eq!(face_name(&face_units(CARD_FACE)), CARD_FACE);
        assert_eq!(face_name(&face_units("")), "");
        let long = "x".repeat(40);
        assert_eq!(face_name(&face_units(&long)).len(), 31);
    }

    #[test]
    fn the_zoom_switches_the_card_font_on_and_off_and_keeps_the_face_when_it_resizes() {
        let (state, console) = console(true);
        let zoom = FontZoom::new(Box::new(console));
        let Some(mut zoom) = zoom else {
            panic!("the fake console can be read");
        };
        assert_eq!(zoom.use_card_font(true), CardFontState::Active);
        assert!(zoom.step(FontStep::Larger));
        let face = current(&state).map(|font| face_name(&font.face));
        assert_eq!(face.as_deref(), Some(CARD_FACE));
        assert_eq!(zoom.use_card_font(false), CardFontState::Off);
        let font = current(&state);
        assert_eq!(
            font.map(|font| face_name(&font.face)).as_deref(),
            Some("Consolas")
        );
        assert_eq!(font.map(|font| font.height), Some(20));
    }

    #[test]
    fn dropping_the_zoom_gives_the_old_font_back() {
        let (state, console) = console(true);
        let zoom = FontZoom::new(Box::new(console));
        let Some(mut zoom) = zoom else {
            panic!("the fake console can be read");
        };
        assert_eq!(zoom.use_card_font(true), CardFontState::Active);
        drop(zoom);
        assert_eq!(current(&state), Some(info("Consolas")));
    }
}
