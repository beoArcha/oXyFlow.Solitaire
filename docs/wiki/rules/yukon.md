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

- **Classic:** the rules above. Other members of the Yukon family (Alaska, Moosehide, Australian
  Patience) are planned.

## Russian Solitaire

The same layout, deal and group moves as Classic, with one change: a column builds downward in the
**same suit** instead of alternating colors. A 6 of hearts goes on a 7 of hearts only.

- **Tableau:** Descending by one rank, in the same suit (for example, the 6 of clubs on the 7 of clubs).
- **Group moves:** As in Classic. Only the bottom card of the moved group must fit the same-suit rule.
- **Empty column, foundations, face-down cards, stock:** As in Classic.
