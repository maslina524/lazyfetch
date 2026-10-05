use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_family = "unix")] {
        mod unix;
    }
}

#[derive(Debug)]
pub enum AmPm {
    Am, Pm
}

impl AmPm {
    pub const fn from_hour(hour: u8) -> Self {
        if hour < 12 {
            Self::Am
        } else {
            Self::Pm
        }
    }
}

impl core::fmt::Display for AmPm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let string = match self {
            Self::Am => "am",
            Self::Pm => "pm"
        }.to_owned();

        write!(f, "{string}")
    }
}

#[derive(Debug)]
pub struct DatetimeInfo {
    pub year: u16,
    pub month: u16,
    pub month_name: String,
    pub month_name_short: String,

    pub hour: u16,
    pub minute: u16,
    pub second: u16,

    pub week: u16,
    pub weekday: String,
    pub weekday_short: String,
    pub day_in_year: u16,
    pub day_in_month: u16,
    pub day_in_week: u16,

    pub offset_utc: String,
    pub timezone_name: String, 
    pub am_pm: AmPm
}