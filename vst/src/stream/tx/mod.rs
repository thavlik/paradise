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

pub fn write_message_header(buf: &mut [u8], size: Option<usize>, timestamp: Option<std::time::Duration>) -> usize {
    let mut offset = 0;
    if let Some(size) = size {
        if size > std::usize::MAX {
            panic!("tx buffer overflow");
        }
        buf[offset] = (size >> 8) as u8;
        offset += 1;
        buf[offset] = (size >> 0) as u8;
        offset += 1;
    }
    if let Some(timestamp) = timestamp {
        let timestamp = timestamp.as_nanos();
        buf[offset] = ((timestamp >> 48) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 40) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 32) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 24) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 16) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 8) & 0xFF) as u8;
        offset += 1;
        buf[offset] = ((timestamp >> 0) & 0xFF) as u8;
        offset += 1;
        buf[offset] = 0; // Status
        offset += 1;
    }
    offset
}