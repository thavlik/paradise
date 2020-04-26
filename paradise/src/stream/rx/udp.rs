use super::*;

pub struct UdpRxStream<B> where B: RxBuffer {
    clock: std::sync::atomic::AtomicU64,
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
    sync: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl<B> UdpRxStream<B> where B: 'static + RxBuffer {
    pub fn new(port: u16) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        sock.set_nonblocking(true)?;
        let (s, stop_recv) = crossbeam::crossbeam_channel::unbounded();
        let sync = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let stream = std::sync::Arc::new(Self {
            stop: s,
            clock: std::default::Default::default(),
            buf: std::sync::Arc::new(B::new()),
            sync: sync.clone(),
        });
        //crate::runtime::Runtime::get()
        //    .rt
        //    .lock()
        //    .unwrap()
        //    .block_on(async {
        //    });
        tokio::task::spawn(Self::entry(stream.buf.clone(), sock, sync, stop_recv));
        Ok(stream)
    }

    async fn entry(b: std::sync::Arc<B>, sock: std::net::UdpSocket, sync: std::sync::Arc<std::sync::atomic::AtomicU64>, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        // TODO: set clock. Right now all samples are accepted.
        let mut clock: u64 = 0;
        loop {
            std::thread::yield_now();
            select! {
                recv(stop) -> _ => return,
                default => {},
            }
            let (amt, _src) = match sock.recv_from(&mut buf[..]) {
                Ok(value) => value,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        continue
                    },
                    _ => {
                        println!("recv_from: {:?}", e);
                        continue
                    }
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
                println!("discarding late sample buffer");
                continue;
            }
            let status = hdr[7];
            let data = &buf[8..amt - 8];
            if data.len() % 4 != 0 {
                println!("data buffer is not divisible by four");
                continue;
            }
            let num_samples = data.len() / 4;
            let samples: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
            b.accumulate(timestamp, samples);
        }
    }
}

impl<B> std::ops::Drop for UdpRxStream<B>
    where B: RxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> RxStream for UdpRxStream<B> where B: 'static + RxBuffer {
    fn process(&self, output_buffer: &mut [f32]) -> u64 {
        // Swap out the current receive buffer
        self.buf.flush(output_buffer);
        self.sync.load(std::sync::atomic::Ordering::SeqCst)
    }
}