use super::*;

pub mod locking;
pub mod udp;

pub trait TxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new() -> Self;

    /// Accumulates the data into the send buffer. Called by plugin.
    fn accumulate(&self, input_buffer: &[f32]);

    /// Flushes the send buffer into `buffer`. Called by network thread.
    fn flush(&self, buffer: &mut [f32]) -> usize;
}

pub trait TxStream<B> where B: TxBuffer {
    fn process(&self, input_buffer: &[f32]);
}

