# Yukon

Sources: [Yukon (solitaire)](https://en.wikipedia.org/wiki/Yukon_(solitaire)) and the PySolFC rule
description.

## Goal

Move all 52 cards to the four foundations.

## Layout

- 52 cards, all dealt to the tableau.
- 7 tableau columns: the first holds 1 face-up card; column *i* (2 to 7) holds *i* − 1 face-down
  cards under 5 face-up cards. 21 cards are face down, 31 face up.
- 4 foundations.
- No stock and no waste.

## Moves

- **Tableau:** Descending, alternating colors.
- **Group moves:** Any face-up card moves together with every card above it, in or out of sequence.
  Only the bottom card of the moved group must fit the target.
- **Empty column:** Takes only a king, or a group with a king at the bottom.
- **Foundations:** By suit, ascending from ace to king. An ace goes to any empty foundation.
- **Allowed moves:**
  - Tableau to tableau (a group of any size).
  - Tableau to foundation (one card).
  - Foundation to tableau (one card).
- **Revealing cards:** After cards leave a column, a face-down card on top flips automatically.

## Stock

Yukon has no stock, and drawing from a stock is not supported. There is no redeal.

## Winning

52 cards on the foundations.

## Winnable deals

The solver does not cover Yukon yet (P4-10f). With "Winnable deals only" on, a Yukon game is dealt
from a random seed and the game menu says so next to the game's name.

## Variants

- **Classic:** the rules above. The other members of the family follow below.

## Russian Solitaire

The same layout, deal and group moves as Classic, with one change: a column builds downward in the
**same suit** instead of alternating colors. A 6 of hearts goes on a 7 of hearts only.

- **Tableau:** Descending by one rank, in the same suit (for example, the 6 of clubs on the 7 of clubs).
- **Group moves:** As in Classic. Only the bottom card of the moved group must fit the same-suit rule.
- **Empty column, foundations, face-down cards, stock:** As in Classic.

## Alaska

The same layout, deal and group moves as Classic, with one change: a column builds up or down by one
rank in the **same suit**. A 6 of hearts goes on a 7 of hearts or a 5 of hearts, and nothing else.

- **Tableau:** One rank higher or lower, in the same suit (for example, the 6 of clubs on the 5 or the 7 of clubs).
- **Group moves:** As in Classic. Only the bottom card of the moved group must fit the same-suit rule.
- **Empty column, foundations, face-down cards, stock:** As in Classic.

## Moosehide

The same layout, deal and group moves as Classic, with one change: a column builds down by one rank in
**any suit except the card's own**. A 6 of clubs goes on a 7 of spades, but a 6 of spades does not.

- **Tableau:** One rank lower, in any suit but the suit of the card it goes on (for example, the 6 of
  clubs or the 6 of hearts on the 7 of spades, but not the 6 of spades).
- **Group moves:** As in Classic. Only the bottom card of the moved group must fit the rule.
- **Empty column, foundations, face-down cards, stock:** As in Classic.

## Australian Patience

The Yukon group moves with a stock and a different deal. The seven columns hold **four face-up cards each**
(28 cards, none face down); the other 24 cards form the stock.

- **Stock:** Turn the stock one card at a time onto the waste. There is one pass only and no redeal: once
  the stock is empty, the waste stays where it is. The top card of the waste can go to a column or a
  foundation.
- **Tableau:** One rank lower in the **same suit** (a 6 of hearts goes on a 7 of hearts only).
- **Group moves:** As in Classic. Only the bottom card of the moved group must fit the same-suit rule.
- **Empty column:** Only a king, or a group with a king at the bottom.
- **Foundations:** Ace to king by suit, as in Classic.
- **Auto-finish:** Offered only when the stock and the waste are empty, so that no unseen card is left.
