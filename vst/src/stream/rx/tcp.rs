use super::*;
use std::io::prelude::*;

pub struct TcpRxStream<B> where B: RxBuffer {
    clock: std::sync::atomic::AtomicU64,
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
    sync: crossbeam::crossbeam_channel::Sender<u64>,
}

impl<B> TcpRxStream<B> where B: 'static + RxBuffer {
    pub fn new(port: u16) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let buf  = std::sync::Arc::new(B::new());
        let (stop_send, stop_recv) = crossbeam::crossbeam_channel::unbounded();
        let (sync_send, sync_recv) = crossbeam::crossbeam_channel::unbounded();
        let mut listener = std::net::TcpListener::bind(&addr)?;
        let stream = std::sync::Arc::new(Self {
            buf: buf.clone(),
            stop: stop_send,
            clock: std::default::Default::default(),
            sync: sync_send,
        });
        crate::runtime::Runtime::get()
            .rt
            .lock()
            .unwrap()
            .block_on(async {
                tokio::task::spawn(Self::entry(stream.buf.clone(), listener, sync_recv, stop_recv))
            });
        Ok(stream)
    }

    async fn entry(b: std::sync::Arc<B>, listener: std::net::TcpListener, sync: crossbeam::crossbeam_channel::Receiver<u64>, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        // TODO: set clock. Right now all samples are accepted.
        let mut clock: u64 = 0;
        for stream in listener.incoming() {
            //let (stop_send, stop_recv) = crossbeam::crossbeam_channel::unbounded();
            //let (sync_send, sync_recv) = crossbeam::crossbeam_channel::unbounded();
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    //tokio::task::spawn(Self::entry(buf.clone(), stream, sync_recv, stop_recv));
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        loop {
            std::thread::yield_now();
            select! {
                recv(stop) -> _ => return,
                recv(sync) -> time => clock = time.unwrap(),
                default => {},
            }
            let amt = match stream.read(&mut buf[..]) {
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

impl<B> std::ops::Drop for TcpRxStream<B>
    where B: RxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> RxStream for TcpRxStream<B> where B: 'static + RxBuffer {
    fn process(&self, output_buffer: &mut [f32]) {
        // Swap out the current receive buffer
        self.buf.flush(output_buffer);
    }
}