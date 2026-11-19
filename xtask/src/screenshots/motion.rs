//! The animations: a move, the auto-finish of a won game, and the win.

use oxysol_core::solver::{SolveBudget, Verdict, solve};
use oxysol_core::{DrawCount, Session, Variant};
use oxysol_tui::crossterm::event::KeyCode;
use oxysol_tui::i18n::Locale;
use oxysol_tui::storage::GameId;
use oxysol_tui::theme::ThemeId;
use oxysol_tui::{Modal, SavedGame, Settings};

use super::driver::{Driver, Setup};
use super::svg::Frame;
use super::{Body, Image, ScreenshotError};

const SIZE: (u16, u16) = (120, 34);
const MOVE_SEED: u64 = 7;
const FINISH_VARIANT: Variant = Variant::Klondike(DrawCount::One);
const FINISH_SEED: u64 = 65;
const CLOCK_STEP_MS: u64 = 1000;
const STEP_FRAME_MS: u32 = 220;
const HOLD_MS: u32 = 1800;
const WIN_FRAMES: usize = 8;
const MAX_STEPS: usize = 300;
const AFTER_WIN_STEPS: usize = 3;

pub fn all(locale: Locale) -> Result<Vec<Image>, ScreenshotError> {
    let mut finish = finishing_frames(locale)?;
    let split = finish.len().saturating_sub(WIN_FRAMES);
    let win = finish.split_off(split);
    Ok(vec![
        animation("anim-move", moving_frames(locale)),
        animation("anim-auto-finish", finish),
        animation("anim-win", win),
    ])
}

fn animation(name: &'static str, mut frames: Vec<Frame>) -> Image {
    if let Some(last) = frames.last_mut() {
        last.millis = HOLD_MS;
    }
    Image {
        name,
        body: Body::Animation(frames),
    }
}

fn driver(locale: Locale, saved: Option<SavedGame>) -> Driver {
    Driver::new(Setup {
        settings: Settings {
            locale,
            theme: ThemeId::Dark,
            ..Settings::default()
        },
        seed: MOVE_SEED,
        saved,
        width: SIZE.0,
        height: SIZE.1,
    })
}

fn snap(driver: &mut Driver, millis: u32) -> Frame {
    Frame {
        grid: driver.frame(),
        millis,
    }
}

fn moving_frames(locale: Locale) -> Vec<Frame> {
    let mut d = driver(locale, None);
    d.keys(&[KeyCode::Enter, KeyCode::Enter]);
    let mut frames = vec![snap(&mut d, 900)];
    for _ in 0..2 {
        d.keys(&[KeyCode::Right; 3]);
        frames.push(snap(&mut d, 260));
    }
    d.key(KeyCode::Enter);
    frames.push(snap(&mut d, 900));
    for key in [KeyCode::Right, KeyCode::Up] {
        d.key(key);
        frames.push(snap(&mut d, 600));
    }
    d.key(KeyCode::Enter);
    frames.push(snap(&mut d, HOLD_MS));
    frames
}

fn finishing_frames(locale: Locale) -> Result<Vec<Frame>, ScreenshotError> {
    let mut d = driver(locale, Some(forced_win_save()?));
    d.key(KeyCode::Enter);
    let mut frames = vec![snap(&mut d, 900)];
    let mut after_win = 0;
    for _ in 0..MAX_STEPS {
        d.wait(CLOCK_STEP_MS);
        frames.push(snap(&mut d, STEP_FRAME_MS));
        if matches!(d.app().modal(), Some(Modal::Win { .. })) {
            after_win += 1;
            if after_win > AFTER_WIN_STEPS {
                break;
            }
        }
    }
    Ok(frames)
}

/// A saved game at the first position the engine proves won, so that resuming it starts the
/// auto-finish.
fn forced_win_save() -> Result<SavedGame, ScreenshotError> {
    let Verdict::Winnable { moves } = solve(
        FINISH_VARIANT,
        FINISH_SEED,
        SolveBudget::default_for(FINISH_VARIANT),
    ) else {
        return Err(ScreenshotError::NoWinningLine {
            variant: "Klondike",
            seed: FINISH_SEED,
        });
    };
    let mut session = Session::new(FINISH_VARIANT, FINISH_SEED);
    for mv in moves {
        if session.forced_finish().is_some() {
            break;
        }
        session.apply(mv).map_err(ScreenshotError::IllegalLine)?;
    }
    Ok(SavedGame {
        id: GameId(1),
        record: session.record(),
        elapsed_ms: 45_000,
        undo_allowed: true,
    })
}
