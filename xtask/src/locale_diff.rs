//! `locale-diff [<lang>] [--ref <git-ref>]`: lists the keys a locale lacks, adds or writes with other
//! placeholders than `en-US` (decision T-38). Exit code 0 only when nothing differs. Files are read
//! from the working tree, or from a git ref with `--ref`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxysol_tui::i18n::diff::{self, Diff};
use oxysol_tui::i18n::{I18nError, Locale, parse_catalog};

#[derive(Debug, thiserror::Error)]
pub enum LocaleDiffError {
    #[error("unknown locale `{0}`; the folders are {1}")]
    UnknownLocale(String, String),
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot start `git`")]
    Spawn(#[source] std::io::Error),
    #[error("`git show {spec}` failed (exit code {code:?}); is the ref right?")]
    GitShow { spec: String, code: Option<i32> },
    #[error("`{spec}` is not UTF-8")]
    NotUtf8 { spec: String },
    #[error("cannot parse the locale files of {locale}")]
    Catalog {
        locale: &'static str,
        #[source]
        source: Box<I18nError>,
    },
    #[error("{count} locale(s) differ from en-US")]
    Differs { count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub lang: Option<String>,
    pub git_ref: Option<String>,
}

/// Reads the flags after `locale-diff`; `None` when they do not fit.
pub fn parse_options(flags: &[String]) -> Option<Options> {
    let mut options = Options {
        lang: None,
        git_ref: None,
    };
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        if flag == "--ref" && options.git_ref.is_none() {
            options.git_ref = Some(rest.next()?.clone());
        } else if !flag.starts_with("--") && options.lang.is_none() {
            options.lang = Some(flag.clone());
        } else {
            return None;
        }
    }
    Some(options)
}

fn read_text(
    root: &Path,
    git_ref: Option<&str>,
    relative: &str,
) -> Result<String, LocaleDiffError> {
    match git_ref {
        None => {
            let path = root.join(relative);
            fs::read_to_string(&path).map_err(|source| LocaleDiffError::Io { path, source })
        }
        Some(git_ref) => {
            let spec = format!("{git_ref}:{relative}");
            let output = Command::new("git")
                .arg("show")
                .arg(&spec)
                .current_dir(root)
                .output()
                .map_err(LocaleDiffError::Spawn)?;
            if !output.status.success() {
                return Err(LocaleDiffError::GitShow {
                    spec,
                    code: output.status.code(),
                });
            }
            String::from_utf8(output.stdout).map_err(|_| LocaleDiffError::NotUtf8 { spec })
        }
    }
}

fn read_catalog(
    root: &Path,
    git_ref: Option<&str>,
    locale: Locale,
) -> Result<oxysol_tui::i18n::Catalog, LocaleDiffError> {
    let mut texts = Vec::new();
    for area in Locale::area_names() {
        let relative = format!("locales/{}/{area}.toml", locale.id());
        texts.push((area, read_text(root, git_ref, &relative)?));
    }
    parse_catalog(texts.iter().map(|(area, text)| (*area, text.as_str()))).map_err(|source| {
        LocaleDiffError::Catalog {
            locale: locale.id(),
            source: Box::new(source),
        }
    })
}

fn compare(root: &Path, git_ref: Option<&str>, locale: Locale) -> Result<Diff, LocaleDiffError> {
    let source = read_catalog(root, git_ref, Locale::EnUs)?;
    let translated = read_catalog(root, git_ref, locale)?;
    Ok(diff::diff(&source, &translated, locale.plural_rule()))
}

fn set(keys: &std::collections::BTreeSet<String>) -> String {
    let names: Vec<String> = keys.iter().map(|name| format!("{{{name}}}")).collect();
    format!("[{}]", names.join(", "))
}

/// The summary line of a diff: `de-DE: 2 missing, 0 extra, 1 placeholder mismatch`.
pub fn summary(locale: Locale, result: &Diff) -> String {
    if result.is_clean() {
        return format!("{}: clean", locale.id());
    }
    format!(
        "{}: {} missing, {} extra, {} placeholder mismatch(es)",
        locale.id(),
        result.missing.len(),
        result.extra.len(),
        result.mismatched.len()
    )
}

/// The summary line followed by one line per differing key.
pub fn render(locale: Locale, result: &Diff) -> Vec<String> {
    let mut lines = vec![summary(locale, result)];
    lines.extend(result.missing.iter().map(|key| format!("  missing  {key}")));
    lines.extend(result.extra.iter().map(|key| format!("  extra    {key}")));
    lines.extend(result.mismatched.iter().map(|item| {
        format!(
            "  placeholders  {}: expected {}, found {}",
            item.key,
            set(&item.expected),
            set(&item.found)
        )
    }));
    lines
}

fn find_locale(lang: &str) -> Result<Locale, LocaleDiffError> {
    Locale::from_id(lang).ok_or_else(|| {
        let ids: Vec<&str> = Locale::ALL.iter().map(|locale| locale.id()).collect();
        LocaleDiffError::UnknownLocale(lang.to_owned(), ids.join(", "))
    })
}

pub fn run(root: &Path, options: &Options) -> Result<(), LocaleDiffError> {
    let git_ref = options.git_ref.as_deref();
    let (locales, detailed) = match &options.lang {
        Some(lang) => (vec![find_locale(lang)?], true),
        None => (Locale::ALL.into_iter().skip(1).collect(), false),
    };
    let mut differing = 0;
    for locale in locales {
        let result = compare(root, git_ref, locale)?;
        differing += usize::from(!result.is_clean());
        let lines = if detailed {
            render(locale, &result)
        } else {
            vec![summary(locale, &result)]
        };
        lines.iter().for_each(|line| println!("{line}"));
    }
    match differing {
        0 => Ok(()),
        count => Err(LocaleDiffError::Differs { count }),
    }
}

#[cfg(test)]
mod tests;
