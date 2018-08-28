extern crate cpu_monitor;

use std::io;
use std::thread;
use std::time::Duration;

use cpu_monitor::CpuInstant;

const CR_CODE: &'static str = "\x1b[G";
const CLEAR_CODE: &'static str = "\x1b[K";

fn main() -> Result<(), io::Error> {
    let period = Duration::from_secs(1);
    println!("CPU monitor - time period is {:?}", period);
    let mut start = CpuInstant::now()?;
    loop {
        thread::sleep(period);
        let end = CpuInstant::now()?;
        let duration = end.clone() - start;
        print!("{}Usage: {:.0}%{}", CR_CODE, duration.non_idle() * 100., CLEAR_CODE);
        io::Write::flush(&mut io::stdout()).unwrap();
        start = end;
    }
}
