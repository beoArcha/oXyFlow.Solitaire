//! Relaunching the game in a new console window (`--new-window`, decision T-28). The commands to
//! try are built by pure functions of a [`Context`]; only [`open`] starts a process.

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use thiserror::Error;

/// Flags that ask for a new window; both are removed before the arguments are forwarded.
pub const FLAGS: [&str; 2] = ["--new-window", "-n"];

/// Environment variables of the application that a fresh shell on macOS must be given again.
pub const FORWARDED_ENV: [&str; 2] = ["OXYSOL_HOME", "OXYSOL_LOG"];

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

const TERMINAL_VARIABLE: &str = "TERMINAL";
const DASH_E: &str = "-e";
const DOUBLE_DASH: &str = "--";
const OSASCRIPT: &str = "osascript";

/// The new window could not be opened.
#[derive(Debug, Error)]
pub enum NewWindowError {
    #[error("the path of the running executable is unknown")]
    CurrentExe(#[source] io::Error),
    #[error("the working directory is unknown")]
    CurrentDir(#[source] io::Error),
    #[error("no console window could be started (tried: {})", .tried.join(", "))]
    Unavailable {
        tried: Vec<String>,
        #[source]
        last: Option<io::Error>,
    },
}

impl NewWindowError {
    /// What was tried, for the message shown to the player: the programs, comma separated.
    pub fn tried(&self) -> String {
        match self {
            Self::Unavailable { tried, .. } => tried.join(", "),
            Self::CurrentExe(_) | Self::CurrentDir(_) => String::new(),
        }
    }
}

/// The operating system family that decides how a window is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOs {
    Windows,
    Linux,
    MacOs,
}

impl HostOs {
    /// The family this binary was built for.
    pub const fn current() -> Self {
        if cfg!(windows) {
            HostOs::Windows
        } else if cfg!(target_os = "macos") {
            HostOs::MacOs
        } else {
            HostOs::Linux
        }
    }
}

/// Everything the commands are built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub os: HostOs,
    pub exe: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub terminal: Option<String>,
}

impl Context {
    /// The context of this process; `raw_args` are the arguments as typed, without the program.
    pub fn current(raw_args: &[String]) -> Result<Self, NewWindowError> {
        Ok(Self {
            os: HostOs::current(),
            exe: std::env::current_exe().map_err(NewWindowError::CurrentExe)?,
            cwd: std::env::current_dir().map_err(NewWindowError::CurrentDir)?,
            args: forwarded(raw_args),
            env: FORWARDED_ENV
                .iter()
                .filter_map(|name| Some((name.to_string(), std::env::var(name).ok()?)))
                .collect(),
            terminal: std::env::var(TERMINAL_VARIABLE).ok(),
        })
    }
}

/// One command to try: a program, its arguments, and whether Windows should give it a new console.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub new_console: bool,
}

impl Plan {
    fn label(&self) -> String {
        self.program.to_string_lossy().into_owned()
    }
}

/// `args` without the flags that ask for a new window.
pub fn forwarded(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|arg| !FLAGS.contains(&arg.as_str()))
        .cloned()
        .collect()
}

fn with_separator(program: &str, separator: &str, context: &Context) -> Plan {
    let mut args = vec![
        OsString::from(separator),
        context.exe.clone().into_os_string(),
    ];
    args.extend(context.args.iter().map(OsString::from));
    Plan {
        program: OsString::from(program),
        args,
        new_console: false,
    }
}

fn linux_plans(context: &Context) -> Vec<Plan> {
    let from_environment = context
        .terminal
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .map(|name| with_separator(name, DASH_E, context));
    let others = [
        ("x-terminal-emulator", DASH_E),
        ("gnome-terminal", DOUBLE_DASH),
        ("konsole", DASH_E),
        ("xterm", DASH_E),
    ]
    .map(|(program, separator)| with_separator(program, separator, context));
    from_environment.into_iter().chain(others).collect()
}

/// Quotes `word` for a POSIX shell.
pub fn shell_quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// Escapes `text` for the inside of an AppleScript string literal.
pub fn applescript_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The shell line that Terminal runs: the same directory, the same variables, then the game in
/// place of the shell, so the window closes when the game exits cleanly.
pub fn shell_line(context: &Context) -> String {
    let mut words = vec![format!(
        "cd {} &&",
        shell_quote(&context.cwd.to_string_lossy())
    )];
    words.extend(
        context
            .env
            .iter()
            .map(|(name, value)| format!("{name}={}", shell_quote(value))),
    );
    words.push("exec".to_owned());
    words.push(shell_quote(&context.exe.to_string_lossy()));
    words.extend(context.args.iter().map(|arg| shell_quote(arg)));
    words.join(" ")
}

fn macos_plans(context: &Context) -> Vec<Plan> {
    let script = format!(
        "tell application \"Terminal\" to do script \"{}\"",
        applescript_escape(&shell_line(context))
    );
    vec![Plan {
        program: OsString::from(OSASCRIPT),
        args: vec![
            OsString::from("-e"),
            OsString::from(script),
            OsString::from("-e"),
            OsString::from("tell application \"Terminal\" to activate"),
        ],
        new_console: false,
    }]
}

fn windows_plans(context: &Context) -> Vec<Plan> {
    vec![Plan {
        program: context.exe.clone().into_os_string(),
        args: context.args.iter().map(OsString::from).collect(),
        new_console: true,
    }]
}

/// The commands to try, in order, until one starts.
pub fn plans(context: &Context) -> Vec<Plan> {
    match context.os {
        HostOs::Windows => windows_plans(context),
        HostOs::Linux => linux_plans(context),
        HostOs::MacOs => macos_plans(context),
    }
}

fn command_of(plan: &Plan) -> Command {
    let mut command = Command::new(&plan.program);
    command.args(&plan.args);
    #[cfg(windows)]
    if plan.new_console {
        std::os::windows::process::CommandExt::creation_flags(&mut command, CREATE_NEW_CONSOLE);
    }
    #[cfg(not(windows))]
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    command
}

/// Starts the first plan that works and returns at once; the new window outlives this process.
pub fn open(context: &Context) -> Result<(), NewWindowError> {
    let mut tried = Vec::new();
    let mut last = None;
    for plan in plans(context) {
        match command_of(&plan).spawn() {
            Ok(_child) => return Ok(()),
            Err(error) => {
                tracing::debug!(program = %plan.label(), %error, "the console window cannot be started");
                tried.push(plan.label());
                last = Some(error);
            }
        }
    }
    Err(NewWindowError::Unavailable { tried, last })
}

#[cfg(test)]
mod tests;
