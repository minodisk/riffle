use rayon::prelude::*;
use riffle_core::{candidate, decode::decode_rgb, reader::read_preview, scan, sharpness, xmp, Flag};
use std::path::{Path, PathBuf};

struct Row {
    name: String,
    pick: bool,
    lap: f64,
    lap_var: f64,     // lapvar / variance of luma
    lap_mean2: f64,   // lapvar / mean^2
    lap_noise: f64,   // lapvar / (Immerkaer noise sigma)^2
    edge_w: f64,      // mean edge width (px), lower = sharper
    edge_w_rel: f64,  // edge width / window side
}

fn stats(g: &[u8], stride: usize, w: &sharpness::Window) -> (f64, f64) {
    let (mut s, mut s2, mut n) = (0f64, 0f64, 0f64);
    for y in w.y..w.y + w.height {
        for x in w.x..w.x + w.width {
            let v = g[y * stride + x] as f64;
            s += v;
            s2 += v * v;
            n += 1.0;
        }
    }
    let m = s / n;
    (m, s2 / n - m * m)
}

fn immerkaer(g: &[u8], stride: usize, w: &sharpness::Window) -> f64 {
    let p = |x: usize, y: usize| g[y * stride + x] as f64;
    let mut sum = 0f64;
    for y in w.y + 1..w.y + w.height - 1 {
        for x in w.x + 1..w.x + w.width - 1 {
            let v = p(x - 1, y - 1) - 2.0 * p(x, y - 1) + p(x + 1, y - 1)
                - 2.0 * p(x - 1, y) + 4.0 * p(x, y) - 2.0 * p(x + 1, y)
                + p(x - 1, y + 1) - 2.0 * p(x, y + 1) + p(x + 1, y + 1);
            sum += v.abs();
        }
    }
    let n = ((w.width - 2) * (w.height - 2)) as f64;
    (std::f64::consts::PI / 2.0).sqrt() * sum / (6.0 * n)
}

/// Marziliano-style edge width along rows and columns: at each local maximum
/// of the Sobel gradient (above the window's 90th percentile), walk both ways
/// while the intensity stays monotonic; the width is the distance between the
/// two extrema.
fn edge_width(g: &[u8], stride: usize, w: &sharpness::Window) -> f64 {
    let p = |x: usize, y: usize| g[y * stride + x] as i32;
    let (x0, y0, x1, y1) = (w.x + 1, w.y + 1, w.x + w.width - 1, w.y + w.height - 1);
    let gx = |x: usize, y: usize| {
        (p(x + 1, y - 1) + 2 * p(x + 1, y) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x - 1, y) + p(x - 1, y + 1))
    };
    let gy = |x: usize, y: usize| {
        (p(x - 1, y + 1) + 2 * p(x, y + 1) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x, y - 1) + p(x + 1, y - 1))
    };
    let mut mags = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            mags.push(gx(x, y).abs().max(gy(x, y).abs()));
        }
    }
    if mags.is_empty() {
        return f64::NAN;
    }
    mags.sort_unstable();
    let t = mags[mags.len() * 9 / 10].max(16);
    let (mut total, mut count) = (0f64, 0f64);
    // Rows (horizontal gradient).
    for y in y0..y1 {
        for x in x0 + 1..x1 - 1 {
            let m = gx(x, y).abs();
            if m < t || m < gx(x - 1, y).abs() || m <= gx(x + 1, y).abs() {
                continue;
            }
            let up = gx(x, y) > 0;
            let mut l = x;
            while l > w.x && ((p(l - 1, y) < p(l, y)) == up) && p(l - 1, y) != p(l, y) {
                l -= 1;
            }
            let mut r = x;
            while r + 1 < w.x + w.width && ((p(r + 1, y) > p(r, y)) == up) && p(r + 1, y) != p(r, y) {
                r += 1;
            }
            total += (r - l) as f64;
            count += 1.0;
        }
    }
    // Columns (vertical gradient).
    for x in x0..x1 {
        for y in y0 + 1..y1 - 1 {
            let m = gy(x, y).abs();
            if m < t || m < gy(x, y - 1).abs() || m <= gy(x, y + 1).abs() {
                continue;
            }
            let up = gy(x, y) > 0;
            let mut u = y;
            while u > w.y && ((p(x, u - 1) < p(x, u)) == up) && p(x, u - 1) != p(x, u) {
                u -= 1;
            }
            let mut d = y;
            while d + 1 < w.y + w.height && ((p(x, d + 1) > p(x, d)) == up) && p(x, d + 1) != p(x, d) {
                d += 1;
            }
            total += (d - u) as f64;
            count += 1.0;
        }
    }
    if count == 0.0 { f64::NAN } else { total / count }
}

fn row(path: &Path) -> Option<Row> {
    let flag = std::fs::read(xmp::sidecar_path(path)).ok().and_then(|b| xmp::read_flag(&b).ok())?;
    let pick = match flag {
        Flag::Pick => true,
        Flag::Reject => false,
        _ => return None,
    };
    let cue = scan::extract_faces(path).ok()?;
    let face = cue.face?;
    let lap = cue.eye_sharpness?;
    let (_, preview) = read_preview(path).ok()?;
    let (rgb, width, height) = decode_rgb(&preview).ok()?;
    let gray = candidate::luma(&rgb, width, height);
    let win = candidate::eye_window(width, height, &face);
    let (mean, var) = stats(&gray, width, &win);
    let sigma = immerkaer(&gray, width, &win);
    let ew = edge_width(&gray, width, &win);
    Some(Row {
        name: path.file_name()?.to_string_lossy().into_owned(),
        pick,
        lap,
        lap_var: lap / var.max(1e-6),
        lap_mean2: lap / (mean * mean).max(1e-6),
        lap_noise: lap / (sigma * sigma).max(1e-6),
        edge_w: ew,
        edge_w_rel: ew / win.width as f64,
    })
}

/// AUC of `score` for pick (higher score = more likely in focus).
fn auc(rows: &[Row], score: impl Fn(&Row) -> f64) -> f64 {
    let pos: Vec<f64> = rows.iter().filter(|r| r.pick).map(&score).filter(|v| v.is_finite()).collect();
    let neg: Vec<f64> = rows.iter().filter(|r| !r.pick).map(&score).filter(|v| v.is_finite()).collect();
    let mut s = 0f64;
    for a in &pos {
        for b in &neg {
            s += if a > b { 1.0 } else if a == b { 0.5 } else { 0.0 };
        }
    }
    s / (pos.len() * neg.len()) as f64
}

fn main() {
    let dirs: Vec<String> = std::env::args().skip(1).collect();
    let mut paths: Vec<PathBuf> = Vec::new();
    for d in &dirs {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            if scan::is_raw_file(&e.path()) {
                paths.push(e.path());
            }
        }
    }
    paths.sort();
    let rows: Vec<Row> = paths.par_iter().filter_map(|p| row(p)).collect();
    println!("name,pick,lap,lap_var,lap_mean2,lap_noise,edge_w,edge_w_rel");
    for r in &rows {
        println!("{},{},{:.2},{:.4},{:.5},{:.3},{:.3},{:.5}", r.name, r.pick as u8, r.lap, r.lap_var, r.lap_mean2, r.lap_noise, r.edge_w, r.edge_w_rel);
    }
    let picks = rows.iter().filter(|r| r.pick).count();
    eprintln!("faced labeled frames: {} (in focus {}, off {})", rows.len(), picks, rows.len() - picks);
    eprintln!("AUC laplacian variance (current) : {:.3}", auc(&rows, |r| r.lap));
    eprintln!("AUC lap / luma variance          : {:.3}", auc(&rows, |r| r.lap_var));
    eprintln!("AUC lap / mean^2                 : {:.3}", auc(&rows, |r| r.lap_mean2));
    eprintln!("AUC lap / noise sigma^2          : {:.3}", auc(&rows, |r| r.lap_noise));
    eprintln!("AUC edge width (px, narrower)    : {:.3}", auc(&rows, |r| -r.edge_w));
    eprintln!("AUC edge width / window side     : {:.3}", auc(&rows, |r| -r.edge_w_rel));
}
