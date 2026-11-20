use std::path::PathBuf;

use super::*;

fn findings(text: &str) -> Vec<Finding> {
    let file = SourceFile {
        path: PathBuf::from("docs/sample.md"),
        text: text.to_owned(),
    };
    check(&file)
}

fn lines_of(text: &str) -> Vec<usize> {
    findings(text).iter().map(|found| found.line).collect()
}

fn start() -> String {
    format!("{} HEAD", "<".repeat(7))
}

fn separator() -> String {
    "=".repeat(7)
}

fn end() -> String {
    format!("{} feature", ">".repeat(7))
}

fn joined(lines: &[&str]) -> String {
    lines.iter().map(|line| format!("{line}\n")).collect()
}

#[test]
fn conflict_block_is_flagged_on_each_marker_line_with_file_and_line() {
    let text = joined(&[
        "fn f() {}",
        &start(),
        "ours",
        &separator(),
        "theirs",
        &end(),
    ]);
    let found = findings(&text);
    assert_eq!(lines_of(&text), vec![2, 4, 6]);
    assert_eq!(found[0].file, PathBuf::from("docs/sample.md"));
    assert_eq!(found[0].rule, "conflict-markers");
    assert_eq!(found[0].severity, Severity::Error);
}

#[test]
fn bare_markers_without_label_are_flagged() {
    let text = joined(&[
        &"<".repeat(7),
        "ours",
        &"=".repeat(7),
        "theirs",
        &">".repeat(7),
    ]);
    assert_eq!(lines_of(&text), vec![1, 3, 5]);
}

#[test]
fn start_marker_alone_is_flagged() {
    assert_eq!(lines_of(&joined(&[&start(), "ours"])), vec![1]);
}

#[test]
fn end_marker_without_start_is_flagged() {
    assert_eq!(lines_of(&joined(&["text", &end()])), vec![2]);
}

#[test]
fn crlf_line_endings_are_flagged() {
    let text = format!("{}\r\nours\r\n{}\r\n", start(), end());
    assert_eq!(lines_of(&text), vec![1, 3]);
}

#[test]
fn markdown_heading_underline_is_not_flagged() {
    assert_eq!(findings("Title\n=======\n\nBody text.\n"), Vec::new());
    assert_eq!(findings("Title\n=====\n"), Vec::new());
}

#[test]
fn separator_outside_an_open_block_is_not_flagged() {
    let text = joined(&[&start(), "ours", &end(), "Title", &separator(), "", "Text"]);
    assert_eq!(lines_of(&text), vec![1, 3]);
}

#[test]
fn indented_marker_is_not_flagged() {
    let text = joined(&[&format!(" {}", start()), "text"]);
    assert_eq!(findings(&text), Vec::new());
}

#[test]
fn run_of_eight_characters_is_not_flagged() {
    let text = joined(&[&"<".repeat(8), &"=".repeat(8), &">".repeat(8)]);
    assert_eq!(findings(&text), Vec::new());
}

#[test]
fn separator_with_trailing_text_is_not_flagged_inside_a_block() {
    let text = joined(&[&start(), &format!("{} text", "=".repeat(7)), "more"]);
    assert_eq!(lines_of(&text), vec![1]);
}

#[test]
fn ordinary_markup_is_not_flagged() {
    assert_eq!(findings("fn f() {}\n<div>\n  =\n"), Vec::new());
}
