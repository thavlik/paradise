use super::*;

struct Chunk {
    timestamp: u64,
    samples: Vec<f32>,
}

struct LockingRxBufferState {
    pub chunks: Vec<Chunk>,
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
        //self.state[cycle(&self.parity)]
        //    .lock()
        //    .unwrap()
        self.get_state()
    }

    fn get_state(&self) -> std::sync::MutexGuard<LockingRxBufferState> {
        let parity = self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2;
        self.state[parity].lock().unwrap()
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
                    chunks: vec![Chunk {
                        timestamp: 0,
                        samples: vec![0.0; 4 * 192000],
                    }],
                }),
                std::sync::Mutex::new(LockingRxBufferState {
                    discard: 0,
                    oldest: 0,
                    chunks: vec![Chunk {
                        timestamp: 0,
                        samples: vec![0.0; 4 * 192000],
                    }],
                }),
            ],
        }
    }

    fn flush(&self, output_buffer: &mut [f32]) {
        let mut state = self.cycle();
        let mut written: usize = 0;
        let mut consumed = 0;
        for chunk in state.chunks.iter_mut() {
            let remaining = output_buffer.len() - written;
            if remaining <= 0 {
                // No need to process additional samples
                break;
            }
            let amt = remaining.min(chunk.samples.len());
            unsafe {
                std::ptr::copy_nonoverlapping(
                    chunk.samples[..amt].as_ptr(),
                    output_buffer[written..written + amt].as_mut_ptr(),
                    amt,
                );
            }
            written += amt;
            if chunk.samples.len() == amt {
                // This entire chunk has been consumed
                consumed += 1;
            } else {
                // Part of the chunk has been consumed
                chunk.samples.drain(..amt);
            }
        }
        state.chunks.drain(..consumed);
        //let num_samples = samples.len();
        //if num_samples == 0 {
        //    return;
        //}
        //let n = output_buffer.len().min(num_samples);
        //let i = num_samples - n;
        //unsafe { std::ptr::copy_nonoverlapping(samples[i..].as_ptr(), output_buffer.as_mut_ptr(), n) };
        //state.discard = state.oldest;
        //state.chunks.drain(..);
    }

    fn accumulate(&self, timestamp: u64, in_samples: &[f32]) {
        let mut state = self.get_state();
        if timestamp < state.discard {
            println!("discarding sample");
            return;
        }
        state.oldest = state.oldest.min(timestamp);
        // Determine where the samples belong
        let i = match state
            .chunks
            .iter()
            .enumerate()
            .find(|(_, chunk)| chunk.timestamp > timestamp)
        {
            Some((i, _)) => i,
            None => state.chunks.len(),
        };
        // Insert the samples such that all elements are order
        // according to timestamp.
        state.chunks.insert(
            i,
            Chunk {
                timestamp,
                samples: Vec::from(in_samples),
            },
        );
    }
}
