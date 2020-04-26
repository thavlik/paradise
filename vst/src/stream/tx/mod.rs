use super::*;

pub mod locking;
pub mod tcp;
pub mod udp;

pub trait TxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new() -> Self;

    /// Accumulates the data into the send buffer. Called by plugin.
    fn accumulate(&self, input_buffer: &[f32]);

    /// Flushes the send buffer into `buffer`. Called by network thread.
    fn flush(&self, buffer: &mut [f32]) -> usize;
}

pub trait TxStream {
    fn process(&self, input_buffer: &[f32]);
}

pub fn write_message_header(buf: &mut [u8], size: Option<usize>, clock: &std::time::Instant) {
    let timestamp = clock.elapsed().as_nanos();
    let mut offset = 0;
    match size {
        Some(size) => {

        },
        None => {},
    };
    buf[offset] = ((timestamp >> 48) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 40) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 32) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 24) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 16) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 8) & 0xFF) as u8;
    buf[offset] = ((timestamp >> 0) & 0xFF) as u8;
    buf[offset] = 0; // Status
}