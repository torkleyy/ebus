use std::time::Duration;

use energy_bus::{
    Buffer, Crc, EbusDriver, MasterTelegram, ProcessResult, Telegram, TelegramFlag, Transmit,
};

use crate::helper::AutoLoopback;

mod helper;

struct TestTransmitter {
    sent: Vec<u8>,
}

impl Transmit for TestTransmitter {
    type Error = ();

    fn clear_buffer(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

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
    let mut res = ProcessResult::None;
    for &reply_byte in reply {
        res = driver
            .process(reply_byte, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }

    res
}

fn test_send_and_reply(tel: MasterTelegram, reply: &[u8]) -> ProcessResult {
    let mut composed = vec![0x00, reply.len() as u8];
    composed.extend_from_slice(reply);

    let mut reply_raw = vec![];
    for byte in composed {
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

    let crc = Crc::new(0x9B).add_multiple(&reply_raw).calc_crc();
    assert_ne!(crc, 0xAA);
    assert_ne!(crc, 0xA9);
    reply_raw.push(crc);

    test_send_and_reply_raw(tel, &reply_raw)
}

fn example1() -> MasterTelegram {
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

#[test]
fn test_example1() {
    let res = test_send_and_reply(example1(), &[0xA9, 0xDA]);
    assert!(matches!(res.as_reply().unwrap(), [0xA9, 0xDA]));
}

#[test]
fn test_example1_auto_lb() {
    use ProcessResult::*;

    let mut d = AutoLoopback::new();
    let msg = example1();
    let res = d.process(0xAA, Some(&msg));
    assert_eq!(res.len(), 10);
    let res = d.process_multiple(&[0x00, 0x02, 0xA9, 0x00, 0xDA, 0x82], Some(&msg));
    assert!(
        matches!(dbg!(&res[5][..]), [.., Reply { data }, None, None] if data.as_bytes() == &[0xA9, 0xDA])
    );
}

#[test]
fn test_example1_timeout() {
    let res = test_send_and_reply_raw(
        example1(),
        &[0x00, 0x02, 0xA9, 0x00, 0xDA, 0xAA], // missing CRC
    );

    assert!(matches!(res, ProcessResult::Timeout));
}

#[test]
fn test_master_retry_lock() {
    let mut transmitter = TestTransmitter { sent: vec![] };
    let msg = example1();

    let mut driver = EbusDriver::new(Duration::from_micros(123), 0x9B, 0x5C);
    for _ in 0..50 {
        driver.process(0xAA, &mut transmitter, sleep, None).unwrap();
    }
    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    let res = driver
        .process(0x0F, &mut transmitter, sleep, Some(&msg))
        .unwrap();

    assert!(matches!(res, ProcessResult::None));

    transmitter.sent.clear();
    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();

    assert!(transmitter.sent.is_empty());

    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    assert_eq!(*transmitter.sent.last().unwrap(), msg.telegram.src);
}

#[test]
fn interrupt_lock() {
    let mut transmitter = TestTransmitter { sent: vec![] };
    let msg = example1();

    let mut driver = EbusDriver::new(Duration::from_micros(123), 0x9B, 0x5C);
    for _ in 0..50 {
        driver.process(0xAA, &mut transmitter, sleep, None).unwrap();
    }
    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    driver
        .process(msg.telegram.src, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    driver
        .process(0xFF, &mut transmitter, sleep, Some(&msg))
        .unwrap();

    let len = transmitter.sent.len();
    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();

    assert_eq!(transmitter.sent.len(), len);
}
