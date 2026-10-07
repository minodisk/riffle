//! Failures that would otherwise leave no trace in `Riffle.log`: Rust panics
//! and the frontend's uncaught errors, each written under a fixed, greppable
//! prefix.

use std::backtrace::{Backtrace, BacktraceStatus};
use std::panic::Location;

/// Log every panic, on any thread, as one `panic:` line before the default
/// hook prints it to stderr. Panics the core catches and turns into an `Err`
/// (corrupt-file decodes) are logged too, so a `panic:` line is a crash only
/// when no recovery follows. It is installed from `.setup()`, after the log
/// plugin, so a panic before `setup` only reaches stderr.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string payload>");
        let line = panic_line(std::thread::current().name(), info.location(), payload);
        let backtrace = Backtrace::capture();
        if backtrace.status() == BacktraceStatus::Captured {
            log::error!("{line}\n{backtrace}");
        } else {
            log::error!("{line}");
        }
        previous(info);
    }));
}

fn panic_line(thread: Option<&str>, location: Option<&Location>, payload: &str) -> String {
    let location = location.map_or_else(
        || "unknown".to_string(),
        |l| format!("{}:{}:{}", l.file(), l.line(), l.column()),
    );
    format!(
        "panic: thread={} location={location} payload={}",
        thread.unwrap_or("unnamed"),
        payload.replace(['\r', '\n'], " ")
    )
}

/// Write a frontend failure line to `Riffle.log`. It is `async` with owned
/// arguments so it does not run on the main thread.
#[tauri::command]
pub async fn log_frontend(kind: String, line: String) {
    match kind.as_str() {
        "uncaught-js" => log::error!("uncaught-js: {line}"),
        _ => log::warn!("{kind}: {line}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_line_names_the_thread_location_and_payload() {
        let location = Location::caller();
        let line = panic_line(Some("scan"), Some(location), "boom");
        assert_eq!(
            line,
            format!(
                "panic: thread=scan location={}:{}:{} payload=boom",
                location.file(),
                location.line(),
                location.column()
            )
        );
    }

    #[test]
    fn panic_line_without_a_thread_name_or_location() {
        assert_eq!(
            panic_line(None, None, "<non-string payload>"),
            "panic: thread=unnamed location=unknown payload=<non-string payload>"
        );
    }

    #[test]
    fn panic_line_collapses_newlines_in_the_payload() {
        assert_eq!(
            panic_line(Some("main"), None, "a\nb\r\nc"),
            "panic: thread=main location=unknown payload=a b  c"
        );
    }
}
