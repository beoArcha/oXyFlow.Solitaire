**English** · [Polski](pl/features.md) · [Deutsch](de/features.md) · [Español](es/features.md) · [Français](fr/features.md) · [Italiano](it/features.md) · [Nederlands](nl/features.md) · [Português (Brasil)](pt-br/features.md) · [Українська](uk/features.md)

# Features

## Four games

- **Klondike**: draw 1 or draw 3.
- **Spider**: 1, 2 or 4 suits.
- **FreeCell**.
- **Pyramid**: relaxed or classic.

One binary, one menu, one set of keys.

## Keyboard and mouse

Every action works with the keyboard and with the mouse. Click a card to pick it up and click a target to drop it,
double-click to send a card to a foundation, drag a card onto its target, right-click to put it back.
The full key list is in the help window (`?`).

## Card size

Make the cards bigger with `]` and smaller with `[`; `0` goes back to the automatic size, which fits the window.
Ctrl+wheel and the Card size setting do the same. A size that does not fit the window falls back by itself.
In the classic Windows console, Ctrl+Plus, Ctrl+Minus and Ctrl+0 change the console font size.

## Help while you play

- **Guide panel** (`g`): the goal of the game, the keys, and what the focused card can do. It sits beside the board
  on terminals 130 columns wide or more, and opens in a window on narrower ones.
- **Possible moves panel** (`v`): every move you can make now, each with its key.
- **Hints** (`h`): the game marks a move on the board and names it.
- **Auto-finish**: when the game is certainly won, the remaining moves are played out for you. It can be switched off
  in Settings.
- **Undo and redo** (`u`, `r`), with an option to play without undo.

## Seeds, winnable deals, statistics

Every deal has a **seed**. Copy it (`c`) to replay or share a deal, or start a game from any seed
with "New game from seed". An option makes a new game search for a deal that is proven winnable.
The **statistics** screen shows results per game and a history of every deal, newest first, and lets you play a
seed again.

## Themes for comfort and accessibility

Five themes, chosen in Settings. Each one is shown here on the same position.

**Dark** and **Light** for everyday play.

![The Dark theme](img/en/theme-dark.svg)

![The Light theme](img/en/theme-light.svg)

**High contrast** is made for players with low vision: white text on pure black with bright yellow and cyan accents, a frame around
the playing field, and a text contrast ratio of at least 7:1 (the other themes keep at least 4.5:1; tests check both).

![The High contrast theme](img/en/theme-high-contrast.svg)

**Monokai** and **Dracula** are available as well.

![The Monokai theme](img/en/theme-monokai.svg)

![The Dracula theme](img/en/theme-dracula.svg)

**Color is never the only carrier of information.** Suits are drawn as signs (♠ ♥ ♦ ♣) next to the rank, the focused
item is marked with a symbol, picked cards and drop targets have their own marks, and a line under the board says in
words where the cursor is and what it can do. The game stays usable for color-blind players. Dedicated color-vision
modes are planned; they are not in the game yet.

## Languages

The game interface is available in **English** and **Polish** today; German, French, Italian, Brazilian Portuguese,
Ukrainian, Dutch and Spanish are in progress. Pick the language with the flag selector in the top-left corner of the
main menu, or in Settings; it applies at once.

This website is available in nine languages: the links at the top of every page switch between them.

## What it is not yet

oXySol is a pre-release. Expect rough edges, and tell us about them in the
[issue tracker](https://github.com/beoArcha/oXyFlow.Solitare/issues).
