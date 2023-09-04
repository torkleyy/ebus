#![allow(dead_code)]

use std::{iter::once, time::Duration};

use energy_bus::{
    Buffer, Crc, EbusDriver, MasterTelegram, ProcessResult, RequestToken, Telegram, TelegramFlag,
    Transmit,
};

#[derive(Default)]
struct LoopbackTransmitter {
    sent: Vec<u8>,
    loopback: Vec<u8>,
}

impl Transmit for LoopbackTransmitter {
    type Error = ();

    fn clear_buffer(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transmit_raw(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.sent.extend_from_slice(bytes);
        self.loopback.extend_from_slice(bytes);

        Ok(())
    }
}

pub struct AutoLoopback {
    driver: EbusDriver,
    transmit: LoopbackTransmitter,
}

fn sleep(_: Duration) {}

impl AutoLoopback {
    pub fn new() -> Self {
        let mut this = AutoLoopback {
            driver: EbusDriver::new(Duration::from_micros(123), 0x9B, 0x5C),
            transmit: Default::default(),
        };

        for _ in 0..50 {
            this.process(0xAA, None);
        }

        this
    }

    pub fn process(&mut self, byte: u8, msg: Option<&MasterTelegram>) -> Vec<ProcessResult> {
        let mut results = vec![self
            .driver
            .process(byte, &mut self.transmit, sleep, msg)
            .unwrap()];

        results.extend(self.process_bus(msg));
        results
    }

    pub fn process_bus(&mut self, msg: Option<&MasterTelegram>) -> Vec<ProcessResult> {
        let mut results = vec![];

        for _ in 0..500 {
            if self.transmit.loopback.is_empty() {
                return results;
            }

            let iter = self
                .transmit
                .loopback
                .drain(..)
                .collect::<Vec<_>>()
                .into_iter()
                .map(|byte| {
                    // caveat: we are not dropping our message, even when it was already sent / timed out
                    self.driver
                        .process(byte, &mut self.transmit, sleep, msg)
                        .unwrap()
                });
            results.extend(iter);
        }

        panic!("infinite loop detected");
    }

    pub fn last_sent(&self) -> Option<u8> {
        self.transmit.sent.last().cloned()
    }

    pub fn process_multiple(
        &mut self,
        bytes: &[u8],
        msg: Option<&MasterTelegram>,
    ) -> Vec<Vec<ProcessResult>> {
        bytes
            .iter()
            .cloned()
            .map(|byte| self.process(byte, msg))
            .collect()
    }

    pub fn send_external_bytes(&mut self, bytes: &[u8]) {
        self.transmit.loopback.extend_from_slice(bytes);
    }

    pub fn send_external_msg(&mut self, tele: &MasterTelegram) {
        self.send_external_bytes(&[0xAA]);

        let msg = &tele.telegram;

        let mut v: Vec<u8> = [
            msg.src,
            msg.dest,
            (msg.service & 0xFF) as u8,
            (msg.service >> 8) as u8,
        ]
        .into_iter()
        .chain(once(
            msg.data.as_bytes().len() as u8 + (tele.flags & TelegramFlag::NeedsDataCrc) as u8,
        ))
        .chain(if tele.flags & TelegramFlag::NeedsDataCrc {
            Some(Crc::new(0x5C).add_multiple(msg.data.as_bytes()).calc_crc())
        } else {
            None
        })
        .chain(msg.data.as_bytes().iter().cloned())
        .flat_map(|byte| escape(byte))
        .collect();

        let crc = Crc::new(0x9B).add_multiple(&v).calc_crc();
        v.extend(escape(crc));

        v.iter().for_each(|b| print!("0x{b:X} "));
        println!();

        self.send_external_bytes(&v);
    }

    pub fn reply_as_slave(&mut self, data: &[u8], token: RequestToken) {
        self.driver
            .reply_as_slave(data, &mut self.transmit, token)
            .unwrap();
    }
}

pub fn escape(byte: u8) -> Vec<u8> {
    match byte {
        0xA9 => vec![0xA9, 0x0],
        0xAA => vec![0xA9, 0x1],
        byte => vec![byte],
    }
}

pub fn example1() -> MasterTelegram {
    MasterTelegram {
        telegram: Telegram {
            src: 0xFF,
            dest: 0x51,
            service: 0x5022,
            data: Buffer::from_slice(&[15, 0]),
        },
        flags: TelegramFlag::NeedsDataCrc | TelegramFlag::ExpectReply,
    }
}
