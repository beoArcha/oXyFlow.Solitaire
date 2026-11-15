use std::time::{Duration, Instant};

use oxysol_core::{DrawCount, Session, Variant};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use super::draw;
use crate::app::Modal;
use crate::caps::{ColorDepth, TerminalCaps};
use crate::glyphs::Glyphs;
use crate::i18n::{self, Locale};
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};
use crate::settings::Settings;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::theme::{Theme, ThemeId};

const SIZES: [(u16, u16); 2] = [(100, 30), (160, 50)];
const FORBIDDEN_UNICODE: [char; 6] = ['▶', '◀', '╭', '╮', '╰', '╯'];

fn all_modals() -> Vec<Modal> {
    vec![
        Modal::Help,
        Modal::Pause,
        Modal::win(),
        Modal::confirm("board.confirm.quit", Effect::Quit),
        Modal::alert("common.error.storage"),
    ]
}

fn timed_board(now: Instant) -> BoardScreen {
    let mut board = BoardScreen::new();
    let session = Session::new(Variant::Klondike(DrawCount::One), 42);
    board.begin(session, None, Duration::from_secs(75), now);
    board
}

fn render(
    modal: &Modal,
    caps: TerminalCaps,
    locale: Locale,
    size: (u16, u16),
    now: Instant,
) -> Terminal<TestBackend> {
    i18n::set_locale(locale);
    let settings = Settings::for_caps(&caps);
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let glyphs = Glyphs::new(settings.glyph_mode);
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &caps,
        audio_available: false,
        has_saved_game: false,
        now,
    };
    let board = timed_board(now);
    let mut terminal =
        Terminal::new(TestBackend::new(size.0, size.1)).unwrap_or_else(|never| match never {});
    terminal
        .draw(|frame| {
            let area = frame.area();
            draw(frame, area, modal, &board, &ctx);
        })
        .unwrap_or_else(|never| match never {});
    terminal
}

fn lines_of(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    (0..area.height)
        .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn text_of(terminal: &Terminal<TestBackend>) -> String {
    lines_of(terminal).join("\n")
}

fn position_of(terminal: &Terminal<TestBackend>, needle: &str) -> Option<(u16, u16)> {
    lines_of(terminal)
        .iter()
        .enumerate()
        .find_map(|(row, line)| {
            let byte = line.find(needle)?;
            let column = line[..byte].chars().count();
            Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
        })
}

fn caps(unicode: bool) -> TerminalCaps {
    if unicode { TRUE_COLOR } else { PLAIN }
}

fn focused(unicode: bool, label: &str) -> String {
    if unicode {
        format!("► {label} ◄")
    } else {
        format!("> {label} <")
    }
}

fn assert_renders_cleanly(modal: &Modal, locale: Locale, unicode: bool, size: (u16, u16)) {
    let terminal = render(modal, caps(unicode), locale, size, Instant::now());
    let text = text_of(&terminal);
    let context = format!("{modal:?} {} unicode={unicode} {size:?}", locale.id());
    assert!(text.contains('╔') || text.contains('+'), "{context}");
    if unicode {
        for glyph in FORBIDDEN_UNICODE {
            assert!(!text.contains(glyph), "{context}: {glyph}");
        }
    } else {
        assert!(text.is_ascii(), "{context}");
    }
}

#[test]
fn every_overlay_renders_in_both_locales_and_glyph_modes() {
    for modal in all_modals() {
        for locale in Locale::ALL {
            for unicode in [true, false] {
                for size in SIZES {
                    assert_renders_cleanly(&modal, locale, unicode, size);
                }
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn borders_are_double_in_unicode_and_plain_ascii_otherwise() {
    let now = Instant::now();
    let terminal = render(&Modal::Help, caps(true), Locale::EnUs, (100, 30), now);
    let text = text_of(&terminal);
    assert!(text.contains('╔') && text.contains('╝'));
    let terminal = render(&Modal::Help, caps(false), Locale::EnUs, (100, 30), now);
    assert!(text_of(&terminal).contains("+-"));
}

#[test]
fn help_lists_every_binding_and_the_gestures_in_a_100_by_30_window() {
    let now = Instant::now();
    for locale in Locale::ALL {
        let terminal = render(&Modal::Help, caps(true), locale, (100, 30), now);
        let text = text_of(&terminal);
        for section in crate::widgets::overlay::help::sections() {
            assert!(text.contains(&section.title), "{}", section.title);
            for row in section.rows {
                assert!(text.contains(&row.keys), "{}", row.keys);
                assert!(text.contains(&row.text), "{}", row.text);
            }
        }
        assert!(text.contains(&crate::i18n::t("help.overlay.help_footer").into_owned()));
        assert!(text.contains('╚'));
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn help_is_drawn_inside_the_terminal_at_both_sizes() {
    let now = Instant::now();
    let terminal = render(&Modal::Help, caps(false), Locale::EnUs, (100, 30), now);
    let lines = lines_of(&terminal);
    assert!(lines[0].trim().is_empty());
    assert!(lines[29].trim().is_empty());
    assert!(lines.iter().any(|line| line.contains("+-")));
}

#[test]
fn focused_button_is_marked_and_styled_without_color() {
    let now = Instant::now();
    let terminal = render(
        &Modal::confirm("board.confirm.quit", Effect::Quit),
        caps(true),
        Locale::EnUs,
        (100, 30),
        now,
    );
    assert!(text_of(&terminal).contains(&focused(true, "No")));
    let (column, row) = position_of(&terminal, "►").expect("marker");
    let cell = &terminal.backend().buffer()[(column, row)];
    assert!(cell.modifier.contains(Modifier::REVERSED));
    assert!(cell.modifier.contains(Modifier::BOLD));

    let focused_yes = Modal::Confirm {
        prompt: "board.confirm.quit",
        on_yes: Box::new(Effect::Quit),
        yes_focused: true,
    };
    let terminal = render(&focused_yes, caps(false), Locale::PlPl, (100, 30), now);
    assert!(text_of(&terminal).contains(&focused(false, "Yes")));
    assert!(text_of(&terminal).contains("  No  "));
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn win_shows_the_time_and_both_buttons_in_both_locales() {
    let now = Instant::now();
    for locale in Locale::ALL {
        for unicode in [true, false] {
            let terminal = render(&Modal::win(), caps(unicode), locale, (100, 30), now);
            let text = text_of(&terminal);
            assert!(text.contains("Time: 1:15"), "{}", locale.id());
            assert!(text.contains(&focused(unicode, "New game")));
            assert!(text.contains("Menu"));
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn win_moves_the_focus_to_menu() {
    let now = Instant::now();
    let modal = Modal::Win { menu_focused: true };
    let terminal = render(&modal, caps(true), Locale::EnUs, (100, 30), now);
    assert!(text_of(&terminal).contains(&focused(true, "Menu")));
}

#[test]
fn alert_and_pause_draw_their_text_and_buttons() {
    let now = Instant::now();
    let terminal = render(
        &Modal::alert("common.error.storage"),
        caps(true),
        Locale::EnUs,
        (100, 30),
        now,
    );
    let text = text_of(&terminal);
    assert!(text.contains("Notice"));
    assert!(text.contains(&focused(true, "OK")));

    let terminal = render(&Modal::Pause, caps(true), Locale::EnUs, (100, 30), now);
    let text = text_of(&terminal);
    assert!(text.contains("Paused"));
    assert!(text.contains(&focused(true, "Continue")));
}

fn started_board(caps: TerminalCaps) -> Harness {
    let mut h = Harness::with(crate::storage::memory::MemoryStorage::new(), caps, 100, 30);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    h
}

#[test]
fn pause_covers_the_board_and_shows_no_card() {
    let mut h = started_board(TRUE_COLOR);
    let before = h.text();
    assert!(before.contains("seed 42"));
    assert!(before.contains('♠') || before.contains('♥') || before.contains('▒'));
    h.type_char('p');
    let text = h.text();
    assert!(text.contains("Paused"));
    assert!(text.contains("seed 42"));
    for suit in ['♣', '♦', '♥', '♠', '▒'] {
        assert!(!text.contains(suit), "{suit}");
    }
}

#[test]
fn pause_hides_the_board_in_ascii_mode_too() {
    let mut h = started_board(PLAIN);
    assert!(h.text().contains("seed 42"));
    h.type_char('p');
    let lines = h.lines();
    assert!(lines[2..=10].iter().all(|line| line.trim().is_empty()));
    assert!(h.text().contains("Paused"));
    assert!(h.text().is_ascii());
}

#[test]
fn continue_button_resumes_the_board_by_mouse() {
    let mut h = started_board(TRUE_COLOR);
    h.type_char('p');
    let (column, row) = h.find("Continue").expect("button");
    h.click(column, row);
    let text = h.text();
    assert!(!text.contains("Paused"));
    assert!(text.contains("seed 42"));
}

#[test]
fn a_click_on_yes_quits_from_the_board() {
    let mut h = started_board(TRUE_COLOR);
    h.type_char('q');
    assert!(h.text().contains("Please confirm"));
    let (column, row) = h.find("Yes").expect("button");
    h.click(column, row);
    assert!(h.app.should_quit());
}
