use super::*;

struct Chunk {
    timestamp: u64,
    samples: Vec<f32>,
}

struct LockingRxBufferState {
    chunks: Vec<Chunk>,
    samples: Vec<f32>,
    discard: u64,
    oldest: u64,
}

pub struct LockingRxBuffer {
    parity: std::sync::atomic::AtomicUsize,
    state: [std::sync::Mutex<LockingRxBufferState>; 2],
}

unsafe impl std::marker::Send for LockingRxBuffer {}

unsafe impl std::marker::Sync for LockingRxBuffer {}

impl LockingRxBuffer {
    fn cycle(&self) -> std::sync::MutexGuard<LockingRxBufferState> {
        self.state[cycle(&self.parity)]
            .lock()
            .unwrap()
    }

    fn get_state(&self) -> std::sync::MutexGuard<LockingRxBufferState> {
        let parity = self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2;
        self.state[parity]
            .lock()
            .unwrap()
    }
}


impl RxBuffer for LockingRxBuffer {
    fn new() -> Self {
        Self {
            parity: std::default::Default::default(),
            state: [
                std::sync::Mutex::new(LockingRxBufferState {
                    discard: 0,
                    oldest: 0,
                    chunks: Vec::new(),
                    samples: Vec::new(),
                }),
                std::sync::Mutex::new(LockingRxBufferState {
                    discard: 0,
                    oldest: 0,
                    chunks: Vec::new(),
                    samples: Vec::new(),
                })
            ],
        }
    }

    fn flush(&self, output_buffer: &mut [f32]) {
        let mut state = self.cycle();
        let num_samples = state.samples.len();
        if num_samples == 0 {
            return;
        }
        let i = num_samples - output_buffer.len().min(num_samples);
        output_buffer.copy_from_slice(&state.samples[i..]);
        state.chunks.clear();
        state.samples.clear();
        state.discard = state.oldest;
    }

    fn accumulate(&self, timestamp: u64, in_samples: &[f32]) {
        let mut state = self.get_state();
        if timestamp < state.discard {
            println!("discarding sample");
            return;
        }
        state.oldest = state.oldest.min(timestamp);
        // Determine where the samples belong
        let i = match state.chunks.iter()
            .enumerate()
            .find(|(_, chunk)| chunk.timestamp > timestamp) {
            Some((i, _)) => i,
            None => state.chunks.len(),
        };
        // Insert the samples such that all elements are order
        // according to timestamp.
        state.chunks.insert(i, Chunk {
            timestamp,
            samples: Vec::from(in_samples),
        });
        if i == state.chunks.len() {
            // Simple extension of the output buffer
            state.samples.extend_from_slice(in_samples);
            return;
        }
        // Count the number of samples that are already in order
        let offset = state.chunks[..i].iter()
            .fold(0, |n, b| n + b.samples.len());
        // Truncate the output buffer to that many samples
        state.samples.resize(offset, 0.0);
        // Re-extend the output buffer with the newly sorted samples
        let mut samples = std::mem::replace(&mut state.samples, Vec::new());
        state.chunks[i..].iter()
            .for_each(|b| samples.extend_from_slice(&b.samples[..]));
        state.samples = samples;
    }
}