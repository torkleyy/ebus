use std::time::Duration;

use energy_bus::{EbusDriver, MasterTelegram, ProcessResult, Transmit};

#[derive(Default)]
struct LoopbackTransmitter {
    sent: Vec<u8>,
    loopback: Vec<u8>,
}

impl Transmit for LoopbackTransmitter {
    type Error = ();

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
        let mut results = vec![];

        results.push(
            self.driver
                .process(byte, &mut self.transmit, sleep, msg)
                .unwrap(),
        );

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
}
