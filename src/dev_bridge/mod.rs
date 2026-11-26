//! Synchronous JSON-lines developer bridge (`--dev-bridge`, feature `dev-bridge`).
//! Reads one JSON command per line from `stdin` and writes one JSON response per line to `stdout`.

mod input_map;
pub mod log;
pub mod protocol;
#[cfg(test)]
mod tests;

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use oxysol_tui::{Headless, Input, UiSnapshot};
use thiserror::Error;

use self::input_map::{StartTarget, build_start_config, parse_key_event, parse_mouse_inputs};
use self::log::DevLog;
use self::protocol::{
    BridgeEvent, Command, KeyArgs, MouseArgs, ResizeArgs, Response, SnapshotArgs, StartArgs,
    TickArgs, diff_events,
};

/// Fatal I/O or start-up error on the bridge stream or dev log.
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("dev bridge stream I/O failed")]
    Io(#[from] io::Error),
    #[error("cannot open dev log `{path}`")]
    LogOpen {
        path: String,
        #[source]
        source: io::Error,
    },
}

/// Recoverable protocol or command error returned as `ok: false` in the JSON response.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("malformed JSON command: {0}")]
    InvalidJson(String),
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
    #[error("no session started; send `start` first")]
    NotStarted,
    #[error("invalid terminal dimensions {cols}x{rows}")]
    InvalidDimensions { cols: u16, rows: u16 },
    #[error("unknown game `{0}`")]
    InvalidGame(String),
    #[error("unknown color mode `{0}`")]
    InvalidColors(String),
    #[error("unknown language `{0}`")]
    InvalidLocale(String),
    #[error("unknown theme `{0}`")]
    InvalidTheme(String),
    #[error("unknown key `{0}`")]
    InvalidKey(String),
    #[error("unknown modifier `{0}`")]
    InvalidModifier(String),
    #[error("unknown mouse action `{0}`")]
    InvalidMouseAction(String),
    #[error("headless step failed")]
    HeadlessIo(#[from] io::Error),
}

/// Runs the dev bridge synchronously over `stdin` and `stdout`, optionally opening `--dev-log`.
pub fn run(default_seed: Option<u64>, dev_log_path: Option<&Path>) -> Result<(), BridgeError> {
    let mut dev_log = dev_log_path.map(DevLog::open).transpose()?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_with_options(stdin.lock(), stdout.lock(), default_seed, dev_log.as_mut())
}

/// Runs the dev bridge loop over arbitrary buffered `reader` and `writer` streams.
pub fn run_streams<R: BufRead, W: Write>(reader: R, writer: W) -> Result<(), BridgeError> {
    run_with_options(reader, writer, None, None)
}

/// Runs the dev bridge loop with an optional default seed and structured dev log.
pub fn run_with_options<R: BufRead, W: Write>(
    reader: R,
    writer: W,
    default_seed: Option<u64>,
    mut dev_log: Option<&mut DevLog>,
) -> Result<(), BridgeError> {
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut session = BridgeSession::new(default_seed);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let was_started = session.headless.is_some();
        let (response, cmd_name, stop) = session.process_line(trimmed);
        if let Some(log) = dev_log.as_deref_mut() {
            let after = session.snapshot();
            log.record_step(
                trimmed,
                cmd_name.as_deref(),
                was_started,
                after.as_ref(),
                &response,
            )?;
        }
        let json = serde_json::to_string(&response).map_err(io::Error::other)?;
        writeln!(writer, "{json}")?;
        writer.flush()?;
        if stop {
            break;
        }
    }
    Ok(())
}

struct BridgeSession {
    default_seed: Option<u64>,
    headless: Option<Headless>,
}

impl BridgeSession {
    fn new(default_seed: Option<u64>) -> Self {
        Self {
            default_seed,
            headless: None,
        }
    }

    fn frame_id(&self) -> Option<u64> {
        self.headless.as_ref().map(Headless::frame_id)
    }

    fn snapshot(&self) -> Option<UiSnapshot> {
        self.headless.as_ref().map(Headless::snapshot)
    }

    fn process_line(&mut self, line: &str) -> (Response, Option<String>, bool) {
        let cmd = match parse_command(line) {
            Ok(cmd) => cmd,
            Err(err) => {
                let cmd_name = match &err {
                    ProtocolError::UnknownCommand(name) => Some(name.clone()),
                    _ => None,
                };
                return (
                    Response::error(self.frame_id(), err.to_string()),
                    cmd_name,
                    false,
                );
            }
        };
        let cmd_name = Some(command_name(&cmd).to_owned());
        if matches!(cmd, Command::Quit) {
            return (self.handle_quit(), cmd_name, true);
        }
        match self.execute(cmd) {
            Ok(response) => (response, cmd_name, false),
            Err(err) => (
                Response::error(self.frame_id(), err.to_string()),
                cmd_name,
                false,
            ),
        }
    }

    fn execute(&mut self, cmd: Command) -> Result<Response, ProtocolError> {
        match cmd {
            Command::Start(args) => self.handle_start(args),
            Command::Key(args) => self.handle_key(args),
            Command::Mouse(args) => self.handle_mouse(args),
            Command::Resize(args) => self.handle_resize(args),
            Command::Tick(args) => self.handle_tick(args),
            Command::Snapshot(args) => self.handle_snapshot(args),
            Command::Quit => unreachable!(),
        }
    }

    fn handle_start(&mut self, args: StartArgs) -> Result<Response, ProtocolError> {
        let include_text = args.include_text;
        let include_state = args.include_state;
        let (config, target) = build_start_config(&args, self.default_seed)?;
        let mut headless = Headless::new(config)?;
        if let StartTarget::Game(variant) = target {
            headless.start_game(variant, args.seed.or(self.default_seed), true);
        }
        let response = response_of(&headless, Vec::new(), include_text, include_state);
        self.headless = Some(headless);
        Ok(response)
    }

    fn handle_key(&mut self, args: KeyArgs) -> Result<Response, ProtocolError> {
        let key_event = parse_key_event(&args.key, &args.modifiers)?;
        let headless = self.headless_mut()?;
        let before = headless.snapshot();
        headless.step(Input::Key(key_event))?;
        Ok(step_response(
            headless,
            &before,
            Vec::new(),
            args.include_text,
            args.include_state,
        ))
    }

    fn handle_mouse(&mut self, args: MouseArgs) -> Result<Response, ProtocolError> {
        let inputs = parse_mouse_inputs(&args.action, args.x, args.y, &args.modifiers)?;
        let headless = self.headless_mut()?;
        let before = headless.snapshot();
        for input in inputs {
            headless.step(input)?;
        }
        Ok(step_response(
            headless,
            &before,
            Vec::new(),
            args.include_text,
            args.include_state,
        ))
    }

    fn handle_resize(&mut self, args: ResizeArgs) -> Result<Response, ProtocolError> {
        if args.cols == 0 || args.rows == 0 {
            return Err(ProtocolError::InvalidDimensions {
                cols: args.cols,
                rows: args.rows,
            });
        }
        let headless = self.headless_mut()?;
        let before = headless.snapshot();
        headless.resize(args.cols, args.rows)?;
        Ok(step_response(
            headless,
            &before,
            vec![BridgeEvent::Resized {
                cols: args.cols,
                rows: args.rows,
            }],
            args.include_text,
            args.include_state,
        ))
    }

    fn handle_tick(&mut self, args: TickArgs) -> Result<Response, ProtocolError> {
        let headless = self.headless_mut()?;
        let before = headless.snapshot();
        headless.tick(args.ms)?;
        Ok(step_response(
            headless,
            &before,
            Vec::new(),
            args.include_text,
            args.include_state,
        ))
    }

    fn handle_snapshot(&self, args: SnapshotArgs) -> Result<Response, ProtocolError> {
        let headless = self.headless.as_ref().ok_or(ProtocolError::NotStarted)?;
        Ok(response_of(
            headless,
            Vec::new(),
            args.include_text,
            args.include_state,
        ))
    }

    fn handle_quit(&self) -> Response {
        match &self.headless {
            Some(headless) => response_of(headless, vec![BridgeEvent::Quit], false, true),
            None => Response {
                frame_id: None,
                ok: true,
                error: None,
                state: None,
                text_frame: None,
                events: vec![BridgeEvent::Quit],
            },
        }
    }

    fn headless_mut(&mut self) -> Result<&mut Headless, ProtocolError> {
        self.headless.as_mut().ok_or(ProtocolError::NotStarted)
    }
}

fn step_response(
    headless: &Headless,
    before: &UiSnapshot,
    mut extra: Vec<BridgeEvent>,
    include_text: bool,
    include_state: bool,
) -> Response {
    let after = headless.snapshot();
    let mut events = diff_events(before, &after);
    events.append(&mut extra);
    if headless.should_quit() {
        events.push(BridgeEvent::Quit);
    }
    Response {
        frame_id: Some(headless.frame_id()),
        ok: true,
        error: None,
        state: include_state.then_some(after),
        text_frame: include_text.then(|| headless.text_frame()),
        events,
    }
}

fn response_of(
    headless: &Headless,
    events: Vec<BridgeEvent>,
    include_text: bool,
    include_state: bool,
) -> Response {
    Response {
        frame_id: Some(headless.frame_id()),
        ok: true,
        error: None,
        state: include_state.then(|| headless.snapshot()),
        text_frame: include_text.then(|| headless.text_frame()),
        events,
    }
}

fn command_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Start(_) => "start",
        Command::Key(_) => "key",
        Command::Mouse(_) => "mouse",
        Command::Resize(_) => "resize",
        Command::Tick(_) => "tick",
        Command::Snapshot(_) => "snapshot",
        Command::Quit => "quit",
    }
}

fn parse_command(line: &str) -> Result<Command, ProtocolError> {
    let known = [
        "start", "key", "mouse", "resize", "tick", "snapshot", "quit",
    ];
    match serde_json::from_str::<Command>(line) {
        Ok(cmd) => Ok(cmd),
        Err(err) => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line)
                && let Some(cmd_name) = value.get("cmd").and_then(serde_json::Value::as_str)
                && !known.contains(&cmd_name)
            {
                return Err(ProtocolError::UnknownCommand(cmd_name.to_owned()));
            }
            Err(ProtocolError::InvalidJson(err.to_string()))
        }
    }
}
