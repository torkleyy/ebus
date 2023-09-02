use std::time::Duration;

use energy_bus::{EbusDriver, ProcessResult, Telegram, Transmit};

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

#[test]
fn test_example1() {
    let mut transmitter = TestTransmitter { sent: vec![] };
    let msg = Telegram {
        src: 0xFF,
        dest: 0x51,
        service: 0x5022,
        data: &[15, 0],
        needs_data_crc: true,
        expect_reply: true,
    };

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
    for &reply_byte in &[0x00, 0x02, 0xA9, 0x00, 0xDA] {
        driver
            .process(reply_byte, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }

    let res = driver
        .process(0x82, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    match res {
        ProcessResult::Reply { buf, len } => {
            let buf = &buf[..len as usize];

            assert_eq!(&[0xA9, 0xDA], buf);
        }
        other => unreachable!("{:?}", other),
    }
}

#[test]
fn test_example1_timeout() {
    let mut transmitter = TestTransmitter { sent: vec![] };
    let msg = Telegram {
        src: 0xFF,
        dest: 0x51,
        service: 0x5022,
        data: &[15, 0],
        needs_data_crc: true,
        expect_reply: true,
    };

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
    for &reply_byte in &[0x00, 0x02, 0xA9, 0x00, 0xDA] {
        driver
            .process(reply_byte, &mut transmitter, sleep, Some(&msg))
            .unwrap();
    }

    let res = driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    match res {
        ProcessResult::Timeout => {}
        other => unreachable!("{:?}", other),
    }
}
