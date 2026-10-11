use core::str::FromStr;

use crate::config::Config;

#[derive(Debug, Clone, Copy, Default)]
pub struct Percent(u8);

impl Percent {
    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn new(mut percent: u8) -> Self {
        if percent > 100 {
            percent = 100;
        }
        Self(percent)
    }

    pub const fn new_check(percent: u8) -> Option<Self> {
        if percent > 100 {
            None
        } else {
            Some(Self(percent))
        }
    }
}

impl FromStr for Percent {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .trim_end_matches('%')
            .parse::<u8>()
            .map_err(|_| ())
            .and_then(|v| Self::new_check(v).ok_or(()))
    }
}

impl From<Percent> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Percent) -> Self {
        val.get() as Self
    }
}

impl core::fmt::Display for Percent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = Config::get().format_percent(*self);
        write!(f, "{val}")
    }
}
