use super::*;

pub struct LockingTxBuffer {
    parity: std::sync::atomic::AtomicUsize,
    buf: [std::sync::Mutex<Vec<f32>>; 2],
}

impl LockingTxBuffer {
    fn current(&self) -> usize {
        self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2
    }

    fn cycle(&self) -> std::sync::MutexGuard<Vec<f32>> {
        self.buf[cycle(&self.parity)]
            .lock()
            .unwrap()
    }
}

impl TxBuffer for LockingTxBuffer {
    fn new() -> Self {
        Self {
            parity: std::default::Default::default(),
            buf: [
                std::sync::Mutex::new(Vec::new()),
                std::sync::Mutex::new(Vec::new()),
            ],
        }
    }

    fn accumulate(&self, input_buffer: &[f32]) {
        self.buf[self.current()]
            .lock()
            .unwrap()
            .extend_from_slice(input_buffer);
    }

    fn flush(&self, buffer: &mut [f32]) -> usize {
        let mut buf = self.cycle();
        let len = buf.len();
        if len == 0 {
            return 0;
        }
        if buffer.len() < len {
            panic!("tx buffer overrun");
        }
        let buffer = &mut buffer[..len];
        let buffer = buffer.as_mut_ptr();
        unsafe { std::ptr::copy_nonoverlapping(buf.as_ptr(), buffer, len) };
        buf.clear();
        len
    }
}
