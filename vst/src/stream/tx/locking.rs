use super::*;

pub struct LockingTxBuffer {
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
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
    fn new(rt: &tokio::runtime::Runtime) -> Self {
        Self {
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new())),
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
        if buffer.len() < len {
            panic!("tx buffer overrun");
        }
        unsafe { std::ptr::copy_nonoverlapping(buf.as_ptr(), buffer.as_mut_ptr(), len) };
        buf.clear();
        len
    }
}
