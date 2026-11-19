//! Entry point of the `oxysol` binary: arguments, paths, logging, the panic hook, storage, and
//! then the terminal interface.

use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;

use oxysol::about;
use oxysol::audio;
use oxysol::cli::{self, Cli, ColorArg};
use oxysol::clipboard::SystemClipboard;
use oxysol::console;
use oxysol::error::AppError;
use oxysol::paths::AppPaths;
use oxysol::solver::ThreadSolver;
use oxysol::storage::SqliteStorage;
use oxysol::{logging, new_window, panic_hook, seed};
use oxysol_tui::caps::{self, ColorDepth, Platform, TerminalCaps};
use oxysol_tui::i18n::{self, Locale};
use oxysol_tui::{Deps, NullSolver, Solver};

fn main() -> ExitCode {
    i18n::set_locale(locale_from_env());
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let outcome = cli::parse(raw.iter().cloned())
        .map_err(AppError::from)
        .and_then(|args| launch(&args, &raw));
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
    let paths = AppPaths::detect()?;
    let level = std::env::var(logging::LEVEL_VARIABLE).ok();
    let diagnostics = start_logging(&paths, level.as_deref());
    panic_hook::install(paths.log_file(), diagnostics);
    if console::owns_window() {
        console::widen_owned_window();
    }
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        data_dir = %paths.data_dir().display(),
        "oXySol starting"
    );
    if let Some(seed) = args.seed {
        tracing::info!(seed, "seed given on the command line");
    }
    panic_hook::panic_if_requested();
    let storage = SqliteStorage::open(&paths).map_err(|error| {
        tracing::error!(target: "oxysol::critical", %error, "the database cannot be opened");
        AppError::from(error)
    })?;
    let caps = terminal_caps(args);
    let deps = Deps {
        storage: Box::new(storage),
        audio: audio::create(&caps),
        clipboard: Box::new(SystemClipboard::new(caps.remote)),
        caps,
        seeds: Box::new(seed::fresh),
        start_seed: args.seed,
        sound_styles: audio::bank::style_list(),
        solver: start_solver(),
    };
    oxysol_tui::run(deps).map_err(|error| {
        tracing::error!(%error, "the terminal interface stopped");
        AppError::from(error)
    })
}

fn start_solver() -> Box<dyn Solver> {
    match ThreadSolver::spawn() {
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
    if value.is_some_and(|text| text.starts_with("pl")) {
        Locale::PlPl
    } else {
        Locale::EnUs
    }
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
    fn polish_environment_selects_polish_and_everything_else_english() {
        assert_eq!(locale_for(Some("pl_PL.UTF-8")), Locale::PlPl);
        assert_eq!(locale_for(Some("en_US.UTF-8")), Locale::EnUs);
        assert_eq!(locale_for(None), Locale::EnUs);
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
        for locale in Locale::ALL {
            i18n::set_locale(locale);
            for key in keys {
                assert_ne!(i18n::t(key), key, "{} lacks {key}", locale.id());
            }
        }
        i18n::set_locale(Locale::EnUs);
    }
}
