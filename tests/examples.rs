use std::time::Duration;

use ebus::{EbusDriver, Telegram, Transmit};

const EXAMPLE_EXCHANGE1: &[u8] = &[0xAAu8, 0x00]; // TODO

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

    let mut driver = EbusDriver::new(123, 0x9B, 0x5C);

    driver
        .process(0xAA, &mut transmitter, sleep, Some(&msg))
        .unwrap();

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
        .process(0xB1, &mut transmitter, sleep, Some(&msg))
        .unwrap();
    match res {
        ebus::ProcessResult::Reply { buf, len } => {
            let buf = &buf[..len as usize];

            assert_eq!(EXAMPLE_EXCHANGE1, buf);
        }
        other => unreachable!("{:?}", other),
    }
}
