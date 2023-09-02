#![no_std]
#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! Here is an example how once could integrate this library:
//!
//! ```rust,no_run
#![doc = include_str!("../examples/integration.rs")]
//! ```

use core::{fmt::Debug, time::Duration};

pub use crc::Crc;
pub use telegram::{Buffer, MasterTelegram, Telegram, TelegramFlag, TelegramFlags};

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
            state: State::Start,
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
        next_msg: Option<&MasterTelegram>,
    ) -> Result<ProcessResult, T::Error> {
        if word == SYN {
            let was_timeout = self.state.master_is_awaiting();

            if self.process_syn() && next_msg.is_some() {
                #[allow(clippy::unnecessary_unwrap)]
                let msg = next_msg.unwrap();
                let src = msg.telegram.src;

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
        } else if self.state.is_start() {
            // do nothing
            Ok(ProcessResult::None)
        } else {
            // we are not in idle state, there must be a msg
            let msg = next_msg.unwrap();
            self.process_slow(word, transmit, sleep, msg)
        }
    }

    /// Reply to a received master-slave telegram
    pub fn reply_as_slave<T: Transmit>(
        &mut self,
        data: &[u8],
        transmit: &mut T,
        _token: RequestToken,
    ) -> Result<(), T::Error> {
        if data.len() > 16 {
            log::warn!("replying with more than 16 bytes");
        }

        transmit.transmit_encode(&[ACK_OK, data.len() as u8])?;

        let mut crc = Crc::new(self.crc_poly_telegram);
        transmit.transmit_encode_with_crc(data, &mut crc)?;
        transmit.transmit_encode(&[crc.calc_crc()])?;

        self.state = State::Replied;

        Ok(())
    }

    /// Returns `true` if we may lock the bus
    fn process_syn(&mut self) -> bool {
        if self.state.has_bus_lock() {
            log::warn!("unexpected SYN while holding bus lock");
            self.reset_syn();
        } else if self.state.is_acquiring() {
            log::warn!("unexpected double SYN prevented lock");
            self.reset_syn();
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
        msg: &MasterTelegram,
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
                self.reset_wait_syn();

                return Ok(ProcessResult::None);
            }
        } else if word == ESCAPE_PREFIX {
            self.flags.add(Flag::WasEscapePrefix);
            return Ok(ProcessResult::None);
        }

        match &mut self.state {
            State::Unknown => {
                // just wait for next SYN
            }
            State::Start => {
                // we are not acquiring the lock, so we just listen
                self.state = State::GotSrc { src: word };
            }
            State::AcquiringLock => {
                if word == msg.telegram.src {
                    let expect = self.send_data(transmit, msg)?;
                    self.state = State::DataLoopback { expect };
                    sleep(Duration::from_millis(10));
                } else {
                    let prio_class = word & 0x0F;
                    let own_prio = msg.telegram.src & 0x0F;

                    if prio_class == own_prio {
                        // instantly try again on next SYN
                        self.state = State::Unknown;
                    } else {
                        log::warn!("Failed to acquire lock");
                        self.fairness_counter = 2;
                        sleep(Duration::from_millis(20));
                        self.state = State::GotSrc { src: word };
                    }
                }
            }
            State::DataLoopback { expect } => {
                *expect -= 1;
                if *expect == 0 {
                    self.state = State::AwaitingAck;
                }
                sleep(Duration::from_millis(0));
            }
            State::AwaitingAck => match word {
                ACK_OK => {
                    if msg.flags & TelegramFlag::ExpectReply {
                        self.state = State::AwaitingLen;
                    } else {
                        self.success(transmit)?;

                        return Ok(ProcessResult::MasterAckOk);
                    }
                }
                x => {
                    log::warn!("telegram not acknowledged");
                    if x != ACK_ERR {
                        log::warn!("expected ack, got non-ack byte: 0x{word:X}");
                    }
                    self.reset_wait_syn();

                    return Ok(ProcessResult::MasterAckErr);
                }
            },
            State::AwaitingLen => {
                if word > 16 {
                    log::warn!("got slave response with len > 16");
                    self.reset_wait_syn();
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
                        data: Buffer::from_parts(*buf, *len),
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
            State::GotSrc { src } => {
                self.state = State::GotDst {
                    src: *src,
                    dst: word,
                }
            }
            State::GotDst { src, dst } => {
                self.state = State::GotSvc1 {
                    src: *src,
                    dst: *dst,
                    svc1: word,
                }
            }
            State::GotSvc1 { src, dst, svc1 } => {
                self.state = State::GotSvc2 {
                    src: *src,
                    dst: *dst,
                    svc: u16::from_le_bytes([*svc1, word]),
                }
            }
            State::GotSvc2 { src, dst, svc } => {
                self.state = State::GettingBytes {
                    src: *src,
                    dst: *dst,
                    svc: *svc,
                    len: word,
                    cursor: 0,
                    buf: [0; 16],
                }
            }
            State::GettingBytes {
                src,
                dst,
                svc,
                len,
                cursor,
                buf,
            } => {
                buf[*cursor as usize] = word;
                *cursor += 1;

                if *cursor >= *len {
                    let res = ProcessResult::Request {
                        telegram: Telegram {
                            src: *src,
                            dest: *dst,
                            service: *svc,
                            data: Buffer::from_parts(*buf, *len),
                        },
                        token: RequestToken { _priv: () },
                    };
                    self.state = State::GotTelegram;
                    return Ok(res);
                }
            }
            State::GotTelegram => {
                // TODO: could sniff here
                self.state = State::Unknown;
            }
            State::Replied => match word {
                ACK_OK => {
                    self.reset_wait_syn();
                    return Ok(ProcessResult::SlaveAckOk);
                }
                x => {
                    log::warn!("reply not acknowledged");
                    if x != ACK_ERR {
                        log::warn!("expected ack, got non-ack byte: 0x{word:X}");
                    }
                    self.reset_wait_syn();

                    return Ok(ProcessResult::SlaveAckErr);
                }
            },
            State::ReplyLoopback { expect } => todo!(),
        }

        Ok(ProcessResult::None)
    }

    fn send_data<T: Transmit>(
        &mut self,
        transmit: &mut T,
        msg: &MasterTelegram,
    ) -> Result<u8, T::Error> {
        let mut tele_crc = Crc::new(self.crc_poly_telegram);
        tele_crc.add(msg.telegram.src);
        let mut counter = 0;

        let svc = msg.telegram.service.to_le_bytes();
        let data = msg.telegram.data.as_bytes();
        let len = data.len() as u8;
        counter += transmit.transmit_encode_with_crc(
            &[
                msg.telegram.dest,
                svc[0],
                svc[1],
                len + (msg.flags & TelegramFlag::NeedsDataCrc) as u8,
            ],
            &mut tele_crc,
        )?;
        // TODO: how to handle empty data?
        if msg.flags & TelegramFlag::NeedsDataCrc {
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
        // we do not reset to syn state, because we wait until we receive it back
        self.state.reset_unknown();
        self.fairness_counter = FAIRNESS_MAX;

        Ok(())
    }

    pub fn reset_wait_syn(&mut self) {
        self.flags.clear();
        self.state.reset_unknown();
    }

    /// this should be called if we receive SYN
    pub fn reset_syn(&mut self) {
        self.flags.clear();
        self.state.reset_syn();
    }
}

enum State {
    /// We are waiting for next SYN
    Unknown,
    /// We just got SYN
    Start,
    // === master states ===
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
    // === slave states ===
    GotSrc {
        src: u8,
    },
    GotDst {
        src: u8,
        dst: u8,
    },
    GotSvc1 {
        src: u8,
        dst: u8,
        svc1: u8,
    },
    GotSvc2 {
        src: u8,
        dst: u8,
        svc: u16,
    },
    GettingBytes {
        src: u8,
        dst: u8,
        svc: u16,
        len: u8,
        cursor: u8,
        buf: [u8; 16],
    },
    /// The master half of master-slave was received.
    GotTelegram,
    /// We are waiting to get ACK back.
    ReplyLoopback {
        /// the number of bytes we expect to get echoed back (is counted down)
        expect: u8,
    },
    /// We are waiting to get ACK back.
    Replied,
}

impl State {
    pub fn has_bus_lock(&self) -> bool {
        !matches!(self, State::Start)
    }

    pub fn is_start(&self) -> bool {
        matches!(self, State::Start)
    }

    pub fn is_acquiring(&self) -> bool {
        matches!(self, State::AcquiringLock)
    }

    pub fn master_is_awaiting(&self) -> bool {
        matches!(
            self,
            Self::AwaitingAck
                | Self::AwaitingCrc { .. }
                | Self::AwaitingLen
                | Self::ReceivingData { .. }
        )
    }

    pub fn reset_unknown(&mut self) {
        *self = State::Unknown;
    }

    pub fn reset_syn(&mut self) {
        *self = State::Start;
    }
}

#[derive(Debug)]
pub enum ProcessResult {
    None,
    /// We replied as slave, master acknowledged
    SlaveAckOk,
    /// We replied as slave, master did not acknowledge
    SlaveAckErr,
    /// We sent master-slave, slave acknowledged
    MasterAckOk,
    /// We sent master-slave, slave did not acknowledge
    MasterAckErr,
    /// Expected recipient to send, but AUTO-SYN occurred
    Timeout,
    /// CRC check of reply failed
    CrcError,
    /// Master-slave request
    Request {
        telegram: Telegram,
        token: RequestToken,
    },
    /// Slave sent reply
    Reply {
        data: Buffer,
    },
}

impl ProcessResult {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn as_request(&self) -> Option<&Telegram> {
        if let Self::Request { telegram, .. } = self {
            Some(telegram)
        } else {
            None
        }
    }

    pub fn as_reply(&self) -> Option<&[u8]> {
        if let Self::Reply { data } = self {
            Some(data.as_bytes())
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RequestToken {
    _priv: (),
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
