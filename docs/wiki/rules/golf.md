# Golf

Sources: [Golf (patience)](https://en.wikipedia.org/wiki/Golf_(patience)) and the
[GoodSol Golf help](https://goodsol.com/pgshelp/golf.htm). Golf is a traditional public-domain game.

## Goal

Clear all 35 cards from the seven columns by playing them onto the waste.

## Layout

- 35 cards in 7 columns of 5, all face up.
- The other 17 cards form the stock. The deal turns the first stock card onto the waste to start
  it, so 16 cards stay in the stock.
- No foundation. One waste pile.

## Moves

- **Column to waste:** The top card of a column goes onto the waste when its rank is **one higher or
  one lower** than the top card of the waste, in any suit (a 7 goes onto a 6 or an 8). Only the top
  card of a column can be played, one card at a time.
- **King and ace are not adjacent:** There is no wrap-around. Only a queen goes onto a king and only
  a 2 onto an ace. (The rule is one constant in the engine, `games::golf::KING_ACE_ADJACENT`, so a
  wrap-around variant needs no new move code.)
- **Stock:** Turn one card at a time onto the waste. One pass only, no redeal. Cards never leave the
  waste.
- **Empty columns** are not refilled.

## Winning

Every column is empty, whatever is left in the stock. With the stock empty and no column card that
fits, the game is stuck.

## Winnable deals

The solver (decision T-45) knows the whole deal, stock order included. With "Winnable deals only" on,
a new game is a deal the solver has proven winnable, and the statistics check (`w` / `W`) answers for
Golf games too. About 56 % of deals are winnable with full knowledge of the stock.

## Variants

- **Classic:** the rules above.
