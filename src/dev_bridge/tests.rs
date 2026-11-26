use std::io::Cursor;

use oxysol_tui::crossterm::event::{KeyCode, KeyModifiers};
use oxysol_tui::{Input, OverlayKind, ScreenKind};

use super::input_map::{parse_key_event, parse_mouse_inputs};
use super::run_streams;

fn drive(lines: &[&str]) -> Vec<serde_json::Value> {
    let input = lines.join("\n");
    let mut output = Vec::new();
    run_streams(Cursor::new(input.as_bytes()), &mut output).unwrap();
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn key_mapping_handles_named_keys_chars_and_modifiers() {
    let enter = parse_key_event("Enter", &[]).unwrap();
    assert_eq!(enter.code, KeyCode::Enter);
    assert_eq!(enter.modifiers, KeyModifiers::NONE);

    let shift_tab = parse_key_event("Tab", &["shift".to_owned()]).unwrap();
    assert_eq!(shift_tab.code, KeyCode::BackTab);

    let shift_r = parse_key_event("r", &["shift".to_owned()]).unwrap();
    assert_eq!(shift_r.code, KeyCode::Char('R'));

    let ctrl_z = parse_key_event("z", &["ctrl".to_owned()]).unwrap();
    assert_eq!(ctrl_z.code, KeyCode::Char('z'));
    assert_eq!(ctrl_z.modifiers, KeyModifiers::CONTROL);

    assert!(parse_key_event("NotAKey", &[]).is_err());
    assert!(parse_key_event("Enter", &["hyper".to_owned()]).is_err());
}

#[test]
fn mouse_mapping_splits_click_and_double_click() {
    let click = parse_mouse_inputs("click", 4, 5, &[]).unwrap();
    assert_eq!(click.len(), 2);
    assert!(matches!(click[0], Input::Mouse(_)));

    let dbl = parse_mouse_inputs("double_click", 4, 5, &[]).unwrap();
    assert_eq!(dbl.len(), 4);

    assert!(parse_mouse_inputs("triple_click", 4, 5, &[]).is_err());
}

#[test]
fn malformed_and_unknown_commands_keep_loop_alive() {
    let responses = drive(&[
        "{not valid json",
        r#"{"cmd":"snapshot"}"#,
        r#"{"cmd":"start","game":"menu"}"#,
        r#"{"cmd":"bogus"}"#,
        r#"{"cmd":"key","key":"?"}"#,
        r#"{"cmd":"quit"}"#,
    ]);
    assert_eq!(responses.len(), 6);
    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[1]["ok"], false);
    assert_eq!(responses[2]["ok"], true);
    assert_eq!(responses[3]["ok"], false);
    assert_eq!(responses[4]["ok"], true);
    assert_eq!(
        responses[4]["state"]["active_overlay"],
        serde_json::to_value(OverlayKind::Help).unwrap()
    );
    assert_eq!(responses[5]["ok"], true);
    assert_eq!(
        responses[5]["state"]["screen"],
        serde_json::to_value(ScreenKind::MainMenu).unwrap()
    );
}

#[test]
fn eof_ends_the_loop_cleanly() {
    let responses = drive(&[r#"{"cmd":"start"}"#]);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["ok"], true);
}
