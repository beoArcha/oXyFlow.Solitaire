# FreeCell

## Goal

Move all 52 cards to the four foundations.

## Layout

- 52 cards, all face up.
- 8 columns: the first 4 hold 7 cards, the other 4 hold 6.
- 4 cells.
- 4 foundations.

## Moves

- **Foundations:** By suit, ascending from the ace. No moves from foundations.
- **Tableau:** Descending, alternating colors. An empty column takes any card or run.
- **Cells:** A cell holds one card.
- **Allowed moves:**
  - Column to cell and cell to column.
  - Column to foundation.
  - Cell to foundation.
  - Column to column.
- **Moving runs between columns:** Moving *n* cards between columns needs a valid run and *n* ≤ (free cells + 1) × 2^(empty columns); an empty target column does not count as empty.

## Stock

FreeCell has no stock, and drawing from a stock is not supported.

## Winning

52 cards on the foundations.

## Variants

FreeCell has no variants.
