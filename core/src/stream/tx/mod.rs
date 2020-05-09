use super::*;

pub mod udp;

pub trait TxStream {
    fn process(&self, input_buffer: &[f32], clock: u64);
}

pub fn write_message_header(
    buf: &mut [u8],
    size: Option<usize>,
    timestamp: u64,
    status: u8,
) -> usize {
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
    buf[offset] = status; // Status
    offset += 1;
    offset
}
