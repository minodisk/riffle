//! Partially decode only the area around the focus point out of JpgFromRaw.
//! Calls libjpeg-turbo's jpeg_crop_scanline + jpeg_skip_scanlines directly.

use anyhow::{bail, Result};
use mozjpeg_sys as sys;
use std::mem;

pub struct Crop {
    pub rgb: Vec<u8>,
    pub width: usize,
    pub height: usize,
    /// Top-left actually produced. Cropping snaps to MCU boundaries, so it shifts.
    pub x: usize,
    pub y: usize,
}

/// Cut out size x size centered on (cx, cy). The region is clamped to the image.
pub fn decode_crop(jpeg: &[u8], cx: usize, cy: usize, size: usize) -> Result<Crop> {
    unsafe {
        let mut err: sys::jpeg_error_mgr = mem::zeroed();
        let mut cinfo: sys::jpeg_decompress_struct = mem::zeroed();
        cinfo.common.err = sys::jpeg_std_error(&mut err);
        sys::jpeg_create_decompress(&mut cinfo);

        let guard = scopeguard(&mut cinfo as *mut _);

        sys::jpeg_mem_src(&mut cinfo, jpeg.as_ptr(), jpeg.len() as _);
        if sys::jpeg_read_header(&mut cinfo, true as _) != 1 {
            bail!("jpeg_read_header failed");
        }
        cinfo.out_color_space = sys::J_COLOR_SPACE::JCS_RGB;
        sys::jpeg_start_decompress(&mut cinfo);

        let (iw, ih) = (cinfo.output_width as usize, cinfo.output_height as usize);
        let half = size / 2;
        let x0 = cx.saturating_sub(half).min(iw.saturating_sub(size));
        let y0 = cy.saturating_sub(half).min(ih.saturating_sub(size));
        let w = size.min(iw);
        let h = size.min(ih);

        // Horizontal: snapped to MCU boundaries, so trust the values it hands back.
        let mut xoff = x0 as sys::JDIMENSION;
        let mut width = w as sys::JDIMENSION;
        sys::jpeg_crop_scanline(&mut cinfo, &mut xoff, &mut width);

        // Vertical: skip ahead to the first row we need.
        if y0 > 0 {
            sys::jpeg_skip_scanlines(&mut cinfo, y0 as _);
        }

        let stride = width as usize * 3;
        let mut rgb = vec![0u8; stride * h];
        let mut read = 0usize;
        while read < h {
            let mut row = rgb.as_mut_ptr().add(read * stride);
            let n = sys::jpeg_read_scanlines(&mut cinfo, &mut row, 1);
            if n == 0 {
                break;
            }
            read += n as usize;
        }
        rgb.truncate(stride * read);

        // Abort without reading the rest; otherwise it expands the whole image.
        sys::jpeg_abort_decompress(&mut cinfo);
        drop(guard);

        Ok(Crop {
            rgb,
            width: width as usize,
            height: read,
            x: xoff as usize,
            y: y0,
        })
    }
}

/// Makes sure jpeg_destroy_decompress always runs.
struct ScopeGuard(*mut sys::jpeg_decompress_struct);
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        unsafe { sys::jpeg_destroy_decompress(&mut *self.0) }
    }
}
fn scopeguard(p: *mut sys::jpeg_decompress_struct) -> ScopeGuard {
    ScopeGuard(p)
}
