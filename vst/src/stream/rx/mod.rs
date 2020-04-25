use super::*;

pub mod locking;


pub trait RxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new(rt: &tokio::runtime::Runtime) -> Self;

    /// Accumulates the data into the current write buffer.
    /// Called by network thread.
    fn accumulate(&self, timestamp: u64, samples: &[f32]);

    /// Flushes the data in the current write buffer to output_buffer.
    /// Called by plugin.
    fn flush(&self, output_buffer: &mut [f32]);
}

pub struct RxStream<B> where B: RxBuffer {
    sock: std::net::UdpSocket,
    clock: std::sync::atomic::AtomicU64,
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: B,
}

impl<B> std::ops::Drop for RxStream<B>
    where B: RxBuffer {
    fn drop(&mut self) {
        //self.stop.send(());
    }
}

impl<B> RxStream<B> where B: 'static + RxBuffer {
    pub fn new(port: u16, rt: &tokio::runtime::Runtime) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        let (s, r) = crossbeam::crossbeam_channel::unbounded();
        let stream = std::sync::Arc::new(Self {
            sock,
            stop: s,
            clock: std::default::Default::default(),
            buf: B::new(rt),
        });
        rt.spawn(Self::entry(stream.clone(), r));
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
        // Don't accumulate samples older than the oldest timestamp observed in the previous flushed
        let clock = self.clock.load(std::sync::atomic::Ordering::SeqCst);
        let delta = timestamp - clock;
        if delta < 0 {
            // Current timestamp is higher than incoming.
            // Discard this sample.
            warn!("discarding late sample");
            return;
        }
        let status = hdr[7];
        let data = &receive_buf[8..amt - 8];
        if data.len() % 4 != 0 {
            panic!("data buffer is not divisible by four")
        }
        let num_samples = data.len() / 4;
        let samples: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
        self.buf.accumulate(timestamp, samples);
    }

    pub fn process(&self, output_buffer: &mut [f32]) {
        // Swap out the current receive buffer
        self.buf.flush(output_buffer);
    }

    async fn entry(stream: std::sync::Arc<Self>, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        loop {
            match stop.try_recv() {
                Ok(_) => {
                    return;
                }
                Err(e) => match e {
                    crossbeam::channel::TryRecvError::Empty => {}
                    crossbeam::channel::TryRecvError::Disconnected => {
                        // Stop stream send channel was dropped.
                        return;
                    }
                }
            };
            stream.receive(&mut buf[..]);
            std::thread::yield_now();
        }
    }
}