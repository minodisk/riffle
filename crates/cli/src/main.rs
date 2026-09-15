mod arw;
mod partial;

use anyhow::{anyhow, bail, Result};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("info") => info(Path::new(&args[1])),
        Some("focusbox") => focusbox(Path::new(&args[1]), Path::new(&args[2])),
        Some("bench") => bench(&args[1..]),
        Some("crop") => crop(Path::new(&args[1]), Path::new(&args[2])),
        _ => bail!("usage: riffle-cli <info|focusbox|crop|bench> <file.ARW> [out.png]"),
    }
}

fn info(path: &Path) -> Result<()> {
    let buf = std::fs::read(path)?;
    let a = arw::parse(&buf)?;
    println!("orientation: {}", a.orientation);
    println!("preview: {:?}", a.preview);
    println!("full:    {:?}", a.full);
    Ok(())
}

/// Leave the Sony MakerNote to exiftool while we are still validating.
fn focus_location(path: &Path) -> Result<(u32, u32, u32, u32)> {
    let out = std::process::Command::new("exiftool")
        .args(["-s3", "-FocusLocation"])
        .arg(path)
        .output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let v: Vec<u32> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if v.len() != 4 {
        bail!("FocusLocation not found: {text:?}");
    }
    Ok((v[0], v[1], v[2], v[3]))
}

fn decode_rgb(jpeg: &[u8]) -> Result<(Vec<u8>, usize, usize)> {
    let d = mozjpeg::Decompress::new_mem(jpeg)?.rgb()?;
    let (w, h) = (d.width(), d.height());
    let mut d = d;
    let pixels: Vec<[u8; 3]> = d.read_scanlines()?;
    d.finish()?;
    let flat = pixels.into_iter().flatten().collect();
    Ok((flat, w, h))
}

/// Rotate an RGB buffer per the Orientation tag. Only 1/6/8 show up in ARW files.
fn apply_orientation(rgb: &[u8], w: usize, h: usize, orientation: u16) -> (Vec<u8>, usize, usize) {
    let px = |x: usize, y: usize| {
        let i = (y * w + x) * 3;
        [rgb[i], rgb[i + 1], rgb[i + 2]]
    };
    match orientation {
        6 | 8 => {
            let (nw, nh) = (h, w);
            let mut out = vec![0u8; nw * nh * 3];
            for y in 0..nh {
                for x in 0..nw {
                    // 6 = Rotate 90 CW, 8 = Rotate 270 CW
                    let (sx, sy) = if orientation == 6 {
                        (y, nw - 1 - x)
                    } else {
                        (nh - 1 - y, x)
                    };
                    let p = px(sx, sy);
                    let i = (y * nw + x) * 3;
                    out[i..i + 3].copy_from_slice(&p);
                }
            }
            (out, nw, nh)
        }
        _ => (rgb.to_vec(), w, h),
    }
}

fn draw_rect(rgb: &mut [u8], w: usize, h: usize, x0: i64, y0: i64, bw: i64, bh: i64) {
    let mut put = |x: i64, y: i64| {
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let i = (y as usize * w + x as usize) * 3;
            rgb[i] = 255;
            rgb[i + 1] = 0;
            rgb[i + 2] = 0;
        }
    };
    for t in 0..3 {
        for x in x0..x0 + bw {
            put(x, y0 + t);
            put(x, y0 + bh - t);
        }
        for y in y0..y0 + bh {
            put(x0 + t, y);
            put(x0 + bw - t, y);
        }
    }
}

fn focusbox(path: &Path, out: &Path) -> Result<()> {
    let buf = std::fs::read(path)?;
    let a = arw::parse(&buf)?;
    let e = a.preview.ok_or_else(|| anyhow!("no preview in {path:?}"))?;

    let t = Instant::now();
    let (rgb, w, h) = decode_rgb(a.slice(&buf, e))?;
    println!("preview {w}x{h} decoded in {:?}", t.elapsed());

    let (fw, fh, fx, fy) = focus_location(path)?;
    let frame: u32 = 219;
    println!("focus: sensor {fw}x{fh} at ({fx},{fy}) frame {frame}");

    // FocusLocation is in unrotated sensor coordinates, so draw on the unrotated preview first.
    let sx = w as f64 / fw as f64;
    let sy = h as f64 / fh as f64;
    let bw = (frame as f64 * sx) as i64;
    let bh = (frame as f64 * sy) as i64;
    let mut rgb = rgb;
    draw_rect(
        &mut rgb,
        w,
        h,
        (fx as f64 * sx) as i64 - bw / 2,
        (fy as f64 * sy) as i64 - bh / 2,
        bw,
        bh,
    );

    // Rotate to display orientation, then write it out.
    let (rgb, w, h) = apply_orientation(&rgb, w, h, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?} ({w}x{h}, orientation {})", a.orientation);
    Ok(())
}

const CROP_SIZE: usize = 512;

fn stats(label: &str, mut ms: Vec<f64>) {
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = ms.len();
    let mean = ms.iter().sum::<f64>() / n as f64;
    println!(
        "{label:<28} n={n:<4} mean {mean:6.1}ms  median {:6.1}ms  p95 {:6.1}ms  max {:6.1}ms",
        ms[n / 2],
        ms[((n as f64 * 0.95) as usize).min(n - 1)],
        ms[n - 1]
    );
}

fn bench(paths: &[String]) -> Result<()> {
    let mut t_preview = Vec::new();
    let mut t_full = Vec::new();
    let mut t_crop = Vec::new();

    for p in paths {
        let path = Path::new(p);
        let buf = std::fs::read(path)?;
        let a = arw::parse(&buf)?;

        if let Some(e) = a.preview {
            let jpeg = a.slice(&buf, e);
            let t = Instant::now();
            decode_rgb(jpeg)?;
            t_preview.push(t.elapsed().as_secs_f64() * 1000.0);
        }

        if let Some(e) = a.full {
            let jpeg = a.slice(&buf, e);
            let t = Instant::now();
            decode_rgb(jpeg)?;
            t_full.push(t.elapsed().as_secs_f64() * 1000.0);

            let (fw, _fh, fx, fy) = focus_location(path)?;
            // FocusLocation is in sensor coordinates; convert using the full JPEG width.
            let scale = 1.0; // JpgFromRaw is the same 7008 width FocusLocation reports
            let _ = fw;
            let t = Instant::now();
            let c = partial::decode_crop(
                jpeg,
                (fx as f64 * scale) as usize,
                (fy as f64 * scale) as usize,
                CROP_SIZE,
            )?;
            t_crop.push(t.elapsed().as_secs_f64() * 1000.0);
            let _ = c;
        }
    }

    println!();
    if !t_preview.is_empty() {
        stats("1. preview 1616px decode", t_preview);
    }
    if !t_full.is_empty() {
        stats("2. JpgFromRaw full decode", t_full);
    }
    if !t_crop.is_empty() {
        stats("3. partial decode 512px", t_crop);
    }
    Ok(())
}

/// Write out the partial decode so it can be checked by eye.
fn crop(path: &Path, out: &Path) -> Result<()> {
    let buf = std::fs::read(path)?;
    let a = arw::parse(&buf)?;
    let e = a.full.ok_or_else(|| anyhow!("no JpgFromRaw in {path:?}"))?;
    let (_fw, _fh, fx, fy) = focus_location(path)?;

    let t = Instant::now();
    let c = partial::decode_crop(a.slice(&buf, e), fx as usize, fy as usize, CROP_SIZE)?;
    println!(
        "crop {}x{} at ({},{}) in {:?}",
        c.width,
        c.height,
        c.x,
        c.y,
        t.elapsed()
    );

    let (rgb, w, h) = apply_orientation(&c.rgb, c.width, c.height, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?}");
    Ok(())
}
