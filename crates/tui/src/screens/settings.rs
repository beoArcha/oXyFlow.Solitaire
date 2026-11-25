//! The settings screen: three tabs (`Audio & Visual`, `Popular` by default, `Gameplay`), a live
//! preview pane, and key/mouse handling. Changes leave as [`Effect::Preview`] for interface rows
//! (kept only after confirmation) or [`Effect::Apply`] for all other rows.

mod arrows;
mod change;
mod font_rows;
mod list;
pub(crate) mod options;
mod plan;
mod preview;
mod priority_row;
pub(crate) mod tabs;
#[cfg(test)]
mod tests;
mod values;
mod view;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};

use self::arrows::Arrows;
use self::list::first_shown;
use self::options::{Field, Step};
use self::plan::{Pane, Places, plan};
use self::tabs::{Tab, TabBar, draw_tabs};
use self::values::changed;
use self::view::{
    CHROME_ROWS, draw_description, draw_list, draw_reason, draw_title, footer_segments,
    is_unavailable_field, list_width,
};
use super::focus::step;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::mouse::{contains, left_press, wheel_action};
use crate::widgets::footer::{self, FooterMap, KeyHint};

pub(crate) const FOOTER: [KeyHint; 3] = [
    (Action::Select, "settings.footer.change"),
    (Action::Back, "help.action.back"),
    (Action::Help, "help.action.help"),
];

/// State of the settings screen: active tab, per-tab focus and scroll, and last drawn hit boxes.
#[derive(Debug, Default)]
pub struct SettingsScreen {
    tab: Tab,
    focus: [usize; Tab::ALL.len()],
    first_shown: [usize; Tab::ALL.len()],
    previewing: bool,
    tab_bar: Option<TabBar>,
    rows: Vec<(usize, Rect)>,
    arrows: Option<Arrows>,
    footer: FooterMap,
}

fn change_field(field: Option<Field>, step: Step, ctx: &ViewCtx<'_>) -> Effect {
    let Some(field) = field else {
        return Effect::None;
    };
    if is_unavailable_field(field, ctx) {
        return Effect::None;
    }
    let settings = changed(field, ctx.settings, step, ctx.sound_styles);
    if field.is_interface() {
        Effect::Preview(settings)
    } else {
        Effect::Apply(settings)
    }
}

impl SettingsScreen {
    /// A settings screen opened on the `Popular` tab with its first row focused.
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells the screen whether interface changes wait for confirmation (the note under the list).
    pub(crate) fn set_previewing(&mut self, previewing: bool) {
        self.previewing = previewing;
    }

    fn active_fields(&self) -> &'static [Field] {
        self.tab.fields()
    }

    fn focus_index(&self) -> usize {
        let len = self.active_fields().len();
        self.focus[self.tab.index()].min(len.saturating_sub(1))
    }

    fn focused_field(&self) -> Option<Field> {
        self.active_fields().get(self.focus_index()).copied()
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.arrows = None;
    }

    fn move_focus(&mut self, forward: bool) {
        let len = self.active_fields().len();
        let current = self.focus_index();
        self.focus[self.tab.index()] = step(current, len, forward);
    }

    /// Handles a key press mapped to `action`. `Tab` and `Shift+Tab` (and `[` / `]`) switch tabs,
    /// `Up` and `Down` move focus between rows of the active tab, `Left` and `Right` change the
    /// focused row, `Enter` and `Space` step forward, and `Esc` returns to the main menu.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            Some(Action::Left) => change_field(self.focused_field(), Step::Previous, ctx),
            Some(Action::Right) | Some(Action::Select) => {
                change_field(self.focused_field(), Step::Next, ctx)
            }
            Some(Action::FocusNext) | Some(Action::ZoomIn) => {
                self.switch_tab(self.tab.next());
                Effect::None
            }
            Some(Action::FocusPrevious) | Some(Action::ZoomOut) => {
                self.switch_tab(self.tab.previous());
                Effect::None
            }
            Some(Action::Down) => {
                self.move_focus(true);
                Effect::None
            }
            Some(Action::Up) => {
                self.move_focus(false);
                Effect::None
            }
            _ => Effect::None,
        }
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }

    /// Handles a mouse event: wheel moves row focus, clicks on tab labels or arrows switch tabs,
    /// clicks on row arrows step back or forward, and clicks on a row step forward.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        if let Some(action) = wheel_action(event) {
            self.move_focus(action == Action::Down);
            return Effect::None;
        }
        let Some(cell) = left_press(event) else {
            return Effect::None;
        };
        if let Some(tab) = self.tab_hit(cell) {
            self.switch_tab(tab);
            return Effect::None;
        }
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

    fn tab_hit(&self, cell: (u16, u16)) -> Option<Tab> {
        let bar = self.tab_bar?;
        bar.arrow_target(self.tab, cell)
            .or_else(|| bar.tab_at(cell))
    }

    fn step_row(&mut self, index: usize, step: Step, ctx: &ViewCtx<'_>) -> Effect {
        self.focus[self.tab.index()] = index;
        let field = self.active_fields().get(index).copied();
        change_field(field, step, ctx)
    }

    /// Draws the title, the tab bar, the active tab's rows, the preview pane (or hidden note), the
    /// description, the status/preview note, and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let fields = self.active_fields();
        let list_width = list_width(fields, area.width, ctx);
        let body_rows = area.height.saturating_sub(CHROME_ROWS);
        let plan = plan(area.width, body_rows, list_width, fields.len());
        let (band_rows, below_rows) = match plan.pane {
            Pane::Side { rows, .. } => (plan.list_rows.max(rows), 0),
            Pane::Below { rows, .. } => (plan.list_rows, rows),
            Pane::Hidden => (plan.list_rows, 0),
        };
        let [
            _,
            title_area,
            _,
            tab_area,
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(band_rows),
            Constraint::Length(1),
            Constraint::Length(below_rows),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let focus = self.focus_index();
        let shown = usize::from(plan.list_rows);
        let tab_index = self.tab.index();
        self.first_shown[tab_index] =
            first_shown(fields.len(), focus, shown, self.first_shown[tab_index]);
        draw_title(
            frame,
            title_area,
            shown < fields.len(),
            focus,
            fields.len(),
            ctx,
        );
        self.tab_bar = Some(draw_tabs(frame, tab_area, self.tab, ctx));

        let places = Places::new(plan.pane, list_width, (list_area, below_area));
        let drawn = draw_list(
            frame,
            places.list,
            fields,
            focus,
            self.first_shown[tab_index],
            area.width,
            ctx,
        );
        self.rows = drawn.rows;
        self.arrows = drawn.arrows;

        match places.pane {
            Some(pane) => preview::draw(frame, pane, ctx),
            None => preview::draw_hidden_note(frame, note_area, plan.pane, ctx),
        }
        draw_description(frame, description_area, self.focused_field(), ctx);
        draw_reason(
            frame,
            reason_area,
            self.previewing,
            self.focused_field(),
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
}
