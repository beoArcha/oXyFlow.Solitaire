//! Matching of the paths on a card's `Files` line. `*` and `?` match inside one path segment and
//! `**` matches any number of whole segments, zero included.

pub fn matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    segments_match(&pattern, &path)
}

fn segments_match(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| segments_match(rest, &path[skip..])),
        Some((segment, rest)) => path.split_first().is_some_and(|(first, tail)| {
            wildcard(segment.as_bytes(), first.as_bytes()) && segments_match(rest, tail)
        }),
    }
}

fn wildcard(pattern: &[u8], text: &[u8]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((&b'*', rest)) => (0..=text.len()).any(|skip| wildcard(rest, &text[skip..])),
        Some((&b'?', rest)) => text
            .split_first()
            .is_some_and(|(_, tail)| wildcard(rest, tail)),
        Some((literal, rest)) => text
            .split_first()
            .is_some_and(|(first, tail)| first == literal && wildcard(rest, tail)),
    }
}
