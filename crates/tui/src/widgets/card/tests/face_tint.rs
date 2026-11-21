//! Every card face tint at every card size in every theme: the face and the suit colors come from
//! the tint, and no cell of the card is drawn in another color.

use super::*;
use crate::settings::CardFaceTint;
use crate::theme::Token;

fn tinted(id: ThemeId, tint: CardFaceTint) -> Fixture {
    Fixture {
        theme: Theme::new(id, TRUE_COLOR.color),
        glyphs: Glyphs::new(GlyphMode::Unicode),
        settings: Settings {
            card_face: tint,
            ..Settings::default()
        },
    }
}

fn draw_card(fixture: &Fixture, size: (u16, u16), card: Card) -> (Buffer, Rect) {
    let area = Rect::new(0, 0, size.0, size.1);
    let mut buf = Buffer::empty(area);
    draw(
        &mut buf,
        area,
        Face::Up(card),
        Mark::default(),
        None,
        &fixture.ctx(),
    );
    (buf, area)
}

fn assert_card_in_colors(fixture: &Fixture, tint: CardFaceTint, size: (u16, u16), suit: Suit) {
    let (buf, area) = draw_card(fixture, size, card(Rank::Ace, suit));
    let expected = fixture
        .theme
        .card_face_style(tint, fixture.glyphs.suit_token(suit));
    let mut inked = 0;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &buf[(x, y)];
            assert_eq!(
                cell.bg,
                expected.bg.unwrap_or_default(),
                "{tint:?} {size:?} {suit:?} at {x},{y}"
            );
            if cell.symbol() != " " {
                inked += 1;
                assert_eq!(
                    cell.fg,
                    expected.fg.unwrap_or_default(),
                    "{tint:?} {size:?} {suit:?} at {x},{y}"
                );
            }
        }
    }
    assert!(inked > 0, "{tint:?} {size:?} {suit:?}");
}

#[test]
fn every_tint_draws_every_card_size_in_every_theme_in_its_own_colors() {
    for id in ThemeId::ALL {
        for tint in CardFaceTint::ALL {
            let fixture = tinted(id, tint);
            for size in SIZES {
                for suit in [Suit::Hearts, Suit::Spades] {
                    assert_card_in_colors(&fixture, tint, size, suit);
                }
            }
        }
    }
}

#[test]
fn a_tint_other_than_the_theme_changes_the_face_color() {
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    let own = theme.card_face_style(CardFaceTint::Theme, Token::CardRed);
    for tint in CardFaceTint::ALL.into_iter().skip(1) {
        let style = theme.card_face_style(tint, Token::CardRed);
        assert_ne!(style.bg, own.bg, "{tint:?}");
    }
}
