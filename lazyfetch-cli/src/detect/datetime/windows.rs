use alloc::{borrow::ToOwned, string::String};

use crate::{
    detect::datetime::{AmPm, DatetimeInfo},
    format,
    windows::error::ErrorCode,
    windows::link::{
        DYNAMIC_TIME_ZONE_INFORMATION, GetDynamicTimeZoneInformation, GetLocalTime,
        GetTimeZoneInformation, SYSTEMTIME, TIME_ZONE_INFORMATION,
    },
};

impl DatetimeInfo {
    pub fn new() -> Self {
        let mut time = SYSTEMTIME::default();
        // SAFETY: Completely safe
        unsafe { GetLocalTime(&raw mut time) };

        let year = time.wYear;
        let month = time.wMonth;
        let month_name = Self::month_name(time.wMonth);
        let month_name_short = month_name[..3].to_owned();

        let hour = time.wHour;
        let minute = time.wMinute;
        let second = time.wSecond;

        let day_in_year = Self::day_in_year(&time);
        let week = day_in_year / 7 + 1;
        let weekday = Self::day_of_week(time.wDayOfWeek);
        let weekday_short = weekday[..3].to_owned();

        let day_in_month = time.wDay;
        let day_in_week = time.wDayOfWeek;

        let offset_utc = Self::offset_utc_string();
        let timezone_name = Self::time_zone_string();
        let am_pm = AmPm::from_hour(hour as u8);

        Self {
            year,
            month,
            month_name,
            month_name_short,
            hour,
            minute,
            second,
            week,
            weekday,
            weekday_short,
            day_in_year,
            day_in_month,
            day_in_week,
            offset_utc,
            timezone_name,
            am_pm,
        }
    }

    fn month_name(num: u16) -> String {
        match num {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => unreachable!(),
        }
        .to_owned()
    }

    fn day_of_week(num: u16) -> String {
        match num {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => unreachable!(),
        }
        .to_owned()
    }

    fn day_in_year(time: &SYSTEMTIME) -> u16 {
        let mut days_in_month: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let is_leap = time.wYear.is_multiple_of(400)
            || (time.wYear.is_multiple_of(4) && time.wYear.is_multiple_of(100));

        if is_leap {
            days_in_month[1] = 29;
        }

        let mut day_os_year = 0u16;
        for i in days_in_month.iter().take(time.wMonth as usize - 1) {
            day_os_year += *i as u16;
        }
        day_os_year += time.wDay;

        day_os_year
    }

    fn time_zone() -> i16 {
        let mut tzi = TIME_ZONE_INFORMATION::default();

        // SAFETY: Completely safe
        let ret = unsafe { GetTimeZoneInformation(&raw mut tzi) };
        if ret == 0xFFFF_FFFF {
            ErrorCode::last().panic()
        }

        let mut bias_mins = tzi.Bias;

        if ret == 2 {
            bias_mins += tzi.DaylightBias;
        }

        (-bias_mins) as i16
    }

    fn offset_utc_string() -> String {
        let mut tz = Self::time_zone();
        let symb = if tz < 0 { '-' } else { '+' };
        tz = tz.abs();
        let hours = tz / 60;
        let mins = tz % 60;

        format!("{symb}{hours:02}{mins:02}")
    }

    fn is_daylight() -> bool {
        let mut tzi = TIME_ZONE_INFORMATION::default();

        // SAFETY: Completely safe
        let ret = unsafe { GetTimeZoneInformation(&raw mut tzi) };
        if ret == 0xFFFF_FFFF {
            ErrorCode::last().panic()
        }

        let current_bias = tzi.Bias
            + if tzi.DaylightBias != 0 {
                tzi.DaylightBias
            } else {
                tzi.StandardBias
            };

        tzi.DaylightBias != 0 && current_bias != (tzi.Bias + tzi.StandardBias)
    }

    fn time_zone_string() -> String {
        let mut tzi = DYNAMIC_TIME_ZONE_INFORMATION::default();
        // SAFETY: Completely safe
        unsafe { GetDynamicTimeZoneInformation(&raw mut tzi) };

        let is_daylight = Self::is_daylight();
        let buf = if is_daylight {
            tzi.DaylightName
        } else {
            tzi.StandardName
        };

        String::from_utf16_lossy(&buf).rsplit('\0').collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::detect::datetime::DatetimeInfo;

    #[test]
    fn date_test() {
        let date = DatetimeInfo::new();
        println!("{date:#?}");
    }
}
