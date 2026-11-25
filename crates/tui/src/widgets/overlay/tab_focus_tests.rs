//! Tests that the active tab and the focused button differ without color across every overlay
//! window with tabs or buttons, in every theme, color-vision mode, color depth, and glyph mode,
//! and that keyboard and mouse focus both update the rendered state.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use crate::app::{Modal, Screen};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale, t, tf};
use crate::screens::Effect;
use crate::settings::{ColorVision, DockSide, GlyphMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;
use crate::theme::ThemeId;

fn harness(theme: ThemeId, vision: ColorVision, depth: ColorDepth, unicode: bool) -> Harness {
    let caps = TerminalCaps {
        color: depth,
        unicode,
        remote: false,
    };
    let settings = Settings {
        theme,
        color_vision: vision,
        glyph_mode: if unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        },
        ..Settings::for_caps(&caps)
    };
    let mut storage = MemoryStorage::new();
    storage.settings = Some(settings);
    Harness::with(storage, caps, 100, 30)
}

fn strip_color_signature(h: &mut Harness, needle: &str) -> Vec<(String, Modifier)> {
    let (col, row) = h
        .find(needle)
        .unwrap_or_else(|| panic!("missing {needle:?} in:\n{}", h.text()));
    let buf = h.terminal.backend().buffer();
    let len = u16::try_from(needle.chars().count()).unwrap_or(0);
    (col..col + len)
        .map(|x| {
            let cell = &buf[(x, row)];
            (cell.symbol().to_owned(), cell.modifier)
        })
        .collect()
}

fn button_markers(unicode: bool, label: &str) -> String {
    if unicode {
        format!("► {label} ◄")
    } else {
        format!("> {label} <")
    }
}

fn assert_focused_button(h: &mut Harness, unicode: bool, label: &str, context: &str) {
    let sig = strip_color_signature(h, &button_markers(unicode, label));
    assert!(
        sig.iter().all(|(_, m)| {
            m.contains(Modifier::BOLD | Modifier::REVERSED) && !m.contains(Modifier::UNDERLINED)
        }),
        "{context}: focused button modifiers: {sig:?}"
    );
}

fn assert_wgl4_and_ascii_controls(
    h: &mut Harness,
    unicode: bool,
    tab_needle: &str,
    btn_needle: &str,
    context: &str,
) {
    for line in h.lines() {
        for ch in line.chars() {
            assert!(ch.is_ascii() || is_wgl4(ch), "{context}: non-WGL4 {ch:?}");
        }
    }
    if !unicode {
        for needle in [tab_needle, btn_needle] {
            for ch in needle.chars() {
                assert!(ch.is_ascii(), "{context}: non-ASCII {ch:?} in {needle:?}");
            }
        }
    }
}

fn assert_tab_and_button_differ_without_color(
    h: &mut Harness,
    tab_needle: &str,
    button_needle: &str,
    context: &str,
) {
    let tab = strip_color_signature(h, tab_needle);
    let button = strip_color_signature(h, button_needle);
    assert_ne!(tab, button, "{context}");
    assert!(
        tab.iter().all(|(_, m)| {
            m.contains(Modifier::BOLD | Modifier::UNDERLINED) && !m.contains(Modifier::REVERSED)
        }),
        "{context}: active tab modifiers: {tab:?}"
    );
    assert!(
        button.iter().all(|(_, m)| {
            m.contains(Modifier::BOLD | Modifier::REVERSED) && !m.contains(Modifier::UNDERLINED)
        }),
        "{context}: focused button modifiers: {button:?}"
    );
}

#[test]
fn active_tab_and_focused_button_differ_without_color_in_every_theme_vision_and_glyph_mode() {
    i18n::set_locale(Locale::EnUs);
    let close = tf("help.rules.close", &[("key", "Esc")]);
    for theme in ThemeId::ALL {
        for vision in ColorVision::ALL {
            for depth in [ColorDepth::Ansi16, ColorDepth::TrueColor] {
                for unicode in [true, false] {
                    let ctx = format!("{theme:?} {vision:?} {depth:?} unicode={unicode}");
                    let btn = button_markers(unicode, &close);
                    let mut h = harness(theme, vision, depth, unicode);
                    h.type_char('?');
                    assert_tab_and_button_differ_without_color(&mut h, "[Keys]", &btn, &ctx);
                    assert_wgl4_and_ascii_controls(&mut h, unicode, "[Keys]", &btn, &ctx);

                    h.press(KeyCode::Tab);
                    assert_tab_and_button_differ_without_color(&mut h, "[Klondike]", &btn, &ctx);
                    let top_rules = strip_color_signature(&mut h, "[Rules]");
                    assert!(
                        top_rules.iter().all(|(_, m)| m.is_empty()),
                        "{ctx}: unfocused top tab should have empty modifiers: {top_rules:?}"
                    );
                    assert_wgl4_and_ascii_controls(&mut h, unicode, "[Klondike]", &btn, &ctx);

                    h.press(KeyCode::Tab);
                    assert_tab_and_button_differ_without_color(
                        &mut h,
                        "[Command line]",
                        &btn,
                        &ctx,
                    );
                    assert_wgl4_and_ascii_controls(&mut h, unicode, "[Command line]", &btn, &ctx);
                }
            }
        }
    }
}

#[test]
fn keyboard_and_mouse_update_active_tab_while_focused_button_stays_distinct() {
    i18n::set_locale(Locale::EnUs);
    let btn = button_markers(true, "Close [Esc]");
    let mut h = harness(ThemeId::HighContrast, ColorVision::Normal, ColorDepth::TrueColor, true);
    h.type_char('?');
    assert_tab_and_button_differ_without_color(&mut h, "[Keys]", &btn, "open keys");

    h.press(KeyCode::Tab);
    assert_tab_and_button_differ_without_color(&mut h, "[Klondike]", &btn, "tab to rules");

    h.press(KeyCode::Right);
    assert_tab_and_button_differ_without_color(&mut h, "[Spider]", &btn, "right to spider");
    assert!(!h.text().contains("[Klondike]"));

    let (col, row) = h.find("FreeCell").expect("freecell tab");
    h.click(col, row);
    assert_tab_and_button_differ_without_color(&mut h, "[FreeCell]", &btn, "click freecell");

    let (col, row) = h.find("Command line").expect("command line tab");
    h.click(col, row);
    assert_tab_and_button_differ_without_color(&mut h, "[Command line]", &btn, "click cmd");
}

#[test]
fn docked_help_active_tab_and_close_button_differ_without_color() {
    i18n::set_locale(Locale::EnUs);
    let caps = TerminalCaps {
        color: ColorDepth::TrueColor,
        unicode: true,
        remote: false,
    };
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        help_dock: DockSide::Right,
        ..Settings::for_caps(&caps)
    });
    let mut h = Harness::with(storage, caps, 130, 40);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    h.type_char('?');
    assert_tab_and_button_differ_without_color(&mut h, "◄ Keys ►", "Close [Esc]", "docked keys");
    h.press(KeyCode::F(6));
    assert_tab_and_button_differ_without_color(&mut h, "◄ Keys ►", "Close [Esc]", "docked focus");
}

#[test]
fn quick_options_keyboard_and_mouse_focus_update_buttons_distinctly() {
    i18n::set_locale(Locale::EnUs);
    let save = t("board.options.save").into_owned();
    let close = tf("help.rules.close", &[("key", "Esc")]);
    for unicode in [true, false] {
        let mut h = harness(ThemeId::Dark, ColorVision::Normal, ColorDepth::TrueColor, unicode);
        h.press(KeyCode::Enter);
        h.press(KeyCode::Enter);
        h.type_char('o');
        assert!(matches!(h.app.modal(), Some(Modal::Quick(_))));
        assert!(h.text().contains(&format!("  {save}  ")));
        assert!(h.text().contains(&format!("  {close}  ")));

        h.press(KeyCode::Up);
        assert_focused_button(&mut h, unicode, &close, "quick close");
        assert!(h.text().contains(&format!("  {save}  ")));

        h.press(KeyCode::Left);
        assert_focused_button(&mut h, unicode, &save, "quick save");
        assert!(h.text().contains(&format!("  {close}  ")));

        let (col, row) = h.find(&save).expect("save button");
        h.click(col, row);
        assert!(h.app.modal().is_none());
    }
}

#[test]
fn settings_dialogs_and_confirmations_update_button_focus_by_keyboard_and_mouse() {
    i18n::set_locale(Locale::EnUs);
    let yes = t("common.dialog.yes").into_owned();
    let no = t("common.dialog.no").into_owned();

    let mut h = harness(ThemeId::Dark, ColorVision::Normal, ColorDepth::TrueColor, true);
    h.app
        .apply(Effect::Open(Modal::confirm("board.confirm.quit", Effect::Quit)));
    assert_focused_button(&mut h, true, &no, "confirm initial no");
    assert!(h.text().contains(&format!("  {yes}  ")));

    h.press(KeyCode::Tab);
    assert_focused_button(&mut h, true, &yes, "confirm tab yes");
    assert!(h.text().contains(&format!("  {no}  ")));

    let (col, row) = h.find(&yes).expect("yes button");
    h.click(col, row);
    assert!(h.app.should_quit());

    let mut settings_h =
        harness(ThemeId::HighContrast, ColorVision::Monochrome, ColorDepth::Ansi16, false);
    for _ in 0..3 {
        settings_h.press(KeyCode::Down);
    }
    settings_h.press(KeyCode::Enter);
    assert_eq!(settings_h.app.screen(), Screen::Settings);
    settings_h.app.apply(Effect::Open(Modal::confirm_or(
        "settings.confirm.keep_interface",
        Effect::LeaveSettings {
            keep: true,
            to: Screen::MainMenu,
        },
        Effect::LeaveSettings {
            keep: false,
            to: Screen::MainMenu,
        },
    )));
    assert_focused_button(&mut settings_h, false, &no, "settings confirm initial no");
    settings_h.press(KeyCode::Left);
    assert_focused_button(&mut settings_h, false, &yes, "settings confirm left yes");
    let (col, row) = settings_h.find(&no).expect("no button");
    settings_h.click(col, row);
    assert_eq!(settings_h.app.screen(), Screen::MainMenu);

    let mut gfx_h = harness(ThemeId::Light, ColorVision::Tritanopia, ColorDepth::TrueColor, true);
    let check =
        crate::widgets::overlay::GraphicsCheck::new(&Settings::default(), false, Effect::None);
    gfx_h.app.apply(Effect::Open(Modal::GraphicsCheck(check)));
    assert_focused_button(&mut gfx_h, true, &no, "gfx initial no");
    gfx_h.press(KeyCode::Tab);
    assert_focused_button(&mut gfx_h, true, &yes, "gfx tab yes");
    let (col, row) = gfx_h.find(&no).expect("no button");
    gfx_h.click(col, row);
    assert!(matches!(gfx_h.app.modal(), Some(Modal::Alert { .. })));
    let ok = t("common.dialog.ok").into_owned();
    assert_focused_button(&mut gfx_h, true, &ok, "gfx declined ok");
}
