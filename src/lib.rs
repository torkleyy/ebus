#![no_std]

use core::time::Duration;

pub trait EbusInterface {
    const POLY_TELEGRAM: u8;
    const POLY_DATA: u8;

    type Error;

    /// Transmit all bytes in the buffer.
    fn tx_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Transmit a single byte.
    fn tx_single(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.tx_bytes(&[byte])
    }

    /// Receive some bytes with a specified timeout.
    ///
    /// ## Parameters
    ///
    /// * `bytes`: the buffer to write to
    /// * `block_for`: duration to wait until at least one byte is received
    ///     * specify `Duration::ZERO` for non blocking
    ///     * specify `Duration::MAX` for infinite blocking
    ///
    /// ## Returns
    ///
    /// * `Ok(0)` on timeout
    /// * `Ok(n)` if n bytes have been received
    /// * `Err(_)` if general IO error occured
    fn rx_bytes(&mut self, bytes: &mut [u8], block_for: Duration) -> Result<usize, Self::Error>;

    /// Receive enough bytes to fill `bytes` with a specified timeout.
    ///
    /// ## Parameters
    ///
    /// * `bytes`: the buffer to write to
    /// * `block_for`: duration to wait until at least one byte is received
    ///     * specify `Duration::ZERO` for non blocking
    ///     * specify `Duration::MAX` for infinite blocking
    /// * `max_trys`: the number of attempts to fill the buffer. Defaults to the length of `bytes`.
    ///
    /// ## Returns
    ///
    /// * `Ok(n)` on successful read of the byte
    /// * `Err(_)` if general IO error occured
    fn rx_single(&mut self, block_for: Duration) -> Result<u8, Self::Error> {
        let mut bytes = [0];
        self.rx_bytes(&mut bytes, block_for)?;

        Ok(bytes[0])
    }
}

pub struct EbusDriver<I> {
    pub interface: I,
    pub state: EbusState,
}

pub struct EbusState {
    pub flags: u8,
}

impl EbusState {
    pub fn add(&mut self, flag: Flag) {
        self.flags |= 1 << flag as u8;
    }

    pub fn remove(&mut self, flag: Flag) {
        self.flags &= !(1 << flag as u8);
    }

    pub fn has(&mut self, flag: Flag) -> bool {
        (self.flags & (flag as u8)) != 0
    }
}

#[repr(u8)]
pub enum Flag {
    /// Are we allowed to send?
    BusLock = 0,
    /// Last byte we received was 0x9F prefix
    WasEscapePrefix = 1,
}
