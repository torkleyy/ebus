mod helper;

use energy_bus::ProcessResult;
use helper::{example1, AutoLoopback};

#[test]
fn example1_ok() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

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
