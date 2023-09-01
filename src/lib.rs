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
    /// * `Ok(true)` on successful reads of all bytes
    /// * `Ok(false)` if not all bytes could be received in time
    /// * `Err(_)` if general IO error occured
    fn rx_bytes_exact(
        &mut self,
        bytes: &mut [u8],
        block_for: Duration,
        max_trys: Option<u8>,
    ) -> Result<bool, Self::Error> {
        let max_trys = max_trys.unwrap_or(bytes.len() as u8);
        for _ in 0..max_trys {}

        Ok(false)
    }
}

pub struct EbusHandler {}
