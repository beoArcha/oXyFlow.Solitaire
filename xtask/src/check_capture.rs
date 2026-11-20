//! Runs one cargo step: its output goes to the terminal as it is produced and the last lines are kept.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::error::XtaskError;

pub const TAIL_LINES: usize = 30;

pub struct Captured {
    pub passed: bool,
    pub code: Option<i32>,
    pub tail: Vec<String>,
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

pub fn run_cargo(cargo: &str, root: &Path, args: &[&str]) -> Result<Captured, XtaskError> {
    let mut child = Command::new(cargo)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| XtaskError::Spawn {
            program: cargo.to_owned(),
            source,
        })?;
    let (sender, receiver) = mpsc::channel();
    if let Some(stdout) = child.stdout.take() {
        spawn_reader(stdout, Stream::Stdout, sender.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_reader(stderr, Stream::Stderr, sender.clone());
    }
    drop(sender);
    let mut tail = VecDeque::with_capacity(TAIL_LINES);
    for (stream, line) in receiver {
        match stream {
            Stream::Stdout => println!("{line}"),
            Stream::Stderr => eprintln!("{line}"),
        }
        if tail.len() == TAIL_LINES {
            tail.pop_front();
        }
        tail.push_back(line);
    }
    let status = child.wait().map_err(|source| XtaskError::Wait {
        program: cargo.to_owned(),
        source,
    })?;
    Ok(Captured {
        passed: status.success(),
        code: status.code(),
        tail: tail.into(),
    })
}

fn spawn_reader<R: Read + Send + 'static>(
    pipe: R,
    stream: Stream,
    sender: Sender<(Stream, String)>,
) {
    thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            if sender.send((stream, line)).is_err() {
                break;
            }
        }
    });
}
