use clap::Parser;
use image::{ImageBuffer, Rgb};
use palette::{IntoColor, Lch, Mix, Srgb};

#[derive(Parser)]
#[command(about = "Generate a PNG filled with a vertical LCH gradient")]
#[command(after_help = "\
COLORS: START[:END] where each color is a hex RGB value, e.g. #ff0000 or #ff0000:#0000ff
DIMENSIONS: either HEIGHTxWIDTH (e.g. 600x800) or HEIGHT WIDTH as separate args
GRID: COLSxROWS cell size in pixels, e.g. --grid 10x10 draws lines every 10 pixels")]
struct Args {
    /// Color(s): START or START:END hex RGB, e.g. #ff0000 or #ff0000:#0000ff
    colors: String,

    /// Dimensions and output path: HEIGHT WIDTH OUTPUT  or  HEIGHTxWIDTH OUTPUT
    #[arg(num_args = 2..=3)]
    rest: Vec<String>,

    /// Grid cell size in pixels, e.g. 10x10
    #[arg(long)]
    grid: Option<String>,

    /// Hex color for grid lines, e.g. #000000
    #[arg(long, default_value = "#000000")]
    grid_color: String,
}

fn parse_hex_color(s: &str) -> Result<[u8; 3], String> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Err(format!("expected 6 hex digits, got {}", s.len()));
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
    Ok([r, g, b])
}

fn parse_colors(s: &str) -> Result<([u8; 3], [u8; 3]), String> {
    match s.split_once(':') {
        Some((start, end)) => Ok((parse_hex_color(start)?, parse_hex_color(end)?)),
        None => {
            let rgb = parse_hex_color(s)?;
            Ok((rgb, rgb))
        }
    }
}

fn parse_dimensions(rest: &[String]) -> Result<(u32, u32, &str), String> {
    match rest {
        [dims, output] => {
            let (h_str, w_str) = dims
                .split_once('x')
                .ok_or_else(|| format!("expected HEIGHTxWIDTH, got '{dims}'"))?;
            let w = parse_positive_int(w_str, "width")?;
            let h = parse_positive_int(h_str, "height")?;
            Ok((w, h, output.as_str()))
        }
        [h_str, w_str, output] => {
            let w = parse_positive_int(w_str, "width")?;
            let h = parse_positive_int(h_str, "height")?;
            Ok((w, h, output.as_str()))
        }
        _ => unreachable!("clap enforces 2..=3 args"),
    }
}

fn parse_positive_int(s: &str, label: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(0) => Err(format!("{label} must be > 0")),
        Ok(n) => Ok(n),
        Err(_) => Err(format!("invalid {label} '{s}'")),
    }
}

fn parse_grid(s: &str) -> Result<(u32, u32), String> {
    let (cols_str, rows_str) = s
        .split_once('x')
        .ok_or_else(|| format!("expected COLSxROWS, got '{s}'"))?;
    let cols = parse_positive_int(cols_str, "grid columns")?;
    let rows = parse_positive_int(rows_str, "grid rows")?;
    Ok((cols, rows))
}

fn rgb_to_lch(rgb: [u8; 3]) -> Lch {
    let srgb = Srgb::new(
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    );
    srgb.into_color()
}

fn lch_to_rgb(lch: Lch) -> [u8; 3] {
    let srgb: Srgb<f32> = lch.into_color();
    [
        (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
        (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
        (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

#[rustfmt::skip]
const BAYER_8X8: [u8; 64] = [
     0, 32,  8, 40,  2, 34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44,  4, 36, 14, 46,  6, 38,
    60, 28, 52, 20, 62, 30, 54, 22,
     3, 35, 11, 43,  1, 33,  9, 41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47,  7, 39, 13, 45,  5, 37,
    63, 31, 55, 23, 61, 29, 53, 21,
];

fn lch_to_rgb_dithered(lch: Lch, x: u32, y: u32) -> [u8; 3] {
    let srgb: Srgb<f32> = lch.into_color();
    let d = BAYER_8X8[(y as usize % 8) * 8 + (x as usize % 8)] as f32 / 64.0 - 0.5;
    [
        ((srgb.red.clamp(0.0, 1.0) * 255.0 + d).round().clamp(0.0, 255.0)) as u8,
        ((srgb.green.clamp(0.0, 1.0) * 255.0 + d).round().clamp(0.0, 255.0)) as u8,
        ((srgb.blue.clamp(0.0, 1.0) * 255.0 + d).round().clamp(0.0, 255.0)) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_hex_color ---

    #[test]
    fn hex_color_with_hash() {
        assert_eq!(parse_hex_color("#ff0000"), Ok([255, 0, 0]));
    }

    #[test]
    fn hex_color_without_hash() {
        assert_eq!(parse_hex_color("00ff00"), Ok([0, 255, 0]));
    }

    #[test]
    fn hex_color_mixed_case() {
        assert_eq!(parse_hex_color("#0000FF"), Ok([0, 0, 255]));
    }

    #[test]
    fn hex_color_wrong_length() {
        assert!(parse_hex_color("#fff").is_err());
    }

    #[test]
    fn hex_color_invalid_chars() {
        assert!(parse_hex_color("#zzzzzz").is_err());
    }

    // --- parse_colors ---

    #[test]
    fn colors_start_and_end() {
        assert_eq!(parse_colors("#ff0000:#0000ff"), Ok(([255, 0, 0], [0, 0, 255])));
    }

    #[test]
    fn colors_start_only() {
        assert_eq!(parse_colors("#ff0000"), Ok(([255, 0, 0], [255, 0, 0])));
    }

    #[test]
    fn colors_invalid_start() {
        assert!(parse_colors("#zzzzzz:#0000ff").is_err());
    }

    #[test]
    fn colors_invalid_end() {
        assert!(parse_colors("#ff0000:#zzzzzz").is_err());
    }

    // --- parse_positive_int ---

    #[test]
    fn positive_int_valid() {
        assert_eq!(parse_positive_int("42", "x"), Ok(42));
    }

    #[test]
    fn positive_int_zero_rejected() {
        assert!(parse_positive_int("0", "x").is_err());
    }

    #[test]
    fn positive_int_negative_rejected() {
        assert!(parse_positive_int("-1", "x").is_err());
    }

    #[test]
    fn positive_int_non_numeric_rejected() {
        assert!(parse_positive_int("abc", "x").is_err());
    }

    // --- parse_dimensions ---

    #[test]
    fn dimensions_combined_form() {
        let args = vec!["300x600".to_string(), "out.png".to_string()];
        let (w, h, path) = parse_dimensions(&args).unwrap();
        assert_eq!((w, h, path), (600, 300, "out.png"));
    }

    #[test]
    fn dimensions_separate_form() {
        let args = vec!["300".to_string(), "600".to_string(), "out.png".to_string()];
        let (w, h, path) = parse_dimensions(&args).unwrap();
        assert_eq!((w, h, path), (600, 300, "out.png"));
    }

    #[test]
    fn dimensions_zero_rejected() {
        let args = vec!["0x600".to_string(), "out.png".to_string()];
        assert!(parse_dimensions(&args).is_err());
    }

    // --- parse_grid ---

    #[test]
    fn grid_valid() {
        assert_eq!(parse_grid("10x20"), Ok((10, 20)));
    }

    #[test]
    fn grid_missing_separator() {
        assert!(parse_grid("10").is_err());
    }

    #[test]
    fn grid_zero_rejected() {
        assert!(parse_grid("0x10").is_err());
    }

    // --- rgb_to_lch / lch_to_rgb round-trip ---

    fn round_trips(rgb: [u8; 3]) {
        let result = lch_to_rgb(rgb_to_lch(rgb));
        for i in 0..3 {
            assert!(
                (rgb[i] as i16 - result[i] as i16).abs() <= 1,
                "channel {i}: expected {}, got {} (input {:?})",
                rgb[i], result[i], rgb
            );
        }
    }

    #[test]
    fn lch_round_trip_red() {
        round_trips([255, 0, 0]);
    }

    #[test]
    fn lch_round_trip_green() {
        round_trips([0, 255, 0]);
    }

    #[test]
    fn lch_round_trip_blue() {
        round_trips([0, 0, 255]);
    }

    #[test]
    fn lch_round_trip_white() {
        round_trips([255, 255, 255]);
    }

    #[test]
    fn lch_round_trip_black() {
        round_trips([0, 0, 0]);
    }

    #[test]
    fn lch_round_trip_mid_gray() {
        round_trips([128, 128, 128]);
    }
}

fn main() {
    let args = Args::parse();

    let (start_rgb, end_rgb) = parse_colors(&args.colors).unwrap_or_else(|e| {
        eprintln!("Invalid colors '{}': {e}", args.colors);
        std::process::exit(1);
    });

    let (width, height, output) = parse_dimensions(&args.rest).unwrap_or_else(|e| {
        eprintln!("Invalid dimensions: {e}");
        std::process::exit(1);
    });

    let grid = args.grid.as_deref().map(|s| {
        parse_grid(s).unwrap_or_else(|e| {
            eprintln!("Invalid grid '{s}': {e}");
            std::process::exit(1);
        })
    });

    let [gr, gg, gb] = parse_hex_color(&args.grid_color).unwrap_or_else(|e| {
        eprintln!("Invalid grid-color '{}': {e}", args.grid_color);
        std::process::exit(1);
    });

    let lch_start = rgb_to_lch(start_rgb);
    let lch_end = rgb_to_lch(end_rgb);
    let dither = start_rgb != end_rgb;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        if let Some((cols, rows)) = grid {
            if x % cols == 0 || y % rows == 0 {
                return Rgb([gr, gg, gb]);
            }
        }
        let t = if height <= 1 { 0.0_f32 } else { y as f32 / (height - 1) as f32 };
        let mixed = lch_start.mix(lch_end, t);
        let [r, g, b] = if dither { lch_to_rgb_dithered(mixed, x, y) } else { lch_to_rgb(mixed) };
        Rgb([r, g, b])
    });

    img.save(output).unwrap_or_else(|e| {
        eprintln!("Failed to save '{output}': {e}");
        std::process::exit(1);
    });

    println!("Saved {width}x{height} PNG to {output}");
}
