#[derive(Debug, Clone)]
pub struct Stream<'zlib> {
    memory: &'zlib [u8],
    byte_idx: usize,
    bit_idx: usize,
}

impl<'zlib> Stream<'zlib> {
    pub const fn new(memory: &'zlib [u8]) -> Self {
        Self { memory, byte_idx: 0, bit_idx: 0 }
    }

    pub const fn align_to_byte(&mut self) {
        if self.bit_idx != 0 {
            self.bit_idx = 0;
            self.byte_idx += 1;
        }
    }

    pub const fn read_byte(&mut self) -> u8 {
        if self.bit_idx != 0 {
            self.bit_idx = 0;
            self.byte_idx += 1;
        }
        let byte = self.memory[self.byte_idx];
        self.byte_idx += 1;
        byte
    }

    pub fn read_bytes(&mut self, count: usize) -> u64 {
        let mut bytes = 0u64;
        for i in 0..count {
            bytes |= (self.read_byte() as u64) << (8 * i);
        }
        bytes
    }

    pub const fn read_bit(&mut self) -> u8 {
        let bit = (self.memory[self.byte_idx] >> self.bit_idx) & 1;
        self.bit_idx += 1;
        if self.bit_idx > 7 {
            self.bit_idx = 0;
            self.byte_idx += 1;
        }
        bit
    }

    pub fn read_bits(&mut self, count: usize) -> u64 {
        let mut bits = 0;
        for i in 0..count {
            bits |= (self.read_bit() as u64) << i;
        }
        bits
    }
}