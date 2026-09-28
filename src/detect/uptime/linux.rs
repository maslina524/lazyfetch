use alloc::string::String;

use crate::{
    detect::uptime::{UptimeInfo, BOOT_TIMESTAMP}, 
    format,
    imp::env::timestamp_secs,
    linux::libc::{Tm, c_time, get_sysinfo, localtime_r, time}
};

const DAY_MS: u64 = 1000 * 60 * 60 * 24;
const HOUR_MS: u64 = 1000 * 60 * 60;
const MIN_MS: u64 = 1000 * 60;
const SEC_MS: u64 = 1000;

impl UptimeInfo {
    pub fn new() -> Self {
        let info = get_sysinfo();

        let uptime_secs = info.uptime as u64;
        let ms = uptime_secs * 1000;

        let _ = BOOT_TIMESTAMP.set(uptime_secs);

        let days = ms / DAY_MS;
        let rem = ms % DAY_MS;
        let hours = (rem / HOUR_MS) as u8;
        let rem = rem % HOUR_MS;
        let mins = (rem / MIN_MS) as u8;
        let rem = rem % MIN_MS;
        let secs = (rem / SEC_MS) as u8;
        let ms_remainder = (rem % SEC_MS) as u16;

        let years = (days / 365) as u16;
        #[allow(clippy::cast_precision_loss)]
        let years_fraction = (days as f32) / 365.0;

        let boot_time = Self::boot_time(uptime_secs);

        let formatted = Self::formatted(days as u32, hours, mins, secs);

        Self {
            years,
            days: days as u32,
            hours,
            mins,
            secs,
            ms: ms_remainder,
            boot_time,
            days_of_year: days as u32,
            years_fraction,
            formatted,
        }
    }

    fn boot_time(uptime_secs: u64) -> String {
        let mut now_ts: c_time = 0;
        time(&raw mut now_ts);
        
        let boot_ts = now_ts as u64 - uptime_secs;

        let mut tm = Tm::default();
        localtime_r((&raw const boot_ts).cast(), &raw mut tm);

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::detect::uptime::UptimeInfo;

    #[test]
    fn uptime_test() {
        let uptime = UptimeInfo::new();
        println!("{uptime:#?}");
    }
}