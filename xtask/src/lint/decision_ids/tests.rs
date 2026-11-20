use std::path::PathBuf;

use super::*;

const DECISIONS_TEXT: &str = "# Decisions\n\
\n\
| ID | Decision |\n\
|---|---|\n\
| R-01 | First. |\n\
| T-01 | Second. |\n\
\n\
### D-06 Card graphics\n";

fn source(path: &str, text: &str) -> SourceFile {
    SourceFile {
        path: PathBuf::from(path),
        text: text.to_owned(),
    }
}

fn decisions_only(text: &str) -> Vec<SourceFile> {
    vec![source(DECISIONS, text)]
}

fn warning(path: &str, line: usize, message: &str) -> Finding {
    Finding {
        rule: RULE,
        severity: Severity::Warning,
        file: PathBuf::from(path),
        line,
        message: message.to_owned(),
    }
}

#[test]
fn duplicate_table_row_is_an_error_at_the_second_definition() {
    let text = "| R-01 | First. |\n| R-01 | Again. |\n";
    assert_eq!(
        check(&decisions_only(text)),
        vec![Finding {
            rule: RULE,
            severity: Severity::Error,
            file: PathBuf::from(DECISIONS),
            line: 2,
            message: "duplicate decision ID R-01, first defined at line 1".to_owned(),
        }]
    );
}

#[test]
fn heading_and_table_row_with_the_same_id_are_a_duplicate() {
    let text = "### D-06 Card graphics\n\n| D-06 | Again. |\n";
    assert_eq!(
        check(&decisions_only(text)),
        vec![Finding {
            rule: RULE,
            severity: Severity::Error,
            file: PathBuf::from(DECISIONS),
            line: 3,
            message: "duplicate decision ID D-06, first defined at line 1".to_owned(),
        }]
    );
}

#[test]
fn known_reference_is_silent() {
    let files = vec![
        source(DECISIONS, DECISIONS_TEXT),
        source("docs/tech/architecture.md", "See R-01, T-01 and D-06.\n"),
    ];
    assert_eq!(check(&files), Vec::new());
}

#[test]
fn unknown_reference_is_a_warning_with_file_and_line() {
    let files = vec![
        source(DECISIONS, DECISIONS_TEXT),
        source("docs/task_list/phase-9.md", "first line\nSee T-99 here.\n"),
    ];
    assert_eq!(
        check(&files),
        vec![warning(
            "docs/task_list/phase-9.md",
            2,
            "reference to unknown decision T-99"
        )]
    );
}

#[test]
fn references_in_agents_md_are_checked_and_other_files_are_not() {
    let files = vec![
        source(DECISIONS, DECISIONS_TEXT),
        source("AGENTS.md", "Rule R-98.\n"),
        source("docs/data.txt", "T-97 is not a Markdown file.\n"),
        source("src/lib.rs", "// R-96\n"),
    ];
    assert_eq!(
        check(&files),
        vec![warning(
            "AGENTS.md",
            1,
            "reference to unknown decision R-98"
        )]
    );
}

#[test]
fn only_a_bounded_id_is_a_reference() {
    let text = "UTF-8 XR-01 R-01x phase-R-01a T-16_ R-01, D-06.\n";
    let mut files = vec![source(DECISIONS, DECISIONS_TEXT)];
    files.push(source("docs/tech/ui.md", text));
    assert_eq!(check(&files), Vec::new());
}

#[test]
fn id_with_a_dash_suffix_is_still_a_reference() {
    let files = vec![
        source(DECISIONS, DECISIONS_TEXT),
        source("docs/tech/ui.md", "See R-99-b.\n"),
    ];
    assert_eq!(
        check(&files),
        vec![warning(
            "docs/tech/ui.md",
            1,
            "reference to unknown decision R-99"
        )]
    );
}

#[test]
fn non_id_first_cells_and_other_columns_do_not_define_an_id() {
    let files = vec![
        source(DECISIONS, "| ID | Decision |\n| X | R-02 |\n"),
        source("docs/notes.md", "| R-03 | Table outside decisions. |\n"),
        source("docs/tech/ui.md", "R-02 R-03\n"),
    ];
    assert_eq!(
        check(&files),
        vec![
            warning(DECISIONS, 2, "reference to unknown decision R-02"),
            warning("docs/notes.md", 1, "reference to unknown decision R-03"),
            warning("docs/tech/ui.md", 1, "reference to unknown decision R-02"),
            warning("docs/tech/ui.md", 1, "reference to unknown decision R-03"),
        ]
    );
}

#[test]
fn a_missing_decisions_file_makes_every_reference_unknown() {
    let files = vec![source("docs/notes.md", "See R-01.\n")];
    assert_eq!(
        check(&files),
        vec![warning(
            "docs/notes.md",
            1,
            "reference to unknown decision R-01"
        )]
    );
}
