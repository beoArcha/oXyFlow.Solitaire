# Accessibility

oXySol is designed to work completely with either keyboard or mouse, without relying on color alone to convey game state.

## Keyboard, mouse and focus

- Every action has a key and a mouse action; buttons show their key, and `?` lists every key.
- The focused item is marked by a symbol (`►`, or `>` in ASCII). In windows, the active tab is shown in brackets,
  bold and underlined, and the focused button is framed by `▶ ◄` in reverse video, so the two never look alike.
- The line under the board says in words where the cursor is and what it can do there.

## Color vision

- **Color vision** (Settings, Audio & Visual): Normal, Protanopia, Deuteranopia, Tritanopia or Monochrome. Each mode
  replaces the colors that carry meaning (focus, target, hint, success, error) with colors that stay distinct for that
  kind of vision; tests simulate each one.
- **Four-color deck**: black spades, red hearts, blue diamonds and green clubs, in every mode and card size.
- Results carry a sign as well as a color: `✓` or `✗` (`+` or `x` in ASCII).

## Low-vision mode

**Low vision** is a single switch that configures a readable visual preset while leaving every individual option adjustable afterwards:

- **First start:** on the first launch (or with a settings file from an earlier version), an inline banner on the main menu offers to turn Low vision on (`Yes` / `Not now`). Either choice saves so the banner does not appear again.
- **Settings:** **Low vision** sits in **Audio & Visual** and in **Popular**.

### What turning it on sets

- **7:1 contrast:** switches the theme to **High contrast**, raises every text pair of any chosen theme to at least a 7:1 contrast ratio, and replaces muted text (`TextMuted`) with full-contrast text (`Text`).
- **Largest fitting card size:** sets **Card size** to **Auto** (up to 13×11 when the window allows).
- **Thick aura:** sets **Aura size** to **Thick** (two cells around focused, carried, and target cards) and draws halos in bold.
- **Bold cards and larger marks:** draws rank and suit labels, suit art, and focus/target borders in bold, doubles the focus marker (`►►` / `>>`) where space allows, puts target `+` marks in all four corners of target cards, and widens empty target slot markers (`+++`), without dimming covered cards.
- **Wider fan steps:** widens tableau, fanned waste, and pyramid steps when room permits, while still compressing and falling back to a smaller card size if needed so the last card of every column stays fully visible.
- **Larger console font:** in a classic Windows console opened by the game, raises **Font size** to at least 24 px (if smaller or at Console default). In Windows Terminal and other terminals, use the terminal's own zoom shortcut (`Ctrl+=` / `Ctrl+-`, `Ctrl+Shift+=` / `Ctrl+-`, or `Cmd+=` / `Cmd+-`).

### Adjusting and turning off

While **Low vision** is on, you can freely change the theme, card size, aura size, or console font size. Turning **Low vision** off restores the exact values of those settings from before it was turned on.
