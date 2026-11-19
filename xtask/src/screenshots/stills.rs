//! The still screenshots: every screen, and the board in every theme.

use oxysol_tui::Settings;
use oxysol_tui::crossterm::event::KeyCode;
use oxysol_tui::i18n::Locale;
use oxysol_tui::theme::ThemeId;

use super::driver::{Driver, Setup};
use super::{Body, Image};

const SEED: u64 = 7;
const BOARD: (u16, u16) = (140, 40);
const SCREEN: (u16, u16) = (100, 34);

const SPIDER_TWO_SUITS: usize = 3;
const FREECELL: usize = 5;
const PYRAMID_RELAXED: usize = 6;

const THEMES: [(&str, ThemeId); 5] = [
    ("theme-dark", ThemeId::Dark),
    ("theme-light", ThemeId::Light),
    ("theme-high-contrast", ThemeId::HighContrast),
    ("theme-monokai", ThemeId::Monokai),
    ("theme-dracula", ThemeId::Dracula),
];

pub fn all(locale: Locale) -> Vec<Image> {
    let mut images = vec![
        still("menu", menu(locale)),
        still("klondike", board(locale, ThemeId::Dark, 0)),
        still("spider", board(locale, ThemeId::Dark, SPIDER_TWO_SUITS)),
        still("freecell", board(locale, ThemeId::Dark, FREECELL)),
        still("pyramid", board(locale, ThemeId::Dark, PYRAMID_RELAXED)),
        still("settings", settings(locale)),
        still("help", help(locale)),
        still("guide", hinted(locale, ThemeId::Dark)),
    ];
    for (name, theme) in THEMES {
        images.push(still(name, opening(locale, theme)));
    }
    images
}

fn still(name: &'static str, mut driver: Driver) -> Image {
    Image {
        name,
        body: Body::Still(driver.frame()),
    }
}

fn driver(locale: Locale, theme: ThemeId, size: (u16, u16)) -> Driver {
    Driver::new(Setup {
        settings: Settings {
            locale,
            theme,
            ..Settings::default()
        },
        seed: SEED,
        saved: None,
        width: size.0,
        height: size.1,
    })
}

fn menu(locale: Locale) -> Driver {
    driver(locale, ThemeId::Dark, SCREEN)
}

fn settings(locale: Locale) -> Driver {
    let mut d = driver(locale, ThemeId::Dark, SCREEN);
    d.keys(&[KeyCode::Down, KeyCode::Down, KeyCode::Down, KeyCode::Enter]);
    d
}

fn help(locale: Locale) -> Driver {
    let mut d = driver(locale, ThemeId::Dark, SCREEN);
    d.text("?");
    d
}

/// A new game of the entry at `index` of the game selection, dealt from the fixed seed.
fn board(locale: Locale, theme: ThemeId, index: usize) -> Driver {
    let mut d = driver(locale, theme, BOARD);
    d.key(KeyCode::Enter);
    for _ in 0..index {
        d.key(KeyCode::Down);
    }
    d.key(KeyCode::Enter);
    d
}

fn hinted(locale: Locale, theme: ThemeId) -> Driver {
    let mut d = board(locale, theme, 0);
    d.text("h");
    d
}

fn opening(locale: Locale, theme: ThemeId) -> Driver {
    let mut d = board(locale, theme, 0);
    d.keys(&[KeyCode::Right; 6]);
    d.keys(&[KeyCode::Enter, KeyCode::Right, KeyCode::Up, KeyCode::Enter]);
    d.text("h");
    d
}
