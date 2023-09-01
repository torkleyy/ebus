#![no_std]

use core::{fmt::Debug, marker::PhantomData, time::Duration};

use telegram::Telegram;

mod crc;
mod telegram;

const SYN: u8 = 0xAA;
const ACK_OK: u8 = 0x00;
const ACK_ERR: u8 = 0xFF;
const ESCAPE_PREFIX: u8 = 0xA9;

pub trait EbusConfig {
    const POLY_TELEGRAM: u8;
    const POLY_DATA: u8;
}

pub struct EbusDriver<C> {
    pub marker: PhantomData<C>,
    pub state: EbusState,
}

impl<C> EbusDriver<C> {
    pub fn new() -> Self {
        EbusDriver {
            marker: Default::default(),
            state: Default::default(),
        }
    }

    pub fn process<E>(
        &mut self,
        mut word: u8,
        mut transmit_byte: impl FnMut(&[u8]) -> Result<(), E>,
        sleep: impl Fn(Duration),
        next_msg: Option<&Telegram<'_>>,
    ) -> Result<ProcessResult, E> {
        let mut transmit = |bytes: &[u8]| {
            // the (exclusive) index up to which we have tranmitted
            let mut last_transmit = 0;

            for (i, &byte) in bytes.iter().enumerate() {
                if byte == SYN || byte == ESCAPE_PREFIX {
                    if i != last_transmit {
                        transmit_byte(&bytes[last_transmit..i])?;
                        last_transmit = i + 1;
                    }

                    let escape_code = if byte == SYN { 0x01 } else { 0x00 };
                    transmit_byte(&[ESCAPE_PREFIX, escape_code])?;
                }
            }

            Ok(())
        };

        if word == SYN {
            if self.state.process_syn() && next_msg.is_some() {
                let msg = next_msg.unwrap();
                let src = msg.src;

                transmit(&[src])?;
                self.state.add(Flag::LockStarted);
            } else {
                sleep(Duration::from_millis(10));
            }

            return Ok(ProcessResult::Idle);
        }

        if self.state.check_remove(Flag::WasEscapePrefix) {
            if word == 0x00 {
                word = ESCAPE_PREFIX;
            } else if word == 0x01 {
                word = SYN;
            } else {
                log::warn!("detected invalid escape sequence");
                self.state.reset();
            }
        }

        if self.state.has(Flag::LockStarted) {
            let msg = next_msg.unwrap();
            if word == msg.src {
                self.state.remove(Flag::LockStarted);
                self.state.add(Flag::BusLock);
            } else {
                log::warn!("Failed to acquire lock");
                self.state.lock_counter = 2;
                sleep(Duration::from_millis(20));
            }
        } else if self.state.has(Flag::BusLock) {
            self.handle_telegram(word, transmit_byte, sleep, next_msg.unwrap())?;
        }

        Ok(ProcessResult::Idle)
    }

    fn handle_telegram<E>(
        &mut self,
        mut word: u8,
        mut transmit_byte: impl FnMut(&[u8]) -> Result<(), E>,
        sleep: impl Fn(Duration),
        msg: &Telegram<'_>,
    ) -> Result<(), E> {
        Ok(())
    }
}

pub enum ProcessResult {
    Idle,
}

#[derive(Clone, Debug, Default)]
pub struct EbusState {
    pub flags: u8,
    pub lock_counter: u8,
}

impl EbusState {
    /// Returns `true` if we may lock the bus
    pub fn process_syn(&mut self) -> bool {
        if self.has(Flag::BusLock) {
            log::warn!("unexpected SYN while holding bus lock");
            self.reset();
        } else if self.has(Flag::LockStarted) {
            log::warn!("unexpected double SYN prevented lock");
            self.reset();
        } else {
            if self.is_locked() {
                self.lock_counter -= 1;
            } else {
                return true;
            }
        }

        false
    }

    fn is_locked(&self) -> bool {
        self.lock_counter > 0
    }

    pub fn reset(&mut self) {
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
        (self.flags & (flag as u8)) != 0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Flag {
    /// Are we allowed to send?
    BusLock = 0,
    /// Last byte we received was 0x9F prefix
    WasEscapePrefix = 1,
    /// Source address sent to acquire lock
    LockStarted = 2,
}
