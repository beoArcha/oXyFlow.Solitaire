# TriPeaks

Sources: [Tri Peaks (solitaire)](https://en.wikipedia.org/wiki/Tri_Peaks_(solitaire)) and traditional
public-domain rules.

## Goal

Clear all 28 cards from the three overlapping peaks by playing them onto the waste.

## Layout

- 28 cards dealt into three overlapping peaks of 4 rows: 3, 6, 9, and 10 cards from top to bottom.
  Only the bottom row of 10 cards starts face up; a covered card turns face up when both cards
  covering it are gone.
- The other 24 cards form the talon: the first card is turned onto the waste to start it, and the
  remaining 23 stay in the stock.
- No foundation. One waste pile.

## Moves

- **Peak to waste:** Any uncovered face-up peak card goes onto the waste when its rank is **one
  higher or one lower** than the top card of the waste, in any suit (a 7 goes onto a 6 or an 8).
- **King and ace wrap around:** A king may be played onto an ace, and an ace may be played onto a
  king. (The rule is one constant in the engine, `games::tripeaks::KING_ACE_ADJACENT`.)
- **Stock:** Turn one card at a time onto the waste. One pass only, no redeal. Cards never leave the
  waste.

## Winning

All 28 peak cards are cleared, whatever is left in the stock. With the stock empty and no peak card
that fits, the game is stuck.

## Winnable deals

The solver knows the whole deal, stock order included. With "Winnable deals only" on, a new game is
a deal the solver has proven winnable, and the statistics check answers for TriPeaks games too.
About 94 % of deals are winnable with full knowledge of the stock (seeds 0–49: 94 % winnable, 6 %
unwinnable, 0 % unknown).
