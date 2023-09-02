use std::time::Duration;

use energy_bus::{
    Buffer, Crc, EbusDriver, MasterTelegram, ProcessResult, Telegram, TelegramFlag, Transmit,
};

struct TestTransmitter {
    sent: Vec<u8>,
}

impl Transmit for TestTransmitter {
    type Error = ();

    fn transmit_raw(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.sent.extend_from_slice(bytes);

        Ok(())
    }
}

fn sleep(_: Duration) {}

fn test_send_and_reply_raw(tel: MasterTelegram, reply: &[u8]) -> ProcessResult {
    let mut transmitter = TestTransmitter { sent: vec![] };
    let msg = tel;

    let mut driver = EbusDriver::new(Duration::from_micros(123), 0x9B, 0x5C);

    // deal with fairness counter
    for _ in 0..51 {
        driver
            .process(0xAA, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }

    loop {
        if transmitter.sent.is_empty() {
            break;
        }

        let word = transmitter.sent.remove(0);
        driver
            .process(word, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }

    // write reply
    for &reply_byte in reply {
        driver
            .process(reply_byte, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }
    let crc = Crc::new(0x9B).add_multiple(reply).calc_crc();
    driver
        .process(crc, &mut transmitter, sleep, Some(&msg))
        .unwrap();

    driver
        .process(0x82, &mut transmitter, sleep, Some(&msg))
        .unwrap()
}

fn test_send_and_reply(tel: MasterTelegram, reply: &[u8]) -> ProcessResult {
    let mut reply_raw = vec![0x00, reply.len() as u8];
    for &byte in reply {
        match byte {
            0xA9 => {
                reply_raw.push(0xA9);
                reply_raw.push(0x00);
            }
            0xAA => {
                reply_raw.push(0xA9);
                reply_raw.push(0x01);
            }
            byte => {
                reply_raw.push(byte);
            }
        }
    }

    test_send_and_reply_raw(tel, &reply_raw)
}

#[test]
fn test_example1() {
    let res = test_send_and_reply(
        MasterTelegram {
            telegram: Telegram {
                src: 0xFF,
                dest: 0x51,
                service: 0x5022,
                data: Buffer::from_slice(&[15, 0]),
            },
            flags: TelegramFlag::NeedsDataCrc | TelegramFlag::ExpectReply,
        },
        &[0xA9, 0xDA],
    );
    assert!(matches!(
        res,
        ProcessResult::Reply {
            buf: [0xA9, 0xDA, ..],
            len: 2
        }
    ));
}

#[test]
fn test_example1_timeout() {
    let res = test_send_and_reply_raw(
        MasterTelegram {
            telegram: Telegram {
                src: 0xFF,
                dest: 0x51,
                service: 0x5022,
                data: Buffer::from_slice(&[15, 0]),
            },
            flags: TelegramFlag::NeedsDataCrc | TelegramFlag::ExpectReply,
        },
        &[0x00, 0x02, 0xA9, 0x00, 0xDA], // missing CRC
    );

    assert!(matches!(res, ProcessResult::Timeout));
}
