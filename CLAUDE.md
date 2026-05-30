# CLAUDE.md — color-png

GitHub: https://github.com/njacobs5074/color-png

## What this project does

Generates PNG images with a vertical linear color gradient between two hex RGB colors,
interpolated in LCH color space. Single binary, single source file.

## Key files

| File | Purpose |
|---|---|
| `src/main.rs` | All logic: CLI parsing, color conversion, image generation |
| `Cargo.toml` | Dependencies: `clap` (CLI), `image` (PNG output), `palette` (LCH color space) |

## Architecture

Everything lives in `src/main.rs`. The main pipeline is:

1. Parse CLI args with `clap` (derive API)
2. `parse_hex_color` → `[u8; 3]`
3. `rgb_to_lch` converts both endpoints to `palette::Lch`
4. `image::ImageBuffer::from_fn` iterates pixels; each row computes `t = y / (height-1)` and calls `lch_start.mix(lch_end, t)`
5. `lch_to_rgb` converts back, clamping to sRGB gamut
6. Grid lines (if `--grid`) are drawn on top before returning the pixel

## Common commands

```bash
cargo build                  # compile
cargo run -- "#ff0000" "#0000ff" 300x600 out.png   # run
cargo clippy                 # lint
cargo fmt                    # format
```

## Extending this project

- **Horizontal gradients**: pass `x` instead of `y` to compute `t` in the pixel closure; consider adding a `--direction` flag
- **Multiple color stops**: replace the two-color `Args` fields with a `Vec<String>` of stops and segment `t` across intervals
- **Shortest-hue-arc interpolation**: `palette::Lch`'s `Mix` impl does naive linear hue interpolation; for hues crossing the 0/360 boundary, manual wrapping may be needed
