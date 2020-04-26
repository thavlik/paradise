use super::*;

pub mod locking;


pub trait RxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new() -> Self;

    /// Accumulates the data into the current write buffer.
    /// Called by network thread.
    fn accumulate(&self, timestamp: u64, samples: &[f32]);

    /// Flushes the data in the current write buffer to output_buffer.
    /// Called by plugin.
    fn flush(&self, output_buffer: &mut [f32]);
}

pub struct RxStream<B> where B: RxBuffer {
    clock: std::sync::atomic::AtomicU64,
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
    sync: crossbeam::crossbeam_channel::Sender<u64>,
}

impl<B> std::ops::Drop for RxStream<B>
    where B: RxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> RxStream<B> where B: 'static + RxBuffer {
    pub fn new(port: u16) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        let (s, stop_recv) = crossbeam::crossbeam_channel::unbounded();
        let (sync_send, sync_recv) = crossbeam::crossbeam_channel::unbounded();
        let stream = std::sync::Arc::new(Self {
            stop: s,
            clock: std::default::Default::default(),
            buf: std::sync::Arc::new(B::new()),
            sync: sync_send,
        });
        crate::runtime::Runtime::get()
            .rt
            .lock()
            .unwrap()
            .block_on(async {
                //tokio::task::spawn(Self::entry(stream.buf.clone(), sock, sync_recv, stop_recv))
            });
        Ok(stream)
    }

    pub fn process(&self, output_buffer: &mut [f32]) {
        // Swap out the current receive buffer
        self.buf.flush(output_buffer);
    }

    async fn entry(b: std::sync::Arc<B>, sock: std::net::UdpSocket, sync: crossbeam::crossbeam_channel::Receiver<u64>, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        // Retain a strong reference to the runtime so it doesn't
        let _rt = crate::runtime::Runtime::get();
        // TODO: set clock. Right now all samples are accepted.
        let mut clock: u64 = 0;
        loop {
            select! {
                recv(stop) -> _ => return,
                recv(sync) -> time => clock = time.unwrap(),
                default => {},
            }
            tokio::task::yield_now().await;
            continue;
            let (amt, _src) = match sock.recv_from(&mut buf[..]) {
                Ok(value) => value,
                Err(e) => {
                    println!("recv_from: {:?}", e);
                    continue
                }
            };
            let hdr = &buf[..8];
            let timestamp = ((hdr[0] as u64) << 48) |
                ((hdr[1] as u64) << 40) |
                ((hdr[2] as u64) << 32) |
                ((hdr[3] as u64) << 24) |
                ((hdr[4] as u64) << 16) |
                ((hdr[5] as u64) << 8) |
                ((hdr[6] as u64) << 0);
            // Don't accumulate samples older than the oldest timestamp observed in the previous flushed
            let delta = timestamp - clock;
            if delta < 0 {
                // Current timestamp is higher than incoming.
                // Discard this sample.
                warn!("discarding late sample");
                return;
            }
            let status = hdr[7];
            let data = &buf[8..amt - 8];
            if data.len() % 4 != 0 {
                panic!("data buffer is not divisible by four")
            }
            let num_samples = data.len() / 4;
            let samples: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
            b.accumulate(timestamp, samples);
            //stream.receive(&sock, &mut buf[..]);
            tokio::task::yield_now().await;
            break;
        }
    }
}