use core::str::FromStr;

use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub enum Temperature {
    Celsius(f32),
    Fahrenheit(f32),
    Kelvin(f32),
}

impl Temperature {
    pub const fn get(self) -> f32 {
        match self {
            Self::Celsius(t) | Self::Fahrenheit(t) | Self::Kelvin(t) => t,
        }
    }

    pub const fn symbol(self) -> char {
        match self {
            Self::Celsius(_) => 'C',
            Self::Fahrenheit(_) => 'F',
            Self::Kelvin(_) => 'K',
        }
    }

    pub const fn as_celsius(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => t,
            Self::Fahrenheit(t) => (t - 32.0) * 5.0 / 9.0,
            Self::Kelvin(t) => t - 273.15,
        };
        Self::Celsius(temp)
    }

    pub const fn as_fahrenheit(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => (t * 9.0 / 5.0) + 32.0,
            Self::Fahrenheit(t) => t,
            Self::Kelvin(t) => (t - 273.15) * 9.0 / 5.0 + 32.0,
        };
        Self::Fahrenheit(temp)
    }

    pub const fn as_kelvin(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => t + 273.15,
            Self::Fahrenheit(t) => (t - 32.0) * 5.0 / 9.0 + 273.15,
            Self::Kelvin(t) => t,
        };
        Self::Kelvin(temp)
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self::Celsius(0.0)
    }
}

impl FromStr for Temperature {
    type Err = ();
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim().trim_start_matches('+');
        let (val_str, suffix) = if s.ends_with("°C") {
            (s.trim_end_matches("°C"), "°C")
        } else if s.ends_with("°F") {
            (s.trim_end_matches("°F"), "°F")
        } else if s.ends_with("°K") {
            (s.trim_end_matches("°K"), "°K")
        } else {
            return Err(());
        };
        let val = val_str.parse::<f32>().map_err(|_| ())?;
        match suffix {
            "°C" => Ok(Self::Celsius(val)),
            "°F" => Ok(Self::Fahrenheit(val)),
            "°K" => Ok(Self::Kelvin(val)),
            _ => unreachable!(),
        }
    }
}

impl From<Temperature> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Temperature) -> Self {
        val.get() as Self
    }
}

impl core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = Config::get().format_temperature(*self);
        write!(f, "{val}")
    }
}