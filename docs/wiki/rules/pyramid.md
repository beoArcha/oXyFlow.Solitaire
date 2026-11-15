# Pyramid

## Goal

Remove cards by pairing available cards whose values sum to 13, or by removing individual Kings, until the victory condition for the chosen variant is satisfied.

## Layout

Pyramid is played with a single 52-card deck:
- **Pyramid:** 28 cards dealt face up in 7 rows. Row *r* (starting at row 0) contains *r* + 1 cards, forming a triangle from 1 card in the top row to 7 cards in the seventh row.
- **Stock:** The remaining 24 cards form the stock pile, with its top card face up.
- **Waste:** An initially empty discard pile where cards drawn from the stock are placed.
- **Foundation:** The pile where paired and removed cards are retired from play.

A pyramid card at row *r*, index *i* is available only after both cards covering it in the row below—at positions (*r* + 1, *i*) and (*r* + 1, *i* + 1)—have been removed. The cards in the bottom row (row 6) are uncovered and available from the start.

At any point, the available cards in play are:
- Any available pyramid cards.
- The top card of the stock.
- The top card of the waste.

Card values:
- Ace: 1
- Cards 2 through 10: face value
- Jack: 11
- Queen: 12
- King: 13

## Moves

- **Pair Removal:** Any two different available cards summing to 13 can be removed together to the foundation. Available cards may be paired from the pyramid, stock, or waste; in particular, the top of the stock and the top of the waste form a valid pair.
- **King Removal:** An available King has a value of 13 on its own and is removed individually to the foundation without needing a second card.
- **Transfer:** Moving or transferring cards between table positions is unsupported.

## Stock

- Drawing moves the top card of the stock to the waste.
- When the stock is empty and passes remain, the waste returns to the stock in the original order.
- After the final pass has been used, the stock is empty and no further draws can be made.

## Winning

- **Relaxed:** The game is won when the pyramid is empty (all 28 pyramid cards are removed).
- **Classic:** The game is won when all 52 cards of the deck are on the foundation.

## Variants

- **Relaxed:** 3 passes through the stock; won when the pyramid is empty.
- **Classic:** 1 pass through the stock; won when all 52 cards are on the foundation.
