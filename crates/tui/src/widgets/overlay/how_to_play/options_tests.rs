//! Tests that the How to play Rules tab follows the active game options (centred and docked,
//! opened directly or via Help), across all 9 locales.

use oxysol_core::{DrawCount, GameOptions, StockPasses, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::content::{self, Kind, TEXT_WIDTH};
use super::state::{HowToPlay, Rules};
use crate::app::Modal;
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale};
use crate::screens::Effect;
use crate::settings::{DockSide, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

fn joined(rules: Rules, options: GameOptions) -> String {
    content::rows_with_options(rules, options)
        .into_iter()
        .map(|row| row.text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn harness_with(options: GameOptions, dock: DockSide, width: u16, height: u16) -> Harness {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        game_options: options,
        help_dock: dock,
        ..Settings::default()
    });
    Harness::with(storage, TRUE_COLOR, width, height)
}

fn click_text(h: &mut Harness, needle: &str) {
    let Some((col, row)) = h.find(needle) else {
        panic!("{needle} not found in:\n{}", h.text());
    };
    h.click(col, row);
}

fn strip_panel_chrome(screen: &[String], min_col: usize) -> String {
    screen
        .iter()
        .map(|line| {
            let slice: String = line.chars().skip(min_col).collect();
            slice
                .replace(['║', '│', '█', '░'], " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_modal_text(h: &mut Harness, min_col: usize) -> String {
    h.press(KeyCode::Home);
    let mut pages = vec![strip_panel_chrome(&h.lines(), min_col)];
    for _ in 0..8 {
        h.press(KeyCode::PageDown);
        let page = strip_panel_chrome(&h.lines(), min_col);
        if pages.last().is_some_and(|prev| prev == &page) {
            break;
        }
        pages.push(page);
    }
    pages
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn default_options_match_default_rows_for_every_game() {
    for rules in Rules::ALL {
        assert_eq!(
            content::rows_with_options(rules, GameOptions::DEFAULT),
            content::rows(rules),
            "{rules:?}"
        );
    }
}

#[test]
fn klondike_options_replace_matching_lines_and_drop_default_only_tip() {
    let any_empty = GameOptions {
        klondike_any_empty: true,
        ..GameOptions::DEFAULT
    };
    let text = joined(Rules::Klondike, any_empty);
    assert!(text.contains(&*i18n::t("rules.klondike.moves_2_any_empty")));
    assert!(!text.contains(&*i18n::t("rules.klondike.moves_2")));
    assert!(!text.contains(&*i18n::t("rules.klondike.tips_3")));
    assert!(text.contains(&*i18n::t("rules.klondike.moves_5")));

    let three_passes = GameOptions {
        klondike_passes: StockPasses::Three,
        ..GameOptions::DEFAULT
    };
    let text = joined(Rules::Klondike, three_passes);
    assert!(text.contains(&*i18n::t("rules.klondike.moves_5_passes_3")));
    assert!(!text.contains(&*i18n::t("rules.klondike.moves_5")));
    assert!(text.contains(&*i18n::t("rules.klondike.moves_2")));
    assert!(text.contains(&*i18n::t("rules.klondike.tips_3")));

    let one_pass = GameOptions {
        klondike_passes: StockPasses::One,
        ..GameOptions::DEFAULT
    };
    let text = joined(Rules::Klondike, one_pass);
    assert!(text.contains(&*i18n::t("rules.klondike.moves_5_passes_1")));
    assert!(!text.contains(&*i18n::t("rules.klondike.moves_5")));
}

#[test]
fn spider_and_golf_options_replace_matching_lines_for_every_combination() {
    let spider = GameOptions {
        spider_deal_empty: true,
        ..GameOptions::DEFAULT
    };
    let spider_text = joined(Rules::Spider, spider);
    assert!(spider_text.contains(&*i18n::t("rules.spider.moves_5_deal_empty")));
    assert!(!spider_text.contains(&*i18n::t("rules.spider.moves_5")));

    let golf_cases = [
        (false, false, "rules.golf.moves_2"),
        (true, false, "rules.golf.moves_2_wrap"),
        (false, true, "rules.golf.moves_2_king_stops"),
        (true, true, "rules.golf.moves_2_wrap_king_stops"),
    ];
    let all_keys: Vec<String> = golf_cases
        .iter()
        .map(|(_, _, key)| i18n::t(key).into_owned())
        .collect();
    for (index, (wrap, king_stops, key)) in golf_cases.iter().enumerate() {
        let options = GameOptions {
            golf_wrap: *wrap,
            golf_king_stops: *king_stops,
            ..GameOptions::DEFAULT
        };
        let text = joined(Rules::Golf, options);
        let expected = &all_keys[index];
        assert!(text.contains(expected), "wrap={wrap} stops={king_stops}");
        for (other_idx, other) in all_keys.iter().enumerate() {
            if other_idx != index {
                assert!(!text.contains(other), "unexpected {key} in {text}");
            }
        }
    }
}

#[test]
fn other_games_stay_unchanged_under_non_default_options() {
    let non_default = GameOptions {
        klondike_passes: StockPasses::One,
        klondike_any_empty: true,
        spider_deal_empty: true,
        golf_wrap: true,
        golf_king_stops: true,
    };
    for rules in [
        Rules::FreeCell,
        Rules::Pyramid,
        Rules::Yukon,
        Rules::TriPeaks,
    ] {
        assert_eq!(
            content::rows_with_options(rules, non_default),
            content::rows(rules),
            "{rules:?}"
        );
    }
}

#[test]
fn every_option_fits_100_by_30_in_all_offered_locales() {
    let option_sets = [
        GameOptions::DEFAULT,
        GameOptions {
            klondike_passes: StockPasses::Three,
            klondike_any_empty: true,
            spider_deal_empty: true,
            golf_wrap: true,
            golf_king_stops: false,
        },
        GameOptions {
            klondike_passes: StockPasses::One,
            golf_king_stops: true,
            ..GameOptions::DEFAULT
        },
        GameOptions {
            klondike_passes: StockPasses::One,
            klondike_any_empty: true,
            spider_deal_empty: true,
            golf_wrap: true,
            golf_king_stops: true,
        },
    ];
    for locale in Locale::offered() {
        i18n::set_locale(locale);
        for options in option_sets {
            for rules in [Rules::Klondike, Rules::Spider, Rules::Golf] {
                let rows = content::rows_with_options(rules, options);
                assert_eq!(rows.iter().filter(|r| r.kind == Kind::Heading).count(), 5);
                for row in &rows {
                    let width = row.lead.chars().count() + row.text.chars().count();
                    assert!(width <= TEXT_WIDTH, "{locale:?} {rules:?}: {width}");
                    let line = format!("{}{}", row.lead, row.text);
                    assert!(!line.contains("rules.") && !line.contains('{'), "{line}");
                }
                for caps in [TRUE_COLOR, PLAIN] {
                    let mut h = Harness::with(MemoryStorage::new(), caps, 100, 30);
                    let modal = Modal::HowToPlay(HowToPlay::with_options(rules, options));
                    h.app.apply(Effect::Open(modal));
                    let lines = h.lines();
                    assert_eq!(lines.len(), 30);
                    assert!(
                        lines
                            .iter()
                            .flat_map(|l| l.chars())
                            .all(|ch| ch.is_ascii() || is_wgl4(ch)),
                        "{locale:?} {rules:?}"
                    );
                }
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn menu_and_tab_switching_follow_settings_options_when_no_game_is_running() {
    let settings_options = GameOptions {
        klondike_passes: StockPasses::One,
        klondike_any_empty: true,
        spider_deal_empty: true,
        golf_wrap: true,
        golf_king_stops: true,
    };
    let mut h = harness_with(settings_options, DockSide::Centre, 100, 30);
    h.type_char('i');
    let klondike = collect_modal_text(&mut h, 0);
    assert!(klondike.contains("Any card or run can go to an empty column."));
    assert!(klondike.contains("Only 1 pass through the stock is allowed"));
    assert!(!klondike.contains("Empty a column only when you have a king"));

    h.press(KeyCode::Right);
    assert!(collect_modal_text(&mut h, 0).contains("even while a column is empty."));

    for _ in 0..4 {
        h.press(KeyCode::Right);
    }
    assert!(
        collect_modal_text(&mut h, 0)
            .contains("a king or a 2 goes onto an ace, while nothing goes onto a king.")
    );
    h.press(KeyCode::Esc);

    h.type_char('?');
    h.press(KeyCode::Tab);
    assert!(collect_modal_text(&mut h, 0).contains("Only 1 pass through the stock is allowed"));
    h.press(KeyCode::BackTab);
    click_text(&mut h, "Rules");
    assert!(collect_modal_text(&mut h, 0).contains("Any card or run can go to an empty column."));
}

#[test]
fn running_game_options_override_changed_settings_across_direct_and_help_tabs() {
    let started = GameOptions {
        klondike_passes: StockPasses::Three,
        klondike_any_empty: true,
        ..GameOptions::DEFAULT
    };
    let mut h = harness_with(started, DockSide::Centre, 100, 30);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(7),
        undo_allowed: true,
    });
    h.app.apply(Effect::Apply(Settings {
        game_options: GameOptions {
            klondike_passes: StockPasses::One,
            ..GameOptions::DEFAULT
        },
        ..Settings::default()
    }));

    h.type_char('i');
    let text = collect_modal_text(&mut h, 0);
    assert!(text.contains("3 passes through the stock in total."));
    assert!(text.contains("Any card or run can go to an empty column."));
    assert!(!text.contains("Only 1 pass through the stock"));
    h.press(KeyCode::Esc);

    h.type_char('?');
    h.press(KeyCode::Tab);
    assert!(collect_modal_text(&mut h, 0).contains("3 passes through the stock in total."));
    h.press(KeyCode::Esc);

    h.type_char('?');
    click_text(&mut h, "Rules");
    assert!(collect_modal_text(&mut h, 0).contains("Any card or run can go to an empty column."));
}

#[test]
fn docked_rules_tab_and_help_switch_keep_active_options() {
    let game_options = GameOptions {
        golf_wrap: true,
        ..GameOptions::DEFAULT
    };
    let mut h = harness_with(game_options, DockSide::Right, 140, 40);
    h.app.apply(Effect::Start {
        variant: Variant::Golf,
        seed: Some(7),
        undo_allowed: true,
    });
    h.app.apply(Effect::Apply(Settings {
        help_dock: DockSide::Right,
        game_options: GameOptions {
            golf_king_stops: true,
            ..GameOptions::DEFAULT
        },
        ..Settings::default()
    }));

    h.type_char('?');
    assert!(h.text().contains("Dock: right"));
    h.press(KeyCode::F(6));
    h.press(KeyCode::Tab);
    let text = collect_modal_text(&mut h, 100);
    assert!(text.contains("King and ace wrap around:"));
    assert!(text.contains("an ace or a queen goes onto a king"));
    assert!(!text.contains("nothing goes onto a king"));

    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.type_char('i');
    assert!(collect_modal_text(&mut h, 100).contains("an ace or a queen goes onto a king"));

    h.press(KeyCode::BackTab);
    let Some((col, row)) = h.find("◄ Keys ►") else {
        panic!("docked Keys tab not found in:\n{}", h.text());
    };
    h.click(col + 7, row);
    assert!(collect_modal_text(&mut h, 100).contains("an ace or a queen goes onto a king"));

    let mut h_spider = harness_with(
        GameOptions {
            spider_deal_empty: true,
            ..GameOptions::DEFAULT
        },
        DockSide::Right,
        140,
        40,
    );
    h_spider.app.apply(Effect::Start {
        variant: Variant::Spider(SuitCount::One),
        seed: Some(7),
        undo_allowed: true,
    });
    h_spider.type_char('i');
    h_spider.press(KeyCode::F(6));
    assert!(collect_modal_text(&mut h_spider, 100).contains("even while a column is empty."));
}
