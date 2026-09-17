//! Decode an embedded JPEG to RGB and rotate it for display.

use anyhow::Result;

pub fn decode_rgb(jpeg: &[u8]) -> Result<(Vec<u8>, usize, usize)> {
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
