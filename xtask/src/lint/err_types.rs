//! Rule `err_types`: error types in public APIs.
//!
//! Flags public functions and methods returning vague error types such as `String`,
//! `&str`, `()`, `Box<dyn ...>`, or `anyhow::*`.

use std::path::PathBuf;
use syn::visit::{self, Visit};
use syn::{GenericArgument, ImplItemFn, ItemFn, Path, PathArguments, ReturnType, Type, Visibility};

use super::{Finding, Severity, SourceFile};

const RULE_NAME: &str = "err-types";

pub fn check(file: &SourceFile) -> Vec<Finding> {
    let Ok(file_ast) = syn::parse_file(&file.text) else {
        return Vec::new();
    };

    let mut visitor = ErrTypesVisitor {
        file_path: file.path.clone(),
        findings: Vec::new(),
    };
    visitor.visit_file(&file_ast);
    visitor.findings
}

struct ErrTypesVisitor {
    file_path: PathBuf,
    findings: Vec<Finding>,
}

impl ErrTypesVisitor {
    fn check_signature(&mut self, vis: &Visibility, sig: &syn::Signature) {
        if !matches!(vis, Visibility::Public(_)) {
            return;
        }
        let ReturnType::Type(_, return_ty) = &sig.output else {
            return;
        };
        if is_vague_error_type(return_ty) {
            self.findings.push(Finding {
                rule: RULE_NAME,
                severity: Severity::Error,
                file: self.file_path.clone(),
                line: sig.ident.span().start().line,
                message: format!("public function `{}` returns vague error type", sig.ident),
            });
        }
    }
}

impl<'ast> Visit<'ast> for ErrTypesVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.check_signature(&node.vis, &node.sig);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.check_signature(&node.vis, &node.sig);
        visit::visit_impl_item_fn(self, node);
    }
}

fn peel_paren(mut ty: &Type) -> &Type {
    while let Type::Paren(paren) = ty {
        ty = &paren.elem;
    }
    ty
}

fn path_starts_with_anyhow(path: &Path) -> bool {
    path.segments
        .first()
        .is_some_and(|seg| seg.ident == "anyhow")
}

fn is_anyhow_type(ty: &Type) -> bool {
    let Type::Path(type_path) = peel_paren(ty) else {
        return false;
    };
    type_path.qself.is_none() && path_starts_with_anyhow(&type_path.path)
}

fn parse_result_args(ty: &Type) -> Option<(&Type, &Type)> {
    let Type::Path(type_path) = peel_paren(ty) else {
        return None;
    };
    let last_seg = type_path.path.segments.last()?;
    if type_path.qself.is_some() || last_seg.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last_seg.arguments else {
        return None;
    };
    let mut type_args = args.args.iter().filter_map(|arg| match arg {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    let ok_ty = type_args.next()?;
    let err_ty = type_args.next()?;
    if type_args.next().is_none() {
        Some((ok_ty, err_ty))
    } else {
        None
    }
}

fn is_string_type(ty: &Type) -> bool {
    let Type::Path(type_path) = peel_paren(ty) else {
        return false;
    };
    type_path.qself.is_none()
        && type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "String" && seg.arguments.is_empty())
}

fn is_str_ref_type(ty: &Type) -> bool {
    let Type::Reference(type_ref) = peel_paren(ty) else {
        return false;
    };
    let Type::Path(type_path) = peel_paren(&type_ref.elem) else {
        return false;
    };
    type_path.qself.is_none()
        && type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "str" && seg.arguments.is_empty())
}

fn is_unit_type(ty: &Type) -> bool {
    matches!(peel_paren(ty), Type::Tuple(tuple) if tuple.elems.is_empty())
}

fn is_box_dyn_type(ty: &Type) -> bool {
    let Type::Path(type_path) = peel_paren(ty) else {
        return false;
    };
    let Some(last_seg) = type_path.path.segments.last() else {
        return false;
    };
    if type_path.qself.is_some() || last_seg.ident != "Box" {
        return false;
    }
    let PathArguments::AngleBracketed(generic_args) = &last_seg.arguments else {
        return false;
    };
    generic_args.args.iter().any(|arg| match arg {
        GenericArgument::Type(inner) => {
            matches!(peel_paren(inner), Type::TraitObject(to) if to.dyn_token.is_some())
        }
        _ => false,
    })
}

fn is_vague_error_type(ty: &Type) -> bool {
    let ty = peel_paren(ty);
    if is_anyhow_type(ty) {
        return true;
    }
    let Some((_ok_ty, err_ty)) = parse_result_args(ty) else {
        return false;
    };
    is_string_type(err_ty)
        || is_str_ref_type(err_ty)
        || is_unit_type(err_ty)
        || is_box_dyn_type(err_ty)
        || is_anyhow_type(err_ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(text: &str) -> Vec<Finding> {
        let file = SourceFile {
            path: PathBuf::from("src/test.rs"),
            text: text.to_string(),
        };
        check(&file)
    }

    #[test]
    fn test_forbidden_forms() {
        let cases = [
            "pub fn f() -> Result<(), String> { Ok(()) }",
            "pub fn f() -> Result<i32, String> { Ok(0) }",
            "pub fn f() -> Result<(), &str> { Ok(()) }",
            "pub fn f() -> Result<(), &'static str> { Ok(()) }",
            "pub fn f() -> Result<(), ()> { Ok(()) }",
            "pub fn f() -> Result<bool, ()> { Ok(true) }",
            "pub fn f() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }",
            "pub fn f() -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }",
            "pub fn f() -> Result<(), Box<dyn Error>> { Ok(()) }",
            "pub fn f() -> anyhow::Result<()> { Ok(()) }",
            "pub fn f() -> anyhow::Result<i32> { Ok(0) }",
            "pub fn f() -> anyhow::Error { loop {} }",
            "pub fn f() -> ::anyhow::Result<()> { Ok(()) }",
            "pub fn f() -> Result<(), anyhow::Error> { Ok(()) }",
        ];
        for code in cases {
            let findings = check_source(code);
            assert_eq!(findings.len(), 1, "failed for: {code}");
            assert_eq!(findings[0].rule, RULE_NAME);
            assert_eq!(findings[0].severity, Severity::Error);
            assert_eq!(findings[0].line, 1);
        }
    }

    #[test]
    fn test_forbidden_method() {
        let src = "struct S;\nimpl S {\n    pub fn m(&self) -> Result<(), String> { Ok(()) }\n}";
        let findings = check_source(src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 3);
    }

    #[test]
    fn test_allowed_forms() {
        let cases = [
            "fn private_fn() -> Result<(), String> { Ok(()) }",
            "pub(crate) fn crate_fn() -> Result<(), String> { Ok(()) }",
            "pub fn good_fn() -> Result<(), MoveError> { Ok(()) }",
            "pub fn ok_string() -> Result<String, MoveError> { Ok(String::new()) }",
            "pub fn ok_str() -> Result<&str, MoveError> { Ok(\"\") }",
            "pub fn plain_fn() -> String { String::new() }",
            "pub fn unit_fn() {}",
            "struct S;\nimpl S { fn m(&self) -> Result<(), String> { Ok(()) } }",
            "struct S;\nimpl S { pub fn m(&self) -> Result<(), MoveError> { Ok(()) } }",
            "invalid syntax {{{ ???",
        ];
        for code in cases {
            let findings = check_source(code);
            assert!(findings.is_empty(), "expected 0 findings for: {code}");
        }
    }
}
