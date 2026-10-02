#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frequency {
    Hz(u16),
    MHz(f32),
    GHz(f32),
}

impl Frequency {
    pub fn from_hz(hz: u64) -> Self {
        let mut divisions = 0;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Mantissa is 52 bits, 2^52 = a lot"
        )]
        let mut f_hz = hz as f64;

        while f_hz >= 1000.0 && divisions < 3 {
            f_hz /= 1000.0;
            divisions += 1;
        }

        match divisions {
            0 => Self::Hz(hz as u16),
            1 => Self::MHz(f_hz as f32),
            2 => Self::GHz(f_hz as f32),
            _ => unreachable!(),
        }
    }

    pub fn as_hz(self) -> u64 {
        match self {
            Self::Hz(h) => h as u64,
            Self::MHz(h) => (h * 1000.0) as u64,
            Self::GHz(h) => (h * 1000.0 * 1000.0) as u64,
        }
    }
}

impl From<Frequency> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Frequency) -> Self {
        val.as_hz() as Self
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::Hz(0)
    }
}

impl core::fmt::Display for Frequency {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Hz(b) => write!(f, "{b} Hz"),
            Self::MHz(b) => write!(f, "{b:.02} MHz"),
            Self::GHz(b) => write!(f, "{b:.02} GHz"),
        }
    }
}