//! Drawing of the seed screen: the title, the prompt, the page's list, the message under it and the
//! key line.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::{CHOICES, LIST_FOOTER, OPTIONS_FOOTER, Page, SeedScreen};
use crate::i18n::{t, tf};
use crate::screens::main_menu::seed_levels;
use crate::screens::{ViewCtx, variant_label_key};
use crate::theme::Token;
use crate::widgets::footer::{self, KeyHint};
use crate::widgets::menu_list::{Highlight, MenuList, Row};

/// The title, the prompt and the message above and below a list.
struct Heading<'a> {
    title: &'a str,
    prompt: &'a str,
    notice: &'a str,
}

impl SeedScreen {
    fn title(&self) -> String {
        let game = t(variant_label_key(self.variant));
        tf("menu.seed.title_game", &[("game", game.as_ref())])
    }

    /// Draws the current page.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let title = self.title();
        let notice = self
            .notice
            .map(|key| t(key).into_owned())
            .unwrap_or_default();
        match &mut self.page {
            Page::Typing(field) => field.draw(frame, area, ctx, &title),
            Page::Options => {
                let rows: Vec<Row> = CHOICES
                    .iter()
                    .map(|choice| Row::new(t(choice.label_key()).into_owned()))
                    .collect();
                let prompt = t("menu.seed.options_prompt").into_owned();
                let heading = Heading {
                    title: &title,
                    prompt: &prompt,
                    notice: &notice,
                };
                self.draw_list(frame, area, ctx, &heading, &rows, &OPTIONS_FOOTER);
            }
            Page::Played => {
                let rows = self.played.rows();
                let prompt = t("menu.seed.played_prompt").into_owned();
                let heading = Heading {
                    title: &title,
                    prompt: &prompt,
                    notice: &notice,
                };
                self.draw_list(frame, area, ctx, &heading, &rows, &LIST_FOOTER);
            }
            Page::Levels(entries) => {
                let rows = seed_levels::rows(entries);
                let prompt = t("menu.seed.levels_prompt").into_owned();
                let heading = Heading {
                    title: &title,
                    prompt: &prompt,
                    notice: &notice,
                };
                self.draw_list(frame, area, ctx, &heading, &rows, &LIST_FOOTER);
            }
        }
    }

    fn draw_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ctx: &ViewCtx<'_>,
        heading: &Heading<'_>,
        rows: &[Row],
        keys: &[KeyHint],
    ) {
        let list_height = MenuList::height(rows).max(1);
        let [
            _,
            title_area,
            _,
            prompt_area,
            _,
            list_area,
            _,
            notice_area,
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
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        draw_line(
            frame,
            title_area,
            heading.title,
            ctx.theme.style(Token::Accent, Token::Background),
        );
        draw_line(frame, prompt_area, heading.prompt, ctx.theme.base());
        if rows.is_empty() {
            draw_line(
                frame,
                list_area,
                &t("menu.seed.levels_none"),
                ctx.theme.style(Token::TextMuted, Token::Background),
            );
        } else {
            self.list
                .draw(frame, list_area, rows, ctx, Highlight::Focused);
        }
        draw_line(
            frame,
            notice_area,
            heading.notice,
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(keys),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }
}

fn draw_line(frame: &mut Frame, area: Rect, text: &str, style: Style) {
    let line = Line::styled(text.to_owned(), style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}
