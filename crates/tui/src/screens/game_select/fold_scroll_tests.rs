use oxysol_core::{DrawCount, Variant, YukonRules};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::catalog::Family;
use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::screens::Effect;
use crate::screens::focus::marked_text;
use crate::screens::variant_label_key;
use crate::settings::{GameGroup, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

fn select_with(
    settings: Settings,
    caps: crate::caps::TerminalCaps,
    width: u16,
    height: u16,
) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, caps, width, height);
    h.app.apply(Effect::Goto(Screen::GameSelect));
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn select_30() -> Harness {
    select_with(Settings::default(), TRUE_COLOR, 100, 30)
}

fn wheel(h: &mut Harness, kind: MouseEventKind, column: u16, row: u16) {
    h.send(crate::input::Input::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

fn focused_line(h: &mut Harness) -> String {
    marked_text(&h.lines(), "►")
        .map(|s| {
            let trimmed = s.trim_end();
            trimmed
                .strip_suffix(['█', '░', '#', ':'])
                .map_or(trimmed, str::trim_end)
                .to_owned()
        })
        .unwrap_or_default()
}

fn list_row_of(h: &mut Harness, needle: &str) -> (u16, u16) {
    h.find(needle).unwrap_or_default()
}

#[test]
fn mouse_wheel_scrolls_three_rows_and_keeps_focus_visible() {
    let mut h = select_with(
        Settings {
            favourites: vec![Variant::TriPeaks],
            ..Settings::default()
        },
        TRUE_COLOR,
        100,
        30,
    );
    let (col, row) = list_row_of(&mut h, &t(variant_label_key(Variant::TriPeaks)));
    assert!(h.text().contains("[-] Favorites"));
    let bar_before = bar_column(&mut h, ('█', '░'));
    assert!(!bar_before.is_empty());
    assert_eq!(bar_before.first().map(|(_, sym)| *sym), Some('█'));
    assert!(!h.text().contains('↓') && !h.text().contains('↑'));

    wheel(&mut h, MouseEventKind::ScrollDown, col, row);
    let text = h.text();
    assert!(!text.contains("[-] Favorites"), "{text}");
    let bar_after = bar_column(&mut h, ('█', '░'));
    assert_ne!(bar_after, bar_before);
    let focused = focused_line(&mut h);
    assert!(text.contains(&focused));
    assert!(
        !focused.starts_with('['),
        "expected variant focus: {focused}"
    );

    let spider_one = t(variant_label_key(Variant::Spider(
        oxysol_core::SuitCount::One,
    )));
    h.focus_game_variant(Variant::Spider(oxysol_core::SuitCount::One));
    let (col, row) = list_row_of(&mut h, &spider_one);
    wheel(&mut h, MouseEventKind::ScrollUp, col, row);
    let text = h.text();
    assert!(text.contains("[-] Favorites"), "{text}");
    assert!(focused_line(&mut h).contains(spider_one.as_ref()));
}

#[test]
fn page_up_page_down_home_and_end_move_focus_and_scroll() {
    let mut h = select_30();
    let first = t(variant_label_key(Variant::ALL[0])).into_owned();
    let last = t(variant_label_key(Variant::ALL[Variant::ALL.len() - 1])).into_owned();
    assert!(focused_line(&mut h).contains(&first));
    let top_bar = bar_column(&mut h, ('█', '░'));

    h.press(KeyCode::PageDown);
    assert!(!focused_line(&mut h).contains(&first));
    let mid = focused_line(&mut h);
    assert_ne!(bar_column(&mut h, ('█', '░')), top_bar);

    h.press(KeyCode::End);
    assert!(focused_line(&mut h).contains(&last));
    let end_bar = bar_column(&mut h, ('█', '░'));
    assert_eq!(end_bar.last().map(|(_, sym)| *sym), Some('█'));

    h.press(KeyCode::PageUp);
    let after_pgup = focused_line(&mut h);
    assert!(!after_pgup.contains(&last));
    assert_ne!(after_pgup, mid);

    h.press(KeyCode::Home);
    assert!(focused_line(&mut h).contains(&first));
    assert_eq!(bar_column(&mut h, ('█', '░')), top_bar);
}

fn bar_x(h: &mut Harness, (thumb, groove): (char, char)) -> Option<u16> {
    h.lines().into_iter().find_map(|line| {
        let chars: Vec<char> = line.chars().collect();
        let pos = chars.iter().position(|ch| *ch == thumb || *ch == groove)?;
        u16::try_from(pos).ok()
    })
}

fn bar_column(h: &mut Harness, glyphs: (char, char)) -> Vec<(u16, char)> {
    let Some(x) = bar_x(h, glyphs) else {
        return Vec::new();
    };
    let (thumb, groove) = glyphs;
    h.lines()
        .into_iter()
        .enumerate()
        .filter_map(|(row, line)| {
            let ch = line.chars().nth(usize::from(x))?;
            (ch == thumb || ch == groove)
                .then(|| u16::try_from(row).ok().map(|r| (r, ch)))
                .flatten()
        })
        .collect()
}

#[test]
fn scrollbar_appears_only_when_list_scrolls() {
    let mut short = select_with(Settings::default(), TRUE_COLOR, 100, 45);
    assert!(bar_column(&mut short, ('█', '░')).is_empty());

    let mut folded = select_with(
        Settings {
            folded_families: Family::ALL.into_iter().map(Family::group).collect(),
            ..Settings::default()
        },
        TRUE_COLOR,
        100,
        30,
    );
    assert!(bar_column(&mut folded, ('█', '░')).is_empty());

    let mut tall_enough_to_scroll = select_30();
    assert!(!bar_column(&mut tall_enough_to_scroll, ('█', '░')).is_empty());
}

#[test]
fn clicking_track_pages_and_dragging_thumb_scrolls_in_unicode_and_ascii() {
    for (caps, thumb, groove) in [(TRUE_COLOR, '█', '░'), (PLAIN, '#', ':')] {
        let mut h = select_with(Settings::for_caps(&caps), caps, 100, 30);
        let x = bar_x(&mut h, (thumb, groove)).expect("bar drawn");
        let cells = bar_column(&mut h, (thumb, groove));
        let groove_below = cells
            .iter()
            .rfind(|(_, ch)| *ch == groove)
            .map(|(r, _)| *r)
            .expect("groove below thumb");

        h.click(x, groove_below);
        assert!(
            h.text()
                .contains(t(variant_label_key(Variant::TriPeaks)).as_ref())
        );
        let focused = focused_line(&mut h);
        assert!(h.text().contains(&focused));

        let cells = bar_column(&mut h, (thumb, groove));
        let groove_above = cells
            .iter()
            .find(|(_, ch)| *ch == groove)
            .map(|(r, _)| *r)
            .expect("groove above thumb");
        h.click(x, groove_above);
        assert!(
            h.text()
                .contains(t(variant_label_key(Variant::Klondike(DrawCount::One))).as_ref())
        );

        let cells = bar_column(&mut h, (thumb, groove));
        let thumb_top = cells
            .iter()
            .find(|(_, ch)| *ch == thumb)
            .map(|(r, _)| *r)
            .expect("thumb top");
        let bar_bottom = cells.last().map(|(r, _)| *r).expect("bar bottom");
        h.send(crate::input::Input::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: thumb_top,
            modifiers: KeyModifiers::NONE,
        }));
        h.send(crate::input::Input::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: x,
            row: bar_bottom,
            modifiers: KeyModifiers::NONE,
        }));
        h.send(crate::input::Input::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: bar_bottom,
            modifiers: KeyModifiers::NONE,
        }));
        assert!(
            h.text()
                .contains(t(variant_label_key(Variant::TriPeaks)).as_ref())
        );
        let focused = focused_line(&mut h);
        assert!(h.text().contains(&focused));
    }
}

#[test]
fn scrollbar_never_cuts_row_text_in_any_offered_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        for winnable_only in [false, true] {
            let mut h = select_with(
                Settings {
                    locale,
                    winnable_only,
                    ..Settings::default()
                },
                TRUE_COLOR,
                100,
                30,
            );
            let x = bar_x(&mut h, ('█', '░')).expect("bar drawn");
            let gap_x = usize::from(x.saturating_sub(1));
            for (row, _) in bar_column(&mut h, ('█', '░')) {
                let lines = h.lines();
                let line = &lines[usize::from(row)];
                assert_eq!(
                    line.chars().nth(gap_x),
                    Some(' '),
                    "{locale:?} winnable={winnable_only} row={row}: {line:?}"
                );
            }
            for variant in Variant::ALL {
                h.focus_game_variant(variant);
                let name = t(variant_label_key(variant));
                assert!(
                    h.text().contains(name.as_ref()),
                    "{locale:?} missing {variant:?}"
                );
            }
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn families_fold_and_unfold_by_keys_and_mouse_with_count() {
    let mut h = select_30();
    assert!(h.text().contains("[-] Klondike family"));

    h.press(KeyCode::Left);
    let text = h.text();
    assert!(text.contains("[+] Klondike family (2)"), "{text}");
    assert!(!text.contains("Klondike, Draw 1"), "{text}");
    assert_eq!(h.app.settings().folded_families, vec![GameGroup::Klondike]);
    assert_eq!(focused_line(&mut h), "[+] Klondike family (2)");

    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains("[-] Klondike family"), "{text}");
    assert!(text.contains("Klondike, Draw 1"), "{text}");
    assert!(h.app.settings().folded_families.is_empty());

    h.press(KeyCode::Char(' '));
    assert!(h.text().contains("[+] Klondike family (2)"));

    let (col, row) = list_row_of(&mut h, "[+] Klondike family (2)");
    h.click(col, row);
    assert!(h.text().contains("[-] Klondike family"));

    h.press(KeyCode::Down);
    assert!(focused_line(&mut h).contains("Klondike, Draw 1"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[+] Klondike family (2)"));
    assert_eq!(focused_line(&mut h), "[+] Klondike family (2)");
}

#[test]
fn favourites_group_folds_like_a_family_and_folded_set_survives_restart() {
    let mut h = select_with(
        Settings {
            favourites: vec![Variant::FreeCell],
            ..Settings::default()
        },
        TRUE_COLOR,
        100,
        30,
    );
    h.press(KeyCode::Up);
    assert_eq!(focused_line(&mut h), "[-] Favorites");
    h.press(KeyCode::Enter);
    assert!(h.text().contains("[+] Favorites (1)"));
    assert!(!h.text().contains("FreeCell\n"));

    let saved = h.storage.borrow().settings.clone().unwrap_or_default();
    assert_eq!(saved.folded_families, vec![GameGroup::Favourites]);

    let mut restarted = select_with(saved, TRUE_COLOR, 100, 30);
    assert!(restarted.text().contains("[+] Favorites (1)"));
}

#[test]
fn search_opens_matching_folded_families_and_restores_folding_when_cleared() {
    let mut h = select_with(
        Settings {
            folded_families: vec![GameGroup::Yukon],
            ..Settings::default()
        },
        TRUE_COLOR,
        100,
        30,
    );
    assert!(h.text().contains("[+] Yukon family (5)"));
    assert!(!h.text().contains("Alaska"));

    h.type_char('/');
    for ch in "alaska".chars() {
        h.type_char(ch);
    }
    let text = h.text();
    assert!(text.contains("[-] Yukon family"), "{text}");
    assert!(text.contains("Alaska"), "{text}");
    assert_eq!(h.app.settings().folded_families, vec![GameGroup::Yukon]);

    h.press(KeyCode::Esc);
    let after_clear = h.text();
    assert!(
        after_clear.contains("[+] Yukon family (5)"),
        "{after_clear}"
    );
    assert!(!after_clear.contains("Alaska"), "{after_clear}");
    assert_eq!(h.app.settings().folded_families, vec![GameGroup::Yukon]);
}

#[test]
fn old_settings_file_loads_with_every_family_open() {
    let parsed: Settings = toml::from_str("undo_allowed = false\n").unwrap_or_default();
    assert!(parsed.folded_families.is_empty());
    let mut h = select_with(parsed, TRUE_COLOR, 100, 30);
    for family in Family::ALL {
        let heading = format!("[-] {}", t(family.title_key()));
        let mut found = false;
        for _ in 0..3 {
            if h.text().contains(&heading) {
                found = true;
                break;
            }
            h.press(KeyCode::PageDown);
        }
        assert!(found, "expected {heading} to be open");
        h.press(KeyCode::Home);
    }
}

#[test]
fn footer_and_hints_match_what_acts_in_every_focus_state() {
    let mut h = select_30();

    let text = h.text();
    assert!(text.contains("Enter Play"));
    assert!(text.contains("f Favorite"));
    assert!(text.contains("Esc Back"));
    assert!(text.contains("? Help"));
    assert!(text.contains("u Undo"));
    assert!(text.contains("/ Search"));
    assert!(text.contains("w Winnable deals"));

    h.press(KeyCode::Up);
    let text = h.text();
    assert!(text.contains("Enter Fold"), "{text}");
    assert!(!text.contains("Enter Play"), "{text}");
    assert!(!text.contains("f Favorite"), "{text}");
    assert!(
        text.contains(&t("menu.heading_hint").into_owned()),
        "{text}"
    );
    let (col, row) = h.find("Enter Fold").unwrap_or_default();
    h.click(col + 1, row);
    let text = h.text();
    assert!(text.contains("[+] Klondike family (2)"), "{text}");
    assert!(text.contains("Enter Unfold"), "{text}");
    assert!(!text.contains("f Favorite"), "{text}");

    h.press(KeyCode::Tab);
    let text = h.text();
    assert!(text.contains("Enter Change"), "{text}");
    assert!(!text.contains("Enter Play"), "{text}");
    assert!(!text.contains("f Favorite"), "{text}");
    let (col, row) = h.find("Enter Change").unwrap_or_default();
    h.click(col + 1, row);
    assert!(!h.app.settings().undo_allowed);

    h.type_char('/');
    let text = h.text();
    let footer = h.lines().last().cloned().unwrap_or_default();
    assert!(footer.contains("Esc Clear"), "{footer}");
    assert!(footer.contains("F1 Help"), "{footer}");
    assert!(!footer.contains("Esc Back"), "{footer}");
    assert!(!footer.contains("f Favorite"), "{footer}");
    assert!(!footer.contains("u Undo"), "{footer}");
    assert!(!footer.contains("w Winnable deals"), "{footer}");
    assert!(
        !text.contains(&t("menu.heading_hint").into_owned()),
        "{text}"
    );

    for ch in "zzzz".chars() {
        h.type_char(ch);
    }
    let text = h.text();
    assert!(!text.contains("Enter Play"), "{text}");
    assert!(!text.contains("Enter Fold"), "{text}");
    assert!(text.contains("Esc Clear"), "{text}");
    let (col, row) = h.find("Esc Clear").unwrap_or_default();
    h.click(col + 1, row);
    assert!(!h.text().contains("Esc Clear"));
    assert!(h.text().contains("Esc Back"));

    let _ = MouseButton::Left;
}

#[test]
fn folded_and_heading_states_fit_at_100_by_30_in_every_offered_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = select_with(
            Settings {
                locale,
                folded_families: vec![GameGroup::Klondike],
                ..Settings::default()
            },
            TRUE_COLOR,
            100,
            30,
        );
        let klondike = t("menu.family.klondike").into_owned();
        let folded = tf(
            "menu.family_folded",
            &[("title", &klondike), ("count", "2")],
        );
        assert!(h.text().contains(&format!("[+] {folded}")), "{locale:?}");
        h.press(KeyCode::Up);
        h.press(KeyCode::Up);
        let text = h.text();
        assert!(
            text.contains(&format!("Enter {}", t("menu.footer.unfold"))),
            "{locale:?}: {text}"
        );
        assert!(
            text.contains(&t("menu.heading_hint").into_owned()),
            "{locale:?}: {text}"
        );
    }
    set_locale(Locale::EnUs);
}

#[cfg(feature = "dev")]
#[test]
fn dev_snapshot_reports_heading_and_variant_focus() {
    let mut h = select_30();
    let snap = h.app.snapshot();
    let focus = snap.focus.as_ref();
    assert_eq!(focus.map(|f| f.target.as_str()), Some("variant"));
    assert_eq!(
        focus.and_then(|f| f.label.as_deref()),
        Some(t(variant_label_key(Variant::Klondike(DrawCount::One))).as_ref())
    );

    h.press(KeyCode::Up);
    let snap = h.app.snapshot();
    let focus = snap.focus.as_ref();
    assert_eq!(focus.map(|f| f.target.as_str()), Some("heading"));
    assert_eq!(
        focus.and_then(|f| f.label.as_deref()),
        Some(t("menu.family.klondike").as_ref())
    );
}

#[test]
fn before_seed_footer_shows_seed_options_on_variant_and_fold_on_heading() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(h.text().contains("Enter Seed options"));
    assert!(!h.text().contains("w Winnable deals"));
    h.press(KeyCode::Up);
    assert!(h.text().contains("Enter Fold"));
    assert!(!h.text().contains("Enter Seed options"));
    let _ = YukonRules::Classic;
}
