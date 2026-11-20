//! The priority of the game process and of the solver thread. [`Native`] is the only code that talks
//! to the system (`system`); [`native_value`] is the pure mapping of a level to the number each
//! system takes, and [`ProcessControl`] answers the TUI's `PriorityControl` with it. A level is
//! never real time. On Windows the number is a priority class; on Linux and macOS it is a nice
//! value, where the levels above Normal need privileges and a refusal is a normal result.

use std::io;
use std::sync::Arc;

use oxysol_tui::app::{PriorityControl, PriorityOutcome};
use oxysol_tui::settings::ProcessPriority;
use thiserror::Error;

mod system;

pub use system::SystemPriority;

/// The nice value of the solver thread on Linux: above every level the player can choose (the
/// lowest is 10), so the solver always runs below the UI thread.
pub const SOLVER_NICE: i32 = 15;

const HIGH_PRIORITY_CLASS: i32 = 0x0000_0080;
const ABOVE_NORMAL_PRIORITY_CLASS: i32 = 0x0000_8000;
const NORMAL_PRIORITY_CLASS: i32 = 0x0000_0020;
const BELOW_NORMAL_PRIORITY_CLASS: i32 = 0x0000_4000;
const IDLE_PRIORITY_CLASS: i32 = 0x0000_0040;

/// Why the system did not take a priority.
#[derive(Debug, Error)]
pub enum PriorityError {
    /// The call needs privileges the game does not have.
    #[error("the system wants privileges for this priority")]
    Denied(#[source] io::Error),
    /// The system refused for another reason.
    #[error("the system refused the priority")]
    Refused(#[source] io::Error),
    /// This system has no such call.
    #[error("this system cannot set that priority")]
    Unsupported,
}

/// Which family of numbers a system takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    /// A priority class for `SetPriorityClass`.
    Windows,
    /// A nice value for `setpriority`.
    Unix,
}

/// The number that asks `flavour` for `level`.
pub const fn native_value(flavour: Flavour, level: ProcessPriority) -> i32 {
    match (flavour, level) {
        (Flavour::Windows, ProcessPriority::Highest) => HIGH_PRIORITY_CLASS,
        (Flavour::Windows, ProcessPriority::High) => ABOVE_NORMAL_PRIORITY_CLASS,
        (Flavour::Windows, ProcessPriority::Normal) => NORMAL_PRIORITY_CLASS,
        (Flavour::Windows, ProcessPriority::Low) => BELOW_NORMAL_PRIORITY_CLASS,
        (Flavour::Windows, ProcessPriority::Lowest) => IDLE_PRIORITY_CLASS,
        (Flavour::Unix, ProcessPriority::Highest) => -10,
        (Flavour::Unix, ProcessPriority::High) => -5,
        (Flavour::Unix, ProcessPriority::Normal) => 0,
        (Flavour::Unix, ProcessPriority::Low) => 5,
        (Flavour::Unix, ProcessPriority::Lowest) => 10,
    }
}

/// The system calls behind the priorities. [`SystemPriority`] makes them; tests use a fake.
pub trait Native: Send + Sync {
    /// The numbers this system takes.
    fn flavour(&self) -> Flavour;
    /// Sets the priority of the process to `value` (see [`native_value`]).
    fn set_process(&self, value: i32) -> Result<(), PriorityError>;
    /// Puts the calling thread below the normal priority of the process.
    fn lower_current_thread(&self) -> Result<(), PriorityError>;
}

/// Sets the process priority through [`Native`] for the TUI.
pub struct ProcessControl {
    native: Arc<dyn Native>,
}

impl ProcessControl {
    pub fn new(native: Arc<dyn Native>) -> Self {
        Self { native }
    }
}

impl PriorityControl for ProcessControl {
    fn apply(&mut self, level: ProcessPriority) -> PriorityOutcome {
        let value = native_value(self.native.flavour(), level);
        match self.native.set_process(value) {
            Ok(()) => {
                tracing::debug!(
                    level = level.flag_name(),
                    value,
                    "the process priority is set"
                );
                PriorityOutcome::Applied
            }
            Err(PriorityError::Denied(error)) => {
                tracing::info!(level = level.flag_name(), %error, "the priority needs privileges");
                PriorityOutcome::NeedsPrivileges
            }
            Err(error) => {
                tracing::info!(level = level.flag_name(), %error, "the priority was refused");
                PriorityOutcome::Failed
            }
        }
    }
}

/// Puts the calling thread below the UI thread. A system that cannot do it is not an error: the
/// solver then runs at the priority of the process.
pub fn lower_solver_thread(native: &dyn Native) {
    match native.lower_current_thread() {
        Ok(()) => tracing::debug!("the solver thread runs below the UI thread"),
        Err(error) => tracing::info!(%error, "the solver thread keeps the process priority"),
    }
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
