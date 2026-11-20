use std::path::Path;

use super::*;

fn findings(file: &str, text: &str, known: &[&str]) -> Vec<Finding> {
    broken_links(Path::new(file), text, |path| {
        known.iter().any(|name| Path::new(name) == path)
    })
}

#[test]
fn good_relative_link_passes() {
    let text = "See [the workflow](tech/workflow.md) and ![pic](img/a.svg).";
    let known = ["docs/tech/workflow.md", "docs/img/a.svg"];
    assert!(findings("docs/readme.md", text, &known).is_empty());
}

#[test]
fn broken_relative_link_fails_with_its_line() {
    let text = "ok\nSee [gone](tech/missing.md).";
    let found = findings("docs/readme.md", text, &[]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 2);
    assert_eq!(found[0].severity, Severity::Error);
}

#[test]
fn parent_folders_resolve_against_the_file() {
    let text = "[up](../../AGENTS.md)";
    assert!(findings("docs/tech/x.md", text, &["AGENTS.md"]).is_empty());
    assert_eq!(findings("docs/tech/x.md", text, &[]).len(), 1);
}

#[test]
fn anchors_are_stripped_and_pure_anchors_ignored() {
    let text = "[a](#top) [b](tech/x.md#section)";
    assert!(findings("docs/r.md", text, &["docs/tech/x.md"]).is_empty());
    assert_eq!(findings("docs/r.md", text, &[]).len(), 1);
}

#[test]
fn external_targets_are_ignored() {
    let text = "[a](https://example.com/x) [b](mailto:a@b.c) [c](http://x.y)";
    assert!(findings("README.md", text, &[]).is_empty());
}

#[test]
fn code_spans_and_fences_are_ignored() {
    let text = "`[x](nope.md)`\n```\n[y](nope.md)\n```\n";
    assert!(findings("README.md", text, &[]).is_empty());
}

#[test]
fn titles_and_angle_brackets_are_handled() {
    let text = "[a](a.md \"title\") [b](<b.md>)";
    assert!(findings("README.md", text, &["a.md", "b.md"]).is_empty());
}

#[test]
fn reference_definitions_are_checked() {
    assert_eq!(findings("README.md", "[ref]: missing.md", &[]).len(), 1);
}
