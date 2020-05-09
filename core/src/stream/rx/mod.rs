use super::*;

pub mod udp;

pub trait RxStream {
    fn process(&self, output_buffer: &mut [f32]) -> u64;
}
