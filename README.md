# color-png

A CLI tool that generates PNG images filled with a vertical linear color gradient.
Colors are interpolated in [LCH color space](https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model)
for perceptually uniform transitions.

## Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- <COLORS> <DIMENSIONS> <OUTPUT>
```

## Usage

```
color-png <COLORS> <DIMENSIONS> <OUTPUT> [OPTIONS]
```

| Argument | Description |
|---|---|
| `COLORS` | `START` or `START:END` — hex RGB colors, e.g. `#ff0000` or `#ff0000:#0000ff`. When only a start color is given, the output is a solid fill. |
| `DIMENSIONS` | `HEIGHTxWIDTH` (e.g. `600x800`) or `HEIGHT WIDTH` as two separate args |
| `OUTPUT` | Path for the output PNG file |

### Options

| Flag | Description |
|---|---|
| `--grid <COLSxROWS>` | Draw grid lines every N pixels, e.g. `--grid 10x10` |
| `--grid-color <HEX>` | Color for grid lines (default: `#000000`) |

## Examples

Red-to-blue gradient, 300 wide × 600 tall:

```bash
color-png "#ff0000:#0000ff" 300x600 gradient.png
```

Solid red fill (no gradient):

```bash
color-png "#ff0000" 300x600 solid.png
```

Gradient with a white grid overlay:

```bash
color-png "#ff0000:#0000ff" 300x600 gradient.png --grid 50x50 --grid-color "#ffffff"
```

Dimensions as separate arguments:

```bash
color-png "#ff0000:#0000ff" 600 300 gradient.png
```

## Color interpolation

Gradient colors are interpolated in LCH (Lightness–Chroma–Hue) color space using
the [`palette`](https://crates.io/crates/palette) crate. LCH is a cylindrical
representation of the CIELab color model and produces gradients that look uniform
to the human eye, avoiding the muddy mid-tones that straight RGB interpolation can
introduce.

Out-of-gamut values produced during LCH interpolation are clamped to the sRGB range
before writing to the PNG. Ordered (Bayer 8×8) dithering is applied during
quantization to eliminate visible banding — except when both colors are identical
(solid fill), in which case dithering is skipped. Ordered dithering uses a
deterministic pattern rather than random noise, producing a smoother result with no
color fringing.
