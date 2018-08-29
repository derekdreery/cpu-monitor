This library provides methods for getting the percentage of cpu time spent idle, a.k.a. cpu usage.

It follows the structure of `std::time`, since you can only work with a difference between 2
fixed points in time. Cpu usage and a specific instant is either 0 or 1 (per core), the value of
interest is the proportion of cpu cycles spent not idle over a given period.

# Examples

```rust
extern crate cpu_monitor;

use std::io;
use std::time::Duration;

use cpu_monitor::CpuInstant;

fn main() -> Result<(), io::Error> {
    let start = CpuInstant::now()?;
    std::thread::sleep(Duration::from_millis(100));
    let end = CpuInstant::now()?;
    let duration = end - start;
    println!("cpu: {:.0}%", duration.non_idle() * 100.);
    Ok(())
}
```
