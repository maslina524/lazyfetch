use crate::{
    detect::memory::MemoryInfo, 
    formats::MemorySize, 
    unix::libc::get_sysinfo
};

impl MemoryInfo {
    pub fn new() -> Self {
        let info = get_sysinfo();

        let total_raw = info.totalram;
        let in_use_raw = info.totalram - info.freeram;

        let total = MemorySize::from_bytes(total_raw);
        let in_use = MemorySize::from_bytes(in_use_raw);

        Self { total, in_use }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        detect::memory::MemoryInfo,
        formats::MemorySize
    };

    #[test]
    fn ram_test() {
        let mem = MemoryInfo::new();
        let total = mem.total;
        let in_use = mem.in_use;

        println!("Total: {total}, In use: {in_use}");

        assert_ne!(total, MemorySize::default());
        assert_ne!(in_use, MemorySize::default());
    }
}