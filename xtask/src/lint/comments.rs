//! Rule `comments`: no ordinary comment inside a function or method body, and no decorative
//! separator anywhere. Doc comments and `// SAFETY:` comments are allowed inside bodies.

use std::ops::Range;

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use syn::visit::Visit;

use super::{Finding, Severity, SourceFile};

const SAFETY_PREFIX: &str = "// SAFETY:";
const SEPARATOR_MIN_LEN: usize = 3;

pub fn check(file: &SourceFile) -> Vec<Finding> {
    let bodies = function_bodies(&file.text);
    collect_comments(&file.text)
        .iter()
        .filter_map(|comment| finding_for(file, comment, &bodies))
        .collect()
}

struct Comment<'a> {
    start: usize,
    text: &'a str,
    doc: bool,
}

fn finding_for(
    file: &SourceFile,
    comment: &Comment<'_>,
    bodies: &[Range<usize>],
) -> Option<Finding> {
    let message = if is_separator(comment.text) {
        "decorative separator comment; remove it or name the section in code"
    } else if is_inside_body(comment.start, bodies) && !is_allowed_in_body(comment) {
        "comment inside a function body; use a well-named function or constant instead"
    } else {
        return None;
    };
    Some(Finding {
        rule: "comments",
        severity: Severity::Error,
        file: file.path.clone(),
        line: line_of(&file.text, comment.start),
        message: message.to_owned(),
    })
}

fn is_inside_body(start: usize, bodies: &[Range<usize>]) -> bool {
    bodies.iter().any(|body| body.contains(&start))
}

fn is_allowed_in_body(comment: &Comment<'_>) -> bool {
    comment.doc || comment.text.starts_with(SAFETY_PREFIX)
}

fn collect_comments(text: &str) -> Vec<Comment<'_>> {
    let mut comments = Vec::new();
    let mut start = 0;
    for token in tokenize(text, FrontmatterAllowed::No) {
        let end = start + token.len as usize;
        if let (Some(doc), Some(lexeme)) = (doc_flag(token.kind), text.get(start..end)) {
            comments.push(Comment {
                start,
                text: lexeme,
                doc,
            });
        }
        start = end;
    }
    comments
}

fn doc_flag(kind: TokenKind) -> Option<bool> {
    match kind {
        TokenKind::LineComment { doc_style } | TokenKind::BlockComment { doc_style, .. } => {
            Some(doc_style.is_some())
        }
        _ => None,
    }
}

fn is_separator(lexeme: &str) -> bool {
    let symbols: Vec<char> = separator_body(lexeme)
        .chars()
        .filter(|symbol| !symbol.is_whitespace())
        .collect();
    match symbols.split_first() {
        Some((first, rest)) => {
            symbols.len() >= SEPARATOR_MIN_LEN
                && first.is_ascii_punctuation()
                && rest.iter().all(|symbol| symbol == first)
        }
        None => false,
    }
}

fn separator_body(lexeme: &str) -> &str {
    match lexeme.strip_prefix("/*") {
        Some(inner) => inner.strip_suffix("*/").unwrap_or(inner),
        None => lexeme.strip_prefix("//").unwrap_or(lexeme),
    }
}

fn function_bodies(text: &str) -> Vec<Range<usize>> {
    let Ok(file) = syn::parse_str::<syn::File>(text) else {
        return Vec::new();
    };
    let mut collector = BodyCollector::default();
    collector.visit_file(&file);
    collector.bodies
}

#[derive(Default)]
struct BodyCollector {
    bodies: Vec<Range<usize>>,
}

impl BodyCollector {
    fn add(&mut self, block: &syn::Block) {
        let open = block.brace_token.span.open().byte_range();
        let close = block.brace_token.span.close().byte_range();
        self.bodies.push(open.start..close.end);
    }
}

impl<'ast> Visit<'ast> for BodyCollector {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.add(&node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.add(&node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.add(block);
        }
        syn::visit::visit_trait_item_fn(self, node);
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    text.get(..offset)
        .map_or(1, |before| before.matches('\n').count() + 1)
}

#[cfg(test)]
#[path = "comments/tests.rs"]
mod tests;
