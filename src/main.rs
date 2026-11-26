//! Entry point of the `oxysol` binary: arguments, paths, logging, the panic hook, storage, and
//! then the terminal interface.

use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::Arc;

use oxysol::about;
use oxysol::audio;
use oxysol::cli::{self, Cli, ColorArg};
use oxysol::clipboard::SystemClipboard;
use oxysol::console::font::{ClassicConsole, FontZoom};
use oxysol::console::{self, display};
use oxysol::error::AppError;
use oxysol::paths::AppPaths;
use oxysol::priority::{Native, ProcessControl, SystemPriority};
use oxysol::solver::ThreadSolver;
use oxysol::storage::SqliteStorage;
use oxysol::theme::load_user_theme;
use oxysol::{logging, new_window, panic_hook, seed};
use oxysol_tui::app::{ConsoleFont, DisplayDeps, DisplayFacts, NullConsoleFont, PriorityDeps};
use oxysol_tui::caps::{self, ColorDepth, Platform, TerminalCaps};
use oxysol_tui::i18n::{self, Locale};
use oxysol_tui::settings::TerminalKind;
use oxysol_tui::{Deps, NullSolver, Solver, Zone};
use time::UtcOffset;
use time::error::IndeterminateOffset;

fn main() -> ExitCode {
    i18n::set_locale(locale_from_env());
    let raw: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(raw.iter().cloned()) {
        Ok(args) => {
            #[cfg(feature = "dev-bridge")]
            if !args.show_help && !args.show_version && args.dev_bridge {
                return run_dev_bridge(&args);
            }
            finish(launch(&args, &raw))
        }
        Err(error) => finish(Err(AppError::from(error))),
    }
}

fn finish(outcome: Result<(), AppError>) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let code = report(&error);
            if console::should_pause(true, console::owns_window(), console::stdin_is_terminal()) {
                console::wait_for_enter();
            }
            code
        }
    }
}

#[cfg(feature = "dev-bridge")]
fn run_dev_bridge(args: &Cli) -> ExitCode {
    use std::error::Error as _;

    match oxysol::dev_bridge::run(args.seed, args.dev_log.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let message = match error.source() {
                Some(source) => format!("{error}: {source}"),
                None => error.to_string(),
            };
            write_stderr(&message);
            ExitCode::FAILURE
        }
    }
}

fn launch(args: &Cli, raw: &[String]) -> Result<(), AppError> {
    if args.show_help {
        return print_stdout(&i18n::t("cli.help"));
    }
    if args.show_version {
        return print_stdout(&about::version_text());
    }
    if args.new_window {
        return relaunch(raw);
    }
    start(args)
}

fn relaunch(raw: &[String]) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    let level = std::env::var(logging::LEVEL_VARIABLE).ok();
    let _log = start_logging(&paths, level.as_deref());
    let context = new_window::Context::current(raw)?;
    tracing::info!(exe = %context.exe.display(), "opening oXySol in a new console window");
    new_window::open(&context)?;
    Ok(())
}

fn start_logging(paths: &AppPaths, level: Option<&str>) -> Option<File> {
    match logging::start(paths, level) {
        Ok(file) => Some(file),
        Err(_) => {
            let path = paths.log_file().display().to_string();
            write_stderr(&i18n::tf("cli.log_unavailable", &[("path", path.as_str())]));
            None
        }
    }
}

fn start(args: &Cli) -> Result<(), AppError> {
    let local_offset = read_local_offset();
    let paths = AppPaths::detect()?;
    let level = std::env::var(logging::LEVEL_VARIABLE).ok();
    let diagnostics = start_logging(&paths, level.as_deref());
    panic_hook::install(paths.log_file(), diagnostics);
    if console::owns_window() {
        console::widen_owned_window();
    }
    tracing::info!(
        version = env!("OXYSOL_VERSION"),
        data_dir = %paths.data_dir().display(),
        "oXySol starting"
    );
    if let Some(seed) = args.seed {
        tracing::info!(seed, "seed given on the command line");
    }
    if let Some(level) = args.priority {
        tracing::info!(
            level = level.flag_name(),
            "priority given on the command line"
        );
    }
    panic_hook::panic_if_requested();
    let zone = zone_of(local_offset);
    let storage = SqliteStorage::open(&paths).map_err(|error| {
        tracing::error!(target: "oxysol::critical", %error, "the database cannot be opened");
        AppError::from(error)
    })?;
    let caps = terminal_caps(args);
    let user_theme = load_user_theme(&paths);
    let native: Arc<dyn Native> = Arc::new(SystemPriority);
    let deps = Deps {
        storage: Box::new(storage),
        audio: audio::create(&caps),
        clipboard: Box::new(SystemClipboard::new(caps.remote)),
        caps,
        seeds: Box::new(seed::fresh),
        start_seed: args.seed,
        sound_styles: audio::bank::style_list(),
        user_theme,
        solver: start_solver(Arc::clone(&native)),
        display: display_deps(args, native),
        version: env!("OXYSOL_VERSION"),
        zone,
    };
    oxysol_tui::run(deps).map_err(|error| {
        tracing::error!(%error, "the terminal interface stopped");
        AppError::from(error)
    })
}

/// The system's offset from UTC. It is read first in `start`, before any thread exists, because the
/// `time` crate refuses to read the local offset on some Unix systems once threads are running.
fn read_local_offset() -> Result<UtcOffset, IndeterminateOffset> {
    UtcOffset::current_local_offset()
}

/// The zone of the statistics history; without a known offset the history stays in UTC, and the
/// heading says so.
fn zone_of(offset: Result<UtcOffset, IndeterminateOffset>) -> Zone {
    match offset {
        Ok(offset) => Zone::Local {
            minutes: i32::from(offset.whole_minutes()),
        },
        Err(error) => {
            tracing::info!(%error, "the local UTC offset is unknown; the statistics stay in UTC");
            Zone::Utc
        }
    }
}

fn display_deps(args: &Cli, native: Arc<dyn Native>) -> DisplayDeps {
    let kind = display::terminal_kind(platform(), &|key: &str| std::env::var(key).ok());
    DisplayDeps {
        display: Box::new(display::ConsoleWindow::default()),
        facts: DisplayFacts {
            owns_window: console::owns_window(),
            kind,
            flag: args.full_screen,
        },
        font: console_font(kind),
        priority: PriorityDeps {
            control: Box::new(ProcessControl::new(native)),
            flag: args.priority,
        },
    }
}

/// The font zoom of the classic Windows console; every other terminal zooms its font by itself.
fn console_font(kind: TerminalKind) -> Box<dyn ConsoleFont> {
    if kind != TerminalKind::ClassicConsole {
        return Box::new(NullConsoleFont);
    }
    match FontZoom::new(Box::new(ClassicConsole)) {
        Some(zoom) => Box::new(zoom),
        None => Box::new(NullConsoleFont),
    }
}

fn start_solver(native: Arc<dyn Native>) -> Box<dyn Solver> {
    match ThreadSolver::spawn_with(native) {
        Ok(solver) => Box::new(solver),
        Err(error) => {
            tracing::warn!(%error, "no solver thread; deals are never proven winnable");
            Box::new(NullSolver::default())
        }
    }
}

fn terminal_caps(args: &Cli) -> TerminalCaps {
    let mut caps = caps::detect(&|key: &str| std::env::var(key).ok(), platform());
    if args.ascii {
        caps.unicode = false;
    }
    if let Some(color) = args.colors {
        caps.color = color_depth(color);
    }
    caps
}

fn platform() -> Platform {
    if cfg!(windows) {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOs
    } else {
        Platform::Linux
    }
}

fn color_depth(color: ColorArg) -> ColorDepth {
    match color {
        ColorArg::Ansi16 => ColorDepth::Ansi16,
        ColorArg::Ansi256 => ColorDepth::Ansi256,
        ColorArg::TrueColor => ColorDepth::TrueColor,
    }
}

fn locale_from_env() -> Locale {
    let value = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()));
    locale_for(value.as_deref())
}

fn locale_for(value: Option<&str>) -> Locale {
    value
        .and_then(Locale::from_language_tag)
        .map_or(Locale::EnUs, Locale::or_source)
}

fn print_stdout(text: &str) -> Result<(), AppError> {
    writeln!(io::stdout(), "{text}")?;
    Ok(())
}

fn write_stderr(text: &str) {
    if let Err(write_error) = writeln!(io::stderr(), "{text}") {
        tracing::warn!(%write_error, "stderr cannot be written");
    }
}

fn report(error: &AppError) -> ExitCode {
    let text = match error {
        AppError::Arguments(cause) => {
            let detail = cause.to_string();
            i18n::tf("cli.usage_error", &[("error", detail.as_str())])
        }
        AppError::NewWindow(cause) => {
            let tried = cause.tried();
            i18n::tf("cli.new_window_failed", &[("tried", tried.as_str())])
        }
        AppError::Paths(_) => i18n::t("cli.paths_failed").into_owned(),
        AppError::Storage(_) => i18n::t("cli.storage_failed").into_owned(),
        AppError::Terminal(_) => i18n::t("cli.terminal_failed").into_owned(),
        AppError::Output(_) => i18n::t("cli.output_failed").into_owned(),
    };
    write_stderr(&text);
    error.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_selects_a_complete_locale_by_language_else_english() {
        assert_eq!(locale_for(Some("pl_PL.UTF-8")), Locale::PlPl);
        assert_eq!(locale_for(Some("en_US.UTF-8")), Locale::EnUs);
        assert_eq!(locale_for(Some("en_GB")), Locale::EnUs);
        assert_eq!(locale_for(Some("C")), Locale::EnUs);
        assert_eq!(locale_for(None), Locale::EnUs);
        for (tag, locale) in [
            ("de_AT.UTF-8", Locale::DeDe),
            ("pt_PT", Locale::PtBr),
            ("uk_UA.UTF-8", Locale::UkUa),
        ] {
            assert_eq!(locale_for(Some(tag)), locale.or_source(), "{tag}");
        }
    }

    #[test]
    fn a_known_offset_is_kept_in_whole_minutes_east_of_utc() {
        let offset = |seconds| UtcOffset::from_whole_seconds(seconds).ok();
        let half_hour = offset(5 * 3600 + 30 * 60).map(Ok);
        let west = offset(-5 * 3600).map(Ok);
        assert_eq!(
            zone_of(half_hour.unwrap_or(Err(IndeterminateOffset))),
            Zone::Local { minutes: 330 }
        );
        assert_eq!(
            zone_of(west.unwrap_or(Err(IndeterminateOffset))),
            Zone::Local { minutes: -300 }
        );
    }

    #[test]
    fn every_cli_key_has_text_in_both_locales() {
        let keys = [
            "cli.version",
            "cli.version_legal",
            "cli.help",
            "cli.usage_error",
            "cli.paths_failed",
            "cli.storage_failed",
            "cli.terminal_failed",
            "cli.output_failed",
            "cli.new_window_failed",
            "cli.log_unavailable",
            "cli.panic",
            "cli.press_enter",
        ];
        for locale in Locale::offered() {
            i18n::set_locale(locale);
            for key in keys {
                assert_ne!(i18n::t(key), key, "{} lacks {key}", locale.id());
            }
        }
        i18n::set_locale(Locale::EnUs);
    }
}
