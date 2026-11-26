//! The typing page of the seed screen (after a game is chosen): a field that takes decimal digits,
//! the validation of what was typed, and the two buttons that submit or leave it. Tab visits the
//! field, Continue and Back in that order and wraps. Typed digits go to the field only while the
//! field has the focus.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::i18n::t;
use crate::mouse::{contains, left_press};
use crate::screens::ViewCtx;
use crate::screens::focus::{button_style, marker, step, tab_forward};
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

/// An element of the seed stage that can have the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Field,
    Button(Button),
}

/// The focus ring of the stage, in the order Tab visits it.
const STOPS: [Stop; 3] = [
    Stop::Field,
    Stop::Button(Button::Continue),
    Stop::Button(Button::Back),
];
const CONTINUE: usize = 1;
const BACK: usize = 2;

/// The seed text, the message about the last rejected input, the focus (an index into [`STOPS`])
/// and the button rectangles of the last frame.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SeedField {
    text: String,
    error: Option<SeedError>,
    focus: usize,
    buttons: Vec<(Button, Rect)>,
}

impl SeedField {
    /// Whether the field has the focus, so that typed characters are its text.
    pub fn captures_text(&self) -> bool {
        self.stop() == Stop::Field
    }

    #[cfg(feature = "dev")]
    pub(super) fn dev_focus(&self) -> (&'static str, Option<&'static str>) {
        let target = match self.stop() {
            Stop::Field => "field",
            Stop::Button(Button::Continue) => "continue",
            Stop::Button(Button::Back) => "back",
        };
        (target, self.error.map(SeedError::message_key))
    }

    fn stop(&self) -> Stop {
        STOPS.get(self.focus).copied().unwrap_or(Stop::Field)
    }

    /// Handles a key press. Tab and Shift+Tab move the focus. With the field focused, characters
    /// are typed, Backspace deletes and Enter submits; with a button focused, Enter or Space presses
    /// it and Left and Right switch between the two buttons.
    pub fn handle_key(&mut self, key: &KeyEvent, action: Option<Action>) -> Step {
        if let Some(forward) = action.and_then(tab_forward) {
            self.focus = step(self.focus, STOPS.len(), forward);
            return Step::Stay;
        }
        match (self.stop(), action) {
            (Stop::Field, _) => self.type_key(key),
            (Stop::Button(button), Some(Action::Select)) => self.press(button),
            (Stop::Button(_), Some(Action::Left | Action::Right)) => {
                self.focus = if self.focus == CONTINUE {
                    BACK
                } else {
                    CONTINUE
                };
                Step::Stay
            }
            (Stop::Button(_), _) => Step::Stay,
        }
    }

    fn type_key(&mut self, key: &KeyEvent) -> Step {
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

    /// Handles a click on one of the buttons: the button takes the focus and is pressed.
    pub fn handle_mouse(&mut self, event: &MouseEvent) -> Step {
        let Some(cell) = left_press(event) else {
            return Step::Stay;
        };
        let Some(button) = self
            .buttons
            .iter()
            .find(|(_, area)| contains(*area, cell))
            .map(|(button, _)| *button)
        else {
            return Step::Stay;
        };
        if let Some(index) = STOPS.iter().position(|stop| *stop == Stop::Button(button)) {
            self.focus = index;
        }
        self.press(button)
    }

    fn press(&mut self, button: Button) -> Step {
        match button {
            Button::Continue => self.submit(),
            Button::Back => Step::Back,
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

    /// Draws the title (the game and the seed prompt), the field, the message, the buttons and the
    /// key line.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>, title: &str) {
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
        draw_line(frame, title_area, title, accent);
        draw_line(frame, prompt_area, &t("menu.seed.prompt"), ctx.theme.base());
        self.draw_field(frame, field_area, ctx);
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

    fn draw_field(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let focused = self.stop() == Stop::Field;
        let caret = if focused { "_" } else { "" };
        let shown = format!("{}{caret}", self.text);
        let field = format!("{} [ {shown:<FIELD_WIDTH$}]", marker(ctx, focused));
        let style = if focused {
            ctx.theme.focus()
        } else {
            ctx.theme.base()
        };
        draw_line(frame, area, &field, style);
    }

    fn draw_buttons(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let labels = [
            (Button::Continue, t("menu.seed.continue")),
            (Button::Back, t("menu.seed.back")),
        ];
        let texts: Vec<(Button, String)> = labels
            .iter()
            .map(|(button, label)| {
                let focused = self.stop() == Stop::Button(*button);
                (*button, format!("{} [ {label} ]", marker(ctx, focused)))
            })
            .collect();
        let widths: Vec<u16> = texts
            .iter()
            .map(|(_, text)| u16::try_from(text.chars().count()).unwrap_or(u16::MAX))
            .collect();
        let total = widths.iter().sum::<u16>() + BUTTON_GAP;
        let mut x = area.x + area.width.saturating_sub(total) / 2;
        self.buttons.clear();
        for ((button, _), width) in texts.iter().zip(&widths) {
            self.buttons
                .push((*button, Rect::new(x, area.y, *width, 1)));
            x = x.saturating_add(width + BUTTON_GAP);
        }
        let spans: Vec<Span> = texts
            .iter()
            .enumerate()
            .flat_map(|(index, (button, text))| {
                let gap = if index == 0 { "" } else { "   " };
                let focused = self.stop() == Stop::Button(*button);
                [
                    Span::raw(gap),
                    Span::styled(text.clone(), button_style(ctx, focused)),
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
