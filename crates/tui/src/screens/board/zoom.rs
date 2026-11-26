//! Zoom on the board: the keys `]`, `[` and `0`, and Ctrl+mouse wheel where the terminal delivers
//! it, change the card size setting. The step is decided by the pure `settings::zoom_step` from the
//! size on the board now and the largest size that fits, so a key always changes what the player
//! sees or says why it cannot.

use ratatui::crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};

use super::{BoardScreen, status};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::layout::{self, CardSize};
use crate::screens::Effect;
use crate::settings::{CardSizeSetting, Zoom, Zoomed, zoom_step};

impl BoardScreen {
    /// The zoom key action of a wheel turn with Ctrl held: up zooms in, down zooms out.
    pub(super) fn wheel_zoom(&self, event: &MouseEvent) -> Option<Action> {
        if !event.modifiers.contains(KeyModifiers::CONTROL) {
            return None;
        }
        match event.kind {
            MouseEventKind::ScrollUp => Some(Action::ZoomIn),
            MouseEventKind::ScrollDown => Some(Action::ZoomOut),
            _ => None,
        }
    }

    /// Handles a zoom action. The setting changes through [`Effect::CardSize`]; at the limit of what
    /// the window allows the event slot says so and nothing changes.
    pub(super) fn zoom(&mut self, action: Action) -> Effect {
        let zoom = match action {
            Action::ZoomIn => Zoom::In,
            Action::ZoomOut => Zoom::Out,
            _ => return self.zoom_auto(),
        };
        let Some((shown, largest)) = self.card_sizes() else {
            return Effect::None;
        };
        match zoom_step(shown, largest, zoom) {
            Zoomed::To(size) => Effect::CardSize(size),
            Zoomed::AtLimit => {
                self.status = Some(match zoom {
                    Zoom::In => status::ZOOM_LARGEST,
                    Zoom::Out => status::ZOOM_SMALLEST,
                });
                Effect::None
            }
        }
    }

    fn zoom_auto(&mut self) -> Effect {
        self.status = Some(status::ZOOM_AUTO);
        if self.card_size == CardSizeSetting::Auto {
            Effect::None
        } else {
            Effect::CardSize(CardSizeSetting::Auto)
        }
    }

    /// The card size on the board now and the largest size that fits the window.
    fn card_sizes(&self) -> Option<(CardSize, CardSize)> {
        let session = self.session.as_ref()?;
        let piles = session.game().piles();
        let body = self.chrome().body;
        let shown = layout::compute_for_scale(
            session.variant(),
            &piles,
            body,
            self.card_size.size(),
            self.low_vision,
            self.interface_size,
        );
        let largest = layout::compute_for_scale(
            session.variant(),
            &piles,
            body,
            None,
            self.low_vision,
            self.interface_size,
        );
        Some((shown.card, largest.card))
    }

    /// The game information about the card size: the size on the board when the player chose one,
    /// marked `(auto)` when the window is too small for it and a smaller one is shown.
    pub(super) fn card_size_note(&self) -> Option<String> {
        let chosen = self.card_size.size()?;
        let (shown, _) = self.card_sizes()?;
        let name = t(CardSizeSetting::of(shown).label_key()).into_owned();
        let key = if shown == chosen {
            "board.header.card_size"
        } else {
            "board.header.card_size_auto"
        };
        Some(tf(key, &[("size", &name)]))
    }
}
