//! The graphics check (D-06, P5-02): a dialog with samples in the oXySol card font (the suits of the
//! front style, its court emblems, the tile and the mosaic of the back style) and the question
//! whether they look like cards and not like squares or blanks. "Yes" applies the settings it
//! carries (glyph mode Font, confirmed); "No" keeps the Unicode glyphs and points to the wiki. In
//! Windows Terminal it also gives the font face to set in the profile.

use oxysol_core::{Rank, Suit};

use super::dialog::{Button, Text};
use crate::card_style::CardStyles;
use crate::i18n::{t, tf};
use crate::screens::Effect;
use crate::settings::{GlyphMode, Settings};

/// The dialog: the lines it shows, what "Yes" does and which button has the focus.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsCheck {
    lines: Vec<String>,
    on_yes: Box<Effect>,
    pub(super) yes_focused: bool,
}

fn suits_line(front: &str) -> Option<String> {
    let front = CardStyles::embedded().front_or_default(front)?;
    let symbols = front.suit_symbols(GlyphMode::Font);
    let shown: Vec<String> = Suit::ALL
        .iter()
        .zip(symbols)
        .map(|(suit, symbol)| format!("{symbol} {}", t(&front.suit(*suit).name_key)))
        .collect();
    Some(tf(
        "settings.graphics.suits",
        &[("symbols", &shown.join("  "))],
    ))
}

fn courts_line(front: &str) -> Option<String> {
    let front = CardStyles::embedded().front_or_default(front)?;
    let emblems: Vec<&str> = [Rank::Jack, Rank::Queen, Rank::King]
        .into_iter()
        .filter_map(|rank| front.court_emblem(rank))
        .collect();
    (!emblems.is_empty()).then(|| {
        tf(
            "settings.graphics.courts",
            &[("symbols", &emblems.join(" "))],
        )
    })
}

fn back_line(back: &str) -> Option<String> {
    let back = CardStyles::embedded().back_or_default(back)?;
    let [top, bottom] = back.mosaic_rows(GlyphMode::Font);
    let mosaic = format!("{top} / {bottom}");
    Some(tf(
        "settings.graphics.back",
        &[("tile", back.tile(GlyphMode::Font)), ("mosaic", &mosaic)],
    ))
}

impl GraphicsCheck {
    /// The check for the styles in `settings`. `windows_terminal` adds the hint on the font face.
    /// Focus starts on "No", as in every question.
    pub fn new(settings: &Settings, windows_terminal: bool, on_yes: Effect) -> Self {
        let mut lines = vec![t("settings.graphics.body").into_owned()];
        lines.extend(suits_line(&settings.front_style));
        lines.extend(courts_line(&settings.front_style));
        lines.extend(back_line(&settings.back_style));
        if windows_terminal {
            lines.push(t("settings.graphics.hint_windows_terminal").into_owned());
        }
        lines.push(t("settings.graphics.question").into_owned());
        Self {
            lines,
            on_yes: Box::new(on_yes),
            yes_focused: false,
        }
    }

    /// The lines of the dialog, for the tests.
    #[cfg(test)]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub(super) fn text(&self) -> Text {
        let button = |key: &str, focused: bool| Button {
            label: t(key).into_owned(),
            focused,
        };
        Text {
            title: t("settings.graphics.title").into_owned(),
            lines: self.lines.clone(),
            buttons: vec![
                button("common.dialog.yes", self.yes_focused),
                button("common.dialog.no", !self.yes_focused),
            ],
        }
    }

    /// What button `index` does: "Yes" is the effect it carries, "No" opens the notice that points
    /// to the installation page.
    pub(super) fn press(&self, index: usize) -> Effect {
        if index == 0 {
            (*self.on_yes).clone()
        } else {
            Effect::Open(crate::app::Modal::alert("settings.graphics.declined"))
        }
    }
}
