use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemorySize {
    Byte(u16),
    Kb(f32),
    Mb(f32),
    Gb(f32),
}

impl MemorySize {
    pub fn from_bytes(bytes: u64) -> Self {
        let mut divisions = 0;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Mantissa is 52 bits, 2^52 = 4 petabytes"
        )]
        let mut f_bytes = bytes as f64;

        while f_bytes >= 1024.0 && divisions < 4 {
            f_bytes /= 1024.0;
            divisions += 1;
        }

        match divisions {
            0 => Self::Byte(bytes as u16),
            1 => Self::Kb(f_bytes as f32),
            2 => Self::Mb(f_bytes as f32),
            3 => Self::Gb(f_bytes as f32),
            _ => unreachable!(),
        }
    }

    pub fn from_kilobytes(mut kilobytes: f64) -> Self {
        let mut divisions = 1;

        while kilobytes >= 1024.0 && divisions < 3 {
            kilobytes /= 1024.0;
            divisions += 1;
        }

        match divisions {
            1 => Self::Kb(kilobytes as f32),
            2 => Self::Mb(kilobytes as f32),
            3 => Self::Gb(kilobytes as f32),
            _ => unreachable!(),
        }
    }

    pub fn as_bytes(self) -> u64 {
        match self {
            Self::Byte(b) => b as u64,
            Self::Kb(b) => (b * 1024.0) as u64,
            Self::Mb(b) => (b * 1024.0 * 1024.0) as u64,
            Self::Gb(b) => (b * 1024.0 * 1024.0 * 1024.0) as u64,
        }
    }

    pub fn as_kilobytes(self) -> f64 {
        match self {
            Self::Byte(b) => b as f64 / 1024.0,
            Self::Kb(b) => b as f64,
            Self::Mb(b) => (b * 1024.0) as f64,
            Self::Gb(b) => (b * 1024.0 * 1024.0) as f64,
        }
    }
}

impl From<MemorySize> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: MemorySize) -> Self {
        val.as_bytes() as Self
    }
}

impl Default for MemorySize {
    fn default() -> Self {
        Self::Byte(0)
    }
}

impl core::fmt::Display for MemorySize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Byte(b) => write!(f, "{b} Bytes"),
            Self::Kb(b) => write!(f, "{b:.02} Kb"),
            Self::Mb(b) => write!(f, "{b:.02} Mb"),
            Self::Gb(b) => write!(f, "{b:.02} Gb"),
        }
    }
}

impl PartialOrd for MemorySize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_bytes().partial_cmp(&other.as_bytes())
    }
}
