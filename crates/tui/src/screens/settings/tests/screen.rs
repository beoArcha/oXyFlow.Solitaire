use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::{bare, changed, fitting_size};
use crate::action::Action;
use crate::audio::{AudioStatus, StyleInfo};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::settings::SettingsScreen;
use crate::screens::settings::options::{FIELDS, Field, Step};
use crate::screens::settings::tabs::Tab;
use crate::screens::{Effect, ViewCtx};
use crate::settings::Settings;
use crate::test_support::{TRUE_COLOR, lines_without_pane, sound_styles};
use crate::theme::Theme;

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
    caps: TerminalCaps,
    sound_styles: Vec<StyleInfo>,
}

impl Fixture {
    fn new(settings: Settings, caps: TerminalCaps) -> Self {
        Self {
            theme: Theme::new(settings.theme, ColorDepth::TrueColor),
            glyphs: Glyphs::new(settings.glyph_mode),
            settings,
            caps,
            sound_styles: sound_styles(),
        }
    }

    fn ctx(&self, audio_status: AudioStatus) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &self.caps,
            audio_status,
            sound_styles: &self.sound_styles,
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        }
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn act(screen: &mut SettingsScreen, ctx: &ViewCtx<'_>, action: Action) -> Effect {
    screen.handle(&key(KeyCode::Null), Some(action), ctx)
}

fn switch_screen_tab(screen: &mut SettingsScreen, ctx: &ViewCtx<'_>, target: Tab) {
    let needle = format!("[{}]", t(target.label_key()));
    for _ in 0..Tab::ALL.len() {
        if render(screen, ctx)
            .iter()
            .any(|line| line.contains(&needle))
        {
            return;
        }
        act(screen, ctx, Action::FocusNext);
    }
}

/// Draws the screen in a window wide enough for the preview pane beside the list and high enough for
/// the tallest tab.
fn render(screen: &mut SettingsScreen, ctx: &ViewCtx<'_>) -> Vec<String> {
    let (width, height) = fitting_size();
    let mut terminal =
        Terminal::new(TestBackend::new(width, height)).unwrap_or_else(|never| match never {});
    terminal
        .draw(|frame| screen.draw(frame, frame.area(), ctx))
        .unwrap_or_else(|never| match never {});
    lines_without_pane(terminal.backend().buffer(), ctx.theme)
}

fn position_of(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, text)| {
        let byte = text.find(needle)?;
        let column = text[..byte].chars().count();
        Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
    })
}

fn row_label(field: Field) -> String {
    format!("{} .", t(field.label_key()))
}

fn row_at(lines: &[String], field: Field) -> (u16, u16) {
    let found = position_of(lines, &row_label(field));
    assert!(found.is_some(), "row {field:?} is not drawn");
    found.unwrap_or_default()
}

#[test]
fn a_click_changes_the_clicked_row_forward() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::Ready);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &ctx);
    let (column, row) = row_at(&lines, Field::Theme);
    let effect = screen.handle_mouse(&click(column, row), &ctx);
    assert_eq!(
        effect,
        Effect::Preview(changed(Field::Theme, &fixture.settings, Step::Next))
    );
}

#[test]
fn every_row_changes_by_mouse() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::Ready);
    for tab in [Tab::AudioVisual, Tab::Gameplay] {
        for field in tab
            .fields()
            .iter()
            .copied()
            .filter(|field| !super::is_console_font_row(*field))
        {
            let mut screen = SettingsScreen::new();
            switch_screen_tab(&mut screen, &ctx, tab);
            let lines = render(&mut screen, &ctx);
            let (column, row) = row_at(&lines, field);
            let effect = screen.handle_mouse(&click(column, row), &ctx);
            let next = changed(field, &fixture.settings, Step::Next);
            let expected = if field.is_interface() {
                Effect::Preview(next)
            } else {
                Effect::Apply(next)
            };
            assert_eq!(effect, expected, "{tab:?} {field:?}");
        }
    }
}

#[test]
fn sound_rows_do_nothing_without_audio() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::NoDevice);
    let mut screen = SettingsScreen::new();
    switch_screen_tab(&mut screen, &ctx, Tab::AudioVisual);
    let lines = render(&mut screen, &ctx);
    assert!(
        lines
            .iter()
            .any(|text| text.contains("no working audio device"))
    );
    let mut clicked = SettingsScreen::new();
    switch_screen_tab(&mut clicked, &ctx, Tab::AudioVisual);
    let click_lines = render(&mut clicked, &ctx);
    let (column, row) = row_at(&click_lines, Field::Music);
    assert_eq!(
        clicked.handle_mouse(&click(column, row), &ctx),
        Effect::None
    );
    let sound_style = Tab::AudioVisual
        .fields()
        .iter()
        .position(|field| *field == Field::SoundStyle)
        .unwrap_or_default();
    for _ in 0..sound_style {
        act(&mut screen, &ctx, Action::Down);
    }
    assert_eq!(act(&mut screen, &ctx, Action::Select), Effect::None);
    assert_eq!(act(&mut screen, &ctx, Action::Right), Effect::None);
    switch_screen_tab(&mut screen, &ctx, Tab::Gameplay);
    assert_eq!(
        act(&mut screen, &ctx, Action::Select),
        Effect::Apply(changed(Field::Language, &fixture.settings, Step::Next))
    );
}

#[test]
fn the_reason_names_a_remote_session() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::RemoteSession));
    assert!(lines.iter().any(|text| text.contains("remote session")));
}

#[test]
fn a_build_without_sound_says_sound_is_not_included() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::NotBuilt));
    assert!(
        lines
            .iter()
            .any(|text| text.contains("Sound is not included in this build."))
    );
    assert!(!lines.iter().any(|text| text.contains("audio device")));
}

#[test]
fn the_reason_is_not_drawn_while_sound_is_available() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    assert!(!lines.iter().any(|text| text.contains("Sound is")));
}

#[test]
fn unavailable_rows_are_marked_in_their_text() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let ctx = fixture.ctx(AudioStatus::NoDevice);
    switch_screen_tab(&mut screen, &ctx, Tab::AudioVisual);
    let lines = render(&mut screen, &ctx);
    assert!(
        lines
            .iter()
            .any(|text| { text.contains("Music") && bare(text).ends_with("Off (unavailable)") })
    );
    switch_screen_tab(&mut screen, &ctx, Tab::Popular);
    let lines = render(&mut screen, &ctx);
    assert!(
        lines
            .iter()
            .any(|text| { text.contains("Language") && bare(text).ends_with("English (US)") })
    );
    assert!(
        !lines
            .iter()
            .any(|text| text.contains("Language") && text.contains("(unavailable)"))
    );
}

#[test]
fn every_row_is_drawn_in_both_locales() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::Ready);
    let mut seen = Vec::new();
    for tab in [Tab::AudioVisual, Tab::Gameplay] {
        let mut screen = SettingsScreen::new();
        switch_screen_tab(&mut screen, &ctx, tab);
        let lines = render(&mut screen, &ctx);
        for field in tab.fields() {
            row_at(&lines, *field);
            seen.push(*field);
        }
    }
    assert_eq!(seen.len(), FIELDS.len());
    set_locale(Locale::PlPl);
    let polish = Settings {
        locale: Locale::PlPl,
        ..Settings::default()
    };
    let fixture = Fixture::new(polish, TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    let label = t("settings.row.language").into_owned();
    let value = Locale::PlPl.own_name();
    assert!(
        lines
            .iter()
            .any(|text| { text.contains(&label) && bare(text).ends_with(value) })
    );
    set_locale(Locale::EnUs);
}

fn value_column(line: &str) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let dots = chars.iter().position(|c| *c == '.')?;
    let run_end = dots + chars[dots..].iter().position(|c| *c != '.')?;
    Some(run_end + 1)
}

#[test]
fn the_values_start_in_one_column_across_all_tabs() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::Ready);
    for tab in Tab::ALL {
        let mut screen = SettingsScreen::new();
        switch_screen_tab(&mut screen, &ctx, tab);
        let lines = render(&mut screen, &ctx);
        let columns: Vec<Option<usize>> = tab
            .fields()
            .iter()
            .map(|field| value_column(&lines[usize::from(row_at(&lines, *field).1)]))
            .collect();
        assert!(columns.iter().all(Option::is_some), "{tab:?}: {columns:?}");
        let first = columns.first().copied().flatten();
        assert!(
            columns.iter().all(|column| *column == first),
            "{tab:?}: {columns:?}"
        );
    }
}
