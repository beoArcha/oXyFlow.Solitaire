//! A [`Native`] for tests: it records what was asked and answers as told.

use std::io;
use std::sync::Mutex;

use super::{Flavour, Native, PriorityError};

/// How the fake answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    Accept,
    Deny,
    Refuse,
}

/// What a [`FakeNative`] was asked.
#[derive(Debug, Default)]
pub(crate) struct Calls {
    pub(crate) process: Vec<i32>,
    pub(crate) thread: usize,
}

pub(crate) struct FakeNative {
    flavour: Flavour,
    answer: Answer,
    calls: Mutex<Calls>,
}

impl FakeNative {
    pub(crate) fn new(flavour: Flavour, answer: Answer) -> Self {
        Self {
            flavour,
            answer,
            calls: Mutex::new(Calls::default()),
        }
    }

    pub(crate) fn process_calls(&self) -> Vec<i32> {
        self.calls
            .lock()
            .map(|calls| calls.process.clone())
            .unwrap_or_default()
    }

    pub(crate) fn thread_calls(&self) -> usize {
        self.calls.lock().map_or(0, |calls| calls.thread)
    }

    fn reply(&self) -> Result<(), PriorityError> {
        match self.answer {
            Answer::Accept => Ok(()),
            Answer::Deny => Err(PriorityError::Denied(io::Error::from(
                io::ErrorKind::PermissionDenied,
            ))),
            Answer::Refuse => Err(PriorityError::Refused(io::Error::from(
                io::ErrorKind::InvalidInput,
            ))),
        }
    }
}

impl Native for FakeNative {
    fn flavour(&self) -> Flavour {
        self.flavour
    }

    fn set_process(&self, value: i32) -> Result<(), PriorityError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.process.push(value);
        }
        self.reply()
    }

    fn lower_current_thread(&self) -> Result<(), PriorityError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.thread += 1;
        }
        self.reply()
    }
}
