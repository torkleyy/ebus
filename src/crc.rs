pub struct Crc {
    crc: u8,
    polynom: u8,
}

impl Crc {
    const CRC_WIDTH: u8 = 8;
    const CRC_INIT: u8 = 0x00;

    pub fn new(polynom: u8) -> Self {
        Crc {
            crc: Self::CRC_INIT,
            polynom,
        }
    }

    pub fn add(&mut self, mut byte: u8) {
        let mut polynom;

        for _ in 0..Self::CRC_WIDTH {
            if self.crc & 0x80 != 0 {
                polynom = self.polynom;
            } else {
                polynom = 0;
            }
            self.crc = (self.crc & !0x80) << 1;
            if (byte & 0x80) != 0 {
                self.crc |= 1;
            }
            self.crc ^= polynom;
            byte <<= 1;
        }
    }

    pub fn add_multiple(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add(*byte);
        }
    }

    pub fn calc_crc(&self) -> u8 {
        self.crc
    }
}