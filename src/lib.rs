#[macro_use] extern crate nom;

mod proc;

use std::time;
use std::ops;
use std::io;

#[cfg(not(unix))]
compile_error!("This is a linux-only library, for now.");

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CpuInstant {
    instant: time::Instant,
    stat: proc::Stat,
}

impl CpuInstant {
    pub fn now() -> io::Result<CpuInstant> {
        Ok(CpuInstant {
            instant: time::Instant::now(),
            stat: proc::Stat::from_system()?,
        })
    }

    pub fn instant(&self) -> time::Instant {
        self.instant
    }
}

impl ops::Sub for CpuInstant {
    type Output = CpuDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        CpuDuration {
            duration: self.instant - rhs.instant,
            stat: self.stat - rhs.stat,
        }
    }
}

impl<'a> ops::Sub<&'a CpuInstant> for CpuInstant {
    type Output = CpuDuration;

    fn sub(self, rhs: &'a Self) -> Self::Output {
        CpuDuration {
            duration: self.instant - rhs.instant,
            stat: self.stat - &rhs.stat,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CpuDuration {
    duration: time::Duration,
    stat: proc::StatDelta,
}

impl CpuDuration {
    pub fn duration(&self) -> time::Duration {
        self.duration
    }

    pub fn idle(&self) -> f64 {
        ((self.stat.cpu_totals.idle + self.stat.cpu_totals.iowait) as f64)
            / (self.stat.cpu_totals.total() as f64)
    }

    pub fn non_idle(&self) -> f64 {
        1.0 - self.idle()
    }
}

