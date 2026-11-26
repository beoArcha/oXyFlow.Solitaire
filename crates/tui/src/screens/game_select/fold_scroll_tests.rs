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
    marked_text(&h.lines(), "►").unwrap_or_default()
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
    assert!(h.text().contains("↓"));

    wheel(&mut h, MouseEventKind::ScrollDown, col, row);
    let text = h.text();
    assert!(!text.contains("[-] Favorites"), "{text}");
    assert!(text.contains("↑"), "{text}");
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

    h.press(KeyCode::PageDown);
    assert!(!focused_line(&mut h).contains(&first));
    let mid = focused_line(&mut h);

    h.press(KeyCode::End);
    assert!(focused_line(&mut h).contains(&last));

    h.press(KeyCode::PageUp);
    let after_pgup = focused_line(&mut h);
    assert!(!after_pgup.contains(&last));
    assert_ne!(after_pgup, mid);

    h.press(KeyCode::Home);
    assert!(focused_line(&mut h).contains(&first));
}

fn edge_mark_cell(h: &mut Harness, mark: &str) -> (u16, u16) {
    h.lines()
        .iter()
        .enumerate()
        .find_map(|(row, line)| {
            let trimmed = line.trim_end();
            let prefix = trimmed.strip_suffix(mark)?;
            if !prefix.ends_with("  ") {
                return None;
            }
            let col = trimmed.chars().count().checked_sub(1)?;
            Some((u16::try_from(col).ok()?, u16::try_from(row).ok()?))
        })
        .unwrap_or_default()
}

#[test]
fn clicking_the_edge_marks_scrolls_by_one_page_in_unicode_and_ascii() {
    for (caps, up_mark, down_mark) in [(TRUE_COLOR, "↑", "↓"), (PLAIN, "^", "v")] {
        let mut h = select_with(Settings::for_caps(&caps), caps, 100, 30);
        let (col, row) = edge_mark_cell(&mut h, down_mark);
        assert!(row > 0, "down mark {down_mark} missing");
        h.click(col, row);
        let text = h.text();
        assert!(text.contains(up_mark), "{text}");
        assert!(
            text.contains(t(variant_label_key(Variant::TriPeaks)).as_ref()),
            "{text}"
        );
        let (col, row) = edge_mark_cell(&mut h, up_mark);
        h.click(col, row);
        assert!(
            h.text()
                .contains(t(variant_label_key(Variant::Klondike(DrawCount::One))).as_ref())
        );
    }
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
