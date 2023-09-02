use std::time::Duration;

use energy_bus::{EbusDriver, Telegram, Transmit};

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

    fn transmit_raw(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        // write bytes to UART

        Ok(())
    }
}

fn sleep(_d: Duration) {
    // This function is called by the ebus driver to allow
    // other tasks to preempt this task (because we are not in a time critical state).
    // You may sleep here using RTOS functionality if it matches your execution model.
    // If you use interrupts or async, this function can just be a no-op.
}

fn main() {
    let (_tx, rx) = std::sync::mpsc::channel::<Telegram>();
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
            energy_bus::ProcessResult::AckOk => {
                // successfully sent message with no expected reply
                msg = None; // remove message from queue
            }
            energy_bus::ProcessResult::AckErr => {
                // recipient replied ACK_ERR
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::Timeout => {
                // recipient did not reply within AUTO-SYN
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::CrcError => {
                // recipient sent reply but CRC check failed
                msg = None; // remove message from queue
                            // could also try to requeue this message for later
            }
            energy_bus::ProcessResult::Reply { buf, len } => {
                // success
                dbg!((buf, len));
                msg = None; // remove message from queue
            }
        }
    }
}
