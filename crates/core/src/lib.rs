//! Reusable ARW parsing and JPEG decoding shared by the CLI and the app.

pub mod arw;
pub mod decode;
pub mod dop;
pub mod faces;
pub mod partial;
pub mod reader;
pub mod scan;
pub mod sharpness;
pub mod xmp;

/// A pick / reject judgement, independent of the `0`-`5` stars, as both
/// Lightroom (`xmpDM:good`) and DxO PhotoLab (`ShouldProcess`) model it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Flag {
    #[default]
    None,
    Pick,
    Reject,
}
