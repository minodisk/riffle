use anyhow::{anyhow, bail, Result};
use riffle_core::arw;
use riffle_core::decode::{apply_orientation, decode_rgb};
use riffle_core::partial;
use riffle_core::reader;
use riffle_core::scan;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("info") => info(Path::new(&args[1])),
        Some("focusbox") => focusbox(Path::new(&args[1]), Path::new(&args[2])),
        Some("bench") => bench(&args[1..]),
        Some("crop") => crop(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args.get(3).map(|s| s.parse()).transpose()?,
        ),
        Some("scan") => scan_dir(Path::new(&args[1]), args.get(2).map(|t| t.parse()).transpose()?),
        _ => bail!(
            "usage: riffle-cli <info|focusbox|bench> <file.ARW> [out.png]\n       riffle-cli crop <file.ARW> <out.png> [size]\n       riffle-cli scan <dir> [threads]"
        ),
    }
}

fn info(path: &Path) -> Result<()> {
    let buf = std::fs::read(path)?;
    let a = arw::parse(&buf)?;
    println!("orientation: {}", a.orientation);
    println!("preview: {:?}", a.preview);
    println!("full:    {:?}", a.full);
    println!(
        "capture_time: {}",
        a.shot.capture_time.as_deref().unwrap_or("-")
    );
    println!("subsec: {}", a.shot.subsec.as_deref().unwrap_or("-"));
    match a.shot.focus {
        Some(f) => println!("focus: {} {} {} {}", f.sensor_w, f.sensor_h, f.x, f.y),
        None => println!("focus: -"),
    }
    Ok(())
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

    let f = a
        .shot
        .focus
        .ok_or_else(|| anyhow!("no FocusLocation in {path:?}"))?;
    let (fw, fh, fx, fy) = (f.sensor_w, f.sensor_h, f.x, f.y);
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

            if let Some(f) = a.shot.focus {
                // FocusLocation is in sensor coordinates, and JpgFromRaw has
                // the same 7008 width it reports, so they need no scaling.
                let t = Instant::now();
                let c = partial::decode_crop(jpeg, f.x as usize, f.y as usize, CROP_SIZE)?;
                t_crop.push(t.elapsed().as_secs_f64() * 1000.0);
                let _ = c;
            }
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

/// Extract a whole folder in parallel, with no database: the benchmark for the
/// 30s scan target.
fn scan_dir(dir: &Path, threads: Option<usize>) -> Result<()> {
    let threads =
        threads.unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
    if threads == 0 {
        bail!("threads must be at least 1");
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("arw")))
        .collect();
    paths.sort();
    if paths.is_empty() {
        bail!("no ARW files in {dir:?}");
    }

    let start = Instant::now();
    // Per-file cost is the gap between two completions on the same worker, so
    // each worker's previous completion is kept alongside the totals.
    let last = Mutex::new(vec![start; threads]);
    let acc = Mutex::new((Vec::<f64>::new(), 0usize, 0usize));
    scan::extract_all(
        &paths,
        threads,
        |_, r| {
            let now = Instant::now();
            let ms = {
                let mut last = last.lock().unwrap();
                let w = rayon::current_thread_index().unwrap_or(0);
                let ms = now.duration_since(last[w]).as_secs_f64() * 1000.0;
                last[w] = now;
                ms
            };
            let mut acc = acc.lock().unwrap();
            acc.0.push(ms);
            match r {
                Ok(e) => acc.1 += e.thumbnail.len(),
                Err(_) => acc.2 += 1,
            }
        },
        &AtomicBool::new(false),
    )
    .map_err(|e| anyhow!(e))?;
    let total = start.elapsed();

    let (mut ms, bytes, errors) = acc.into_inner().unwrap();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = paths.len();
    println!(
        "{n} files, {threads} threads, {errors} errors: {:.2}s total, {:.0} files/s",
        total.as_secs_f64(),
        n as f64 / total.as_secs_f64()
    );
    println!(
        "per file on a worker: mean {:.1}ms  p95 {:.1}ms",
        ms.iter().sum::<f64>() / ms.len() as f64,
        ms[((ms.len() as f64 * 0.95) as usize).min(ms.len() - 1)]
    );
    println!(
        "thumbnails: {bytes} bytes total, {} bytes mean",
        bytes / n.max(1)
    );
    Ok(())
}

/// Write out the partial decode so it can be checked by eye.
fn crop(path: &Path, out: &Path, size: Option<usize>) -> Result<()> {
    let size = size.unwrap_or(CROP_SIZE);
    let t = Instant::now();
    let (a, jpeg) = reader::read_full(path)?;
    println!(
        "read JpgFromRaw ({} bytes) in {:?}",
        jpeg.len(),
        t.elapsed()
    );

    let t = Instant::now();
    let c = partial::decode_focus_crop(&jpeg, a.shot.focus, size, size)?;
    println!(
        "crop {}x{} at ({},{}) point ({},{}) in {:?}",
        c.crop.width,
        c.crop.height,
        c.crop.x,
        c.crop.y,
        c.point_x,
        c.point_y,
        t.elapsed()
    );

    let rgb: Vec<u8> = c
        .crop
        .pixels
        .chunks_exact(4)
        .flat_map(|p| p[..3].to_vec())
        .collect();
    let (rgb, w, h) = apply_orientation(&rgb, c.crop.width, c.crop.height, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?} ({w}x{h})");
    Ok(())
}
