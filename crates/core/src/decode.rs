//! Decode an embedded JPEG to RGB and rotate it for display.

use anyhow::Result;

pub fn decode_rgb(jpeg: &[u8]) -> Result<(Vec<u8>, usize, usize)> {
    // mozjpeg reports malformed input by panicking, not by returning an error.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode_rgb_unguarded(jpeg)))
        .map_err(|_| anyhow::anyhow!("panic while decoding the JPEG"))?
}

fn decode_rgb_unguarded(jpeg: &[u8]) -> Result<(Vec<u8>, usize, usize)> {
    let d = mozjpeg::Decompress::new_mem(jpeg)?.rgb()?;
    let (w, h) = (d.width(), d.height());
    let mut d = d;
    let pixels: Vec<[u8; 3]> = d.read_scanlines()?;
    d.finish()?;
    let flat = pixels.into_iter().flatten().collect();
    Ok((flat, w, h))
}

/// Rotate an RGB buffer per the Orientation tag. Only 1/6/8 show up in ARW files.
pub fn apply_orientation(
    rgb: &[u8],
    w: usize,
    h: usize,
    orientation: u16,
) -> (Vec<u8>, usize, usize) {
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

/// Decode a preview JPEG at 2/8 scale (1616x1080 -> 404x270) and re-encode it
/// as a baseline JPEG for the thumbnail cache.
///
/// The output keeps the preview's orientation, i.e. it is **unrotated**: the
/// caller carries the Orientation alongside it, as the preview tier does.
pub fn thumbnail_jpeg(preview_jpeg: &[u8], quality: f32) -> Result<Vec<u8>> {
    let mut d = mozjpeg::Decompress::new_mem(preview_jpeg)?;
    d.scale(2);
    let mut d = d.rgb()?;
    let (w, h) = (d.width(), d.height());
    let pixels: Vec<[u8; 3]> = d.read_scanlines()?;
    d.finish()?;
    let rgb: Vec<u8> = pixels.into_iter().flatten().collect();

    let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    c.set_size(w, h);
    c.set_quality(quality);
    // Baseline, not progressive: the strip decodes these one by one on the UI
    // thread, and mozjpeg's scan optimisation costs more time than the few
    // kilobytes it saves at this size.
    c.set_optimize_scans(false);
    let mut c = c.start_compress(Vec::new())?;
    c.write_scanlines(&rgb)?;
    Ok(c.finish()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode a synthetic gradient so the test needs no image file.
    fn jpeg(w: usize, h: usize) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                rgb.extend_from_slice(&[(x * 255 / w) as u8, (y * 255 / h) as u8, 128]);
            }
        }
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w, h);
        c.set_quality(90.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        c.finish().unwrap()
    }

    #[test]
    fn thumbnail_is_a_quarter_size_jpeg() {
        let out = thumbnail_jpeg(&jpeg(1616, 1080), 80.0).unwrap();
        assert_eq!(&out[..2], &[0xff, 0xd8]);
        let (rgb, w, h) = decode_rgb(&out).unwrap();
        assert_eq!((w, h), (404, 270));
        assert_eq!(rgb.len(), w * h * 3);
    }

    #[test]
    fn malformed_jpeg_is_an_error() {
        assert!(decode_rgb(b"not a jpeg").is_err());
        let truncated = jpeg(64, 64);
        assert!(decode_rgb(&truncated[..truncated.len() / 2]).is_err());
    }
}
