//! The performance overlay's CPU sampler: Riffle's process CPU and the
//! system's, each 0–100 %, emitted as `perf-sample` every 500 ms while the
//! overlay is on, and nothing at all while it is off.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

const PERIOD: Duration = Duration::from_millis(500);

/// The running sampler's stop flag and thread, if any.
#[derive(Default)]
pub struct AppPerf(Mutex<Option<(Arc<AtomicBool>, JoinHandle<()>)>>);

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Sample {
    pub process: f32,
    pub system: f32,
}

/// Scale a process CPU reading, up to 100 % per logical CPU, to 0–100.
fn normalized(raw: f32, cpus: usize) -> f32 {
    (raw / cpus.max(1) as f32).clamp(0.0, 100.0)
}

/// Turns raw readings into samples, dropping the first: sysinfo's CPU usage
/// is a diff against the previous refresh, so the first has nothing to diff.
#[derive(Default)]
struct Readings {
    primed: bool,
}

impl Readings {
    fn sample(&mut self, process: f32, system: f32, cpus: usize) -> Option<Sample> {
        if !std::mem::replace(&mut self.primed, true) {
            return None;
        }
        Some(Sample {
            process: normalized(process, cpus),
            system: system.clamp(0.0, 100.0),
        })
    }
}

/// Start or stop the sampler. Starting while it runs does nothing; stopping
/// only signals the thread, which exits within one period, so the main
/// thread never waits on it.
#[tauri::command]
pub fn set_perf_sampling(app: AppHandle, state: tauri::State<AppPerf>, enabled: bool) {
    let mut running = state.0.lock().unwrap_or_else(|e| e.into_inner());
    if !enabled {
        if let Some((stop, _)) = running.take() {
            stop.store(true, Ordering::Relaxed);
        }
        return;
    }
    if running
        .as_ref()
        .is_some_and(|(_, handle)| !handle.is_finished())
    {
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let handle = std::thread::spawn({
        let stop = stop.clone();
        move || run(&app, &stop)
    });
    *running = Some((stop, handle));
}

fn run(app: &AppHandle, stop: &AtomicBool) {
    let period = PERIOD.max(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    let pid = sysinfo::get_current_pid().ok();
    let mut sys = System::new();
    let mut readings = Readings::default();
    let mut warned = false;
    while !stop.load(Ordering::Relaxed) {
        sys.refresh_cpu_usage();
        let process = pid.map_or(0.0, |pid| {
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                false,
                ProcessRefreshKind::nothing().with_cpu(),
            );
            sys.process(pid).map_or(0.0, |p| p.cpu_usage())
        });
        if let Some(sample) = readings.sample(process, sys.global_cpu_usage(), sys.cpus().len()) {
            if let Err(e) = app.emit("perf-sample", sample) {
                if !std::mem::replace(&mut warned, true) {
                    log::warn!("perf-sample emit failed: {e}");
                }
            }
        }
        std::thread::sleep(period);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_scales_by_cpu_count() {
        assert_eq!(normalized(0.0, 8), 0.0);
        assert_eq!(normalized(800.0, 8), 100.0);
        assert_eq!(normalized(200.0, 8), 25.0);
    }

    #[test]
    fn normalized_clamps_at_100() {
        assert_eq!(normalized(1000.0, 8), 100.0);
    }

    #[test]
    fn normalized_with_no_cpus_does_not_divide_by_zero() {
        assert_eq!(normalized(50.0, 0), 50.0);
        assert_eq!(normalized(500.0, 0), 100.0);
    }

    #[test]
    fn the_first_reading_is_dropped() {
        let mut readings = Readings::default();
        assert_eq!(readings.sample(400.0, 30.0, 4), None);
        assert_eq!(
            readings.sample(400.0, 30.0, 4),
            Some(Sample {
                process: 100.0,
                system: 30.0
            })
        );
        assert_eq!(
            readings.sample(100.0, 120.0, 4),
            Some(Sample {
                process: 25.0,
                system: 100.0
            })
        );
    }
}
