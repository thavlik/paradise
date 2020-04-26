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
        let buf  = std::sync::Arc::new(B::new());
        let (stop_send, stop_recv) = crossbeam::crossbeam_channel::unbounded();
        let (sync_send, sync_recv) = crossbeam::crossbeam_channel::unbounded();
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
                tokio::task::spawn(Self::entry(stream.buf.clone(), port, sync_recv, stop_recv))
            });
        Ok(stream)
    }

    async fn entry(b: std::sync::Arc<B>, port: u16, sync: crossbeam::crossbeam_channel::Receiver<u64>, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        let addr = format!("0.0.0.0:{}", port);
        let mut listener = std::net::TcpListener::bind(&addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        const BUFFER_SIZE: usize = 256_000;
        let mut recv_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        // TODO: set clock. Right now all samples are accepted.
        let mut clock: u64 = 0;
        let mut stream: Option<std::net::TcpStream> = None;
        let mut msg: Vec<u8> = Vec::new();
        loop {
            std::thread::yield_now();
            select! {
                recv(stop) -> _ => return,
                recv(sync) -> time => clock = time.unwrap(),
                default => {},
            }
            match listener.accept() {
                Ok((incoming, _addr)) => {
                    // Existing connection is closed when stream is dropped
                    incoming.set_nonblocking(true).unwrap();
                    stream = Some(incoming);
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => {},
                    _ => {
                        println!("listener.accept(): {:?}", e);
                    },
                }
            };
            match &mut stream {
                Some(stream) => {
                    let amt = match stream.read(&mut recv_buf[..]) {
                        Ok(amt) => amt,
                        Err(e) => { 0 },
                    };
                    msg.extend_from_slice(&recv_buf[..amt]);
                    if msg.len() >= 2 {
                        let len = ((msg[0] as usize) << 8) |
                            ((msg[1] as usize) << 0);
                        if msg.len() >= len {
                            // We have at least the given message
                            let datagram = &msg[..len];
                            let hdr = &datagram[..8];
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
                                msg.drain(..len);
                                continue;
                            }
                            let status = hdr[7];
                            let data = &datagram[8..];
                            if data.len() % 4 != 0 {
                                println!("data buffer is not divisible by four");
                                msg.drain(..len);
                                continue;
                            }
                            let num_samples = data.len() / 4;
                            let samples: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
                            b.accumulate(timestamp, samples);
                            msg.drain(..len);
                        }
                    }
                },
                None => {
                    println!("no incoming stream");
                },
            }
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