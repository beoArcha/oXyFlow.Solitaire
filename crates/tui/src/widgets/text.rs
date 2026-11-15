//! Text helpers shared by the widgets.

/// Words of `text` packed into rows of at most `width` columns; a longer word gets a row of its own.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

const ELLIPSIS: &str = "...";

/// `text` cut to `width` columns; a cut text ends with an ASCII ellipsis.
pub fn clip(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let kept = width.saturating_sub(ELLIPSIS.len());
    let head: String = text.chars().take(kept).collect();
    if kept == 0 {
        return text.chars().take(width).collect();
    }
    format!("{head}{ELLIPSIS}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_packed_into_rows() {
        assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
        assert_eq!(wrap("", 7), Vec::<String>::new());
        assert_eq!(wrap("a verylongword b", 4), vec!["a", "verylongword", "b"]);
    }

    #[test]
    fn a_long_text_ends_with_an_ellipsis() {
        assert_eq!(clip("short", 10), "short");
        assert_eq!(clip("abcdefgh", 6), "abc...");
        assert_eq!(clip("abcdefgh", 2), "ab");
        assert_eq!(clip("abc", 0), "");
    }
}
