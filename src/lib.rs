
#[cfg(windows)]
extern crate winapi;
#[cfg(unix)]
extern crate linux_proc;

use std::time;
use std::ops;
use std::io;

mod imp;

/// Like `std::time::Instant`, but with information about the cpu usage stats.
#[derive(Debug, Copy, Clone)]
pub struct CpuInstant {
    instant: time::Instant,
    cpu_total: f64,
    cpu_idle: f64,
}

impl CpuInstant {
    /// Get the current instant.
    ///
    /// The main constructor method of the crate.
    pub fn now() -> io::Result<CpuInstant> {
        let (cpu_total, cpu_idle) = imp::get_cpu_totals()?;
        Ok(CpuInstant {
            instant: time::Instant::now(),
            cpu_total,
            cpu_idle,
        })
    }

    /// Get the wrapped `time::Instant`.
    pub fn instant(&self) -> time::Instant {
        self.instant
    }
}

impl ops::Sub for CpuInstant {
    type Output = CpuDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        CpuDuration {
            duration: self.instant - rhs.instant,
            cpu_total: self.cpu_total - rhs.cpu_total,
            cpu_idle: self.cpu_idle - rhs.cpu_idle,
        }
    }
}

/// Like `std::time::Duration`, but with information about the cpu usage stats.
///
/// The way to get this is to subtract one `CpuInstant` from another.
#[derive(Debug, Copy, Clone)]
pub struct CpuDuration {
    duration: time::Duration,
    cpu_total: f64,
    cpu_idle: f64,
}

impl CpuDuration {
    /// The gap between samples.
    pub fn duration(&self) -> time::Duration {
        self.duration
    }

    /// The proportion of the time spent idle (between 0 and 1).
    pub fn idle(&self) -> f64 {
        self.cpu_idle / self.cpu_total
    }

    /// The proportion of the time spent not idle (between 0 and 1).
    pub fn non_idle(&self) -> f64 {
        1.0 - self.idle()
    }
}

