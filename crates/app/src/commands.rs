//! The commands the frontend invokes: folder picking, ARW enumeration and
//! preview extraction.

use std::path::{Path, PathBuf};

use tauri::ipc::Response;
use tauri_plugin_dialog::DialogExt;

/// Size of the header that precedes the JPEG bytes in a `preview` payload.
pub const PREVIEW_HEADER_LEN: usize = 8;

/// Tag identifying the payload kind and version: an IFD0 preview JPEG, v1.
pub const PREVIEW_KIND_JPEG_V1: u16 = 1;

/// Build a `preview` payload: a fixed-size little-endian header followed by the
/// embedded JPEG bytes, copied verbatim.
///
/// Header layout (8 bytes, little-endian):
///
/// | offset | size | field                                       |
/// |--------|------|---------------------------------------------|
/// | 0      | 2    | kind/version tag (`PREVIEW_KIND_JPEG_V1`)   |
/// | 2      | 2    | EXIF Orientation (1..8)                     |
/// | 4      | 4    | reserved, zero                              |
///
/// Width and height are not carried: the JPEG itself has them and
/// `createImageBitmap` reports them.
fn payload(orientation: u16, jpeg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PREVIEW_HEADER_LEN + jpeg.len());
    out.extend_from_slice(&PREVIEW_KIND_JPEG_V1.to_le_bytes());
    out.extend_from_slice(&orientation.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(jpeg);
    out
}

fn is_arw(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("arw"))
}

/// List the ARW files directly in `dir`, sorted by file name. Entries that
/// cannot be read are skipped; a directory that cannot be read is an error.
fn list_arw_in(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_arw(p))
        .collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// Extract a file's IFD0 preview JPEG along with the Orientation, reading only
/// the bounded prefix the metadata and the preview need rather than the whole
/// 48 MB file.
fn read_preview(path: &Path) -> Result<(u16, Vec<u8>), String> {
    let (arw, jpeg) =
        riffle_core::reader::read_preview(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((arw.orientation, jpeg))
}

/// Open the native folder picker and resolve once the user answers, or `None`
/// if they cancel.
///
/// The dialog is driven by the main thread's run loop, so the blocking variant
/// must not be used: a synchronous command runs on the main thread, and waiting
/// there freezes the very loop that delivers the dialog's button events. The
/// callback form hands the answer back over a channel this async command awaits
/// instead.
#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.try_send(path);
    });
    rx.recv().await.flatten().map(|p| p.to_string())
}

#[tauri::command]
pub fn list_arw(dir: String) -> Result<Vec<String>, String> {
    list_arw_in(Path::new(&dir))
}

#[tauri::command]
pub async fn preview(path: String) -> Result<Response, String> {
    let bytes = tauri::async_runtime::spawn_blocking(move || read_preview(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;
    Ok(Response::new(payload(bytes.0, &bytes.1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lists_only_arw_files_sorted_by_name() {
        let dir = temp_dir("list");
        for name in ["b.ARW", "a.arw", "c.Arw", "d.jpg", "e.arw.txt"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        std::fs::create_dir(dir.join("sub.arw")).unwrap();

        let files = list_arw_in(&dir).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(names, ["a.arw", "b.ARW", "c.Arw"]);
        assert!(files.iter().all(|p| Path::new(p).is_absolute()));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unreadable_directory_is_an_error() {
        let missing = std::env::temp_dir().join("riffle-app-does-not-exist");
        assert!(list_arw_in(&missing).is_err());
    }

    #[test]
    fn payload_header_encodes_kind_and_orientation() {
        let jpeg = [0xff, 0xd8, 0xff, 0xd9];
        let out = payload(8, &jpeg);
        assert_eq!(out.len(), PREVIEW_HEADER_LEN + jpeg.len());
        assert_eq!(u16::from_le_bytes([out[0], out[1]]), PREVIEW_KIND_JPEG_V1);
        assert_eq!(u16::from_le_bytes([out[2], out[3]]), 8);
        assert_eq!(u32::from_le_bytes([out[4], out[5], out[6], out[7]]), 0);
        assert_eq!(&out[PREVIEW_HEADER_LEN..], &jpeg);
    }

    #[test]
    fn missing_preview_is_an_error() {
        let dir = temp_dir("preview");
        let path = dir.join("broken.arw");
        std::fs::write(&path, b"II\x2a\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
        assert!(read_preview(&path).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
