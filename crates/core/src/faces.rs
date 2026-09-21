//! Offline face and eye detection on the embedded preview with the YuNet
//! model (OpenCV Zoo, `face_detection_yunet_2023mar.onnx`, MIT), run through
//! `tract`. The preview is scaled so its long edge is `INPUT` pixels and
//! padded into an `INPUT` x `INPUT` square; the per-stride outputs are
//! decoded as OpenCV's `FaceDetectorYN` does, thresholded and merged by NMS.
//! Results are in the caller's pixel coordinates.

use std::io::Cursor;
use std::sync::OnceLock;

use anyhow::{anyhow, Context, Result};
use tract_onnx::prelude::*;

const MODEL: &[u8] = include_bytes!("../models/face_detection_yunet_2023mar.onnx");

/// Side of the square model input, in pixels.
pub const INPUT: usize = 320;
/// Minimum detection score kept.
pub const SCORE_THRESHOLD: f32 = 0.6;
/// Boxes overlapping a better one by more than this IoU are dropped.
pub const NMS_THRESHOLD: f32 = 0.3;
const STRIDES: [usize; 3] = [8, 16, 32];

/// A detected face, in the pixel coordinates of the image given to `detect`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub score: f32,
    /// The eye on the left of the image, then the one on the right.
    pub left_eye: (f32, f32),
    pub right_eye: (f32, f32),
}

type Plan = Arc<TypedRunnableModel>;

struct Detector {
    plan: Plan,
    /// Output index of `cls`, `obj`, `bbox`, `kps` for each stride.
    outputs: [[usize; 4]; 3],
}

fn detector() -> Result<&'static Detector> {
    static DETECTOR: OnceLock<std::result::Result<Detector, String>> = OnceLock::new();
    DETECTOR
        .get_or_init(|| build().map_err(|e| format!("{e:#}")))
        .as_ref()
        .map_err(|e| anyhow!("face model: {e}"))
}

fn build() -> Result<Detector> {
    // The file annotates intermediate and output shapes for a 640 px input.
    let model = tract_onnx::onnx()
        .with_ignore_value_info(true)
        .with_ignore_output_shapes(true)
        .model_for_read(&mut Cursor::new(MODEL))?
        .with_input_fact(0, f32::fact([1, 3, INPUT, INPUT]).into())?;
    let names: Vec<String> = model
        .output_outlets()?
        .iter()
        .map(|o| model.outlet_label(*o).unwrap_or_default().to_string())
        .collect();
    let mut outputs = [[0; 4]; 3];
    for (s, stride) in STRIDES.iter().enumerate() {
        for (k, kind) in ["cls", "obj", "bbox", "kps"].iter().enumerate() {
            let name = format!("{kind}_{stride}");
            outputs[s][k] = names
                .iter()
                .position(|n| *n == name)
                .with_context(|| format!("no output {name} in {names:?}"))?;
        }
    }
    let plan = model.into_optimized()?.into_runnable()?;
    Ok(Detector { plan, outputs })
}

/// Detect faces in an RGB image (`width` x `height`, 3 bytes per pixel),
/// best first.
pub fn detect(rgb: &[u8], width: usize, height: usize) -> Result<Vec<Face>> {
    if width == 0 || height == 0 || rgb.len() < width * height * 3 {
        return Ok(Vec::new());
    }
    let d = detector()?;
    let scale = INPUT as f32 / width.max(height) as f32;
    let input = input_tensor(rgb, width, height, scale);
    let out = d.plan.run(tvec!(input.into()))?;
    let mut faces = Vec::new();
    for (s, &stride) in STRIDES.iter().enumerate() {
        let get = |k: usize| -> Result<&[f32]> {
            out[d.outputs[s][k]].try_as_plain_ram()?.as_slice::<f32>()
        };
        decode_stride(
            stride,
            INPUT / stride,
            get(0)?,
            get(1)?,
            get(2)?,
            get(3)?,
            &mut faces,
        );
    }
    let mut faces = nms(faces, NMS_THRESHOLD);
    for f in &mut faces {
        f.x /= scale;
        f.y /= scale;
        f.width /= scale;
        f.height /= scale;
        f.left_eye = (f.left_eye.0 / scale, f.left_eye.1 / scale);
        f.right_eye = (f.right_eye.0 / scale, f.right_eye.1 / scale);
    }
    Ok(faces)
}

/// Box-average the image down by `scale` into the top-left of a zeroed
/// `INPUT` x `INPUT` BGR NCHW tensor of raw 0..255 values, as YuNet expects.
fn input_tensor(rgb: &[u8], width: usize, height: usize, scale: f32) -> Tensor {
    let ow = ((width as f32 * scale).round() as usize).clamp(1, INPUT);
    let oh = ((height as f32 * scale).round() as usize).clamp(1, INPUT);
    let plane = INPUT * INPUT;
    let mut data = vec![0f32; 3 * plane];
    for oy in 0..oh {
        let y0 = oy * height / oh;
        let y1 = ((oy + 1) * height / oh).max(y0 + 1);
        for ox in 0..ow {
            let x0 = ox * width / ow;
            let x1 = ((ox + 1) * width / ow).max(x0 + 1);
            let mut sum = [0u32; 3];
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * width + x) * 3;
                    sum[0] += rgb[i] as u32;
                    sum[1] += rgb[i + 1] as u32;
                    sum[2] += rgb[i + 2] as u32;
                }
            }
            let n = ((y1 - y0) * (x1 - x0)) as f32;
            let o = oy * INPUT + ox;
            data[o] = sum[2] as f32 / n;
            data[plane + o] = sum[1] as f32 / n;
            data[2 * plane + o] = sum[0] as f32 / n;
        }
    }
    tract_ndarray::Array4::from_shape_vec((1, 3, INPUT, INPUT), data)
        .expect("shape matches the buffer")
        .into()
}

/// Decode one stride's grid (`cols` cells per row) into faces above
/// `SCORE_THRESHOLD`, in model-input coordinates.
fn decode_stride(
    stride: usize,
    cols: usize,
    cls: &[f32],
    obj: &[f32],
    bbox: &[f32],
    kps: &[f32],
    faces: &mut Vec<Face>,
) {
    let s = stride as f32;
    for i in 0..cls.len().min(obj.len()) {
        let score = (cls[i].clamp(0.0, 1.0) * obj[i].clamp(0.0, 1.0)).sqrt();
        if score < SCORE_THRESHOLD {
            continue;
        }
        let (c, r) = ((i % cols) as f32, (i / cols) as f32);
        let b = &bbox[i * 4..i * 4 + 4];
        let (cx, cy) = ((c + b[0]) * s, (r + b[1]) * s);
        let (w, h) = (b[2].exp() * s, b[3].exp() * s);
        let k = &kps[i * 10..i * 10 + 4];
        let a = ((k[0] + c) * s, (k[1] + r) * s);
        let e = ((k[2] + c) * s, (k[3] + r) * s);
        let (left_eye, right_eye) = if a.0 <= e.0 { (a, e) } else { (e, a) };
        faces.push(Face {
            x: cx - w / 2.0,
            y: cy - h / 2.0,
            width: w,
            height: h,
            score,
            left_eye,
            right_eye,
        });
    }
}

fn iou(a: &Face, b: &Face) -> f32 {
    let w = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
    let h = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 {
        return 0.0;
    }
    let inter = w * h;
    inter / (a.width * a.height + b.width * b.height - inter)
}

/// Greedy non-maximum suppression, best score first.
fn nms(mut faces: Vec<Face>, threshold: f32) -> Vec<Face> {
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Face> = Vec::new();
    for f in faces {
        if kept.iter().all(|k| iou(k, &f) <= threshold) {
            kept.push(f);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(x: f32, score: f32) -> Face {
        Face {
            x,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            score,
            left_eye: (0.0, 0.0),
            right_eye: (0.0, 0.0),
        }
    }

    #[test]
    fn decodes_a_cell_into_input_coordinates() {
        let cols = 4;
        let n = cols * cols;
        let mut cls = vec![0.0; n];
        let mut obj = vec![0.0; n];
        let mut bbox = vec![0.0; n * 4];
        let mut kps = vec![0.0; n * 10];
        let i = cols + 2;
        cls[i] = 0.9;
        obj[i] = 0.9;
        bbox[i * 4..i * 4 + 4].copy_from_slice(&[0.5, 0.5, 2f32.ln(), 1f32.ln()]);
        kps[i * 10..i * 10 + 4].copy_from_slice(&[0.75, 0.25, 0.25, 0.25]);
        let mut faces = Vec::new();
        decode_stride(8, cols, &cls, &obj, &bbox, &kps, &mut faces);
        assert_eq!(faces.len(), 1);
        let f = faces[0];
        assert!((f.score - 0.9).abs() < 1e-6);
        assert!((f.x - (20.0 - 8.0)).abs() < 1e-4);
        assert!((f.y - (12.0 - 4.0)).abs() < 1e-4);
        assert!((f.width - 16.0).abs() < 1e-4);
        assert!((f.height - 8.0).abs() < 1e-4);
        assert_eq!(f.left_eye, (18.0, 10.0));
        assert_eq!(f.right_eye, (22.0, 10.0));
    }

    #[test]
    fn drops_cells_below_the_threshold() {
        let mut faces = Vec::new();
        decode_stride(8, 1, &[0.5], &[0.5], &[0.0; 4], &[0.0; 10], &mut faces);
        assert!(faces.is_empty());
    }

    #[test]
    fn nms_keeps_the_best_of_overlapping_boxes() {
        let kept = nms(
            vec![face(0.0, 0.7), face(1.0, 0.9), face(50.0, 0.8)],
            NMS_THRESHOLD,
        );
        assert_eq!(kept, vec![face(1.0, 0.9), face(50.0, 0.8)]);
    }

    #[test]
    fn finds_no_face_in_flat_or_checkerboard_images() {
        let (w, h) = (640, 427);
        let flat = vec![128u8; w * h * 3];
        assert!(detect(&flat, w, h).unwrap().is_empty());
        let checker: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                let v = if ((i % w) / 16 + (i / w) / 16) % 2 == 0 {
                    0
                } else {
                    255
                };
                [v, v, v]
            })
            .collect();
        assert!(detect(&checker, w, h).unwrap().is_empty());
    }

    /// Set `RIFFLE_FACE_JPEG` to a JPEG with one clear face, then run with
    /// `--ignored`.
    #[test]
    #[ignore]
    fn detects_a_face_in_a_real_image() {
        let path = std::env::var("RIFFLE_FACE_JPEG").expect("RIFFLE_FACE_JPEG");
        let jpeg = std::fs::read(path).unwrap();
        let (rgb, w, h) = crate::decode::decode_rgb(&jpeg).unwrap();
        let faces = detect(&rgb, w, h).unwrap();
        assert!(!faces.is_empty());
        let f = faces[0];
        for (x, y) in [f.left_eye, f.right_eye] {
            assert!(x >= f.x && x <= f.x + f.width && y >= f.y && y <= f.y + f.height);
        }
    }
}
