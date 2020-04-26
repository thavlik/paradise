use super::*;

pub struct UdpTxStream<B> where B: TxBuffer {
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
}

impl<B> std::ops::Drop for UdpTxStream<B>
    where B: TxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> UdpTxStream<B> where B: 'static + TxBuffer {
    pub fn new(
        dest: std::net::SocketAddr,
        outbound_port: u16,
    ) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        let (s, r) = crossbeam::crossbeam_channel::unbounded();
        let stream = std::sync::Arc::new(Self {
            stop: s,
            buf: std::sync::Arc::new(B::new()),
        });
        crate::runtime::Runtime::get()
            .rt
            .lock()
            .unwrap()
            .block_on(async {
                tokio::task::spawn(Self::entry(stream.buf.clone(), sock, dest, r))
            });
        Ok(stream)
    }

    async fn entry(b: std::sync::Arc<B>, sock: std::net::UdpSocket, dest: std::net::SocketAddr, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
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
            let hdr_len = write_message_header(&mut buf[..], None, Some(clock.elapsed()));
            let data: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(buf[hdr_len..].as_mut_ptr() as _, buf[hdr_len..].len() / 4) };
            let amt = b.flush(data);
            if amt == 0 {
                println!("tx: no bytes to send");
                continue;
            }
            let i = hdr_len + amt * 4;
            match sock.send_to(&buf[..i], dest) {
                Ok(amt) => {
                    // TODO
                },
                Err(e) => {
                    // TODO
                },
            }
        }
    }
}

impl<B> TxStream for UdpTxStream<B> where B: 'static + TxBuffer {
    fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf.accumulate(input_buffer)
    }
}

