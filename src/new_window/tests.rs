use std::ffi::OsString;
use std::path::PathBuf;

use super::*;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn os_strings(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn context(os: HostOs, exe: &str, args: &[&str]) -> Context {
    Context {
        os,
        exe: PathBuf::from(exe),
        cwd: PathBuf::from("/work"),
        args: strings(args),
        env: Vec::new(),
        terminal: None,
    }
}

#[test]
fn both_flags_are_removed_and_every_other_argument_keeps_its_place() {
    let raw = strings(&["--ascii", "--new-window", "--seed", "7", "-n", "--colors", "16"]);
    assert_eq!(
        forwarded(&raw),
        strings(&["--ascii", "--seed", "7", "--colors", "16"])
    );
    assert_eq!(forwarded(&strings(&["--new-window"])), Vec::<String>::new());
}

#[test]
fn windows_starts_the_executable_itself_in_a_new_console() {
    let plans = plans(&context(HostOs::Windows, r"C:\Games\oxysol.exe", &["--ascii"]));
    assert_eq!(
        plans,
        vec![Plan {
            program: OsString::from(r"C:\Games\oxysol.exe"),
            args: os_strings(&["--ascii"]),
            new_console: true,
        }]
    );
}

#[test]
fn linux_tries_the_environment_terminal_first_and_then_the_known_ones_in_order() {
    let mut ctx = context(HostOs::Linux, "/opt/oxysol", &["--seed", "5"]);
    ctx.terminal = Some("alacritty".to_owned());
    let plans = plans(&ctx);
    let programs: Vec<String> = plans.iter().map(Plan::label).collect();
    assert_eq!(
        programs,
        strings(&[
            "alacritty",
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xterm"
        ])
    );
    assert_eq!(plans[0].args, os_strings(&["-e", "/opt/oxysol", "--seed", "5"]));
    assert_eq!(plans[2].args, os_strings(&["--", "/opt/oxysol", "--seed", "5"]));
    assert!(plans.iter().all(|plan| !plan.new_console));
}

#[test]
fn linux_skips_an_unset_or_blank_terminal_variable() {
    for terminal in [None, Some(String::new()), Some("  ".to_owned())] {
        let mut ctx = context(HostOs::Linux, "/opt/oxysol", &[]);
        ctx.terminal = terminal;
        let programs: Vec<String> = plans(&ctx).iter().map(Plan::label).collect();
        assert_eq!(programs.first().map(String::as_str), Some("x-terminal-emulator"));
        assert_eq!(programs.len(), 4);
    }
}

#[test]
fn shell_quoting_survives_spaces_and_single_quotes() {
    assert_eq!(shell_quote("plain"), "'plain'");
    assert_eq!(shell_quote("a b"), "'a b'");
    assert_eq!(shell_quote("it's"), "'it'\\''s'");
}

#[test]
fn applescript_escaping_protects_quotes_and_backslashes() {
    assert_eq!(applescript_escape(r#"a"b\c"#), r#"a\"b\\c"#);
}

#[test]
fn the_macos_shell_line_changes_directory_sets_variables_and_replaces_the_shell() {
    let mut ctx = context(HostOs::MacOs, "/Applications/My Games/oxysol", &["--seed", "9"]);
    ctx.env = vec![("OXYSOL_HOME".to_owned(), "/tmp/o x".to_owned())];
    assert_eq!(
        shell_line(&ctx),
        "cd '/work' && OXYSOL_HOME='/tmp/o x' exec '/Applications/My Games/oxysol' '--seed' '9'"
    );
}

#[test]
fn macos_asks_terminal_to_run_the_quoted_line_through_osascript() {
    let ctx = context(HostOs::MacOs, "/Apps/it's/oxysol", &[]);
    let plans = plans(&ctx);
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].program, OsString::from("osascript"));
    assert_eq!(
        plans[0].args,
        os_strings(&[
            "-e",
            "tell application \"Terminal\" to do script \"cd '/work' && exec '/Apps/it'\\\\''s/oxysol'\"",
            "-e",
            "tell application \"Terminal\" to activate",
        ])
    );
}

#[test]
fn the_failure_names_everything_that_was_tried_and_keeps_the_last_cause() {
    let cause = io::Error::new(io::ErrorKind::NotFound, "missing");
    let error = NewWindowError::Unavailable {
        tried: strings(&["xterm", "konsole"]),
        last: Some(cause),
    };
    assert_eq!(error.tried(), "xterm, konsole");
    assert!(error.to_string().contains("xterm, konsole"));
    assert!(std::error::Error::source(&error).is_some());
}
