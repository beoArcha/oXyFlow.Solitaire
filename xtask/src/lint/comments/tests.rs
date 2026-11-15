use std::path::PathBuf;

use super::*;

fn findings(code: &str) -> Vec<Finding> {
    let file = SourceFile {
        path: PathBuf::from("sample.rs"),
        text: code.to_owned(),
    };
    check(&file)
}

fn assert_clean(code: &str) {
    assert_eq!(findings(code), Vec::new(), "unexpected finding in:\n{code}");
}

#[test]
fn line_comment_inside_function_body_is_error() {
    let found = findings("fn f() {\n    // note\n    let x = 1;\n}\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "comments");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(found[0].line, 2);
    assert!(found[0].message.contains("function body"));
}

#[test]
fn block_comment_inside_function_body_is_error() {
    let found = findings("fn f() {\n    /* note */\n    let x = 1;\n}\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 2);
}

#[test]
fn comment_inside_method_and_trait_default_body_is_error() {
    let method = "struct S;\nimpl S {\n    fn m(&self) {\n        // note\n    }\n}\n";
    let found = findings(method);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 4);

    let default_body = "trait T {\n    fn m(&self) {\n        // note\n    }\n}\n";
    assert_eq!(findings(default_body).len(), 1);
}

#[test]
fn comment_outside_any_function_is_allowed() {
    assert_clean("// top level note\nfn f() {}\n// after the function\n");
    assert_clean("/* block note */\nconst X: u8 = 1;\n");
    assert_clean("trait T {\n    fn required(&self); // outside any body\n}\n");
}

#[test]
fn double_slash_inside_normal_string_is_not_a_comment() {
    assert_clean("fn f() -> &'static str {\n    \"// not a comment\"\n}\n");
}

#[test]
fn double_slash_inside_raw_string_is_not_a_comment() {
    assert_clean("fn f() -> &'static str {\n    r#\"// not /* a comment */\"#\n}\n");
}

#[test]
fn doc_comment_on_item_nested_in_body_is_allowed() {
    assert_clean("fn outer() {\n    /// documented\n    fn inner() {}\n    inner();\n}\n");
    assert_clean("fn outer() {\n    //! inner documentation\n    let _ = 1;\n}\n");
}

#[test]
fn safety_comment_before_unsafe_is_allowed() {
    assert_clean(
        "fn read(p: *const u8) -> u8 {\n    // SAFETY: the caller guarantees p is valid.\n    unsafe { *p }\n}\n",
    );
}

#[test]
fn comment_that_only_looks_like_safety_is_error() {
    let found =
        findings("fn read(p: *const u8) -> u8 {\n    // Safe: fine\n    unsafe { *p }\n}\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 2);
}

#[test]
fn decorative_separators_are_error_everywhere() {
    let code = "// ------\nfn a() {}\n// =====\n/*****/\nfn b() {\n    // ----\n}\n";
    let found = findings(code);
    let lines: Vec<usize> = found.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, vec![1, 3, 4, 6]);
    assert!(
        found
            .iter()
            .all(|finding| finding.severity == Severity::Error)
    );
    assert!(found[0].message.contains("separator"));
    assert!(found[3].message.contains("separator"));
}

#[test]
fn short_and_mixed_comments_are_not_separators() {
    assert!(!is_separator("//"));
    assert!(!is_separator("////"));
    assert!(!is_separator("/**/"));
    assert!(!is_separator("// --"));
    assert!(!is_separator("// ok"));
    assert!(!is_separator("// ---- section ----"));
    assert!(is_separator("// ------"));
    assert!(is_separator("/*****/"));
    assert!(is_separator("/* ====== */"));
}

#[test]
fn file_that_does_not_parse_still_reports_separators() {
    let found = findings("fn broken( {\n// ----\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 2);
}
