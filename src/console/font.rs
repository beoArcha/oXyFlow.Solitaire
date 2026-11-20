//! The font size of the classic Windows console. Ctrl+Plus, Ctrl+Minus and Ctrl+0 reach the game only
//! in a console that has no zoom of its own; there the game makes the font 2 pixels larger or smaller
//! or returns it to the size it started with. Only the size changes, the face stays. The TUI asks
//! through `ConsoleFont`; [`FontZoom`] answers with the pure step of [`target_height`] and a
//! [`Console`], the one place that talks to the system.

use std::io;

use super::card_font;
use oxysol_tui::app::{CardFontState, ConsoleFont, FontStep};

/// The pixels a larger or a smaller font differs by.
pub const STEP: i16 = 2;
/// The smallest font height the game asks for.
pub const MIN_HEIGHT: i16 = 8;
/// The largest font height the game asks for.
pub const MAX_HEIGHT: i16 = 72;

/// The font of a console, as `CONSOLE_FONT_INFOEX` describes it without its own size field. The face
/// is kept as the system gave it, so setting a font back never changes the face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontInfo {
    pub index: u32,
    pub width: i16,
    pub height: i16,
    pub family: u32,
    pub weight: u32,
    pub face: [u16; 32],
}

/// What the game needs from a console window. [`ClassicConsole`] implements it for the classic Windows
/// console; the tests use a fake.
pub trait Console {
    /// The font the console shows now.
    fn font(&self) -> io::Result<FontInfo>;
    /// Makes the console show `font`.
    fn set_font(&mut self, font: &FontInfo) -> io::Result<()>;
}

/// The font height `step` leads to from `current`, or `None` when nothing would change: at a limit,
/// or already at the start size.
pub fn target_height(start: i16, current: i16, step: FontStep) -> Option<i16> {
    let target = match step {
        FontStep::Larger => current.saturating_add(STEP).min(MAX_HEIGHT),
        FontStep::Smaller => current.saturating_sub(STEP).max(MIN_HEIGHT),
        FontStep::Reset => start,
    };
    (target != current).then_some(target)
}

/// Changes the font size of a [`Console`] and remembers the font it started with.
pub struct FontZoom {
    console: Box<dyn Console>,
    start: FontInfo,
    saved: Option<FontInfo>,
}

impl FontZoom {
    /// Reads the font the console has now as the size to return to. `None` when it cannot be read,
    /// for example when the output is not a console.
    pub fn new(console: Box<dyn Console>) -> Option<Self> {
        match console.font() {
            Ok(start) => Some(Self {
                console,
                start,
                saved: None,
            }),
            Err(error) => {
                tracing::info!(%error, "the console font cannot be read; no font zoom");
                None
            }
        }
    }

    fn change(&mut self, step: FontStep) -> io::Result<bool> {
        let mut font = self.console.font()?;
        let Some(height) = target_height(self.start.height, font.height, step) else {
            return Ok(false);
        };
        font.height = height;
        font.width = match step {
            FontStep::Reset => self.start.width,
            FontStep::Larger | FontStep::Smaller => 0,
        };
        self.console.set_font(&font)?;
        Ok(true)
    }
}

impl ConsoleFont for FontZoom {
    fn step(&mut self, step: FontStep) -> bool {
        match self.change(step) {
            Ok(changed) => {
                tracing::debug!(?step, changed, "the console font size was asked to change");
                changed
            }
            Err(error) => {
                tracing::info!(?step, %error, "the console refused the font size");
                false
            }
        }
    }

    fn cell_size(&self) -> Option<(u16, u16)> {
        let font = self.console.font().ok()?;
        let width = u16::try_from(font.width).ok().filter(|width| *width > 0)?;
        let height = u16::try_from(font.height)
            .ok()
            .filter(|height| *height > 0)?;
        Some((width, height))
    }

    fn use_card_font(&mut self, on: bool) -> CardFontState {
        if on {
            card_font::apply(&mut *self.console, &mut self.saved)
        } else {
            card_font::restore(&mut *self.console, &mut self.saved);
            CardFontState::Off
        }
    }
}

impl Drop for FontZoom {
    /// Sets the font back that the card font option replaced, so the console is left as it was found.
    fn drop(&mut self) {
        card_font::restore(&mut *self.console, &mut self.saved);
    }
}

/// The console window of this process, through the system's console font calls.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassicConsole;

impl Console for ClassicConsole {
    fn font(&self) -> io::Result<FontInfo> {
        platform::font()
    }

    fn set_font(&mut self, font: &FontInfo) -> io::Result<()> {
        platform::set_font(font)
    }
}

/// `GetCurrentConsoleFontEx` and `SetCurrentConsoleFontEx` on the standard output of the process.
#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "the console font calls have no safe wrapper in std and the crate avoids windows-sys"
)]
mod platform {
    use std::io::{self, Write};
    use std::mem::size_of;
    use std::os::windows::io::{AsRawHandle, RawHandle};

    use super::FontInfo;

    type Handle = RawHandle;

    #[repr(C)]
    struct ConsoleFontInfoEx {
        size: u32,
        font: u32,
        width: i16,
        height: i16,
        family: u32,
        weight: u32,
        face: [u16; 32],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentConsoleFontEx(
            output: Handle,
            maximum_window: i32,
            info: *mut ConsoleFontInfoEx,
        ) -> i32;
        fn SetCurrentConsoleFontEx(
            output: Handle,
            maximum_window: i32,
            info: *const ConsoleFontInfoEx,
        ) -> i32;
    }

    fn raw_size() -> u32 {
        u32::try_from(size_of::<ConsoleFontInfoEx>()).unwrap_or(0)
    }

    pub fn font() -> io::Result<FontInfo> {
        let mut raw = ConsoleFontInfoEx {
            size: raw_size(),
            font: 0,
            width: 0,
            height: 0,
            family: 0,
            weight: 0,
            face: [0; 32],
        };
        let output = io::stdout().as_raw_handle();
        // SAFETY: `output` is the live stdout handle and `raw` is a sized CONSOLE_FONT_INFOEX that outlives the call.
        let read = unsafe { GetCurrentConsoleFontEx(output, 0, &mut raw) };
        if read == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(FontInfo {
            index: raw.font,
            width: raw.width,
            height: raw.height,
            family: raw.family,
            weight: raw.weight,
            face: raw.face,
        })
    }

    pub fn set_font(font: &FontInfo) -> io::Result<()> {
        let raw = ConsoleFontInfoEx {
            size: raw_size(),
            font: font.index,
            width: font.width,
            height: font.height,
            family: font.family,
            weight: font.weight,
            face: font.face,
        };
        io::stdout().flush()?;
        let output = io::stdout().as_raw_handle();
        // SAFETY: `output` is the live stdout handle and `raw` is a fully initialized CONSOLE_FONT_INFOEX that outlives the call.
        let accepted = unsafe { SetCurrentConsoleFontEx(output, 0, &raw) };
        if accepted == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Without a console API the game cannot change a font.
#[cfg(not(windows))]
mod platform {
    use std::io;

    use super::FontInfo;

    pub fn font() -> io::Result<FontInfo> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn set_font(_font: &FontInfo) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    const START: i16 = 16;

    fn face(name: &str) -> [u16; 32] {
        let mut face = [0_u16; 32];
        for (slot, unit) in face.iter_mut().zip(name.encode_utf16()) {
            *slot = unit;
        }
        face
    }

    fn info(height: i16) -> FontInfo {
        FontInfo {
            index: 7,
            width: 8,
            height,
            family: 54,
            weight: 400,
            face: face("Consolas"),
        }
    }

    #[derive(Default)]
    struct State {
        font: Option<FontInfo>,
        sets: Vec<FontInfo>,
        refuse_set: bool,
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
            if state.refuse_set {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            state.sets.push(*font);
            state.font = Some(*font);
            Ok(())
        }
    }

    fn state_with(font: Option<FontInfo>) -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State {
            font,
            ..State::default()
        }))
    }

    fn zoom_of(state: &Rc<RefCell<State>>) -> FontZoom {
        FontZoom::new(Box::new(Fake(Rc::clone(state)))).expect("the fake console can be read")
    }

    fn height(state: &Rc<RefCell<State>>) -> Option<i16> {
        state.borrow().font.map(|font| font.height)
    }

    #[test]
    fn larger_and_smaller_move_two_pixels() {
        assert_eq!(target_height(START, 16, FontStep::Larger), Some(18));
        assert_eq!(target_height(START, 16, FontStep::Smaller), Some(14));
        assert_eq!(target_height(START, 18, FontStep::Smaller), Some(16));
    }

    #[test]
    fn the_limits_change_nothing_and_never_pass() {
        assert_eq!(target_height(START, MAX_HEIGHT, FontStep::Larger), None);
        assert_eq!(target_height(START, MIN_HEIGHT, FontStep::Smaller), None);
        assert_eq!(
            target_height(START, MAX_HEIGHT - 1, FontStep::Larger),
            Some(MAX_HEIGHT)
        );
        assert_eq!(
            target_height(START, MIN_HEIGHT + 1, FontStep::Smaller),
            Some(MIN_HEIGHT)
        );
    }

    #[test]
    fn reset_returns_to_the_start_size_and_is_a_no_op_there() {
        assert_eq!(target_height(START, 24, FontStep::Reset), Some(START));
        assert_eq!(target_height(START, 10, FontStep::Reset), Some(START));
        assert_eq!(target_height(START, START, FontStep::Reset), None);
    }

    #[test]
    fn the_steps_change_the_size_and_keep_the_face() {
        let state = state_with(Some(info(START)));
        let mut zoom = zoom_of(&state);
        assert!(zoom.step(FontStep::Larger));
        assert!(zoom.step(FontStep::Larger));
        assert_eq!(height(&state), Some(20));
        assert!(zoom.step(FontStep::Smaller));
        assert_eq!(height(&state), Some(18));
        assert!(zoom.step(FontStep::Reset));
        assert_eq!(height(&state), Some(START));
        assert!(!zoom.step(FontStep::Reset));
        let state = state.borrow();
        assert!(state.sets.iter().all(|font| font.face == face("Consolas")));
        assert!(
            state
                .sets
                .iter()
                .all(|font| font.index == 7 && font.family == 54)
        );
    }

    #[test]
    fn a_step_lets_the_system_pick_the_width_and_reset_restores_the_start_width() {
        let state = state_with(Some(info(START)));
        let mut zoom = zoom_of(&state);
        assert!(zoom.step(FontStep::Larger));
        assert_eq!(state.borrow().font.map(|font| font.width), Some(0));
        assert!(zoom.step(FontStep::Reset));
        assert_eq!(state.borrow().font.map(|font| font.width), Some(8));
    }

    #[test]
    fn a_limit_makes_no_call() {
        let state = state_with(Some(info(MAX_HEIGHT)));
        let mut zoom = zoom_of(&state);
        assert!(!zoom.step(FontStep::Larger));
        assert!(state.borrow().sets.is_empty());
    }

    #[test]
    fn a_console_that_refuses_is_not_an_error() {
        let state = state_with(Some(info(START)));
        let mut zoom = zoom_of(&state);
        state.borrow_mut().refuse_set = true;
        assert!(!zoom.step(FontStep::Larger));
        assert_eq!(height(&state), Some(START));
    }

    #[test]
    fn an_unreadable_console_gives_no_zoom() {
        let state = state_with(None);
        assert!(FontZoom::new(Box::new(Fake(state))).is_none());
    }

    #[cfg(not(windows))]
    #[test]
    fn a_platform_without_a_console_api_has_no_font() {
        assert!(ClassicConsole.font().is_err());
        assert!(FontZoom::new(Box::new(ClassicConsole)).is_none());
    }
}
