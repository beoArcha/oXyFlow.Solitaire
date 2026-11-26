//! Drawing of the game selection screen: the title, the search field, the grouped variants, the
//! two toggles and the key hints.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::list::Content;
use super::rows::{self, FocusItem, Row};
use super::{
    CHROME_ROWS, GameSelect, LIST, Purpose, TOGGLE, WINNABLE, WINNABLE_KEY, hints, label_of,
    search, toggles,
};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::{KeySpec, primary_key};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::{self, Segment};
use oxysol_core::Variant;

fn key_button(action: Action, label_key: &str) -> Option<Segment> {
    let key = primary_key(action)?;
    Some(Segment::button(action, format!("{key} {}", t(label_key))))
}

fn push_segment(segments: &mut Vec<Segment>, segment: Segment) {
    if !segments.is_empty() {
        segments.push(Segment::plain("   "));
    }
    segments.push(segment);
}

fn focused_heading_is_folded(rows: &[Row], focused: Option<FocusItem>) -> bool {
    let Some(FocusItem::Heading(target)) = focused else {
        return false;
    };
    rows.iter().any(|row| {
        matches!(
            row,
            Row::Heading {
                heading,
                folded: true,
                ..
            } if *heading == target
        )
    })
}

fn select_label_key(
    purpose: Purpose,
    focus: usize,
    rows: &[Row],
    focused: Option<FocusItem>,
    filtering: bool,
) -> Option<&'static str> {
    if focus != LIST {
        return Some("menu.footer.change");
    }
    if rows.is_empty() {
        return None;
    }
    match focused {
        Some(FocusItem::Variant(_)) => Some(match purpose {
            Purpose::Play => "menu.footer.play",
            Purpose::BeforeSeed => "menu.footer.seed_options",
        }),
        Some(FocusItem::Heading(_)) if filtering => None,
        Some(FocusItem::Heading(_)) => Some(if focused_heading_is_folded(rows, focused) {
            "menu.footer.unfold"
        } else {
            "menu.footer.fold"
        }),
        None => None,
    }
}

impl GameSelect {
    fn title(&self) -> String {
        match (self.seed, self.purpose) {
            (Some(seed), _) => tf("menu.select_seed", &[("seed", &seed.to_string())]),
            (None, Purpose::BeforeSeed) => t("menu.seed.choose_game").into_owned(),
            (None, Purpose::Play) => t("common.select.title").into_owned(),
        }
    }

    fn hint_line(&self, rows: &[Row], ctx: &ViewCtx<'_>) -> String {
        if self.focus != LIST {
            return String::new();
        }
        match self.list.focused_item(rows, &ctx.settings.favourites) {
            Some(FocusItem::Variant(variant)) => hints::hint_text(variant),
            Some(FocusItem::Heading(_)) if !self.captures_text() => {
                t("menu.heading_hint").into_owned()
            }
            _ => String::new(),
        }
    }

    fn footer_segments(&self, rows: &[Row], filtering: bool, ctx: &ViewCtx<'_>) -> Vec<Segment> {
        let focused = self.list.focused_item(rows, &ctx.settings.favourites);
        let mut segments = Vec::new();
        if let Some(label_key) =
            select_label_key(self.purpose, self.focus, rows, focused, filtering)
            && let Some(button) = key_button(Action::Select, label_key)
        {
            push_segment(&mut segments, button);
        }
        let back_key = if self.searching() {
            "menu.footer.clear"
        } else {
            "help.action.back"
        };
        if let Some(button) = key_button(Action::Back, back_key) {
            push_segment(&mut segments, button);
        }
        if self.captures_text() {
            push_segment(
                &mut segments,
                Segment::plain(format!("F1 {}", t("help.action.help"))),
            );
            return segments;
        }
        if let Some(button) = key_button(Action::Help, "help.action.help") {
            push_segment(&mut segments, button);
        }
        if self.focus == LIST
            && matches!(focused, Some(FocusItem::Variant(_)))
            && let Some(button) = key_button(Action::Finish, "menu.footer.favorite")
        {
            push_segment(&mut segments, button);
        }
        if let Some(button) = key_button(Action::Undo, "help.action.undo") {
            push_segment(&mut segments, button);
        }
        push_segment(
            &mut segments,
            Segment::plain(format!("/ {}", t("menu.search.label"))),
        );
        if self.offers_winnable() {
            let winnable = format!(
                "{} {}",
                KeySpec::ch(WINNABLE_KEY).display(),
                t("menu.footer.winnable")
            );
            push_segment(&mut segments, Segment::plain(winnable));
        }
        segments
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect, rows: &[Row], ctx: &ViewCtx<'_>) {
        if rows.is_empty() {
            search::draw_no_match(frame, area, self.query(), ctx);
            return;
        }
        let winnable = self.offers_winnable() && ctx.settings.winnable_only;
        let label = |variant: Variant| label_of(variant, winnable);
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
        self.folded = ctx.settings.folded_families.clone();
        let filtering = !self.query().trim().is_empty();
        let rows = rows::build(
            &ctx.settings.favourites,
            &ctx.settings.folded_families,
            self.query(),
        );
        let room = area.height.saturating_sub(CHROME_ROWS).max(1);
        let list_height = u16::try_from(rows.len()).unwrap_or(u16::MAX).clamp(1, room);
        let [
            _,
            title_area,
            _,
            search_area,
            legend_area,
            list_area,
            _,
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
            self.hint_line(&rows, ctx),
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
        frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);
        let offers_winnable = self.offers_winnable();
        self.toggle = toggles::draw_undo(frame, undo_area, self.focus == TOGGLE, ctx);
        self.winnable = if offers_winnable {
            toggles::draw_winnable(frame, winnable_area, self.focus == WINNABLE, ctx)
        } else {
            Rect::default()
        };
        let winnable_hint = offers_winnable.then_some(winnable_hint_area);
        toggles::draw_hints(frame, undo_hint_area, winnable_hint, ctx);
        let segments = self.footer_segments(&rows, filtering, ctx);
        self.footer = footer::draw(
            frame,
            footer_area,
            &segments,
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }
}
