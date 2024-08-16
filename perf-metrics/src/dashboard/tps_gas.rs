//! This module is used to support the display of TPS and MGas/s.
use crate::metrics::TpsAndGasMessage;
use revm_utils::time_utils::instant::Instant;
use std::ops::{Div, Mul};

#[derive(Debug, Default)]
pub(super) struct TpsAndGasDisplayer {
    pre_txs: u128,
    pre_gas: u128,
    last_txs: u128,
    last_gas: u128,
    pre_instant: Instant,
}

impl TpsAndGasDisplayer {
    const N: u64 = 1000;

    /// Updates the transactions and gas metrics, and prints the result if block_number is a multiple of `N`.
    fn update_tps_and_gas(&mut self, block_number: u64, txs: u128, gas: u128) {
        if block_number % Self::N == 0 {
            self.print_content(block_number, txs, gas);
        }

        self.last_txs = txs;
        self.last_gas = gas;
    }

    /// Starts recording metrics by capturing the current state.
    fn start_record(&mut self) {
        self.pre_txs = self.last_txs;
        self.pre_gas = self.last_gas;
        self.pre_instant = Instant::now();
    }

    /// Stops recording metrics and prints the final result.
    fn stop_record(&mut self, block_number: u64) {
        self.print_content(block_number, self.last_txs, self.last_gas);
    }

    /// Prints the TPS and MGas/s metrics.
    fn print_content(&mut self, block_number: u64, txs: u128, gas: u128) {
        let now = Instant::now();
        let elapsed_ns = now.checked_nanos_since(self.pre_instant).unwrap_or(1.0); // Avoid division by zero
        let delta_txs = txs - self.pre_txs;
        let delta_gas = gas - self.pre_gas;

        let tps = (delta_txs as f64) * 1_000_000_000.0 / elapsed_ns;
        let mgas_ps = (delta_gas as f64) * 1_000_000_000.0 / elapsed_ns;

        self.pre_txs = txs;
        self.pre_gas = gas;
        self.pre_instant = now;

        println!("block_number: {}, TPS : {:.2}", block_number, tps);
        println!("block_number: {}, MGas: {:.3}\n", block_number, mgas_ps);
    }

    /// Processes messages to update metrics or start/stop recording.
    pub(super) fn print(&mut self, block_number: u64, message: TpsAndGasMessage) {
        match message {
            TpsAndGasMessage::Record(record) => {
                self.update_tps_and_gas(record.block_number, record.txs, record.gas)
            }
            TpsAndGasMessage::Switch(switch) => {
                if switch {
                    self.start_record();
                } else {
                    self.stop_record(block_number);
                }
            }
        }
    }
}
