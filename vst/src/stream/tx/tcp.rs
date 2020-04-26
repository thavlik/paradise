use super::*;
use std::io::prelude::*;

pub struct TcpTxStream<B> where B: TxBuffer {
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
}

impl<B> std::ops::Drop for TcpTxStream<B>
    where B: TxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> TcpTxStream<B> where B: 'static + TxBuffer {
    pub fn new(
        addr: std::net::SocketAddr,
    ) -> std::io::Result<std::sync::Arc<Self>> {
        let (stop, stop_recv) = crossbeam::crossbeam_channel::unbounded();
        let s = std::sync::Arc::new(Self {
            stop,
            buf: std::sync::Arc::new(B::new()),
        });
        crate::runtime::Runtime::get()
            .rt
            .lock()
            .unwrap()
            .block_on(async {
                tokio::task::spawn(Self::entry(s.buf.clone(), addr, stop_recv))
            });
        Ok(s)
    }

    async fn entry(b: std::sync::Arc<B>, addr: std::net::SocketAddr, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        let clock = std::time::Instant::now();
        loop {
            std::thread::yield_now();
            match stop.try_recv() {
                Ok(_) => {
                    return;
                }
                Err(e) => match e {
                    crossbeam::channel::TryRecvError::Empty => {},
                    crossbeam::channel::TryRecvError::Disconnected => {
                        // Stop stream send channel was dropped.
                        return;
                    },
                }
            };
            write_message_header(&mut buf[..], Some(0), &clock);
            let data: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(buf[8..].as_mut_ptr() as _, buf[8..].len() / 4) };
            let amt = b.flush(data);
            if amt == 0 {
                println!("tx: no bytes to send");
                continue;
            }
            // Include datagram length in message
            let i = 8 + amt * 4;
            buf[0] = (i >> 24) as u8;
            buf[1] = (i >> 16) as u8;
            buf[2] = (i >> 8) as u8;
            buf[3] = (i >> 0) as u8;
            match stream.write(&buf[..i+4]) {
                Ok(_) => {},
                Err(e) => {
                    println!("tcp stream write: {:?}", e);
                    continue;
                },
            };
        }
    }
}

impl<B> TxStream for TcpTxStream<B> where B: 'static + TxBuffer {
    fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf.accumulate(input_buffer)
    }
}

