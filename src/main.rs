use clap::Parser;
use image::{ImageBuffer, Rgb};

#[derive(Parser)]
#[command(about = "Generate a PNG filled with a solid color")]
#[command(after_help = "DIMENSIONS: either HEIGHTxWIDTH (e.g. 600x800) or HEIGHT WIDTH as separate args")]
struct Args {
    /// Hex color code, e.g. #123456
    color: String,

    /// Dimensions and output path: WIDTH HEIGHT OUTPUT  or  WIDTHxHEIGHT OUTPUT
    #[arg(num_args = 2..=3)]
    rest: Vec<String>,
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

fn main() {
    let args = Args::parse();

    let [r, g, b] = parse_hex_color(&args.color).unwrap_or_else(|e| {
        eprintln!("Invalid color '{}': {}", args.color, e);
        std::process::exit(1);
    });

    let (width, height, output) = parse_dimensions(&args.rest).unwrap_or_else(|e| {
        eprintln!("Invalid dimensions: {e}");
        std::process::exit(1);
    });

    let img = ImageBuffer::from_fn(width, height, |_, _| Rgb([r, g, b]));

    img.save(output).unwrap_or_else(|e| {
        eprintln!("Failed to save '{output}': {e}");
        std::process::exit(1);
    });

    println!("Saved {width}x{height} PNG to {output}");
}
