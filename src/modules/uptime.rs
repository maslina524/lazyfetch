use doc::Docs;

use crate::{
    impl_module,
    impl_display_for_module,
    modules::Module,
    sync::OnceLock,
    detect::uptime::UptimeInfo
};

static UPTIME: OnceLock<Uptime> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Uptime {
    #[doc ="Days after boot"]
    pub days: u32,
    #[doc ="Hours after boot"]
    pub hours: u8,
    #[doc ="Minutes after boot"]
    pub minutes: u8,
    #[doc ="Seconds after boot"]
    pub seconds: u8,
    #[doc ="Milliseconds after boot"]
    pub milliseconds: u16,
    #[doc ="Boot time in local timezone"]
    pub boot_time: &'static str,
    #[doc ="Years integer after boot"]
    pub years: u16,
    #[doc ="Days of year after boot"]
    pub days_of_year: u32,
    #[doc ="Years fraction after boot"]
    pub years_fraction: f32,
    #[doc ="Formatted uptime"]
    pub formatted: &'static str,
}

impl Module for Uptime {
    fn new() -> Self {
        let info = UptimeInfo::new();
        Self {
            days: info.days,
            hours: info.hours,
            minutes: info.mins,
            seconds: info.secs,
            milliseconds: info.ms,
            boot_time: info.boot_time.as_str(),
            years: info.years,
            days_of_year: info.days_of_year,
            years_fraction: info.years_fraction,
            formatted: info.formatted.as_str(),
        }
    }

    fn get() -> &'static Self {
        UPTIME.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "Uptime"
    }

    fn title(&self) -> &'static str {
        "{formatted}"
    }

    fn string_name(&self) -> &'static str {
        "uptime"
    }

    impl_module!(
        days, hours, minutes, seconds, 
        milliseconds, boot_time, years, days_of_year, 
        years_fraction, formatted
    );
}

impl_display_for_module!(Uptime);