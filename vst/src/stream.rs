struct Chunk {
    timestamp: u64,
    samples: Vec<f32>,
}

pub trait Buffer where Self: std::marker::Sync + std::marker::Send {
    fn new(rt: &tokio::runtime::Runtime) -> Self;

    /// Flushes the data in the current write buffer to output_buffer
    fn flush(&self, output_buffer: &mut [f32]);

    /// Accumulates the data into the current write buffer
    fn accumulate(&self, timestamp: u64, samples: &[f32]);
}

pub mod locking {
    use super::*;

    struct LockingBufferState {
        chunks: Vec<Chunk>,
        samples: Vec<f32>,
    }

    pub struct LockingBuffer {
        state: std::sync::Mutex<LockingBufferState>,
    }

    impl Buffer for LockingBuffer {
        fn new(rt: &tokio::runtime::Runtime) -> Self {
            Self {
                state: std::sync::Mutex::new(LockingBufferState {
                    chunks: Vec::new(),
                    samples: Vec::new(),
                })
            }
        }

        fn flush(&self, output_buffer: &mut [f32]) {
            let mut state = self.state.lock().unwrap();
            // Take only most recent samples
            let i = state.samples.len() - output_buffer.len();
            assert_eq!(state.samples[i..].len(), output_buffer.len());
            output_buffer.copy_from_slice(&state.samples[i..]);
            state.chunks.clear();
            state.samples.clear();
        }

        fn accumulate(&self, timestamp: u64, in_samples: &[f32]) {
            let mut state = self.state.lock().unwrap();
            let mut chunks = std::mem::replace(&mut state.chunks, Vec::new());
            let mut samples = std::mem::replace(&mut state.samples, Vec::new());
            // Determine where the samples belong
            let i = match chunks.iter()
                .enumerate()
                .rev()
                .find(|(_, chunk)| timestamp > chunk.timestamp) {
                Some((i, _)) => i+1,
                None => 0,
            };
            // Insert the samples such that all elements are order
            // according to timestamp.
            chunks.insert(i, Chunk{
                timestamp,
                samples: Vec::from(in_samples),
            });
            if i != chunks.len() {
                // Count the number of samples that are already in order
                let offset = chunks[..i].iter()
                    .fold(0, |n, b| n + b.samples.len());
                // Truncate the output buffer to that many samples
                samples.resize(offset, 0.0);
                // Re-extend the output buffer with the newly sorted samples
                chunks[i..].iter()
                    .for_each(|b | samples.extend_from_slice(&b.samples[..]));
            } else {
                // Simple extension of the output buffer
                samples.extend_from_slice(in_samples);
            }
            state.chunks = chunks;
            state.samples = samples;
        }
    }
}

pub use locking::LockingBuffer;

pub struct RxStream<B> where B: Buffer {
    sock: std::net::UdpSocket,
    parity: std::sync::atomic::AtomicUsize,
    clock: std::sync::atomic::AtomicU64,
    buf: [B; 2],
}

impl<'b, B> RxStream<B> where B: 'b + Buffer, Self: 'b {
    pub fn new(port: usize, rt: &'b tokio::runtime::Runtime) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        let stream = std::sync::Arc::new(Self {
            sock,
            parity: std::default::Default::default(),
            clock: std::default::Default::default(),
            buf: [B::new(rt), B::new(rt)],
        });
        //rt.spawn(Self::entry(std::sync::Arc::downgrade(&stream)));
        Ok(stream)
    }

    /// Receive data over the network. A thread is supposed
    /// to call this repeatedly to ensure the socket is
    /// quickly synchronized with the output buffer.
    fn receive(&self, receive_buf: &mut [u8]) {
        let (amt, _src) = match self.sock.recv_from(receive_buf) {
            Ok(value) => value,
            Err(e) => {
                error!("recv_from: {:?}", e);
                return;
            }
        };
        let hdr = &receive_buf[..8];
        let timestamp = ((hdr[0] as u64) << 48) |
            ((hdr[1] as u64) << 40) |
            ((hdr[2] as u64) << 32) |
            ((hdr[3] as u64) << 24) |
            ((hdr[4] as u64) << 16) |
            ((hdr[5] as u64) << 8) |
            ((hdr[6] as u64) << 0);
        let parity: usize = self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2;
        let buf = &self.buf[parity];
        let clock = self.clock.load(std::sync::atomic::Ordering::SeqCst);
        let delta = timestamp - clock;
        if delta < 0 {
            // Current timestamp is higher than incoming.
            // Discard this sample.
            warn!("discarding late sample");
            return;
        }
        let status = hdr[7];
        let data = &receive_buf[8..amt-8];
        if data.len() % 4 != 0 {
            panic!("data buffer is not divisible by four")
        }
        let num_samples = data.len() / 4;
        let samples: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
        buf.accumulate(timestamp, samples);
    }

    fn cycle(&self) -> usize {
        cycle(&self.parity)
    }

    pub fn process(&self, output_buffer: &mut [f32]) {
        // Swap out the current receive buffer
        self.buf[self.cycle()].flush(output_buffer);
    }

    async fn entry(stream: std::sync::Weak<RxStream<B>>) {
        const RECEIVE_BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; RECEIVE_BUFFER_SIZE] =  [0; RECEIVE_BUFFER_SIZE];
        loop {
            match stream.upgrade() {
                Some(stream) => {
                    stream.receive(&mut buf[..]);
                },
                None => {
                    return;
                }
            };
            std::thread::yield_now();
        }
    }
}

pub struct TxStream {
    sock: std::net::UdpSocket,
    dest: std::net::SocketAddr,
    clock: std::time::Instant,
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

impl TxStream {
    pub fn new(
        dest: std::net::SocketAddr,
        outbound_port: u16,
        rt: &tokio::runtime::Runtime,
    ) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        let stream = std::sync::Arc::new(Self {
            sock,
            dest,
            clock: std::time::Instant::now(),
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        });
        rt.spawn(Self::entry(std::sync::Arc::downgrade(&stream)));
        Ok(stream)
    }

    fn write_message_header(&self, buf: &mut Vec<u8>) {
        let timestamp = self.clock.elapsed().as_nanos();
        let slice = [
            ((timestamp >> 48) & 0xFF) as u8,
            ((timestamp >> 40) & 0xFF) as u8,
            ((timestamp >> 32) & 0xFF) as u8,
            ((timestamp >> 24) & 0xFF) as u8,
            ((timestamp >> 16) & 0xFF) as u8,
            ((timestamp >> 8) & 0xFF) as u8,
            ((timestamp >> 0) & 0xFF) as u8,
            0, // Reset & Status
        ];
        buf.extend_from_slice(&slice);
    }

    /// Send audio over UDP
    fn send(&self) -> std::io::Result<usize> {
        let send_buf = {
            let mut buf = self.buf[cycle(&self.parity)]
                .lock()
                .unwrap();
            if buf.len() == 0 {
                // Don't send empty messages
                return Ok(0);
            }
            let mut send_buf = Vec::new();
            self.write_message_header(&mut send_buf);
            let data: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as _, buf.len() * 4) };
            send_buf.extend_from_slice(data);
            buf.clear();
            send_buf
        };
        self.sock.send_to(&send_buf[..], self.dest)
    }

    fn current(&self) -> usize {
        self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2
    }

    pub fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf[self.current()]
            .lock()
            .unwrap()
            .extend_from_slice(input_buffer);
    }

    async fn entry(stream: std::sync::Weak<TxStream>) {
        loop {
            match stream.upgrade() {
                Some(stream) => {
                    stream.send();
                },
                None => {
                    return;
                },
            };
            std::thread::yield_now();
        }
    }
}

fn cycle(parity: &std::sync::atomic::AtomicUsize) -> usize {
    let original: usize = parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let wrapped = original % 2;
    if original > 100_000_000 {
        // Wrap parity back to [0, 1] so there's no risk of overflow.
        // fetch_add returns the old value, so the current value will
        // (functionally) be the complement. This is *only* okay
        // because we know we're the only thread that is writing to
        // parity. Note that the write is non-transactional and could
        // otherwise introduce a race condition.
        parity.store(1 - wrapped, std::sync::atomic::Ordering::SeqCst);
    }
    wrapped
}

#[cfg(test)]
mod test {
    #[test]
    fn test_rev() {
        let v = vec![0, 1, 2];
        let rev: Vec<_> = v.iter().map(|i| *i).rev().collect();
        assert_eq!(v[2], rev[0]);
        assert_eq!(v[1], rev[1]);
        assert_eq!(v[0], rev[2]);
    }

}