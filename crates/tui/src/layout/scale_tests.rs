//! Tests for the interface size scale (`S`, `M`, `L`, `XL`), fallback behavior and rendering
//! across terminal sizes and all complete locales.

use oxysol_core::{DrawCount, PileRef, PyramidRules, SuitCount, Variant, deal};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::{
    BOARD_MIN_WIDTH, CardSize, Class, InterfaceSize, SCALES, SIZES, chrome_panels_for,
    compute_for_scale,
};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;
use crate::screens::Effect;
use crate::settings::{DockSide, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

#[test]
fn default_m_matches_original_constants() {
    let m = InterfaceSize::M.scale();
    assert_eq!(m.card_cap, CardSize::new(13, 11));
    assert_eq!(m.gaps.min_gap, 1);
    assert_eq!(m.gaps.max_gap, 3);
    assert_eq!(m.gaps.group_gap, 2);
    assert_eq!(m.gaps.tripeaks_base_gap, 2);
    assert_eq!(m.chrome.standard_min_width, 130);
    assert_eq!(m.chrome.wide_min_width, 180);
    assert_eq!(m.chrome.guide_width, 30);
    assert_eq!(m.chrome.guide_width_wide, 40);
    assert_eq!(m.chrome.moves_rows, 12);
    assert_eq!(m.chrome.moves_rows_tall, 22);
    assert_eq!(m.chrome.moves_reasons_from, 56);
    assert_eq!(m.dialog.max_text_columns, 64);
    assert_eq!(m.dialog.min_columns, 24);
    assert_eq!(m.help.min_width, 80);
    assert_eq!(m.help.gap, 4);
    assert_eq!(m.help.key_padding, 2);
    assert_eq!(m.how_to_play.text_width, 76);
    assert_eq!(m.guide_window.box_width, 62);
    assert_eq!(m.guide_window.text_width, 58);
    assert_eq!(m.guide_window.text_rows, 17);
    assert_eq!(m.moves_window.box_width, 54);
    assert_eq!(m.quick_options.box_width, 76);
    assert_eq!(m.dock.min_width, 30);
    assert_eq!(m.dock.max_width, 60);

    for card in SIZES {
        assert_eq!(m.fan.row_gap(card), card.row_gap());
        assert_eq!(m.fan.up_step(card), card.up_step());
        assert_eq!(m.fan.down_step(card), card.down_step());
        assert_eq!(m.fan.waste_step(card), card.waste_step());
        assert_eq!(m.fan.wide_up_step(card), card.wide_up_step());
        assert_eq!(m.fan.wide_down_step(card), card.wide_down_step());
        assert_eq!(m.fan.wide_waste_step(card), card.wide_waste_step());
    }
}

#[test]
fn scales_grow_monotonically_from_s_to_xl() {
    for pair in SCALES.windows(2) {
        let [smaller, larger] = pair else {
            unreachable!()
        };
        assert!(smaller.card_cap.width <= larger.card_cap.width);
        assert!(smaller.gaps.min_gap <= larger.gaps.min_gap);
        assert!(smaller.gaps.max_gap <= larger.gaps.max_gap);
        assert!(smaller.gaps.group_gap <= larger.gaps.group_gap);
        assert!(smaller.chrome.guide_width < larger.chrome.guide_width);
        assert!(smaller.chrome.standard_min_width < larger.chrome.standard_min_width);
        assert!(smaller.chrome.wide_min_width < larger.chrome.wide_min_width);
        assert!(smaller.dialog.max_text_columns < larger.dialog.max_text_columns);
        assert!(smaller.help.min_width < larger.help.min_width);
        assert!(smaller.guide_window.box_width < larger.guide_window.box_width);
        assert!(smaller.moves_window.box_width < larger.moves_window.box_width);
        assert!(smaller.quick_options.box_width < larger.quick_options.box_width);
        assert!(smaller.dock.min_width < larger.dock.min_width);
        assert!(smaller.dock.max_width < larger.dock.max_width);
        for card in SIZES {
            assert!(smaller.fan.row_gap(card) <= larger.fan.row_gap(card));
            assert!(smaller.fan.up_step(card) <= larger.fan.up_step(card));
            assert!(smaller.fan.down_step(card) <= larger.fan.down_step(card));
            assert!(smaller.fan.waste_step(card) <= larger.fan.waste_step(card));
        }
    }
}

#[test]
fn downward_and_fit_resolve_largest_fitting_step() {
    let down: Vec<_> = InterfaceSize::L.downward().collect();
    assert_eq!(down, [InterfaceSize::L, InterfaceSize::M, InterfaceSize::S]);
    assert_eq!(
        InterfaceSize::XL.fit(|scale| scale.help.min_width <= 90),
        InterfaceSize::L
    );
    assert_eq!(
        InterfaceSize::XL.fit(|scale| scale.help.min_width <= 60),
        InterfaceSize::S
    );
}

#[test]
fn board_layout_scales_card_cap_gaps_and_panels_across_steps() {
    let piles = deal(Variant::Klondike(DrawCount::Three), 5).piles();
    let large_board = Rect::new(0, 0, 150, 50);
    let small_scaled = compute_for_scale(
        Variant::Klondike(DrawCount::Three),
        &piles,
        large_board,
        None,
        false,
        InterfaceSize::S,
    );
    let xl_scaled = compute_for_scale(
        Variant::Klondike(DrawCount::Three),
        &piles,
        large_board,
        None,
        false,
        InterfaceSize::XL,
    );
    assert_eq!(small_scaled.card, CardSize::new(9, 7));
    assert_eq!(xl_scaled.card, CardSize::new(13, 11));

    let screen = Rect::new(0, 0, 130, 40);
    let s_chrome = chrome_panels_for(screen, true, true, false, InterfaceSize::S);
    let l_chrome = chrome_panels_for(screen, true, true, false, InterfaceSize::L);
    assert_eq!(s_chrome.class, Class::Standard);
    assert_eq!(s_chrome.guide.map(|r| r.width), Some(26));
    assert_eq!(l_chrome.class, Class::Compact);
    assert_eq!(l_chrome.guide, None);

    let wide_screen = Rect::new(0, 0, 200, 60);
    for step in InterfaceSize::ALL {
        let chrome = chrome_panels_for(wide_screen, true, true, false, step);
        assert_eq!(chrome.class, Class::Wide);
        assert_eq!(
            chrome.guide.map(|r| r.width),
            Some(step.scale().chrome.guide_width_wide)
        );
        assert!(chrome.body.width >= BOARD_MIN_WIDTH);
    }
}

#[test]
fn board_layout_falls_back_cleanly_when_viewport_is_narrower_than_xl() {
    let spider = deal(Variant::Spider(SuitCount::Two), 5).piles();
    let narrow = Rect::new(0, 0, 82, 26);
    let geo = compute_for_scale(
        Variant::Spider(SuitCount::Two),
        &spider,
        narrow,
        None,
        false,
        InterfaceSize::XL,
    );
    for pile in &geo.piles {
        assert!(pile.slot.right() <= narrow.right());
        assert!(pile.area.bottom() <= narrow.bottom());
    }
    let second_x = geo.pile(PileRef::Tableau(1)).map_or(0, |p| p.slot.x);
    let first_right = geo.pile(PileRef::Tableau(0)).map_or(0, |p| p.slot.right());
    assert_eq!(second_x - first_right, 1);

    let tripeaks = deal(Variant::TriPeaks, 5).piles();
    let compact_tri = Rect::new(0, 0, 74, 24);
    let tri_geo = compute_for_scale(
        Variant::TriPeaks,
        &tripeaks,
        compact_tri,
        None,
        false,
        InterfaceSize::XL,
    );
    for pile in &tri_geo.piles {
        assert!(pile.slot.right() <= compact_tri.right());
        assert!(pile.slot.bottom() <= compact_tri.bottom());
    }
}

fn harness_for(locale: Locale, step: InterfaceSize, (width, height): (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            interface_size: step,
            help_dock: DockSide::Centre,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    Harness::with(storage, TRUE_COLOR, width, height)
}

#[test]
fn all_steps_render_at_100x30_130x40_and_200x60_across_all_locales_without_overflow() {
    let games = [
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::Two),
        Variant::FreeCell,
        Variant::Pyramid(PyramidRules::Relaxed),
        Variant::TriPeaks,
    ];
    let sizes = [(100, 30), (130, 40), (200, 60)];
    let close_key = primary_key(Action::Back).unwrap_or_default();

    for locale in Locale::offered() {
        for &size in &sizes {
            for step in InterfaceSize::ALL {
                let mut h = harness_for(locale, step, size);
                for &variant in &games {
                    h.app.apply(Effect::Start {
                        variant,
                        seed: Some(5),
                        undo_allowed: true,
                    });
                    let lines = h.lines();
                    assert_eq!(lines.len(), usize::from(size.1));
                    assert!(
                        lines
                            .iter()
                            .all(|line| line.chars().count() == usize::from(size.0)),
                        "{locale:?} {step:?} {size:?} {variant:?}"
                    );
                }

                let close_label = tf("help.rules.close", &[("key", &close_key)]);
                h.type_char('?');
                let keys_text = h.text();
                assert!(
                    keys_text.contains(&close_label),
                    "Help Keys {locale:?} {step:?} {size:?}"
                );
                h.press(KeyCode::Tab);
                assert!(
                    h.text().contains(&close_label),
                    "HowToPlay {locale:?} {step:?} {size:?}"
                );
                h.press(KeyCode::Tab);
                assert!(
                    h.text().contains(&close_label),
                    "CommandLine {locale:?} {step:?} {size:?}"
                );
                h.press(KeyCode::Esc);

                h.type_char('o');
                assert!(
                    h.text().contains(&*t("board.options.save")),
                    "QuickOptions {locale:?} {step:?} {size:?}"
                );
                h.press(KeyCode::Esc);

                h.type_char('v');
                if h.app.modal().is_some() {
                    assert!(
                        h.text().contains(&close_label),
                        "MovesWindow {locale:?} {step:?} {size:?}"
                    );
                    h.press(KeyCode::Esc);
                }
                h.type_char('g');
                if h.app.modal().is_some() {
                    assert!(
                        h.text().contains(&close_label),
                        "GuideWindow {locale:?} {step:?} {size:?}"
                    );
                    h.press(KeyCode::Esc);
                }
            }
        }
    }
    set_locale(Locale::EnUs);
    let mut h = harness_for(Locale::EnUs, InterfaceSize::XL, (100, 30));
    h.app.apply(Effect::Goto(Screen::Settings));
    h.press(KeyCode::BackTab);
    let found = (0..30).any(|_| {
        let seen = h.text().contains(&*t("settings.row.interface_size"));
        if !seen {
            h.press(KeyCode::Down);
        }
        seen
    });
    assert!(found);
}
