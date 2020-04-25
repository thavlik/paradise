use super::*;

pub struct LockingTxBuffer {
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

impl LockingTxBuffer {
    fn current(&self) -> usize {
        self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2
    }
}

impl TxBuffer for LockingTxBuffer {
    fn new(rt: &tokio::runtime::Runtime) -> Self {
        Self {
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new())),
            ],
        }
    }

    fn process(&self, input_buffer: &[f32]) {
        self.buf[self.current()]
            .lock()
            .unwrap()
            .extend_from_slice(input_buffer);
    }

    fn flush(&self, buffer: &mut [f32]) -> usize {
        0
    }
}
