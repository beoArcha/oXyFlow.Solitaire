//! The seed entry stage of the main menu: a field that takes decimal digits, the validation of
//! what was typed, and the two buttons that submit or leave it.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::i18n::t;
use crate::mouse::{contains, left_press};
use crate::screens::ViewCtx;
use crate::theme::Token;

const FIELD_WIDTH: usize = 22;
const MAX_LENGTH: usize = 32;
const BUTTON_GAP: u16 = 3;

/// Why typed text is not a seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedError {
    Empty,
    NotNumeric,
    OutOfRange,
}

impl SeedError {
    /// Localization key of the message shown under the field.
    pub const fn message_key(self) -> &'static str {
        match self {
            SeedError::Empty => "menu.seed.error_empty",
            SeedError::NotNumeric => "menu.seed.error_not_numeric",
            SeedError::OutOfRange => "menu.seed.error_out_of_range",
        }
    }
}

/// Parses typed text as a decimal `u64`.
pub fn parse_seed(text: &str) -> Result<u64, SeedError> {
    if text.is_empty() {
        return Err(SeedError::Empty);
    }
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SeedError::NotNumeric);
    }
    text.parse::<u64>().map_err(|_| SeedError::OutOfRange)
}

/// What the seed stage asks the menu to do after an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Stay,
    Back,
    Submit(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Continue,
    Back,
}

/// The seed text, the message about the last rejected input, and the button rectangles of the
/// last frame.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SeedField {
    text: String,
    error: Option<SeedError>,
    buttons: Vec<(Button, Rect)>,
}

impl SeedField {
    /// A field that already holds `seed`, used when coming back from the game selection.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            text: seed.to_string(),
            ..Self::default()
        }
    }

    /// Handles a key press: characters are typed, Backspace deletes, Enter submits and validates.
    pub fn handle_key(&mut self, key: &KeyEvent) -> Step {
        match key.code {
            KeyCode::Enter => self.submit(),
            KeyCode::Backspace => {
                self.text.pop();
                self.error = None;
                Step::Stay
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.text.chars().count() < MAX_LENGTH {
                    self.text.push(c);
                }
                self.error = None;
                Step::Stay
            }
            _ => Step::Stay,
        }
    }

    /// Handles a click on one of the buttons.
    pub fn handle_mouse(&mut self, event: &MouseEvent) -> Step {
        let Some(cell) = left_press(event) else {
            return Step::Stay;
        };
        match self
            .buttons
            .iter()
            .find(|(_, area)| contains(*area, cell))
            .map(|(button, _)| *button)
        {
            Some(Button::Continue) => self.submit(),
            Some(Button::Back) => Step::Back,
            None => Step::Stay,
        }
    }

    fn submit(&mut self) -> Step {
        match parse_seed(&self.text) {
            Ok(seed) => Step::Submit(seed),
            Err(error) => {
                self.error = Some(error);
                Step::Stay
            }
        }
    }

    /// Draws the title, the field, the message, the buttons and the key line.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [
            _,
            title_area,
            _,
            prompt_area,
            field_area,
            error_area,
            _,
            buttons_area,
            _,
            keys_area,
        ] = Layout::vertical([
            Constraint::Fill(1),
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
        let accent = ctx.theme.style(Token::Accent, Token::Background);
        draw_line(frame, title_area, &t("menu.seed.title"), accent);
        draw_line(frame, prompt_area, &t("menu.seed.prompt"), ctx.theme.base());
        let field = format!("[ {:<FIELD_WIDTH$}]", format!("{}_", self.text));
        draw_line(frame, field_area, &field, ctx.theme.focus());
        let message = self.error.map(|error| t(error.message_key()).into_owned());
        draw_line(
            frame,
            error_area,
            message.as_deref().unwrap_or(""),
            ctx.theme.style(Token::Error, Token::Background),
        );
        self.draw_buttons(frame, buttons_area, ctx);
        let keys = t("menu.seed.keys").into_owned();
        draw_line(
            frame,
            keys_area,
            &keys,
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
    }

    fn draw_buttons(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let labels = [
            (Button::Continue, format!("[ {} ]", t("menu.seed.continue"))),
            (Button::Back, format!("[ {} ]", t("menu.seed.back"))),
        ];
        let widths: Vec<u16> = labels
            .iter()
            .map(|(_, label)| u16::try_from(label.chars().count()).unwrap_or(u16::MAX))
            .collect();
        let total = widths.iter().sum::<u16>() + BUTTON_GAP;
        let mut x = area.x + area.width.saturating_sub(total) / 2;
        self.buttons.clear();
        for ((button, _), width) in labels.iter().zip(&widths) {
            self.buttons
                .push((*button, Rect::new(x, area.y, *width, 1)));
            x = x.saturating_add(width + BUTTON_GAP);
        }
        let spans: Vec<Span> = labels
            .iter()
            .enumerate()
            .flat_map(|(index, (_, label))| {
                let gap = if index == 0 { "" } else { "   " };
                [
                    Span::raw(gap),
                    Span::styled(label.clone(), ctx.theme.focus()),
                ]
            })
            .collect();
        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
    }
}

fn draw_line(frame: &mut Frame, area: Rect, text: &str, style: Style) {
    let line = Line::styled(text.to_owned(), style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}
