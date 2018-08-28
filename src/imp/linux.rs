use std::io;
use linux_proc::stat::Stat;

/// ans.0 is total, ans.1 is idle.
pub fn get_cpu_totals() -> io::Result<(f64, f64)> {
    let stat = Stat::from_system()?;
    let total = stat.cpu_totals.total() as f64;
    let idle = stat.cpu_totals.idle as f64 + stat.cpu_totals.iowait as f64;
    Ok((total, idle))
}
