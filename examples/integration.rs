use std::time::Duration;

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

    fn transmit_raw(&mut self, _bytes: &[u8]) -> Result<(), Self::Error> {
        // write bytes to UART

        Ok(())
    }
}

fn sleep(_d: Duration) {
    // This function is called by the ebus driver to allow
    // other tasks to preempt this task (because we are not in a time critical state).
    // You may sleep here using RTOS functionality if it matches your execution model.
    // If you use interrupts or async, this function can just be a no-op.

    // It is recommended to handle sub-millisecond sleeps differently than
    // multiple-ms sleeps, because on sub-ms delays precision matters for
    // bus arbitration. It could be necessary to busy loop for microsecond sleeps.
}

fn main() {
    let (_tx, rx) = std::sync::mpsc::channel::<MasterTelegram>();
    // give tx to application so it can queue messages

    let mut uart = Transmitter(UartTxDriver);
    let mut driver = EbusDriver::new(ARBITRATION_DELAY, CRC_POLYNOM_TELEGRAM, CRC_POLYNOM_DATA);

    let mut msg = None;
    loop {
        // Here, we block on the receival of a byte which is not ideal.
        // Depending on your device and architecture, you should use interrupts or
        // low latency async code.
        msg = msg.or_else(|| rx.try_recv().ok());
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
                            .reply_as_slave(&[0xDE, 0xAD, 0xBE, 0xEF], &mut uart, sleep, token)
                            .unwrap();
                    }
                    _ => {
                        // ignore
                    }
                }
            }
            energy_bus::ProcessResult::Reply { data } => {
                // success
                dbg!(data);
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
