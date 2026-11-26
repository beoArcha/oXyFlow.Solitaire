//! Loading the optional user theme (`theme.toml` in the data directory) at startup.

use std::io;
use std::path::Path;

use oxysol_tui::theme::UserTheme;

use crate::paths::AppPaths;

/// Reads and checks `theme.toml` in `paths.data_dir()`. A missing file returns
/// [`UserTheme::Missing`]; any read, parse or contrast failure is returned with its details so
/// the UI can log once and alert the player.
pub fn load_user_theme(paths: &AppPaths) -> UserTheme {
    load_user_theme_from(&paths.theme_file())
}

/// Reads and checks a user theme file at `path`.
pub fn load_user_theme_from(path: &Path) -> UserTheme {
    match std::fs::read_to_string(path) {
        Ok(text) => UserTheme::from_toml(path.display().to_string(), &text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => UserTheme::Missing,
        Err(error) => UserTheme::Unreadable {
            path: path.display().to_string(),
            detail: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxysol_tui::theme::{ThemeId, Token};

    fn valid_toml() -> String {
        [
            "field_edge = false",
            "background = { rgb = [18, 22, 34], ansi16 = \"black\" }",
            "surface = { rgb = [36, 42, 58], ansi16 = \"black\" }",
            "text = { rgb = [232, 236, 244], ansi16 = \"white\" }",
            "text_muted = { rgb = [184, 194, 214], ansi16 = \"gray\" }",
            "accent = { rgb = [140, 190, 255], ansi16 = \"light_cyan\" }",
            "border = { rgb = [110, 122, 150], ansi16 = \"gray\" }",
            "border_focus = { rgb = [255, 214, 102], ansi16 = \"light_yellow\" }",
            "focus_fg = { rgb = [20, 24, 32], ansi16 = \"black\" }",
            "focus_bg = { rgb = [255, 214, 102], ansi16 = \"light_yellow\" }",
            "error = { rgb = [255, 160, 150], ansi16 = \"light_red\" }",
            "success = { rgb = [150, 230, 160], ansi16 = \"light_green\" }",
            "table = { rgb = [40, 120, 80], ansi16 = \"green\" }",
            "table_text = { rgb = [236, 246, 240], ansi16 = \"white\" }",
            "slot = { rgb = [170, 220, 190], ansi16 = \"light_green\" }",
            "card_face = { rgb = [250, 250, 244], ansi16 = \"white\" }",
            "card_back = { rgb = [184, 208, 248], ansi16 = \"white\" }",
            "card_back_mark = { rgb = [16, 20, 40], ansi16 = \"black\" }",
            "card_red = { rgb = [166, 20, 20], ansi16 = \"red\" }",
            "card_black = { rgb = [16, 16, 20], ansi16 = \"black\" }",
            "target = { rgb = [255, 214, 102], ansi16 = \"light_yellow\" }",
            "selection = { rgb = [150, 220, 255], ansi16 = \"light_cyan\" }",
            "hint = { rgb = [255, 196, 110], ansi16 = \"light_yellow\" }",
            "aura_focus = { rgb = [255, 246, 184], ansi16 = \"light_yellow\" }",
            "aura_carry = { rgb = [210, 244, 255], ansi16 = \"light_cyan\" }",
        ]
        .join("\n")
    }

    #[test]
    fn missing_file_gives_missing_without_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let paths = AppPaths::in_dir(dir.path());
        assert_eq!(load_user_theme(&paths), UserTheme::Missing);
    }

    #[test]
    fn valid_theme_file_in_data_dir_loads() {
        let dir = tempfile::tempdir().expect("temp dir");
        let paths = AppPaths::in_dir(dir.path());
        std::fs::write(paths.theme_file(), valid_toml()).expect("write theme");
        let loaded = load_user_theme(&paths);
        assert!(matches!(
            loaded,
            UserTheme::Valid(palette) if palette.background.rgb == [18, 22, 34]
                && palette.text == ThemeId::Dark.palette().text
        ));
    }

    #[test]
    fn directory_at_theme_path_gives_unreadable_with_path() {
        let dir = tempfile::tempdir().expect("temp dir");
        let paths = AppPaths::in_dir(dir.path());
        std::fs::create_dir(paths.theme_file()).expect("mkdir");
        let loaded = load_user_theme(&paths);
        assert!(matches!(
            loaded,
            UserTheme::Unreadable { ref path, .. } if path.ends_with("theme.toml")
        ));
        let _ = Token::Background;
    }
}
