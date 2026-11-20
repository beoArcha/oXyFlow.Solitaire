//! Drawing of the game selection screen: the title, the search field, the grouped variants, the
//! two toggles and the key hints.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::list::Content;
use super::rows::{self, Row, items};
use super::{
    CHROME_ROWS, FOOTER, GameSelect, LIST, TOGGLE, WINNABLE, hints, label_of, search, toggles,
};
use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer;
use oxysol_core::Variant;

impl GameSelect {
    fn title(&self) -> String {
        match self.seed {
            Some(seed) => tf("menu.select_seed", &[("seed", &seed.to_string())]),
            None => t("common.select.title").into_owned(),
        }
    }

    fn hint_line(&self, variants: &[Variant]) -> String {
        variants
            .get(self.list.focus())
            .map_or_else(String::new, |variant| hints::hint_text(*variant))
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect, rows: &[Row], ctx: &ViewCtx<'_>) {
        if rows.is_empty() {
            search::draw_no_match(frame, area, self.query(), ctx);
            return;
        }
        let label = |variant: Variant| label_of(variant, ctx);
        let content = Content {
            rows,
            favourites: &ctx.settings.favourites,
            label_of: &label,
        };
        self.list
            .draw(frame, area, &content, self.focus == LIST, ctx);
    }

    /// Draws the title, the search field, the grouped variants, the two toggles and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let rows = rows::build(&ctx.settings.favourites, self.query());
        let variants = items(&rows);
        let room = area.height.saturating_sub(CHROME_ROWS).max(1);
        let list_height = u16::try_from(rows.len()).unwrap_or(u16::MAX).clamp(1, room);
        let [
            _,
            title_area,
            _,
            search_area,
            legend_area,
            list_area,
            hint_area,
            _,
            undo_area,
            winnable_area,
            undo_hint_area,
            winnable_hint_area,
            _,
            footer_area,
        ] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(list_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        let title = Line::styled(
            self.title(),
            ctx.theme.style(Token::Accent, Token::Background),
        );
        frame.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            title_area,
        );
        self.search_area = search::draw_field(frame, search_area, self.search.as_deref(), ctx);
        search::draw_legend(frame, legend_area, ctx);
        self.draw_list(frame, list_area, &rows, ctx);
        let hint = Line::styled(
            self.hint_line(&variants),
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
        frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);
        self.toggle = toggles::draw_undo(frame, undo_area, self.focus == TOGGLE, ctx);
        self.winnable = toggles::draw_winnable(frame, winnable_area, self.focus == WINNABLE, ctx);
        toggles::draw_hints(frame, [undo_hint_area, winnable_hint_area], ctx);
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(&FOOTER),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }
}
