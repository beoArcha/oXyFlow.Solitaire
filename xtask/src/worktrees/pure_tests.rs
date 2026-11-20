//! Tests of the parsing and option helpers of the worktree check; they need no repository.

use super::git::{Upstream, parse_porcelain, parse_upstream};
use super::pulls::finished_states;
use super::{Options, parse_options};

#[test]
fn porcelain_records_carry_branch_detached_and_missing_state() {
    let text = "worktree /r\nHEAD 1\nbranch refs/heads/main\n\n\
        worktree /w/a\nHEAD 2\nbranch refs/heads/feat/a\nlocked\n\n\
        worktree /w/b\nHEAD 3\ndetached\n\n\
        worktree /w/c\nbare\n\n\
        worktree /w/d\nHEAD 4\nbranch refs/heads/feat/d\nprunable gitdir file points to nowhere\n";
    let records = parse_porcelain(text);
    assert_eq!(records.len(), 4);
    assert_eq!(records[0].branch.as_deref(), Some("main"));
    assert_eq!(records[1].branch.as_deref(), Some("feat/a"));
    assert_eq!(records[2].branch, None);
    assert!(records[3].missing);
    assert_eq!(records[3].branch.as_deref(), Some("feat/d"));
}

#[test]
fn upstream_lines_tell_configured_and_gone_branches() {
    let gone = parse_upstream("refs/heads/feat/x\trefs/remotes/origin/feat/x\t[gone]");
    let expected_gone = Upstream {
        configured: true,
        gone: true,
    };
    assert_eq!(gone, Some(("feat/x".to_owned(), expected_gone)));
    let tracking = parse_upstream("refs/heads/a\trefs/remotes/origin/a\t");
    let expected_tracking = Upstream {
        configured: true,
        gone: false,
    };
    assert_eq!(tracking, Some(("a".to_owned(), expected_tracking)));
    let local = parse_upstream("refs/heads/b\t\t");
    assert_eq!(local, Some(("b".to_owned(), Upstream::default())));
    let based = parse_upstream("refs/heads/c\trefs/remotes/origin/main\t");
    assert_eq!(based, Some(("c".to_owned(), Upstream::default())));
    assert_eq!(parse_upstream("HEAD\t\t"), None);
}

#[test]
fn pull_request_states_need_no_open_request_and_one_finished() {
    assert!(finished_states("MERGED\n"));
    assert!(finished_states("CLOSED\nMERGED\n"));
    assert!(!finished_states("OPEN\nMERGED\n"));
    assert!(!finished_states(""));
}

#[test]
fn flags_parse_and_force_needs_remove() {
    let flags =
        |items: &[&str]| -> Vec<String> { items.iter().map(|item| (*item).to_owned()).collect() };
    assert_eq!(
        parse_options(&flags(&[])),
        Some(Options {
            remove: false,
            force: false
        })
    );
    assert_eq!(
        parse_options(&flags(&["--remove", "--force"])),
        Some(Options {
            remove: true,
            force: true
        })
    );
    assert_eq!(parse_options(&flags(&["--force"])), None);
    assert_eq!(parse_options(&flags(&["--bogus"])), None);
}
