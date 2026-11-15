//! The `guide_panel` setting through a real `SqliteStorage`: it round-trips, and a settings row
//! written before the field existed loads with the default (the panel on).

use oxysol::paths::AppPaths;
use oxysol::storage::SqliteStorage;
use oxysol_tui::{Settings, Storage};
use rusqlite::Connection;

fn open(paths: &AppPaths) -> SqliteStorage {
    let storage = SqliteStorage::open(paths);
    assert!(storage.is_ok(), "open failed: {:?}", storage.as_ref().err());
    let Ok(storage) = storage else {
        std::process::abort();
    };
    storage
}

fn paths_in(dir: &tempfile::TempDir) -> AppPaths {
    AppPaths::in_dir(dir.path())
}

#[test]
fn the_guide_setting_survives_a_save_and_a_load() {
    let dir = tempfile::tempdir();
    let Ok(dir) = dir else {
        std::process::abort();
    };
    let paths = paths_in(&dir);
    let mut storage = open(&paths);
    let settings = Settings {
        guide_panel: false,
        ..Settings::default()
    };
    assert!(storage.save_settings(&settings).is_ok());
    drop(storage);
    let storage = open(&paths);
    assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if !s.guide_panel && *s == settings));
}

#[test]
fn settings_saved_before_the_guide_existed_load_with_the_panel_on() {
    let dir = tempfile::tempdir();
    let Ok(dir) = dir else {
        std::process::abort();
    };
    let paths = paths_in(&dir);
    drop(open(&paths));
    {
        let conn = Connection::open(paths.database_file());
        let Ok(conn) = conn else {
            std::process::abort();
        };
        let written = conn.execute(
            "INSERT OR REPLACE INTO settings (id, json) VALUES (1, ?1)",
            ["{\"theme\":\"light\",\"volume\":30}"],
        );
        assert!(written.is_ok());
    }
    let storage = open(&paths);
    assert!(matches!(
        storage.load_settings(),
        Ok(Some(ref s)) if s.guide_panel && s.volume == 30
    ));
}
