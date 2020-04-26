use super::*;

pub mod locking;

pub trait TxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new() -> Self;

    /// Accumulates the data into the send buffer. Called by plugin.
    fn accumulate(&self, input_buffer: &[f32]);

    /// Flushes the send buffer into `buffer`. Called by network thread.
    fn flush(&self, buffer: &mut [f32]) -> usize;
}

pub struct TxStream<B> where B: TxBuffer {
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: std::sync::Arc<B>,
}

impl<B> std::ops::Drop for TxStream<B>
    where B: TxBuffer {
    fn drop(&mut self) {
        self.stop.send(());
    }
}

impl<B> TxStream<B> where B: 'static + TxBuffer {
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



    /// Send audio over UDP
    //fn send(&self, sock: &std::net::UdpSocket, send_buf: &mut [u8]) -> std::io::Result<usize> {
    //    //self.write_message_header(&mut send_buf[..8]);
    //    let data: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(send_buf[8..].as_mut_ptr() as _, send_buf[8..].len() / 4) };
    //    let amt = self.buf.flush(data);
    //    if amt == 0 {
    //        println!("amt == 0");
    //        return Ok(0);
    //    }
    //    let i = 8 + amt * 4;
    //    sock.send_to(&send_buf[..i], self.dest)
    //}


    pub fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf.accumulate(input_buffer)
    }

    fn write_message_header(buf: &mut [u8], clock: &std::time::Instant) {
        let timestamp = clock.elapsed().as_nanos();
        buf[0] = ((timestamp >> 48) & 0xFF) as u8;
        buf[1] = ((timestamp >> 40) & 0xFF) as u8;
        buf[2] = ((timestamp >> 32) & 0xFF) as u8;
        buf[3] = ((timestamp >> 24) & 0xFF) as u8;
        buf[4] = ((timestamp >> 16) & 0xFF) as u8;
        buf[5] = ((timestamp >> 8) & 0xFF) as u8;
        buf[6] = ((timestamp >> 0) & 0xFF) as u8;
        buf[7] = 0; // Status
    }

    async fn entry(b: std::sync::Arc<B>, sock: std::net::UdpSocket, dest: std::net::SocketAddr, stop: crossbeam::crossbeam_channel::Receiver<()>) {
        const BUFFER_SIZE: usize = 256_000;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let clock = std::time::Instant::now();
        loop {
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
            Self::write_message_header(&mut buf[..], &clock);
            let data: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(buf[8..].as_mut_ptr() as _, buf[8..].len() / 4) };
            let amt = b.flush(data);
            if amt == 0 {
                println!("tx: no bytes to send");
                std::thread::yield_now();
                continue;
            }
            let i = 8 + amt * 4;
            sock.send_to(&buf[..i], dest);
            std::thread::yield_now();
        }
    }
}

