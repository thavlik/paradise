use super::*;

pub mod locking;
//pub mod tcp;
pub mod udp;

pub trait RxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new() -> Self;

    /// Accumulates the data into the current write buffer.
    /// Called by network thread.
    fn accumulate(&self, timestamp: u64, samples: &[f32]);

    /// Flushes the data in the current write buffer to output_buffer.
    /// Called by plugin.
    fn flush(&self, output_buffer: &mut [f32]);
}

pub trait RxStream {
    fn process(&self, output_buffer: &mut [f32]) -> u64;
}

