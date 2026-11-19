use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::{bare, changed};
use crate::action::Action;
use crate::audio::{AudioStatus, StyleInfo};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::settings::SettingsScreen;
use crate::screens::settings::options::{FIELDS, Field, Group, Step};
use crate::screens::{Effect, ViewCtx};
use crate::settings::Settings;
use crate::test_support::{TRUE_COLOR, sound_styles};
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

fn render(screen: &mut SettingsScreen, ctx: &ViewCtx<'_>) -> Vec<String> {
    let mut terminal =
        Terminal::new(TestBackend::new(100, 30)).unwrap_or_else(|never| match never {});
    terminal
        .draw(|frame| screen.draw(frame, frame.area(), ctx))
        .unwrap_or_else(|never| match never {});
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    (0..area.height)
        .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn position_of(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, text)| {
        let byte = text.find(needle)?;
        let column = text[..byte].chars().count();
        Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
    })
}

fn row_label(field: Field) -> String {
    t(field.label_key()).into_owned()
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
        Effect::Apply(changed(Field::Theme, &fixture.settings, Step::Next))
    );
}

#[test]
fn every_row_changes_by_mouse() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::Ready);
    for field in FIELDS {
        let mut screen = SettingsScreen::new();
        let lines = render(&mut screen, &ctx);
        let (column, row) = row_at(&lines, field);
        let effect = screen.handle_mouse(&click(column, row), &ctx);
        assert_eq!(
            effect,
            Effect::Apply(changed(field, &fixture.settings, Step::Next)),
            "{field:?}"
        );
    }
}

#[test]
fn sound_rows_do_nothing_without_audio() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let ctx = fixture.ctx(AudioStatus::NoDevice);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &ctx);
    assert!(
        lines
            .iter()
            .any(|text| text.contains("no working audio device"))
    );
    let mut clicked = SettingsScreen::new();
    let lines = render(&mut clicked, &ctx);
    let (column, row) = row_at(&lines, Field::Music);
    assert_eq!(
        clicked.handle_mouse(&click(column, row), &ctx),
        Effect::None
    );
    for _ in 0..8 {
        act(&mut screen, &ctx, Action::Down);
    }
    assert_eq!(act(&mut screen, &ctx, Action::Select), Effect::None);
    assert_eq!(act(&mut screen, &ctx, Action::Right), Effect::None);
    for _ in 0..8 {
        act(&mut screen, &ctx, Action::Up);
    }
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
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::NoDevice));
    assert!(
        lines
            .iter()
            .any(|text| { text.contains("Music") && bare(text).ends_with("Off (unavailable)") })
    );
    assert!(
        lines
            .iter()
            .any(|text| { text.contains("Language") && bare(text).ends_with("English") })
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
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    for field in FIELDS {
        row_at(&lines, field);
    }
    set_locale(Locale::PlPl);
    let polish = Settings {
        locale: Locale::PlPl,
        ..Settings::default()
    };
    let fixture = Fixture::new(polish, TRUE_COLOR);
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    let label = t("settings.row.language").into_owned();
    let value = t("settings.locale.pl_pl").into_owned();
    assert!(
        lines
            .iter()
            .any(|text| { text.contains(&label) && bare(text).ends_with(&value) })
    );
    set_locale(Locale::EnUs);
}

fn heading_y(lines: &[String], group: Group) -> u16 {
    let found = position_of(lines, &t(group.label_key()));
    found.map_or(0, |(_, y)| y)
}

#[test]
fn the_rows_sit_under_their_group_headings_and_the_aid_rows_close_the_list() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    let display = heading_y(&lines, Group::Display);
    let sound = heading_y(&lines, Group::Sound);
    let hints = heading_y(&lines, Group::Hints);
    assert!(display < row_at(&lines, Field::Language).1);
    assert!(row_at(&lines, Field::Colors).1 < sound);
    assert!(sound < row_at(&lines, Field::SoundStyle).1);
    assert!(row_at(&lines, Field::Volume).1 < hints);
    assert!(hints < row_at(&lines, Field::Aura).1);
    let last = FIELDS
        .iter()
        .map(|field| row_at(&lines, *field).1)
        .max()
        .unwrap_or_default();
    assert_eq!(row_at(&lines, Field::AutoFinish).1, last);
}

fn value_column(line: &str) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let dots = chars.iter().position(|c| *c == '.')?;
    let run_end = dots + chars[dots..].iter().position(|c| *c != '.')?;
    Some(run_end + 1)
}

#[test]
fn the_values_start_in_one_column() {
    let fixture = Fixture::new(Settings::default(), TRUE_COLOR);
    let mut screen = SettingsScreen::new();
    let lines = render(&mut screen, &fixture.ctx(AudioStatus::Ready));
    let columns: Vec<Option<usize>> = FIELDS
        .iter()
        .map(|field| value_column(&lines[usize::from(row_at(&lines, *field).1)]))
        .collect();
    assert!(columns.iter().all(Option::is_some), "{columns:?}");
    let first = columns.first().copied().flatten();
    assert!(columns.iter().all(|column| *column == first), "{columns:?}");
}
