use core::ops;

/// Telegram to be sent
#[derive(Clone, Debug)]
pub struct MasterTelegram {
    /// Core telegram data
    pub telegram: Telegram,
    /// Options for the handling of this telegram
    pub flags: TelegramFlags,
}

#[derive(Clone, Debug)]
pub struct Telegram {
    /// QQ - source eBUS address
    pub src: u8,
    /// ZZ - destination eBUS address
    pub dest: u8,
    /// Service command, encoded LSB first
    pub service: u16,
    /// Up to 32 data bytes
    pub data: Buffer,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    data: [u8; 32],
    len: u8,
}

impl Buffer {
    /// Create `Buffer` from byte slice with at most 32 elements.
    ///
    /// ## Panics
    ///
    /// Panics if `bytes.len() > 32`
    #[inline]
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut data = [0; 32];
        data[..bytes.len()].copy_from_slice(bytes);

        Buffer {
            data,
            len: bytes.len() as u8,
        }
    }

    pub const fn from_parts(data: [u8; 32], len: u8) -> Self {
        Buffer { data, len }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.len as usize]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TelegramFlag {
    /// Whether the data is expected to have an additional CRC prepended
    NeedsDataCrc = 0,
    /// Whether or not to wait for the recipient to reply
    ExpectReply = 1,
}

impl ops::BitOr for TelegramFlag {
    type Output = TelegramFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        TelegramFlags(1 << self as u8 | 1 << rhs as u8)
    }
}

impl ops::BitOr<TelegramFlags> for TelegramFlag {
    type Output = TelegramFlags;

    fn bitor(self, rhs: TelegramFlags) -> Self::Output {
        TelegramFlags(1 << self as u8 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelegramFlags(u8);

impl TelegramFlags {
    pub fn none() -> Self {
        TelegramFlags(0)
    }
}

impl ops::BitAnd<TelegramFlag> for TelegramFlags {
    type Output = bool;

    fn bitand(self, rhs: TelegramFlag) -> Self::Output {
        self.0 & (1 << rhs as u8) != 0
    }
}

impl ops::BitOr<TelegramFlag> for TelegramFlags {
    type Output = TelegramFlags;

    fn bitor(self, rhs: TelegramFlag) -> Self::Output {
        TelegramFlags(self.0 | 1 << rhs as u8)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TelegramFlag, TelegramFlags};

    #[test]
    fn test_bitor() {
        let flags = TelegramFlag::ExpectReply | TelegramFlag::NeedsDataCrc;
        assert_eq!(flags.0, 0x3);
    }

    #[test]
    fn test_bitor2() {
        let mut flags = TelegramFlag::ExpectReply | TelegramFlag::NeedsDataCrc;
        flags = flags | TelegramFlag::ExpectReply;
        assert_eq!(flags.0, 0x3);
    }

    #[test]
    fn test_bitor3() {
        let flags = TelegramFlag::ExpectReply | TelegramFlags::none();
        assert_eq!(flags.0, 0x2);
    }
}
