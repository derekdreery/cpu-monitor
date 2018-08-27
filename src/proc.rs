//! Parsers for the contents of the `/proc` directory.
//!
use nom;
use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::time::Duration;
use std::ops;
use std;

/// The stats from `/proc/stat`.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Stat {
    /// Total stats, sum of all cpus.
    pub cpu_totals: StatCpu,
    /// For each cpu, the number of *units* spent in different contexts.
    pub cpus: Vec<StatCpu>,
    /// Number of context switches since the system booted.
    pub context_switches: u64,
    /// Timestamp (in seconds since epoch) that system booted.
    pub boot_time: u64,
    /// The total number of processes and threads created since system booted.
    pub processes: u64,
    /// The total number of processes running on the cpu.
    pub procs_running: u64,
    /// The total number of processes waiting to run on the cpu.
    pub procs_blocked: u64,
    // todo `softirq`
}

impl ops::Sub for Stat {
    type Output = StatDelta;

    fn sub(mut self, rhs: Self) -> Self::Output {
        assert_eq!(self.cpus.len(), rhs.cpus.len(), "different number of cpus");
        self.cpus.iter_mut()
            .zip(rhs.cpus.into_iter())
            .for_each(|(this, rhs)| *this = *this - rhs);
        let cpu_totals = self.cpu_totals - rhs.cpu_totals;
        let context_switches = self.context_switches.checked_sub(rhs.context_switches).unwrap();
        let processes = self.processes.checked_sub(rhs.processes).unwrap();
        let max = ::std::i64::MAX as u64;
        if self.procs_running > max || rhs.procs_running > max {
            panic!("overflow");
        }
        let procs_running = (self.procs_running as i64)
            .checked_sub(rhs.procs_running as i64).unwrap();
        if self.procs_blocked > max || rhs.procs_blocked > max {
            panic!("overflow");
        }
        let procs_blocked = (self.procs_blocked as i64)
            .checked_sub(rhs.procs_blocked as i64).unwrap();
        StatDelta {
            cpu_totals,
            cpus: self.cpus,
            context_switches,
            processes,
            procs_running,
            procs_blocked,
        }
    }
}

impl<'a> ops::Sub<&'a Stat> for Stat {
    type Output = StatDelta;

    fn sub(mut self, rhs: &'a Self) -> Self::Output {
        assert_eq!(self.cpus.len(), rhs.cpus.len(), "different number of cpus");
        self.cpus.iter_mut()
            .zip(rhs.cpus.iter())
            .for_each(|(this, rhs)| *this = *this - *rhs);
        let cpu_totals = self.cpu_totals - rhs.cpu_totals;
        let context_switches = self.context_switches.checked_sub(rhs.context_switches).unwrap();
        let processes = self.processes.checked_sub(rhs.processes).unwrap();
        let max = ::std::i64::MAX as u64;
        if self.procs_running > max || rhs.procs_running > max {
            panic!("overflow");
        }
        let procs_running = (self.procs_running as i64)
            .checked_sub(rhs.procs_running as i64).unwrap();
        if self.procs_blocked > max || rhs.procs_blocked > max {
            panic!("overflow");
        }
        let procs_blocked = (self.procs_blocked as i64)
            .checked_sub(rhs.procs_blocked as i64).unwrap();
        StatDelta {
            cpu_totals,
            cpus: self.cpus,
            context_switches,
            processes,
            procs_running,
            procs_blocked,
        }
    }
}
impl Stat {
    pub fn from_system() -> io::Result<Self> {
        Stat::from_iter(BufReader::new(File::open("/proc/stat")?).lines())
    }

    fn from_iter(mut iter: impl Iterator<Item=io::Result<String>>) -> io::Result<Stat> {
        let cpu_totals = parse_line(&mut iter, |s| StatCpu::from_str(&s))?;
        let mut cpus = Vec::new();
        let mut next_line;
        loop {
            next_line = iter.next().ok_or(io::ErrorKind::InvalidData.into()).and_then(|v| v)?;
            if let Some(cpu_info) = StatCpu::from_str(&next_line) {
                cpus.push(cpu_info);
            } else {
                break;
            }
        }
        // skip interrupts (in next_line)

        let context_switches = parse_line(&mut iter, |s| parse_single("ctxt", s))?;
        let boot_time = parse_line(&mut iter, |s| parse_single("btime", s))?;
        let processes = parse_line(&mut iter, |s| parse_single("processes", s))?;
        let procs_running = parse_line(&mut iter, |s| parse_single("procs_running", s))?;
        let procs_blocked = parse_line(&mut iter, |s| parse_single("procs_blocked", s))?;
        // todo softirq
        Ok(Stat {
            cpu_totals,
            cpus,
            context_switches,
            boot_time,
            processes,
            procs_running,
            procs_blocked
        })
    }
}

/// The change in the stat values over a time period
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StatDelta {
    /// The number of *units* over the time period.
    pub cpu_totals: StatCpu,
    /// For each cpu, the number of *units* spent in different contexts.
    pub cpus: Vec<StatCpu>,
    /// Number of context switches.
    pub context_switches: u64,
    /// The total number of processes and threads created over the time period.
    pub processes: u64,
    /// The change in number of processes running on the cpu.
    pub procs_running: i64,
    /// The change in number of processes waiting to run on the cpu.
    pub procs_blocked: i64,
    // todo `softirq`
}

/// Helper to flatten all the i/o errors
fn parse_line<I, F, Val>(iter: &mut I, mapper: F) -> io::Result<Val>
where I: Iterator<Item=io::Result<String>>,
      F: Fn(String) -> Option<Val>
{
    iter.next()
        .ok_or(io::ErrorKind::InvalidData.into())
        .and_then(|val| val) // flatten
        .and_then(|val| mapper(val).ok_or(io::ErrorKind::InvalidData.into()))
}

/// Info about the number of *units* in the various cpu contexts.
///
/// *units* could be anything, for example cpu cycles, or hundredths of a second. The numbers only
/// really make sense as a proportion of the total.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StatCpu {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
}

impl ops::Sub for StatCpu {
    type Output = StatCpu;

    fn sub(self, rhs: Self) -> Self::Output {
        StatCpu {
            user: self.user.checked_sub(rhs.user).unwrap(),
            nice: self.nice.checked_sub(rhs.nice).unwrap(),
            system: self.system.checked_sub(rhs.system).unwrap(),
            idle: self.idle.checked_sub(rhs.idle).unwrap(),
            iowait: self.iowait.checked_sub(rhs.iowait).unwrap(),
            irq: self.irq.checked_sub(rhs.irq).unwrap(),
            softirq: self.softirq.checked_sub(rhs.softirq).unwrap(),
            steal: self.steal.checked_sub(rhs.steal).unwrap(),
            guest: self.guest.checked_sub(rhs.guest).unwrap(),
        }
    }
}

impl StatCpu {
    fn from_str(input: &str) -> Option<StatCpu> {
        parse_cpu_line(input).map(|(_, answer)| answer).ok()
    }

    pub fn total(&self) -> u64 {
        self.user
            .checked_add(self.nice).unwrap()
            .checked_add(self.system).unwrap()
            .checked_add(self.idle).unwrap()
            .checked_add(self.iowait).unwrap()
            .checked_add(self.irq).unwrap()
            .checked_add(self.softirq).unwrap()
            .checked_add(self.steal).unwrap()
            .checked_add(self.guest).unwrap()
    }

}

named!(parse_cpu_line<&str, StatCpu>, do_parse!(
    tag!("cpu") >>
    opt!(nom::digit0) >>
    call!(nom::space0) >>
    user: call!(parse_u64) >>
    call!(nom::space0) >>
    nice: call!(parse_u64) >>
    call!(nom::space0) >>
    system: call!(parse_u64) >>
    call!(nom::space0) >>
    idle: call!(parse_u64) >>
    call!(nom::space0) >>
    iowait: call!(parse_u64) >>
    call!(nom::space0) >>
    irq: call!(parse_u64) >>
    call!(nom::space0) >>
    softirq: call!(parse_u64) >>
    call!(nom::space0) >>
    steal: call!(parse_u64) >>
    call!(nom::space0) >>
    guest: call!(parse_u64) >>
    (StatCpu { user, nice, system, idle, iowait, irq, softirq, steal, guest })
));


fn parse_single(name: &str, input: impl AsRef<str>) -> Option<u64> {
    let result = do_parse!(input.as_ref(),
        tag!(name) >>
        tag!(" ") >>
        val: call!(parse_u64) >>
        (val)
    );
    result.map(|(_, val)| val).ok()
}

/// Parse a u64.
///
/// Will return successfully if there is no more input.
fn parse_u64(mut input: &str) -> nom::IResult<&str, u64> {
    let mut num = input.get(0..1)
        .and_then(|s| s.chars().next())
        .and_then(|d| d.to_digit(10))
        .map(|d| d as u64)
        .ok_or(nom::Err::Error(error_position!(input, nom::ErrorKind::Custom(0))))?;
    loop {
        input = &input[1..]; // digits are all 1 utf8 byte.
        match input.get(0..1)
            .and_then(|s| s.chars().next())
            .and_then(|d| d.to_digit(10))
            .map(|d| d as u64)
        {
            Some(d) => { num = num * 10 + d },
            None => break,
        }
    }
    Ok((input, num))
}

pub struct DiskStats {
    inner: Vec<DiskStat>
}

impl DiskStats {
    pub fn from_system() -> io::Result<Self> {
        let mut reader = BufReader::new(File::open("/proc/diskstats")?);
        let mut disk_stats = Vec::new();

        unimplemented!()
    }

    pub fn iter(&self) -> impl Iterator<Item=&DiskStat> {
        self.inner.iter()
    }
}

impl IntoIterator for DiskStats {
    type IntoIter = std::vec::IntoIter<DiskStat>;
    type Item = DiskStat;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

pub struct DiskStat {
    pub major: u64,
    pub minor: u64,
    pub name: String,
    pub reads_completed: u64,
    pub reads_merged: u64,
    pub sectors_read: u64,
    // in ms
    pub time_reading: Duration,
    pub writes_completed: u64,
    pub writes_merged: u64,
    pub sectors_written: u64,
    // in ms
    pub time_writing: Duration,
    pub io_in_progress: u64,
    // in ms
    pub time_io: Duration,
    // in ms
    pub time_io_weighted: Duration,
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead};
    use super::Stat;

    #[test]
    fn parse_single() {
        let input = "processes 2453";
        assert_eq!(super::parse_single("processes", input).unwrap(), 2453);
    }

    #[test]
    fn parse_u64() {
        assert!(super::parse_u64("a123").is_err());
        assert_eq!(super::parse_u64("12 "),
                   Result::Ok((" ", 12)));
        assert_eq!(super::parse_u64("12"),
                   Result::Ok(("", 12)));
    }

    #[test]
    fn proc_stat() {
        let raw = "\
cpu  17501 2 6293 8212469 20141 1955 805 0 0 0
cpu0 4713 0 1720 2049410 8036 260 255 0 0 0
cpu1 3866 0 1325 2054893 3673 928 307 0 0 0
cpu2 4966 1 1988 2051243 5596 516 141 0 0 0
cpu3 3955 0 1258 2056922 2835 250 100 0 0 0
intr 1015182 8 8252 0 0 0 0 0 0 1 113449 0 0 198907 0 0 0 18494 0 0 1 0 0 0 29 22 7171 46413 13 0 413 167 528 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
ctxt 2238717
btime 1535128607
processes 2453
procs_running 1
procs_blocked 0
softirq 4257581 64 299604 69 2986 36581 0 3497229 283111 0 137937
";
        let _stat = Stat::from_iter(io::Cursor::new(raw).lines()).unwrap();
    }
}
