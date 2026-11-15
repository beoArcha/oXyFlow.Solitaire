//! Application logo widget.

const ASCII_LOGO: &[&str] = &[
    "        ##   ##            ####           ##    ",
    " ####    ## ##   ##   ##  ##       ####   ##    ",
    "##  ##    ###     ## ##    ####   ##  ##  ##    ",
    "##  ##   ## ##     ###        ##  ##  ##  ##  ##",
    " ####   ##   ##   ###      ####    ####    #### ",
];

const UNICODE_LOGO: &[&str] = &[
    "        ██   ██            ▄████          ██    ",
    " ▄██▄    ▀█ █▀   ██   ██  ██       ▄██▄   ██    ",
    "██  ██    ███     ▀█ █▀    ▀███▄  ██  ██  ██    ",
    "██  ██   ▄█ █▄     ███        ██  ██  ██  ██  ▄▄",
    " ▀██▀   ██   ██   ▀██     ████▀    ▀██▀    ▀██▀ ",
];

/// Returns the title logo lines for the given terminal capability.
#[must_use]
pub fn logo(unicode: bool) -> &'static [&'static str] {
    if unicode { UNICODE_LOGO } else { ASCII_LOGO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_count_and_width_limits() {
        for &unicode in &[false, true] {
            let lines = logo(unicode);
            assert!(lines.len() == 5 || lines.len() == 6);
            let width = lines[0].chars().count();
            assert!(width <= 60);
            for line in lines {
                assert_eq!(line.chars().count(), width);
            }
        }
    }

    #[test]
    fn test_ascii_character_set() {
        for line in logo(false) {
            for ch in line.chars() {
                assert!(ch == '#' || ch == ' ');
            }
        }
    }

    #[test]
    fn test_unicode_character_set() {
        for line in logo(true) {
            for ch in line.chars() {
                assert!(ch == '█' || ch == '▀' || ch == '▄' || ch == ' ');
            }
        }
    }

    #[test]
    fn test_print_logos_for_reviewer() {
        println!(
            "=== ASCII LOGO (width: {}) ===",
            logo(false)[0].chars().count()
        );
        for line in logo(false) {
            println!("{line}");
        }
        println!(
            "=== UNICODE LOGO (width: {}) ===",
            logo(true)[0].chars().count()
        );
        for line in logo(true) {
            println!("{line}");
        }
    }
}
