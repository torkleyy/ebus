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

    pub fn add(&mut self, mut byte: u8) -> &mut Self {
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

        self
    }

    pub fn add_multiple(&mut self, bytes: &[u8]) -> &mut Self {
        for byte in bytes {
            self.add(*byte);
        }

        self
    }

    pub fn calc_crc(&self) -> u8 {
        self.crc
    }
}

#[cfg(test)]
mod tests {
    use crate::Crc;

    #[test]
    fn test_crc0x9b() {
        let mut crc = Crc::new(0x9B);
        crc.add(0x1E);
        assert_eq!(crc.calc_crc(), 0x1E);

        crc.add_multiple(&[15, 0]);
        assert_eq!(crc.calc_crc(), 0xD1);
    }

    #[test]
    fn test_crc0x5c() {
        let mut crc = Crc::new(0x5C);
        crc.add(0x0);
        assert_eq!(crc.calc_crc(), 0x0);

        crc.add_multiple(&[15, 0]);
        assert_eq!(crc.calc_crc(), 0x90);
    }
}
