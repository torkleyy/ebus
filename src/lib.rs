#![no_std]
#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! Here is an example how once could integrate this library:
//!
//! ```rust
#![doc = include_str!("../examples/integration.rs")]
//! ```

use core::{fmt::Debug, time::Duration};

pub use crc::Crc;
pub use telegram::Telegram;

mod crc;
mod telegram;

const SYN: u8 = 0xAA;
const ACK_OK: u8 = 0x00;
const ACK_ERR: u8 = 0xFF;
const ESCAPE_PREFIX: u8 = 0xA9;

const FAIRNESS_MAX: u8 = 50;

pub struct EbusDriver {
    crc_poly_telegram: u8,
    crc_poly_data: u8,
    arbitration_delay: Duration,

    flags: Flags,
    /// Fairness counter, confusingly called "lock counter" in spec.
    ///
    /// Allows bus access if 0, gets reset to FAIRNESS_MAX after successful access.
    fairness_counter: u8,
    state: State,
}

impl EbusDriver {
    pub fn new(arbitration_delay: Duration, crc_poly_telegram: u8, crc_poly_data: u8) -> Self {
        EbusDriver {
            flags: Default::default(),
            fairness_counter: FAIRNESS_MAX,
            state: State::Idle,
            crc_poly_telegram,
            crc_poly_data,
            arbitration_delay,
        }
    }

    pub fn process<T: Transmit>(
        &mut self,
        word: u8,
        transmit: &mut T,
        sleep: impl Fn(Duration),
        next_msg: Option<&Telegram<'_>>,
    ) -> Result<ProcessResult, T::Error> {
        if word == SYN {
            let was_timeout = self.state.is_awaiting();

            if self.process_syn() && next_msg.is_some() {
                #[allow(clippy::unnecessary_unwrap)]
                let msg = next_msg.unwrap();
                let src = msg.src;

                sleep(self.arbitration_delay);
                transmit.transmit_encode(&[src])?;
                self.state = State::AcquiringLock;
            } else {
                self.flags.remove(Flag::WasEscapePrefix);
                sleep(Duration::from_millis(10));
            }

            if was_timeout {
                Ok(ProcessResult::Timeout)
            } else {
                Ok(ProcessResult::None)
            }
        } else if self.state.is_idle() {
            // most common case
            // do nothing
            Ok(ProcessResult::None)
        } else {
            // we are not in idle state, there must be a msg
            let msg = next_msg.unwrap();
            self.process_slow(word, transmit, sleep, msg)
        }
    }

    /// Returns `true` if we may lock the bus
    fn process_syn(&mut self) -> bool {
        if self.state.has_bus_lock() {
            log::warn!("unexpected SYN while holding bus lock");
            self.reset();
        } else if self.state.is_acquiring() {
            log::warn!("unexpected double SYN prevented lock");
            self.reset();
        } else if self.is_allowed_to_lock() {
            return true;
        } else {
            self.fairness_counter -= 1;
        }

        false
    }

    // lower branch weight and speed up `process` for more consistent delay
    #[inline(never)]
    #[cold]
    fn process_slow<T: Transmit>(
        &mut self,
        mut word: u8,
        transmit: &mut T,
        sleep: impl Fn(Duration),
        msg: &Telegram<'_>,
    ) -> Result<ProcessResult, T::Error> {
        // ugly: we have to build the crc for response before converting escape sequences
        if let State::ReceivingData { crc, .. } = &mut self.state {
            crc.add(word);
        }

        if self.flags.check_remove(Flag::WasEscapePrefix) {
            if word == 0x00 {
                word = ESCAPE_PREFIX;
            } else if word == 0x01 {
                word = SYN;
            } else {
                log::warn!("detected invalid escape sequence");
                self.reset();
            }
        } else if word == ESCAPE_PREFIX {
            self.flags.add(Flag::WasEscapePrefix);
            return Ok(ProcessResult::None);
        }

        log::debug!("processing 0x{word:X}");

        match &mut self.state {
            State::Idle => unreachable!(),
            State::AcquiringLock => {
                if word == msg.src {
                    let expect = self.send_data(transmit, msg)?;
                    self.state = State::DataLoopback { expect };
                    sleep(Duration::from_millis(10));
                } else {
                    let prio_class = word & 0x0F;
                    let own_prio = msg.src & 0x0F;

                    if prio_class == own_prio {
                        // instantly try again
                        self.state = State::Idle;
                    } else {
                        log::warn!("Failed to acquire lock");
                        self.fairness_counter = 2;
                        sleep(Duration::from_millis(20));
                        self.state = State::Idle;
                    }
                }
            }
            State::DataLoopback { expect } => {
                log::debug!("loopback: 0x{word:X}");
                *expect -= 1;
                if *expect == 0 {
                    self.state = State::AwaitingAck;
                }
            }
            State::AwaitingAck => match word {
                ACK_OK => {
                    if msg.expect_reply {
                        self.state = State::AwaitingLen;
                    } else {
                        self.success(transmit)?;

                        return Ok(ProcessResult::AckOk);
                    }
                }
                x => {
                    log::warn!("telegram not acknowledged");
                    if x != ACK_ERR {
                        log::warn!("expected ack, got non-ack byte: 0x{word:X}");
                    }
                    self.reset();

                    return Ok(ProcessResult::AckErr);
                }
            },
            State::AwaitingLen => {
                if word > 16 {
                    log::warn!("got slave response with len > 16");
                    self.reset();
                    // TODO: how to handle?
                    sleep(Duration::from_millis(10));
                }

                // TODO: handle 0 len case?

                let mut crc = Crc::new(self.crc_poly_telegram);
                crc.add(word);

                self.state = State::ReceivingData {
                    buf: [0; 16],
                    cursor: 0,
                    total: word,
                    crc,
                };
            }
            State::ReceivingData {
                buf,
                cursor,
                total,
                crc,
            } => {
                buf[*cursor as usize] = word;
                *cursor += 1;

                if *cursor >= *total {
                    self.state = State::AwaitingCrc {
                        buf: *buf,
                        len: *total,
                        crc: crc.calc_crc(),
                    };
                }
            }
            State::AwaitingCrc { crc, buf, len } => {
                let crc_should = *crc;

                if word == crc_should {
                    transmit.transmit_raw(&[ACK_OK])?;
                    sleep(Duration::from_millis(15));
                    let res = ProcessResult::Reply {
                        buf: *buf,
                        len: *len,
                    };
                    self.success(transmit)?;

                    return Ok(res);
                } else {
                    log::warn!("got crc 0x{word:X}, expected 0x{crc_should:X}");
                    transmit.transmit_raw(&[ACK_ERR])?;
                    sleep(Duration::from_millis(15));
                    self.success(transmit)?;
                    return Ok(ProcessResult::CrcError);
                }
            }
        }

        Ok(ProcessResult::None)
    }

    fn send_data<T: Transmit>(
        &mut self,
        transmit: &mut T,
        msg: &Telegram<'_>,
    ) -> Result<u8, T::Error> {
        let mut tele_crc = Crc::new(self.crc_poly_telegram);
        tele_crc.add(msg.src);
        let mut counter = 0;

        let svc = msg.service.to_le_bytes();
        let data = msg.data;
        let len = data.len() as u8;
        counter += transmit.transmit_encode_with_crc(
            &[msg.dest, svc[0], svc[1], len + msg.needs_data_crc as u8],
            &mut tele_crc,
        )?;
        // TODO: how to handle empty data?
        if msg.needs_data_crc {
            let mut data_crc = Crc::new(self.crc_poly_data);
            // TODO: do we have to use encoded bytes here?
            data_crc.add_multiple(data);
            let data_crc = data_crc.calc_crc();
            counter += transmit.transmit_encode_with_crc(&[data_crc], &mut tele_crc)?;
        }
        counter += transmit.transmit_encode_with_crc(data, &mut tele_crc)?;
        counter += transmit.transmit_encode(&[tele_crc.calc_crc()])?;

        Ok(counter)
    }

    fn is_allowed_to_lock(&self) -> bool {
        self.fairness_counter == 0
    }

    fn success<T: Transmit>(&mut self, transmit: &mut T) -> Result<(), T::Error> {
        transmit.transmit_syn()?;
        self.flags.clear();
        self.state.reset();
        self.fairness_counter = FAIRNESS_MAX;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.flags.clear();
        self.state.reset();
    }
}

enum State {
    Idle,
    AcquiringLock,
    DataLoopback {
        /// the number of bytes we expect to get echoed back (is counted down)
        expect: u8,
    },
    AwaitingAck,
    AwaitingLen,
    ReceivingData {
        buf: [u8; 16],
        cursor: u8,
        total: u8,
        crc: Crc,
    },
    AwaitingCrc {
        crc: u8,
        buf: [u8; 16],
        len: u8,
    },
}

impl State {
    pub fn has_bus_lock(&self) -> bool {
        !matches!(self, State::Idle)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, State::Idle)
    }

    pub fn is_acquiring(&self) -> bool {
        matches!(self, State::AcquiringLock)
    }

    pub fn is_awaiting(&self) -> bool {
        matches!(
            self,
            Self::AwaitingAck
                | Self::AwaitingCrc { .. }
                | Self::AwaitingLen
                | Self::ReceivingData { .. }
        )
    }

    pub fn reset(&mut self) {
        *self = State::Idle;
    }
}

#[derive(Clone, Debug)]
pub enum ProcessResult {
    None,
    AckOk,
    AckErr,
    Timeout,
    CrcError,
    Reply { buf: [u8; 16], len: u8 },
}

#[derive(Clone, Debug, Default)]
struct Flags {
    pub flags: u8,
}

impl Flags {
    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn add(&mut self, flag: Flag) {
        self.flags |= 1 << flag as u8;
    }

    pub fn check_remove(&mut self, flag: Flag) -> bool {
        let was_set = self.has(flag);
        self.remove(flag);

        was_set
    }

    pub fn remove(&mut self, flag: Flag) {
        self.flags &= !(1 << flag as u8);
    }

    pub fn has(&mut self, flag: Flag) -> bool {
        (self.flags & (1 << flag as u8)) != 0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Flag {
    /// Last byte we received was 0x9F prefix
    WasEscapePrefix = 0,
}

pub trait Transmit {
    type Error: Debug;

    fn transmit_raw(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    fn transmit_syn(&mut self) -> Result<(), Self::Error> {
        self.transmit_raw(&[SYN])
    }

    #[doc(hidden)]
    fn _transmit_count(&mut self, bytes: &[u8]) -> Result<u8, Self::Error> {
        self.transmit_raw(bytes).map(|_| bytes.len() as u8)
    }
}

trait TransmitExt: Transmit {
    fn transmit_encode_with_options(
        &mut self,
        bytes: &[u8],
        crc: Option<&mut Crc>,
    ) -> Result<u8, Self::Error>;

    fn transmit_encode_with_crc(&mut self, bytes: &[u8], crc: &mut Crc) -> Result<u8, Self::Error> {
        self.transmit_encode_with_options(bytes, Some(crc))
    }

    fn transmit_encode(&mut self, bytes: &[u8]) -> Result<u8, Self::Error> {
        self.transmit_encode_with_options(bytes, None)
    }
}

impl<T> TransmitExt for T
where
    T: Transmit,
{
    fn transmit_encode_with_options(
        &mut self,
        bytes: &[u8],
        mut crc: Option<&mut Crc>,
    ) -> Result<u8, Self::Error> {
        // the (exclusive) index up to which we have tranmitted
        let mut last_transmit = 0;
        // count the number of bytes we transmit
        let mut byte_counter = 0u8;

        let mut transmit = |bytes: &'_ [u8]| {
            if let Some(crc) = &mut crc {
                crc.add_multiple(bytes);
            }
            byte_counter += self._transmit_count(bytes)?;

            Ok(())
        };

        for (i, &byte) in bytes.iter().enumerate() {
            let i = i as u8;
            if byte == SYN || byte == ESCAPE_PREFIX {
                if i != last_transmit {
                    transmit(&bytes[last_transmit as usize..i as usize])?;
                    last_transmit = i + 1;
                }

                let escape_code = if byte == SYN { 0x01 } else { 0x00 };
                transmit(&[ESCAPE_PREFIX, escape_code])?;
            }
        }

        if bytes.len() as u8 != last_transmit {
            transmit(&bytes[last_transmit as usize..])?;
        }

        Ok(byte_counter)
    }
}
