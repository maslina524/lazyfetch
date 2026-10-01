use crate::{
    detect::memory::MemoryInfo, 
    formats::MemorySize,
    parser::LineBased,
    warning
    };

impl MemoryInfo {
    pub fn new() -> Self {
        let info = match LineBased::parse_mem_info() {
            Ok(c) => c,
            Err(e) => {
                warning!("Failed to open /proc/meminfo: {e}");
                return Self::default();
            }
        };

        let total_kb_raw = if let Some(v) = info.get("MemTotal") {
            let value_str = v.trim_end_matches(" kB");
            match value_str.parse::<f64>() {
                Ok(v) => v,
                Err(e) => {
                    warning!("Failed to parse `{value_str}` as f64: {e} (Raw: {v})");
                    0.0
                }
            }
        } else {
            0.0
        };

        let available_kb_raw = if let Some(v) = info.get("MemAvailable") {
            let value_str = v.trim_end_matches(" kB");
            match value_str.parse::<f64>() {
                Ok(v) => v,
                Err(e) => {
                    warning!("Failed to parse `{value_str}` as f64: {e} (Raw: {v})");
                    0.0
                }
            }
        } else {
            0.0
        };

        let total = MemorySize::from_kilobytes(total_kb_raw);
        let in_use = MemorySize::from_kilobytes(total_kb_raw - available_kb_raw);

        Self { total, in_use }
    }
}
