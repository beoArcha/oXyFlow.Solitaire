//! Tests for source selection, the ICO container and the comparison tolerance.

use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Tree};

use super::ico;
use super::render::{Source, render_tree};
use super::{MAX_CHANNEL_DIFFERENCE, SIZES, pixels_match};

const SQUARE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><rect width="8" height="8" fill="#ff0000"/></svg>"##;

fn square(size: u32) -> Pixmap {
    let tree = Tree::from_data(SQUARE.as_bytes(), &Options::default()).unwrap();
    render_tree(&tree, size).unwrap()
}

#[test]
fn sources_are_chosen_by_size() {
    assert_eq!(Source::for_size(16), Source::Small);
    assert_eq!(Source::for_size(24), Source::Small);
    assert_eq!(Source::for_size(32), Source::Medium);
    assert_eq!(Source::for_size(40), Source::Medium);
    assert_eq!(Source::for_size(48), Source::Full);
    assert_eq!(Source::for_size(256), Source::Full);
}

#[test]
fn every_windows_shell_size_is_listed() {
    for size in [16, 24, 32, 48, 256] {
        assert!(SIZES.contains(&size), "{size} px is missing");
    }
}

#[test]
fn rendering_scales_to_the_requested_size() {
    let pixmap = square(20);
    assert_eq!((pixmap.width(), pixmap.height()), (20, 20));
    assert_eq!(&pixmap.data()[..4], &[255, 0, 0, 255]);
}

#[test]
fn ico_round_trips_entries_including_256() {
    let entries = vec![(16, vec![1, 2, 3]), (256, vec![4, 5, 6, 7])];
    let bytes = ico::build(&entries);
    let parsed = ico::parse(&bytes).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0], (16, &[1u8, 2, 3][..]));
    assert_eq!(parsed[1], (256, &[4u8, 5, 6, 7][..]));
    assert_eq!(bytes[2], 1);
}

#[test]
fn truncated_ico_is_an_error() {
    let bytes = ico::build(&[(16, vec![1, 2, 3])]);
    assert!(ico::parse(&bytes[..bytes.len() - 1]).is_err());
    assert!(ico::parse(&[0, 0]).is_err());
}

#[test]
fn comparison_tolerates_rounding_but_not_a_different_picture() {
    let base = square(8);
    let mut nudged = base.clone();
    nudged.data_mut()[0] -= MAX_CHANNEL_DIFFERENCE;
    assert!(pixels_match(&base, &nudged));
    nudged.data_mut()[0] -= 1;
    assert!(!pixels_match(&base, &nudged));
    assert!(!pixels_match(&base, &square(9)));
}

#[test]
fn committed_icon_is_up_to_date() {
    let root = crate::check::workspace_root().expect("workspace root");
    super::run(&root, true).expect("generated icon files match the sources");
}
