use core::time::Duration;

use energy_bus::{EbusDriver, MasterTelegram, Transmit};

// Depends on hardware and latency. Right value must be chosen to ensure
// layering of the first byte after `SYN`
const ARBITRATION_DELAY: Duration = Duration::from_micros(540);

const CRC_POLYNOM_TELEGRAM: u8 = 0x9B;
const CRC_POLYNOM_DATA: u8 = 0x5C;

fn wait_for_next_byte() -> u8 {
    // this should await a byte from UART

    0x00
}

struct UartTxDriver;

struct Transmitter(UartTxDriver);

impl Transmit for Transmitter {
    type Error = ();

    fn clear_buffer(&mut self) -> Result<(), Self::Error> {
        // clear UART tx buffer

        Ok(())
    }

    fn transmit_raw(&mut self, _bytes: &[u8]) -> Result<(), Self::Error> {
        // write bytes to UART

        Ok(())
    }
}

fn sleep(_d: Duration) {
    // This function is called by the ebus driver to
    // correctly layer its own source address with others'
    // in order to lock the bus.
    // It is only called with the arbitration_delay passed
    // to `EbusDriver::new`.
}

fn poll_next_msg() -> Option<MasterTelegram> {
    // poll your message queue here
    None
}

fn main() {
    // create some sort of queue for messages to be sent,
    // or a channel
    // give tx to application so it can queue messages

    let mut uart = Transmitter(UartTxDriver);
    let mut driver = EbusDriver::new(ARBITRATION_DELAY, CRC_POLYNOM_TELEGRAM, CRC_POLYNOM_DATA);

    let mut msg = None;
    loop {
        // Here, we block on the receival of a byte which is not ideal.
        // Depending on your device and architecture, you should use interrupts or
        // low latency async code.
        msg = msg.or_else(|| poll_next_msg());
        let byte = wait_for_next_byte();

        match driver
            .process(byte, &mut uart, sleep, msg.as_ref())
            .expect("handle uart error")
        {
            energy_bus::ProcessResult::None => {}
            energy_bus::ProcessResult::MasterAckOk => {
                // successfully sent message with no expected reply
                msg = None; // remove message from queue
            }
            energy_bus::ProcessResult::MasterAckErr => {
                // recipient replied ACK_ERR
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::Timeout => {
                // recipient did not reply within AUTO-SYN
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::ReplyCrcError => {
                // recipient sent reply but CRC check failed
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::TelegramCrcError => {
                // some master sent telegram but CRC check failed
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::Request { telegram, token } => {
                match telegram.dest {
                    0xFF => {
                        // this is meant for us, but master to master
                        // schedule telegram as reply (add it to queue)
                    }
                    0x04 => {
                        // this is meant for us, reply
                        driver
                            .reply_as_slave(&[0xDE, 0xAD, 0xBE, 0xEF], &mut uart, token)
                            .unwrap();
                    }
                    _ => {
                        // ignore
                    }
                }
            }
            energy_bus::ProcessResult::Reply { data: _ } => {
                // success
                msg = None; // remove message from queue
            }
            energy_bus::ProcessResult::SlaveAckOk => {
                // our reply was acknowledged
            }
            energy_bus::ProcessResult::SlaveAckErr => {
                // our reply was not acknowledged
            }
        }
    }
}
