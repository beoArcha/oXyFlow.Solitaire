# Application icon

The icon is a rounded green tile with three fanned playing cards. The front card shows a gold sun
("Sol") over a red X ("oXy"); the side cards carry a heart and a spade.

## Sources (vector only)

| File | Used for | Notes |
|---|---|---|
| `oxysol.svg` | 48, 64, 96, 128, 256 px | Full detail, 256 × 256 viewBox. |
| `oxysol-32.svg` | 32, 40 px | Simplified, 32 × 32 viewBox, edges aligned to the pixel grid. |
| `oxysol-16.svg` | 16, 20, 24 px | Hand-drawn for 16 × 16, axis-aligned edges on whole pixels. |

The sources use no text or fonts and no filters, only paths, gradients and basic shapes.

## Generated files (committed)

`generated/oxysol.ico` holds PNG-compressed entries at 16, 20, 24, 32, 40, 48, 64, 96, 128 and
256 px. `generated/oxysol-256.png` is the same 256 px image for documentation. On Windows the
root `build.rs` embeds the ICO as the icon of `oxysol.exe`.

## Regenerate

```sh
cargo xtask-art icon           # render and overwrite the generated files
cargo xtask-art icon --check   # fail if the committed files differ from a fresh render
```

Run `cargo xtask-art icon` after every change to an SVG and commit the generated files together with
the sources. The full `cargo xtask check` runs the `--check` form.
