pub struct Crc<const POLY: u8> {
    crc: u8,
}

impl<const POLY: u8> Crc<POLY> {
    const CRC_WIDTH: u8 = 8;
    const CRC_INIT: u8 = 0x00;

    pub fn new() -> Self {
        Crc {
            crc: Self::CRC_INIT,
        }
    }

    pub fn add(&mut self, mut byte: u8) {
        let mut polynom;

        for _ in 0..Self::CRC_WIDTH {
            if self.crc & 0x80 != 0 {
                polynom = POLY;
            } else {
                polynom = 0;
            }
            self.crc = (self.crc & !0x80) << 1;
            if (byte & 0x80) != 0 {
                self.crc |= 1;
            }
            self.crc ^= polynom;
            byte = byte << 1;
        }
    }

    pub fn add_multiple(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add(*byte);
        }
    }

    pub fn crc(&self) -> u8 {
        self.crc
    }
}
