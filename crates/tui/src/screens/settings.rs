//! The settings screen: rows in groups under their headings. Keys and mouse change the focused row;
//! the result leaves as [`Effect::Apply`] and `App` applies and saves it. Headings are not focusable,
//! so the focus moves over the rows only.

mod arrows;
mod font_rows;
mod list;
pub(crate) mod options;
mod plan;
mod preview;
mod priority_row;
#[cfg(test)]
mod tests;
mod values;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use self::arrows::{Arrows, glyphs, locate};
use self::list::{Entry, entries, first_shown};
use self::options::{FIELDS, Field, Step};
use self::plan::{Pane, Places, plan};
use self::values::{changed, is_unavailable, line, unavailable_reason_key};
use super::focus::{marker, step, tab_forward};
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::mouse::{contains, left_press, wheel_action};
use crate::theme::Token;
use crate::theme::table::{felt_for, hex};
use crate::widgets::footer::{self, FooterMap, KeyHint, Segment};

pub(crate) const FOOTER: [KeyHint; 3] = [
    (Action::Select, "settings.footer.change"),
    (Action::Back, "help.action.back"),
    (Action::Help, "help.action.help"),
];

const HEADING_INDENT: &str = "  ";

/// The rows of the screen that are not the list: title, two gaps, the note about a hidden preview, two
/// lines of description, the reason line, footer.
const CHROME_ROWS: u16 = 8;

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

fn is_unavailable_row(index: usize, ctx: &ViewCtx<'_>) -> bool {
    FIELDS
        .get(index)
        .is_some_and(|field| is_unavailable(*field, ctx.audio_status.is_ready(), ctx.font))
}

fn change_row(index: usize, step: Step, ctx: &ViewCtx<'_>) -> Effect {
    let Some(field) = FIELDS.get(index).copied() else {
        return Effect::None;
    };
    if is_unavailable_row(index, ctx) {
        return Effect::None;
    }
    let settings = changed(field, ctx.settings, step, ctx.sound_styles);
    if field.is_interface() {
        Effect::Preview(settings)
    } else {
        Effect::Apply(settings)
    }
}

fn draw_reason(
    frame: &mut Frame,
    area: Rect,
    previewing: bool,
    focus: Option<Field>,
    ctx: &ViewCtx<'_>,
) {
    let text = if previewing {
        t("settings.preview.note").into_owned()
    } else if let Some(note) = table_note(focus, ctx) {
        note
    } else if let Some(key) = unavailable_reason_key(ctx.audio_status) {
        t(key).into_owned()
    } else {
        return;
    };
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    let line = Line::styled(text, style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

/// The warning under the table colour row when its colour fails the contrast tests on the table:
/// it names the nearest passing shade the table is drawn in.
fn table_note(focus: Option<Field>, ctx: &ViewCtx<'_>) -> Option<String> {
    if focus != Some(Field::TableColour) {
        return None;
    }
    let felt = felt_for(ctx.settings.theme, ctx.settings.table_colour);
    felt.shifted
        .then(|| tf("settings.table.reason", &[("shade", &hex(felt.swatch.rgb))]))
}

/// The footer: the arrows that change the focused row (a text hint, since one button holds one key),
/// then the buttons of [`FOOTER`].
fn footer_segments(ctx: &ViewCtx<'_>) -> Vec<Segment> {
    let (left, right) = glyphs(ctx);
    let arrows = tf(
        "settings.footer.arrows",
        &[("arrows", &format!("{left} {right}"))],
    );
    let mut segments = vec![Segment::plain(arrows), Segment::plain("   ")];
    segments.extend(footer::key_segments(&FOOTER));
    segments
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
    /// them), the preview pane beside or under the list (or the note that the window is too small
    /// for it), the explanation of the focused row, the note about a preview or the reason why sound
    /// is unavailable, and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let entries = entries();
        let list_width = self.list_width(&entries, area.width, ctx);
        let body_rows = area.height.saturating_sub(CHROME_ROWS);
        let plan = plan(area.width, body_rows, list_width, entries.len());
        let below_rows = match plan.pane {
            Pane::Below { rows, .. } => rows,
            Pane::Side { .. } | Pane::Hidden => 0,
        };
        let shown = usize::from(plan.list_rows);
        let [
            _,
            title_area,
            _,
            list_area,
            _,
            below_area,
            note_area,
            description_area,
            reason_area,
            _,
            footer_area,
        ] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(plan.list_rows),
            Constraint::Length(1),
            Constraint::Length(below_rows),
            Constraint::Length(1),
            Constraint::Length(2),
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
        let places = Places::new(plan.pane, list_width, (list_area, below_area));
        self.draw_list(frame, places.list, visible, area.width, ctx);
        match places.pane {
            Some(pane) => preview::draw(frame, pane, ctx),
            None => preview::draw_hidden_note(frame, note_area, plan.pane, ctx),
        }
        self.draw_description(frame, description_area, ctx);
        draw_reason(
            frame,
            reason_area,
            self.previewing,
            FIELDS.get(self.focus).copied(),
            ctx,
        );
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer_segments(ctx),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }

    /// The columns of the widest line of the list, never more than `columns`.
    fn list_width(&self, entries: &[Entry], columns: u16, ctx: &ViewCtx<'_>) -> u16 {
        entries
            .iter()
            .map(|entry| text_width(&self.entry_text(*entry, columns, ctx)))
            .max()
            .unwrap_or_default()
            .min(columns)
    }

    fn draw_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        entries: &[Entry],
        columns: u16,
        ctx: &ViewCtx<'_>,
    ) {
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
            let cell = Rect::new(area.x, area.y + dy, area.width, 1);
            let style = match entry {
                Entry::Heading(_) => ctx.theme.style(Token::Accent, Token::Background),
                Entry::Row(index) if index == self.focus => ctx.theme.focus(),
                Entry::Row(index) if is_unavailable_row(index, ctx) => {
                    ctx.theme.style(Token::TextMuted, Token::Background)
                }
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
            Paragraph::new(Line::styled(text, style))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}
