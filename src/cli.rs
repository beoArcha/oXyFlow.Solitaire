//! Command-line argument parsing.

use thiserror::Error;

/// Color depth argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorArg {
    Ansi16,
    Ansi256,
    TrueColor,
}

/// Parsed command-line arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cli {
    pub show_version: bool,
    pub show_help: bool,
    pub ascii: bool,
    pub new_window: bool,
    pub colors: Option<ColorArg>,
    pub seed: Option<u64>,
}

/// Command-line parsing error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CliError {
    #[error("unknown argument: {0}")]
    UnknownArgument(String),
    #[error("missing value for flag `{0}`")]
    MissingValue(&'static str),
    #[error("invalid value `{value}` for flag `{flag}`")]
    InvalidValue { flag: &'static str, value: String },
}

fn parse_colors(val: &str) -> Option<ColorArg> {
    match val {
        "16" => Some(ColorArg::Ansi16),
        "256" => Some(ColorArg::Ansi256),
        "truecolor" => Some(ColorArg::TrueColor),
        _ => None,
    }
}

/// Parses command-line arguments.
///
/// The iterator `args` must exclude the program name.
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, CliError> {
    let mut cli = Cli {
        show_version: false,
        show_help: false,
        ascii: false,
        new_window: false,
        colors: None,
        seed: None,
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--version" => cli.show_version = true,
            "--help" => cli.show_help = true,
            "--ascii" => cli.ascii = true,
            "--new-window" | "-n" => cli.new_window = true,
            "--colors" => {
                let val = it.next().ok_or(CliError::MissingValue("--colors"))?;
                let color = parse_colors(&val).ok_or(CliError::InvalidValue {
                    flag: "--colors",
                    value: val,
                })?;
                cli.colors = Some(color);
            }
            "--seed" => {
                let val = it.next().ok_or(CliError::MissingValue("--seed"))?;
                let seed = val.parse::<u64>().map_err(|_err| CliError::InvalidValue {
                    flag: "--seed",
                    value: val,
                })?;
                cli.seed = Some(seed);
            }
            _ => return Err(CliError::UnknownArgument(arg)),
        }
    }
    Ok(cli)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn test_no_arguments() {
        let parsed = parse(Vec::<String>::new()).unwrap();
        assert_eq!(
            parsed,
            Cli {
                show_version: false,
                show_help: false,
                ascii: false,
                new_window: false,
                colors: None,
                seed: None,
            }
        );
    }

    #[test]
    fn test_version_flag() {
        let parsed = parse(to_args(&["--version"])).unwrap();
        assert!(parsed.show_version);
    }

    #[test]
    fn test_help_flag() {
        let parsed = parse(to_args(&["--help"])).unwrap();
        assert!(parsed.show_help);
    }

    #[test]
    fn test_ascii_flag() {
        let parsed = parse(to_args(&["--ascii"])).unwrap();
        assert!(parsed.ascii);
    }

    #[test]
    fn test_new_window_flag_long_and_short_keep_the_other_flags() {
        for flag in ["--new-window", "-n"] {
            let parsed = parse(to_args(&[
                "--ascii", flag, "--seed", "5", "--colors", "256",
            ]))
            .unwrap();
            assert!(parsed.new_window);
            assert!(parsed.ascii);
            assert_eq!(parsed.seed, Some(5));
            assert_eq!(parsed.colors, Some(ColorArg::Ansi256));
        }
        assert!(!parse(to_args(&["--ascii"])).unwrap().new_window);
    }

    #[test]
    fn test_colors_flag_all_three_values() {
        let parsed16 = parse(to_args(&["--colors", "16"])).unwrap();
        assert_eq!(parsed16.colors, Some(ColorArg::Ansi16));

        let parsed256 = parse(to_args(&["--colors", "256"])).unwrap();
        assert_eq!(parsed256.colors, Some(ColorArg::Ansi256));

        let parsed_tc = parse(to_args(&["--colors", "truecolor"])).unwrap();
        assert_eq!(parsed_tc.colors, Some(ColorArg::TrueColor));
    }

    #[test]
    fn test_colors_without_value() {
        let err = parse(to_args(&["--colors"])).unwrap_err();
        assert_eq!(err, CliError::MissingValue("--colors"));
    }

    #[test]
    fn test_colors_invalid_value() {
        let err = parse(to_args(&["--colors", "8"])).unwrap_err();
        assert_eq!(
            err,
            CliError::InvalidValue {
                flag: "--colors",
                value: "8".to_string(),
            }
        );
    }

    #[test]
    fn test_seed_42() {
        let parsed = parse(to_args(&["--seed", "42"])).unwrap();
        assert_eq!(parsed.seed, Some(42));
    }

    #[test]
    fn test_seed_u64_max() {
        let parsed = parse(to_args(&["--seed", "18446744073709551615"])).unwrap();
        assert_eq!(parsed.seed, Some(18446744073709551615));
    }

    #[test]
    fn test_seed_negative_one() {
        let err = parse(to_args(&["--seed", "-1"])).unwrap_err();
        assert_eq!(
            err,
            CliError::InvalidValue {
                flag: "--seed",
                value: "-1".to_string(),
            }
        );
    }

    #[test]
    fn test_seed_abc() {
        let err = parse(to_args(&["--seed", "abc"])).unwrap_err();
        assert_eq!(
            err,
            CliError::InvalidValue {
                flag: "--seed",
                value: "abc".to_string(),
            }
        );
    }

    #[test]
    fn test_seed_without_value() {
        let err = parse(to_args(&["--seed"])).unwrap_err();
        assert_eq!(err, CliError::MissingValue("--seed"));
    }

    #[test]
    fn test_unknown_argument() {
        let err = parse(to_args(&["--unknown"])).unwrap_err();
        assert_eq!(err, CliError::UnknownArgument("--unknown".to_string()));
    }
}
