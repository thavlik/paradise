use super::*;

pub struct LockingBuffer<T> {
    state: std::sync::Mutex<Vec<T>>,
}

unsafe impl<T> std::marker::Send for LockingBuffer<T> {}

unsafe impl<T> std::marker::Sync for LockingBuffer<T> {}

impl<T> super::Buffer<T> for LockingBuffer<T> where T: Clone {
    fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(vec![]),
        }
    }

    fn flush(&self, output_buffer: &mut [T]) -> usize {
        let mut state = self.state.lock().unwrap();
        std::mem::replace(&mut *state, vec![]).len()
    }

    fn accumulate(&self, in_samples: &[T]) {
        let mut state = self.state.lock().unwrap();
        std::vec::Vec::extend_from_slice(&mut *state, in_samples);
    }
}
