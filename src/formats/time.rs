use crate::imp;

#[derive(Debug, Clone, Copy, Default)]
pub struct Time(u64);

impl Time {
    pub const fn new(time: u64) -> Self {
        Self(time)
    }
}

impl From<Time> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Time) -> Self {
        val.0 as Self
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let string = imp::env::format_timestamp(self.0, None);
        write!(f, "{string}")
    }
}
