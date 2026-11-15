use std::path::PathBuf;

use super::*;

fn first_function_depth(code: &str) -> usize {
    let file: syn::File = syn::parse_str(code).unwrap();
    let mut collector = FunctionCollector::default();
    collector.visit_file(&file);
    let first = collector.functions.first().unwrap();
    let (depth, _) = measure_nesting(first.block);
    depth
}

fn check_source(code: &str) -> Vec<Finding> {
    let file = SourceFile {
        path: PathBuf::from("test.rs"),
        text: code.to_owned(),
    };
    check(&file)
}

#[test]
fn match_in_for_in_if_is_three() {
    let code = r#"
        fn sample() {
            if a {
                for b in c {
                    match d {
                        _ => {}
                    }
                }
            }
        }
    "#;
    assert_eq!(first_function_depth(code), 3);
}

#[test]
fn else_if_chain_is_one() {
    let code = r#"
        fn sample() {
            if a {
                1
            } else if b {
                2
            } else if c {
                3
            } else {
                4
            }
        }
    "#;
    assert_eq!(first_function_depth(code), 1);
}

#[test]
fn block_bodied_closure_adds_one() {
    let block_closure = r#"
        fn sample() {
            let f = || {
                let x = 1;
            };
        }
    "#;
    assert_eq!(first_function_depth(block_closure), 1);

    let non_block_closure = r#"
        fn sample() {
            let f = |x| x + 1;
        }
    "#;
    assert_eq!(first_function_depth(non_block_closure), 0);
}

#[test]
fn nested_function_does_not_add_to_parent() {
    let code = r#"
        fn parent() {
            if a {
                fn child() {
                    if b {
                        if c {
                        }
                    }
                }
            }
        }
    "#;
    let file: syn::File = syn::parse_str(code).unwrap();
    let mut collector = FunctionCollector::default();
    collector.visit_file(&file);
    assert_eq!(collector.functions.len(), 2);

    let parent = &collector.functions[0];
    assert_eq!(parent.name, "parent");
    assert_eq!(measure_nesting(parent.block).0, 1);

    let child = &collector.functions[1];
    assert_eq!(child.name, "child");
    assert_eq!(measure_nesting(child.block).0, 2);
}

#[test]
fn depth_four_yields_no_findings() {
    let code = r#"
        fn depth_4() {
            if a {
                if b {
                    if c {
                        if d {
                        }
                    }
                }
            }
        }
    "#;
    assert_eq!(check_source(code), Vec::new());
}

#[test]
fn depth_five_yields_warning() {
    let code = "fn depth_5() {\n    if a {\n        if b {\n            if c {\n                if d {\n                    if e {\n                    }\n                }\n            }\n        }\n    }\n}";
    let findings = check_source(code);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "nesting");
    assert_eq!(findings[0].severity, Severity::Warning);
    assert_eq!(findings[0].line, 6);
    assert!(findings[0].message.contains("depth_5"));
    assert!(findings[0].message.contains('5'));
}

#[test]
fn depth_seven_yields_error() {
    let code = "fn depth_7() {\n    if a {\n        if b {\n            if c {\n                if d {\n                    if e {\n                        if f {\n                            if g {\n                            }\n                        }\n                    }\n                }\n            }\n        }\n    }\n}";
    let findings = check_source(code);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "nesting");
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(findings[0].line, 8);
    assert!(findings[0].message.contains("depth_7"));
    assert!(findings[0].message.contains('7'));
}

#[test]
fn parse_failure_yields_no_findings() {
    let code = "fn broken( {";
    assert_eq!(check_source(code), Vec::new());
}

#[test]
fn plain_unsafe_and_async_blocks_add_depth() {
    let code = r#"
        fn sample() {
            {
                unsafe {
                    async {
                    }
                }
            }
        }
    "#;
    assert_eq!(first_function_depth(code), 3);
}
