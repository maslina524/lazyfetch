use alloc::string::String;

use crate::{
    detect::uptime::{UptimeInfo, UPTIME_INFO}, 
    format, 
    windows::link::{
        FILETIME, FileTimeToLocalFileTime, FileTimeToSystemTime, 
        GetSystemTimeAsFileTime, GetTickCount64, SYSTEMTIME
    }
};

const DAY_MS: u64 = 1000 * 60 * 60 * 24;
const HOUR_MS: u64 = 1000 * 60 * 60;
const MIN_MS: u64 = 1000 * 60;
const SEC_MS: u64 = 1000;

impl UptimeInfo {
    pub fn new() -> &'static Self {
        UPTIME_INFO.get_or_init(Self::get)
    }

    pub fn get() -> Self {
        // SAFETY: Completely safe
        let ms = unsafe { GetTickCount64() };

        let days = ms / DAY_MS;
        let rem = ms % DAY_MS;
        let hours = (rem / HOUR_MS) as u8;
        let rem = rem % HOUR_MS;
        let mins = (rem / MIN_MS) as u8;
        let rem = rem % MIN_MS;
        let secs = (rem / SEC_MS) as u8;
        let ms_remainder = (rem % SEC_MS) as u16;

        let years = (days / 365) as u16;

        #[allow(clippy::cast_precision_loss, reason = "16_777_216 years is more than enough")]
        let years_fraction = (days as f32) / 365.0;

        let (boot_time, boot_timestamp) = Self::boot_time(ms);
        let formatted = Self::formatted(days as u32, hours, mins, secs);

        Self {
            boot_timestamp,
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

    fn boot_time(ms: u64) -> (String, u64) {
        let mut now_ft = FILETIME::default();
        // SAFETY: Completely safe
        unsafe {
            GetSystemTimeAsFileTime(&raw mut now_ft);
        }
        let now_ns = ((now_ft.dwHighDateTime as u64) << 32) | (now_ft.dwLowDateTime as u64);
        let boot_ns = now_ns - ms * 10_000;

        let boot_ft_utc = FILETIME {
            dwLowDateTime: boot_ns as u32,
            dwHighDateTime: (boot_ns >> 32) as u32,
        };
        let mut boot_ft = FILETIME::default();
        unsafe {
            // SAFETY: Completely safe
            FileTimeToLocalFileTime(&raw const boot_ft_utc, &raw mut boot_ft);
        }

        let boot_timestamp = ((boot_ft.dwHighDateTime as u64) << 32) | (boot_ft.dwLowDateTime as u64);

        let mut st = SYSTEMTIME::default();
        // SAFETY: Completely safe
        unsafe {
            FileTimeToSystemTime(&raw const boot_ft, &raw mut st);
        }

        (
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                st.wYear, st.wMonth, st.wDay,
                st.wHour, st.wMinute, st.wSecond
            ),
            boot_timestamp
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