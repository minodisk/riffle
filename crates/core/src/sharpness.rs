//! A focus-quality score: the variance of the Laplacian of the luma in a
//! window around the focus point. Only meaningful relative to other frames.

use std::panic::{catch_unwind, AssertUnwindSafe};

use anyhow::{anyhow, bail, Result};

use crate::arw::FocusLocation;
use crate::partial::focus_point;

/// Side of the square window the score is taken over, in preview pixels.
pub const WINDOW: usize = 256;

/// A rectangle in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// A `size` x `size` window centred on `(cx, cy)`, clamped into a
/// `width` x `height` image.
pub fn window_at(width: usize, height: usize, cx: usize, cy: usize, size: usize) -> Window {
    let w = size.min(width);
    let h = size.min(height);
    Window {
        x: cx.saturating_sub(w / 2).min(width - w),
        y: cy.saturating_sub(h / 2).min(height - h),
        width: w,
        height: h,
    }
}

/// Variance of the 3x3 Laplacian over the pixels of `window` whose
/// neighbours all lie inside it, so nothing outside the window counts.
/// `gray` is `width` x `height`, one byte per pixel.
pub fn laplacian_variance(gray: &[u8], width: usize, window: Window) -> f64 {
    if window.width < 3 || window.height < 3 {
        return 0.0;
    }
    let mut sum = 0i64;
    let mut sum_sq = 0i64;
    let mut n = 0i64;
    for y in window.y + 1..window.y + window.height - 1 {
        let row = y * width;
        for x in window.x + 1..window.x + window.width - 1 {
            let i = row + x;
            let l = gray[i - width] as i32
                + gray[i + width] as i32
                + gray[i - 1] as i32
                + gray[i + 1] as i32
                - 4 * gray[i] as i32;
            sum += l as i64;
            sum_sq += (l * l) as i64;
            n += 1;
        }
    }
    let mean = sum as f64 / n as f64;
    sum_sq as f64 / n as f64 - mean * mean
}

/// Decode `preview` to grayscale and score the `WINDOW`-sized window on the
/// focus point (the centre when there is none). mozjpeg aborts through a
/// panic on bytes that are not a JPEG; that comes back as `Err` too.
pub fn score_preview(preview: &[u8], focus: Option<FocusLocation>) -> Result<f64> {
    catch_unwind(AssertUnwindSafe(|| score(preview, focus)))
        .map_err(|_| anyhow!("panic while decoding the preview"))?
}

fn score(preview: &[u8], focus: Option<FocusLocation>) -> Result<f64> {
    let mut d = mozjpeg::Decompress::new_mem(preview)?.grayscale()?;
    let (w, h) = (d.width(), d.height());
    let gray: Vec<u8> = d.read_scanlines()?;
    d.finish()?;
    if w < 3 || h < 3 {
        bail!("a {w}x{h} preview has no interior pixel to score");
    }
    let (cx, cy) = focus_point(w, h, focus);
    Ok(laplacian_variance(
        &gray,
        w,
        window_at(w, h, cx, cy, WINDOW),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(w: usize, h: usize, cell: usize) -> Vec<u8> {
        (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    if (x / cell + y / cell).is_multiple_of(2) {
                        0
                    } else {
                        255
                    }
                })
            })
            .collect()
    }

    fn blur(gray: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut out = gray.to_vec();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut s = 0u32;
                for dy in 0..3 {
                    for dx in 0..3 {
                        s += gray[(y + dy - 1) * w + x + dx - 1] as u32;
                    }
                }
                out[y * w + x] = (s / 9) as u8;
            }
        }
        out
    }

    fn jpeg(gray: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_GRAYSCALE);
        c.set_size(w, h);
        c.set_quality(95.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(gray).unwrap();
        c.finish().unwrap()
    }

    fn all(w: usize, h: usize) -> Window {
        Window {
            x: 0,
            y: 0,
            width: w,
            height: h,
        }
    }

    #[test]
    fn a_constant_image_scores_zero() {
        assert_eq!(laplacian_variance(&[128u8; 64 * 64], 64, all(64, 64)), 0.0);
    }

    #[test]
    fn a_blurred_checkerboard_scores_lower() {
        let sharp = checker(64, 64, 4);
        let soft = blur(&sharp, 64, 64);
        assert!(
            laplacian_variance(&sharp, 64, all(64, 64))
                > laplacian_variance(&soft, 64, all(64, 64))
        );
    }

    #[test]
    fn content_outside_the_window_does_not_count() {
        let mut gray = vec![100u8; 64 * 64];
        for y in 0..64 {
            for x in 32..64 {
                gray[y * 64 + x] = if (x + y).is_multiple_of(2) { 0 } else { 255 };
            }
        }
        let left = Window {
            x: 0,
            y: 0,
            width: 32,
            height: 64,
        };
        assert_eq!(laplacian_variance(&gray, 64, left), 0.0);
    }

    #[test]
    fn a_window_near_a_corner_is_clamped_inside() {
        let w = window_at(1616, 1080, 1610, 5, WINDOW);
        assert_eq!(
            w,
            Window {
                x: 1616 - 256,
                y: 0,
                width: 256,
                height: 256
            }
        );
        let small = window_at(100, 80, 0, 0, WINDOW);
        assert_eq!(small, all(100, 80));
    }

    #[test]
    fn without_a_focus_location_the_centre_is_scored() {
        let (w, h) = (1024, 768);
        let mut gray = vec![128u8; w * h];
        for y in h / 2 - 64..h / 2 + 64 {
            for x in w / 2 - 64..w / 2 + 64 {
                gray[y * w + x] = if (x / 4 + y / 4).is_multiple_of(2) {
                    0
                } else {
                    255
                };
            }
        }
        let jpeg = jpeg(&gray, w, h);
        assert!(score_preview(&jpeg, None).unwrap() > 0.0);
        let corner = FocusLocation {
            sensor_w: 1024,
            sensor_h: 768,
            x: 0,
            y: 0,
        };
        assert!(score_preview(&jpeg, Some(corner)).unwrap() < 1.0);
    }

    #[test]
    fn a_preview_too_small_to_score_is_an_error() {
        assert!(score_preview(&jpeg(&[0u8; 4], 2, 2), None).is_err());
    }

    #[test]
    fn a_preview_that_is_not_a_jpeg_is_an_error() {
        assert!(score_preview(&[0u8; 512], None).is_err());
    }
}
