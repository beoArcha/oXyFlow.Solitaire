//! The settings screen: rows in groups under their headings. Keys and mouse change the focused row;
//! the result leaves as [`Effect::Apply`] and `App` applies and saves it. Headings are not focusable,
//! so the focus moves over the rows only.

mod arrows;
mod list;
pub(crate) mod options;
mod sample;
#[cfg(test)]
mod tests;
mod values;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use self::arrows::{Arrows, glyphs, locate};
use self::list::{Entry, entries, first_shown};
use self::options::{FIELDS, Field, Step};
use self::values::{changed, line, unavailable_reason_key};
use super::focus::{marker, step, tab_forward};
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::{CardFontState, Screen};
use crate::i18n::{t, tf};
use crate::mouse::{contains, left_press, wheel_action};
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, KeyHint};

pub(crate) const FOOTER: [KeyHint; 3] = [
    (Action::Select, "settings.footer.change"),
    (Action::Back, "help.action.back"),
    (Action::Help, "help.action.help"),
];

const HEADING_INDENT: &str = "  ";

/// The rows of the screen that are not the list: title, two gaps, sample, description, note, footer.
const CHROME_ROWS: u16 = 7;

/// State of the settings screen: the focused row and where each row was drawn last.
#[derive(Debug, Default)]
pub struct SettingsScreen {
    focus: usize,
    first_shown: usize,
    previewing: bool,
    rows: Vec<(usize, Rect)>,
    arrows: Option<Arrows>,
    footer: FooterMap,
}

fn change_row(index: usize, step: Step, ctx: &ViewCtx<'_>) -> Effect {
    let Some(field) = FIELDS.get(index).copied() else {
        return Effect::None;
    };
    if field.is_audio() && !ctx.audio_status.is_ready() {
        return Effect::None;
    }
    if field == Field::ConsoleFont && ctx.font.card_font == CardFontState::Unavailable {
        return Effect::None;
    }
    let settings = changed(field, ctx.settings, step, ctx.sound_styles);
    if field.is_interface() {
        Effect::Preview(settings)
    } else {
        Effect::Apply(settings)
    }
}

fn draw_reason(frame: &mut Frame, area: Rect, previewing: bool, ctx: &ViewCtx<'_>) {
    let key = if previewing {
        "settings.preview.note"
    } else if let Some(key) = unavailable_reason_key(ctx.audio_status) {
        key
    } else {
        return;
    };
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    let text = Line::styled(t(key).into_owned(), style);
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
}

fn text_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

impl SettingsScreen {
    /// A settings screen focused on its first row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells the screen whether interface changes wait for confirmation (the note under the list).
    pub(crate) fn set_previewing(&mut self, previewing: bool) {
        self.previewing = previewing;
    }

    /// Moves the focus with Up, Down, Tab and Shift+Tab (wrapping at both ends). Returns the row
    /// when Enter activates it.
    fn move_focus(&mut self, action: Action) -> Option<usize> {
        if action == Action::Select {
            return Some(self.focus);
        }
        let forward = match action {
            Action::Down => true,
            Action::Up => false,
            other => tab_forward(other)?,
        };
        self.focus = step(self.focus.min(FIELDS.len() - 1), FIELDS.len(), forward);
        None
    }

    /// Handles a key press mapped to `action`. Tab and Shift+Tab (like Up and Down) move the focus
    /// between the rows, Left and right change the focused row, Enter changes it forward, Esc leads
    /// back to the main menu.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            Some(Action::Left) => change_row(self.focus, Step::Previous, ctx),
            Some(Action::Right) => change_row(self.focus, Step::Next, ctx),
            Some(action) => self
                .move_focus(action)
                .map_or(Effect::None, |index| change_row(index, Step::Next, ctx)),
            None => Effect::None,
        }
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }

    /// Handles a mouse event: the wheel moves focus, a click on an arrow of the focused row steps its
    /// value back or forward, and a click on any other part of a row steps it forward.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        if let Some(action) = wheel_action(event) {
            self.move_focus(action);
            return Effect::None;
        }
        let Some(cell) = left_press(event) else {
            return Effect::None;
        };
        if let Some(hit) = self.arrows {
            if contains(hit.left, cell) {
                return self.step_row(hit.row, Step::Previous, ctx);
            }
            if contains(hit.right, cell) {
                return self.step_row(hit.row, Step::Next, ctx);
            }
        }
        match self.rows.iter().find(|(_, area)| contains(*area, cell)) {
            Some((index, _)) => self.step_row(*index, Step::Next, ctx),
            None => Effect::None,
        }
    }

    fn step_row(&mut self, index: usize, step: Step, ctx: &ViewCtx<'_>) -> Effect {
        self.focus = index;
        change_row(index, step, ctx)
    }

    /// Draws the title, the groups and their rows (scrolled when the screen is too low for all of
    /// them), the aura sample, the explanation of the focused row, the note about a preview or the
    /// reason why sound is unavailable, and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let entries = entries();
        let room = usize::from(area.height.saturating_sub(CHROME_ROWS));
        let shown = entries.len().min(room);
        let [
            _,
            title_area,
            _,
            list_area,
            _,
            sample_area,
            description_area,
            reason_area,
            _,
            footer_area,
        ] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(shown).unwrap_or(0)),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let scrolled = shown < entries.len();
        self.first_shown = first_shown(&entries, self.focus, shown, self.first_shown);
        let mut heading = t("common.settings.title").into_owned();
        if scrolled {
            let position = (self.focus + 1).to_string();
            let total = FIELDS.len().to_string();
            let place = tf(
                "settings.preview.position",
                &[("row", &position), ("total", &total)],
            );
            heading = format!("{heading}  ({place})");
        }
        let title = Line::styled(heading, ctx.theme.style(Token::Accent, Token::Background));
        frame.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            title_area,
        );
        let visible = entries.get(self.first_shown..).unwrap_or_default();
        self.draw_list(frame, list_area, visible, area.width, ctx);
        let sample = Paragraph::new(sample::line(ctx)).alignment(Alignment::Center);
        frame.render_widget(sample, sample_area);
        self.draw_description(frame, description_area, ctx);
        draw_reason(frame, reason_area, self.previewing, ctx);
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(&FOOTER),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }

    fn draw_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        entries: &[Entry],
        columns: u16,
        ctx: &ViewCtx<'_>,
    ) {
        let width = entries
            .iter()
            .map(|entry| text_width(&self.entry_text(*entry, columns, ctx)))
            .max()
            .unwrap_or_default()
            .min(area.width);
        let x = area.x + (area.width - width) / 2;
        self.rows.clear();
        self.arrows = None;
        for (offset, entry) in entries.iter().copied().enumerate() {
            let Ok(dy) = u16::try_from(offset) else {
                break;
            };
            if dy >= area.height {
                break;
            }
            let text = self.entry_text(entry, columns, ctx);
            let cell = Rect::new(x, area.y + dy, width, 1);
            let style = match entry {
                Entry::Heading(_) => ctx.theme.style(Token::Accent, Token::Background),
                Entry::Row(index) if index == self.focus => ctx.theme.focus(),
                Entry::Row(_) | Entry::Gap => ctx.theme.base(),
            };
            if let Entry::Row(index) = entry {
                if index == self.focus {
                    self.arrows = locate(index, &text, cell, glyphs(ctx));
                }
                self.rows.push((index, cell));
            }
            frame.render_widget(Paragraph::new(Line::styled(text, style)), cell);
        }
    }

    fn entry_text(&self, entry: Entry, columns: u16, ctx: &ViewCtx<'_>) -> String {
        match entry {
            Entry::Heading(group) => format!("{HEADING_INDENT}{}", t(group.label_key())),
            Entry::Gap => String::new(),
            Entry::Row(index) => {
                let focused = index == self.focus;
                let text = FIELDS.get(index).map_or_else(String::new, |field| {
                    line(
                        *field,
                        ctx.settings,
                        ctx.audio_status.is_ready(),
                        ctx.sound_styles,
                        (columns, ctx.font),
                        focused.then(|| glyphs(ctx)),
                    )
                });
                format!("{} {text}", marker(ctx, focused))
            }
        }
    }

    fn draw_description(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let text = FIELDS.get(self.focus).map_or_else(
            || t("settings.desc.language").into_owned(),
            |field| field.describe(ctx.font),
        );
        let style = ctx.theme.style(Token::TextMuted, Token::Background);
        frame.render_widget(
            Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center),
            area,
        );
    }
}
