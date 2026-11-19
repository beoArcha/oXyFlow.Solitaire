use oxysol_core::{Rank, Suit};
use ratatui::style::Color;

use super::*;

const FOCUS_SIZES: [(u16, u16); 5] = [(13, 11), (11, 9), (9, 7), (7, 6), (7, 5)];

fn focused() -> Mark {
    Mark {
        focus: true,
        ..Mark::default()
    }
}

fn felt_cells(buf: &Buffer, area: Rect, felt: Color) -> Vec<(u16, u16)> {
    (area.top()..area.bottom())
        .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
        .filter(|(x, y)| buf[(*x, *y)].bg == felt)
        .collect()
}

#[test]
fn focused_card_shows_no_felt_inside_its_rectangle() {
    for mode in MODES {
        let fixture = Fixture::new(mode);
        let Some(felt) = fixture.theme.style(Token::TableText, Token::Table).bg else {
            unreachable!("the theme paints the felt")
        };
        for size in FOCUS_SIZES {
            for (kind, face) in [
                ("up", Face::Up(card(Rank::Nine, Suit::Hearts))),
                ("down", Face::Down),
            ] {
                let (buf, area) = padded_face(mode, size, (face, focused(), None));
                let leaked = felt_cells(&buf, area, felt);
                assert!(
                    leaked.is_empty(),
                    "{mode:?} {size:?} focused {kind}: felt inside the card at {leaked:?}"
                );
            }
        }
    }
}
