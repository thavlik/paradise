use super::*;

pub mod udp;

pub trait RxStream<T> {
    fn process(&self, output_buffer: &mut [T]) -> usize;
}
