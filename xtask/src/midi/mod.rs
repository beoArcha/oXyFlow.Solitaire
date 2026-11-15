//! Score-to-MIDI compilation tool.

pub mod parse;
#[cfg(test)]
pub mod tests;
pub mod write;

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Error during MIDI score processing or compilation.
#[derive(Debug, Error)]
pub enum MidiError {
    #[error("syntax error in `{path}:{line}` near token `{token}`: {message}")]
    Syntax {
        path: PathBuf,
        line: usize,
        token: String,
        message: String,
    },
    #[error("style directory `{path}` does not exist")]
    MissingStyleDir { path: PathBuf },
    #[error("scores directory `{path}` does not exist")]
    MissingScoresDir { path: PathBuf },
    #[error("scores directory `{path}` is empty")]
    EmptyScoresDir { path: PathBuf },
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Compiles scores in `assets/audio/<style>/scores/*.txt` into `.mid` files in `assets/audio/<style>/`.
pub fn run(root: &Path, style: &str) -> Result<(), MidiError> {
    let style_dir = root.join("assets").join("audio").join(style);
    if !style_dir.is_dir() {
        return Err(MidiError::MissingStyleDir { path: style_dir });
    }

    let scores_dir = style_dir.join("scores");
    if !scores_dir.is_dir() {
        return Err(MidiError::MissingScoresDir { path: scores_dir });
    }

    let entries = fs::read_dir(&scores_dir).map_err(|source| MidiError::Io {
        path: scores_dir.clone(),
        source,
    })?;

    let mut score_files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| MidiError::Io {
            path: scores_dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("txt") {
            score_files.push(path);
        }
    }

    if score_files.is_empty() {
        return Err(MidiError::EmptyScoresDir { path: scores_dir });
    }

    score_files.sort();

    for score_path in &score_files {
        let stem = score_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| MidiError::Io {
                path: score_path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid score file name",
                ),
            })?;

        let text = fs::read_to_string(score_path).map_err(|source| MidiError::Io {
            path: score_path.clone(),
            source,
        })?;

        let score = parse::parse_score(&text, score_path)?;
        let out_path = style_dir.join(format!("{stem}.mid"));
        write::write_midi(&score, &out_path)?;
        println!(
            "compiled {} -> {}",
            score_path.display(),
            out_path.display()
        );
    }

    Ok(())
}
