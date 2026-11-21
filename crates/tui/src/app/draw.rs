//! Drawing of the application: the current screen, then the modal layer on top, docked or centred.

use ratatui::Frame;
use ratatui::widgets::Block;

use super::{App, Screen};
use crate::screens::ViewCtx;
use crate::widgets::footer::FooterMap;
use crate::widgets::overlay;
use crate::widgets::too_small::{self, is_too_small};

impl App {
    /// Draws the current screen, then the modal on top. Below the minimum size only the notice
    /// is drawn.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.area = Some((area.width, area.height));
        let ctx = view_ctx!(self);
        frame.render_widget(Block::new().style(ctx.theme.base()), area);
        if is_too_small(area) {
            too_small::draw(frame, area, &ctx);
            return;
        }
        let split = self.dock_split();
        let screen = split.map_or(area, |(_, rest)| rest);
        self.menu
            .set_other_pane_focused(split.is_some() && self.dock.focused);
        match self.screen {
            Screen::MainMenu => self.menu.draw(frame, screen, &ctx),
            Screen::GameSelect => self.select.draw(frame, screen, &ctx),
            Screen::Board => self.board.draw(frame, screen, &ctx),
            Screen::Settings => self.settings_screen.draw(frame, screen, &ctx),
            Screen::Stats => self.stats.draw(frame, screen, &ctx),
        }
        if let Some(modal) = &self.modal {
            match split {
                Some((rect, _)) => {
                    overlay::dock::dock_view::draw(
                        frame,
                        rect,
                        modal,
                        &self.dock,
                        self.settings.help_dock,
                        &ctx,
                    );
                    self.modal_footer = FooterMap::default();
                }
                None => {
                    self.modal_footer = overlay::draw(frame, area, modal, &self.board, &ctx);
                    overlay::dock::dock_view::draw_placement(
                        frame,
                        area,
                        modal,
                        self.settings.help_dock,
                        &ctx,
                    );
                }
            }
        }
    }
}
