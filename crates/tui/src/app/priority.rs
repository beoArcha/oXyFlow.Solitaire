//! The process priority: the contract the application implements, the state that remembers what
//! was asked, and how a refusal reaches the player. The levels are [`ProcessPriority`]; the flag
//! `--priority` holds until the player changes the setting.

use super::App;
use crate::settings::ProcessPriority;

/// What came of asking the system for a priority.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriorityOutcome {
    /// The system took the level, or nothing was asked yet.
    #[default]
    Applied,
    /// Raising the priority needs privileges the game does not have; the priority stays as it was.
    NeedsPrivileges,
    /// The system refused for another reason.
    Failed,
}

impl PriorityOutcome {
    /// Localization key of the note shown beside the value, `None` when all is well.
    pub const fn note_key(self) -> Option<&'static str> {
        match self {
            PriorityOutcome::Applied => None,
            PriorityOutcome::NeedsPrivileges => Some("settings.value.needs_privileges"),
            PriorityOutcome::Failed => Some("settings.value.refused"),
        }
    }
}

/// Sets the priority of the game process. The application (`oxysol`) implements it with the
/// system calls; the TUI never touches them.
pub trait PriorityControl {
    /// Asks the system for `level`. A refusal is a result, not an error.
    fn apply(&mut self, level: ProcessPriority) -> PriorityOutcome;
}

/// A control that changes nothing, for tests and platforms without one.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullPriority;

impl PriorityControl for NullPriority {
    fn apply(&mut self, _level: ProcessPriority) -> PriorityOutcome {
        PriorityOutcome::Applied
    }
}

/// The control and the flag, as `DisplayDeps` carries them.
pub struct PriorityDeps {
    pub control: Box<dyn PriorityControl>,
    /// `--priority <level>`: it holds until the player changes the setting.
    pub flag: Option<ProcessPriority>,
}

impl PriorityDeps {
    /// A control that never changes anything and no flag.
    pub fn none() -> Self {
        Self {
            control: Box::new(NullPriority),
            flag: None,
        }
    }
}

/// The priority state of a running [`App`].
pub(super) struct PriorityState {
    control: Box<dyn PriorityControl>,
    flag: Option<ProcessPriority>,
    asked: Option<ProcessPriority>,
    pub(super) outcome: PriorityOutcome,
}

impl PriorityState {
    pub(super) fn new(deps: PriorityDeps) -> Self {
        Self {
            control: deps.control,
            flag: deps.flag,
            asked: None,
            outcome: PriorityOutcome::Applied,
        }
    }
}

impl App {
    /// Brings the process priority in line with the flag or the setting. A level is asked of the
    /// system once; a refusal stays the outcome the Settings row shows, so nothing repeats per frame.
    pub(super) fn sync_priority(&mut self) {
        let state = &mut self.priority;
        let wanted = state.flag.unwrap_or(self.settings.process_priority);
        if state.asked == Some(wanted) {
            return;
        }
        state.asked = Some(wanted);
        state.outcome = state.control.apply(wanted);
    }

    /// The player changed the setting: a flag was only for the start, so it stops applying.
    pub(super) fn priority_setting_changed(&mut self) {
        self.priority.flag = None;
        self.sync_priority();
    }
}
