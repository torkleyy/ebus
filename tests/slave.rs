mod helper;

use ebus::{Buffer, MasterTelegram, ProcessResult, Telegram, TelegramFlags};
use helper::{example1, AutoLoopback};

#[test]
fn example1_ok() {
    let mut d = AutoLoopback::new();
    let msg = example1();

    d.send_external_msg(&msg);
    let mut results = d.process_bus(None);

    match results.drain(..).last().unwrap() {
        ProcessResult::Request { telegram, token } => {
            assert_eq!(telegram.src, 0xFF);
            d.reply_as_slave(&[0xDE, 0xAD, 0xBE, 0xEF], token)
        }
        other => panic!("{:?}", other),
    }

    let crc = d.last_sent().unwrap();

    d.process_bus(None);

    assert_eq!(d.last_sent(), Some(crc));

    // ACK
    let res = d.process(0x00, None);
    assert!(matches!(&res[..], [ProcessResult::SlaveAckOk]));
}

#[test]
fn time_program() {
    let mut d = AutoLoopback::new();
    let msg = MasterTelegram {
        telegram: Telegram {
            src: 0xFF,
            dest: 0x85,
            service: 0x03F1,
            data: Buffer::from_slice(&[0x04, 0x01, 0x00, 0x01]),
        },
        flags: TelegramFlags::none(),
    };

    let mut results = d.process(0xAA, Some(&msg));

    match results.drain(..).last().unwrap() {
        ProcessResult::None => {}
        other => panic!("{:?}", other),
    }

    assert_eq!(
        d.process(0, Some(&msg)).drain(..).next().unwrap(),
        ProcessResult::MasterAckOk
    );
    log::info!("processed ack");

    d.send_external_msg(&MasterTelegram {
        telegram: Telegram {
            src: 0x30,
            dest: 0xFF,
            service: 0x03F2,
            data: Buffer::from_slice(&[
                0x2C, 0x01, 0xEC, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
        },
        flags: TelegramFlags::none(),
    });
    let mut results = d.process_bus(None);
    match results.drain(..).last().unwrap() {
        ProcessResult::Request { telegram: _, token } => {
            d.reply_ack(token);
        }
        other => panic!("{:?}", other),
    }
}
