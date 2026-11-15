//! Terminal capability detection.

/// Color capability of the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorDepth {
    Ansi16,
    Ansi256,
    TrueColor,
}

/// Target operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

/// Detected terminal capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalCaps {
    pub color: ColorDepth,
    pub unicode: bool,
    pub remote: bool,
}

/// Detects terminal capabilities based on environment variables and platform.
pub fn detect(env: &dyn Fn(&str) -> Option<String>, platform: Platform) -> TerminalCaps {
    let color = detect_color(env, platform);
    let unicode = detect_unicode(env, platform);
    let remote = env("SSH_CONNECTION").is_some() || env("SSH_TTY").is_some();

    TerminalCaps {
        color,
        unicode,
        remote,
    }
}

fn detect_color(env: &dyn Fn(&str) -> Option<String>, platform: Platform) -> ColorDepth {
    if is_truecolor(env) || platform == Platform::Windows {
        ColorDepth::TrueColor
    } else if has_256color(env) {
        ColorDepth::Ansi256
    } else {
        ColorDepth::Ansi16
    }
}

fn is_truecolor(env: &dyn Fn(&str) -> Option<String>) -> bool {
    match env("COLORTERM").as_deref() {
        Some(val) => {
            val == "truecolor"
                || val == "24bit"
                || val.eq_ignore_ascii_case("truecolor")
                || val.eq_ignore_ascii_case("24bit")
        }
        None => false,
    }
}

fn has_256color(env: &dyn Fn(&str) -> Option<String>) -> bool {
    match env("TERM").as_deref() {
        Some(term) => term.contains("256color"),
        None => false,
    }
}

fn detect_unicode(env: &dyn Fn(&str) -> Option<String>, platform: Platform) -> bool {
    if platform == Platform::Windows {
        return true;
    }

    let candidates = ["LC_ALL", "LC_CTYPE", "LANG"];
    let first_non_empty = candidates.into_iter().find_map(|var| match env(var) {
        Some(val) if !val.is_empty() => Some(val),
        _ => None,
    });

    match first_non_empty {
        Some(val) => {
            let lower = val.to_ascii_lowercase();
            lower.contains("utf-8") || lower.contains("utf8")
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase<'a> {
        name: &'a str,
        platform: Platform,
        vars: &'a [(&'a str, &'a str)],
        expected: TerminalCaps,
    }

    #[test]
    fn table_detection_cases() {
        let cases = [
            TestCase {
                name: "Windows with no variables",
                platform: Platform::Windows,
                vars: &[],
                expected: TerminalCaps {
                    color: ColorDepth::TrueColor,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "Linux xterm-256color with LANG=en_US.UTF-8",
                platform: Platform::Linux,
                vars: &[("TERM", "xterm-256color"), ("LANG", "en_US.UTF-8")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi256,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "Linux TERM=xterm with LANG=C (16 colors, no Unicode)",
                platform: Platform::Linux,
                vars: &[("TERM", "xterm"), ("LANG", "C")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: false,
                    remote: false,
                },
            },
            TestCase {
                name: "macOS xterm-256color UTF-8",
                platform: Platform::MacOs,
                vars: &[("TERM", "xterm-256color"), ("LANG", "en_US.UTF-8")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi256,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "COLORTERM=truecolor",
                platform: Platform::Linux,
                vars: &[("COLORTERM", "truecolor")],
                expected: TerminalCaps {
                    color: ColorDepth::TrueColor,
                    unicode: false,
                    remote: false,
                },
            },
            TestCase {
                name: "COLORTERM=24bit",
                platform: Platform::Linux,
                vars: &[("COLORTERM", "24bit")],
                expected: TerminalCaps {
                    color: ColorDepth::TrueColor,
                    unicode: false,
                    remote: false,
                },
            },
            TestCase {
                name: "LC_ALL overriding LANG (LC_ALL=C, LANG=UTF-8)",
                platform: Platform::Linux,
                vars: &[("LC_ALL", "C"), ("LANG", "en_US.UTF-8")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: false,
                    remote: false,
                },
            },
            TestCase {
                name: "LC_ALL overriding LANG (LC_ALL=UTF-8, LANG=C)",
                platform: Platform::Linux,
                vars: &[("LC_ALL", "en_US.UTF-8"), ("LANG", "C")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "LC_ALL empty string falls back to LANG",
                platform: Platform::Linux,
                vars: &[("LC_ALL", ""), ("LANG", "en_US.UTF-8")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "LC_CTYPE used when LC_ALL is unset",
                platform: Platform::Linux,
                vars: &[("LC_CTYPE", "UTF-8"), ("LANG", "C")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: true,
                    remote: false,
                },
            },
            TestCase {
                name: "an SSH session via SSH_CONNECTION",
                platform: Platform::Linux,
                vars: &[("SSH_CONNECTION", "192.168.1.1 1234 192.168.1.2 22")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: false,
                    remote: true,
                },
            },
            TestCase {
                name: "an SSH session via SSH_TTY",
                platform: Platform::Linux,
                vars: &[("SSH_TTY", "/dev/pts/1")],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: false,
                    remote: true,
                },
            },
            TestCase {
                name: "no variables at all on Linux",
                platform: Platform::Linux,
                vars: &[],
                expected: TerminalCaps {
                    color: ColorDepth::Ansi16,
                    unicode: false,
                    remote: false,
                },
            },
        ];

        for case in cases {
            let env = |key: &str| -> Option<String> {
                case.vars
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| (*v).to_string())
            };
            let actual = detect(&env, case.platform);
            assert_eq!(actual, case.expected, "failed test case: {}", case.name);
        }
    }
}
