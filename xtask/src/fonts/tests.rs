//! Tests of the card font builder: registry, glyph files and Python discovery need no Python;
//! the end-to-end test builds a font from two fixture SVGs and is ignored by default.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{FontsError, Glyphs, Paths, Source, build, find_python, glyph_sources, probe_python};

const SVG_HEAD: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1200 2380\">";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("fonts")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xtask-fonts-{name}-{}", std::process::id()));
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn glyphs_of(entries: &[(&str, u32)]) -> Glyphs {
    entries
        .iter()
        .map(|(key, code)| ((*key).to_owned(), *code))
        .collect()
}

#[test]
fn test_find_python_takes_first_candidate_that_works() {
    let probed = RefCell::new(Vec::new());
    let found = find_python(|program| {
        probed.borrow_mut().push(program.to_owned());
        program == "python"
    });
    assert_eq!(found.unwrap(), "python");
    assert_eq!(*probed.borrow(), ["python3", "python"]);
}

#[test]
fn test_find_python_names_what_was_tried() {
    let error = find_python(|_| false).unwrap_err();
    assert!(matches!(error, FontsError::NoPython { .. }));
    assert_eq!(
        error.to_string(),
        "no Python 3.11 or newer with fontTools and skia-pathops found; tried `python3`, `python`; \
         install them with `python -m pip install -r xtask/fonts/requirements.txt`"
    );
}

#[test]
fn test_find_python_probe_reports_missing_program_as_not_found() {
    assert!(!probe_python("oxysol-no-such-python-3.11"));
}

#[test]
fn test_glyph_sources_pair_svg_with_registry_and_skip_entries_without_art() {
    let glyphs = glyphs_of(&[
        ("fronts/international/spade", 0xE6BC),
        ("backs/lattice/tile", 0xE6D8),
        ("fronts/international/heart", 0xE6BD),
    ]);
    let sources = glyph_sources(&fixtures().join("cards"), &glyphs).unwrap();
    assert_eq!(sources.files.len(), 2);
    assert_eq!(
        sources.files[0],
        Source {
            key: "fronts/international/spade".to_owned(),
            code: 0xE6BC,
            svg: fixtures().join("cards/fronts/international/glyphs/spade.svg"),
        }
    );
    assert_eq!(sources.skipped, ["fronts/international/heart"]);
}

#[test]
fn test_glyph_sources_name_the_svg_without_registry_entry() {
    let glyphs = glyphs_of(&[("fronts/international/spade", 0xE6BC)]);
    let error = glyph_sources(&fixtures().join("cards"), &glyphs).unwrap_err();
    match error {
        FontsError::Glyph { path, reason } => {
            assert!(path.ends_with("tile.svg"), "{}", path.display());
            assert!(reason.contains("backs/lattice/tile"), "{reason}");
        }
        other => panic!("expected a glyph error, got {other:?}"),
    }
}

#[test]
fn test_glyph_sources_expand_mosaic_into_four_corners() {
    let dir = scratch("mosaic");
    write(&dir.join("backs/ornament/glyphs/mosaic.svg"), SVG_HEAD);
    let glyphs = glyphs_of(&[
        ("backs/ornament/mosaic-tl", 0xE6DE),
        ("backs/ornament/mosaic-tr", 0xE6DF),
        ("backs/ornament/mosaic-bl", 0xE6E0),
        ("backs/ornament/mosaic-br", 0xE6E1),
    ]);
    let sources = glyph_sources(&dir, &glyphs).unwrap();
    let keys: Vec<&str> = sources
        .files
        .iter()
        .map(|source| source.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "backs/ornament/mosaic-tl",
            "backs/ornament/mosaic-tr",
            "backs/ornament/mosaic-bl",
            "backs/ornament/mosaic-br",
        ]
    );
    assert!(sources.skipped.is_empty());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_glyph_sources_reject_file_name_outside_style_guide() {
    let dir = scratch("names");
    write(&dir.join("fronts/international/glyphs/Spade.svg"), SVG_HEAD);
    let error = glyph_sources(&dir, &Glyphs::new()).unwrap_err();
    match error {
        FontsError::Glyph { path, reason } => {
            assert!(path.ends_with("Spade.svg"), "{}", path.display());
            assert!(reason.contains("[a-z][a-z0-9-]*"), "{reason}");
        }
        other => panic!("expected a glyph error, got {other:?}"),
    }
    fs::remove_dir_all(&dir).ok();
}

fn python_or_skip() -> Option<String> {
    find_python(probe_python).ok()
}

fn fixture_paths(out: &Path, preview: &Path) -> Paths {
    let root = workspace_root();
    Paths {
        cards: fixtures().join("cards"),
        registry: fixtures().join("codepoints.toml"),
        base: root
            .join("assets")
            .join("fonts")
            .join("base")
            .join("CascadiaMono.ttf"),
        out: out.to_path_buf(),
        preview: preview.to_path_buf(),
        script: root.join("xtask").join("fonts").join("build_font.py"),
    }
}

const READ_BACK: &str = "\
import sys
from fontTools.ttLib import TTFont
out = TTFont(sys.argv[1])
base = TTFont(sys.argv[2])
cmap = out.getBestCmap()
names = {r.nameID: r.toUnicode() for r in out['name'].names if r.platformID == 3}
print('glyphs=%d' % len(out.getGlyphOrder()))
print('base_glyphs=%d' % len(base.getGlyphOrder()))
print('e6bc=%s' % cmap.get(0xE6BC))
print('e6d8=%s' % cmap.get(0xE6D8))
print('advances=%d,%d' % (out['hmtx'][cmap[0xE6BC]][0], out['hmtx'][cmap[0xE6D8]][0]))
print('metrics=%d,%d,%d' % (out['head'].unitsPerEm, out['hhea'].ascent, -out['hhea'].descent))
print('family=%s' % names.get(1))
print('cascadia=%s' % any('Cascadia' in names.get(i, '') for i in (1, 2, 3, 4, 6, 16, 17)))
print('base_codes_kept=%s' % all(cmap.get(c) == n for c, n in base.getBestCmap().items()))
";

fn read_back(python: &str, out: &Path, base: &Path) -> Vec<(String, String)> {
    let output = Command::new(python)
        .arg("-c")
        .arg(READ_BACK)
        .arg(out)
        .arg(base)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn fact<'a>(facts: &'a [(String, String)], key: &str) -> &'a str {
    facts
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
        .unwrap_or_else(|| panic!("no fact `{key}`"))
}

#[test]
#[ignore = "needs Python 3.11+ with fontTools and skia-pathops; run: cargo test -p xtask -- --ignored fonts"]
fn test_fonts_build_from_fixture_svgs_and_reads_back() {
    let python = python_or_skip().expect("Python 3.11 or newer is required for this test");
    let dir = scratch("e2e");
    let out = dir.join("oXySolMono-Regular.ttf");
    let preview = dir.join("preview.html");
    build(&fixture_paths(&out, &preview)).unwrap();

    let base = workspace_root().join("assets/fonts/base/CascadiaMono.ttf");
    let facts = read_back(&python, &out, &base);
    let base_glyphs: usize = fact(&facts, "base_glyphs").parse().unwrap();
    assert_eq!(fact(&facts, "glyphs"), (base_glyphs + 2).to_string());
    assert_eq!(fact(&facts, "e6bc"), "uniE6BC");
    assert_eq!(fact(&facts, "e6d8"), "uniE6D8");
    assert_eq!(fact(&facts, "advances"), "1200,1200");
    assert_eq!(fact(&facts, "metrics"), "2048,1900,480");
    assert_eq!(fact(&facts, "family"), "oXySol Mono");
    assert_eq!(fact(&facts, "cascadia"), "False");
    assert_eq!(fact(&facts, "base_codes_kept"), "True");

    let page = fs::read_to_string(&preview).unwrap();
    assert!(page.contains("&#xE6BC;") && page.contains("oXySol Mono"));

    let again = dir.join("again.ttf");
    build(&fixture_paths(&again, &dir.join("again.html"))).unwrap();
    assert_eq!(fs::read(&out).unwrap(), fs::read(&again).unwrap());
    fs::remove_dir_all(&dir).ok();
}

#[test]
#[ignore = "needs Python 3.11+ with fontTools and skia-pathops; run: cargo test -p xtask -- --ignored fonts"]
fn test_fonts_invalid_svg_fails_and_names_file_and_rule() {
    let python = python_or_skip().expect("Python 3.11 or newer is required for this test");
    let dir = scratch("invalid");
    write(
        &dir.join("fronts/international/glyphs/spade.svg"),
        &format!("{SVG_HEAD}<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\"/></svg>"),
    );
    let script = workspace_root().join("xtask/fonts/build_font.py");
    let base = workspace_root().join("assets/fonts/base/CascadiaMono.ttf");
    let output = Command::new(&python)
        .arg(&script)
        .arg("--base")
        .arg(&base)
        .arg("--out")
        .arg(dir.join("out.ttf"))
        .arg("--preview")
        .arg(dir.join("preview.html"))
        .arg("--glyph")
        .arg("fronts/international/spade")
        .arg("E6BC")
        .arg(dir.join("fronts/international/glyphs/spade.svg"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("spade.svg"), "{stderr}");
    assert!(
        stderr.contains("only <path> elements are allowed, found <rect>"),
        "{stderr}"
    );
    fs::remove_dir_all(&dir).ok();
}
