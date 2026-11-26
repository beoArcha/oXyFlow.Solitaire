use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

fn run_bridge_process(lines: &[&str]) -> Result<(i32, Vec<Value>), String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxysol"))
        .arg("--dev-bridge")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn --dev-bridge: {err}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        for line in lines {
            writeln!(stdin, "{line}").map_err(|err| format!("write stdin: {err}"))?;
        }
    } else {
        return Err("stdin not piped".to_owned());
    }

    let output = child
        .wait_with_output()
        .map_err(|err| format!("wait child: {err}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
    let stdout = String::from_utf8(output.stdout).map_err(|err| format!("utf8 stdout: {err}"))?;
    let parsed = stdout
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .map_err(|err| format!("invalid JSON line {line:?}: {err}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((output.status.code().unwrap_or(-1), parsed))
}

fn find_cell(text_frame: &str, needle: &str) -> Option<(u16, u16)> {
    text_frame.lines().enumerate().find_map(|(row, line)| {
        let byte = line.find(needle)?;
        let col = line[..byte].chars().count();
        Some((u16::try_from(col).ok()?, u16::try_from(row).ok()?))
    })
}

fn has_event(response: &Value, kind: &str) -> bool {
    response["events"]
        .as_array()
        .is_some_and(|events| events.iter().any(|ev| ev["kind"] == kind))
}

#[test]
fn dev_bridge_drives_menus_seeded_klondike_keyboard_mouse_resize_and_quit() -> Result<(), String> {
    let (code, responses) = run_bridge_process(&[
        r#"{"cmd":"start","cols":120,"rows":40,"game":"menu","ascii":false,"colors":"truecolor","lang":"en-US","theme":"dark"}"#,
        r#"{"cmd":"key","key":"?"}"#,
        r#"{"cmd":"key","key":"Esc"}"#,
        r#"{"cmd":"key","key":"Down"}"#,
        r#"{"cmd":"key","key":"Down"}"#,
        r#"{"cmd":"key","key":"Down"}"#,
        r#"{"cmd":"key","key":"Enter"}"#,
        r#"{"cmd":"key","key":"Esc"}"#,
        r#"{"cmd":"key","key":"Up"}"#,
        r#"{"cmd":"key","key":"Up"}"#,
        r#"{"cmd":"key","key":"Up"}"#,
        r#"{"cmd":"key","key":"Enter"}"#,
        r#"{"cmd":"key","key":"Esc"}"#,
        r#"{"cmd":"start","cols":120,"rows":40,"game":"klondike","seed":12345,"include_text":true}"#,
        r#"{"cmd":"key","key":"s"}"#,
        r#"{"cmd":"key","key":"a"}"#,
        r#"{"cmd":"resize","cols":80,"rows":24,"include_text":true}"#,
        r#"{"cmd":"tick","ms":1000}"#,
        r#"{"cmd":"snapshot","include_text":true,"include_state":true}"#,
        r#"{"cmd":"quit"}"#,
    ])?;

    assert_eq!(code, 0);
    assert_eq!(responses.len(), 20);
    assert!(responses.iter().all(|r| r["ok"] == true));

    assert_eq!(responses[0]["frame_id"], 0);
    assert_eq!(responses[0]["state"]["screen"], "main_menu");
    assert_eq!(responses[1]["state"]["active_overlay"], "help");
    assert_eq!(responses[2]["state"]["active_overlay"], Value::Null);
    assert_eq!(responses[6]["state"]["screen"], "settings");
    assert_eq!(responses[11]["state"]["screen"], "game_select");

    let klondike = &responses[13];
    assert_eq!(klondike["frame_id"], 0);
    assert_eq!(klondike["state"]["screen"], "board");
    assert_eq!(klondike["state"]["game_stats"]["seed"], 12345);
    assert_eq!(klondike["state"]["game_stats"]["moves"], 0);
    let board_text = klondike["text_frame"]
        .as_str()
        .ok_or_else(|| "missing klondike text_frame".to_owned())?;

    let after_stock = &responses[14];
    assert_eq!(after_stock["state"]["game_stats"]["moves"], 1);
    assert!(has_event(after_stock, "move_applied"));

    let after_resize = &responses[16];
    let resized_text = after_resize["text_frame"]
        .as_str()
        .ok_or_else(|| "missing resized text_frame".to_owned())?;
    assert!(resized_text.contains("Terminal too small"));
    assert!(has_event(after_resize, "resized"));

    assert_eq!(responses[17]["state"]["game_stats"]["elapsed_ms"], 1000);
    assert!(responses[18]["text_frame"].is_string());
    assert!(responses[18]["state"].is_object());
    assert!(has_event(&responses[19], "quit"));

    let (x, y) = find_cell(board_text, "x24")
        .map(|(col, row)| (col, row.saturating_sub(1)))
        .ok_or_else(|| format!("stock counter not found in board:\n{board_text}"))?;
    let mouse_cmd =
        format!(r#"{{"cmd":"mouse","action":"click","x":{x},"y":{y},"include_state":true}}"#);
    let (mouse_code, mouse_responses) = run_bridge_process(&[
        r#"{"cmd":"start","cols":120,"rows":40,"game":"klondike","seed":12345}"#,
        &mouse_cmd,
        r#"{"cmd":"mouse","action":"move","x":10,"y":5}"#,
        r#"{"cmd":"mouse","action":"double_click","x":10,"y":10}"#,
        r#"{"cmd":"mouse","action":"right_click","x":10,"y":10}"#,
        r#"{"cmd":"quit"}"#,
    ])?;
    assert_eq!(mouse_code, 0);
    assert_eq!(mouse_responses.len(), 6);
    assert!(mouse_responses.iter().all(|r| r["ok"] == true));
    assert_eq!(mouse_responses[1]["state"]["game_stats"]["moves"], 1);
    assert_eq!(mouse_responses[2]["state"]["hover"]["x"], 10);
    assert_eq!(mouse_responses[2]["state"]["hover"]["y"], 5);
    Ok(())
}

#[test]
fn dev_bridge_handles_malformed_json_unknown_commands_and_illegal_move_events() -> Result<(), String>
{
    let (code, responses) = run_bridge_process(&[
        "{bad json",
        r#"{"cmd":"unknown_cmd"}"#,
        r#"{"cmd":"key","key":"Enter"}"#,
        r#"{"cmd":"start","cols":120,"rows":40,"game":"klondike","seed":12345}"#,
        r#"{"cmd":"key","key":"u"}"#,
        r#"{"cmd":"quit"}"#,
    ])?;
    assert_eq!(code, 0);
    assert_eq!(responses.len(), 6);
    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[1]["ok"], false);
    assert_eq!(responses[2]["ok"], false);
    assert_eq!(responses[3]["ok"], true);
    assert_eq!(responses[4]["ok"], true);
    assert!(responses[4]["state"]["last_error"].is_string());
    assert!(has_event(&responses[4], "move_rejected"));
    assert_eq!(responses[5]["ok"], true);
    Ok(())
}
