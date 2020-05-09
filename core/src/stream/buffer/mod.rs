pub mod locking;

pub trait Buffer<T>
    where
        Self: std::marker::Sync + std::marker::Send,
        T: Clone,
{
    fn new() -> Self;

    fn accumulate(&self, samples: &[T]);

    fn flush(&self, output_buffer: &mut [T]) -> usize;
}
