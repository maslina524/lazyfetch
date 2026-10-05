use core::{
    ptr,
    ffi::CStr
};

use alloc::{
    borrow::ToOwned,
    string::String
};

use crate::{
    detect::datetime::{DatetimeInfo, AmPm},
    unix::libc::{time, localtime},
    format
};

impl DatetimeInfo {
    pub fn new() -> Self {
        let time = time(ptr::null_mut());
        let info_ptr = localtime(&raw const time);
        // SAFETY: Libc always returns a valid pointer
        let info = unsafe { &*info_ptr };

        let year = info.tm_year as u16 + 1900;
        let month = info.tm_mon as u16;

        let month_name = Self::month_name(month);
        let month_name_short = month_name[..3].to_owned();

        let hour = info.tm_hour as u16;
        let minute = info.tm_min as u16;
        let second = info.tm_sec as u16;

        let week = (info.tm_wday as u16 + 6) % 7 + 1;
        let weekday = Self::day_of_week(week);
        let weekday_short = weekday[..3].to_owned();

        let day_in_year = info.tm_yday as u16 + 1;
        let day_in_month = info.tm_mday as u16;
        let day_in_week = (info.tm_wday as u16 + 6) % 7 + 1;

        let offset_utc = Self::offset_utc_string(info.tm_gmtoff);

        let timezone_name = if info.tm_zone.is_null() {
            "Unknown".to_owned()
        } else {
            // SAFETY: Libs are guaranteed to store a valid cstr
            let tz_c_str = unsafe { CStr::from_ptr(info.tm_zone) };
            tz_c_str.to_string_lossy().into_owned()
        };
        
        let am_pm = AmPm::from_hour(hour as u8);

        Self {
            year, month, month_name, month_name_short,
            hour, minute, second, week,
            weekday, weekday_short, day_in_year, day_in_month,
            day_in_week, offset_utc, timezone_name, am_pm
        }
    }

    fn offset_utc_string(mut offset: i64) -> String {
        let symb = if offset < 0 { '-' } else { '+' };
        offset = offset.abs();
        let hours = offset / 60;
        let mins = offset % 60;

        format!("{symb}{hours:02}{mins:02}")
    }

    fn month_name(num: u16) -> String {
        match num {
            0  => "January",
            1  => "February",
            2  => "March",
            3  => "April",
            4  => "May",
            5  => "June",
            6  => "July",
            7  => "August",
            8  => "September",
            9  => "October",
            10 => "November",
            11 => "December",
            _ => unreachable!()
        }.to_owned()
    }

    fn day_of_week(num: u16) -> String {
        match num {
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            7 => "Sunday",
            _ => unreachable!()
        }.to_owned()
    }
}