use super::*;

pub mod locking;

pub trait TxBuffer
    where Self: std::marker::Sync + std::marker::Send {
    fn new(rt: &tokio::runtime::Runtime) -> Self;

    /// Accumulates the data into the send buffer. Called by plugin.
    fn accumulate(&self, input_buffer: &[f32]);

    /// Flushes the send buffer into `buffer`. Called by network thread.
    fn flush(&self, buffer: &mut [f32]) -> usize;
}

pub struct TxStream<B> where B: TxBuffer {
    sock: std::net::UdpSocket,
    dest: std::net::SocketAddr,
    clock: std::time::Instant,
    stop: crossbeam::crossbeam_channel::Sender<()>,
    buf: B,
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
        rt: &tokio::runtime::Runtime,
    ) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        let (s, r) = crossbeam::crossbeam_channel::unbounded();
        let stream = std::sync::Arc::new(Self {
            sock,
            dest,
            stop: s,
            clock: std::time::Instant::now(),
            buf: B::new(rt),
        });
        rt.spawn(Self::entry(stream.clone(), r));
        Ok(stream)
    }

    fn write_message_header(&self, buf: &mut [u8]) {
        let timestamp = self.clock.elapsed().as_nanos();
        buf[0] = ((timestamp >> 48) & 0xFF) as u8;
        buf[1] = ((timestamp >> 40) & 0xFF) as u8;
        buf[2] = ((timestamp >> 32) & 0xFF) as u8;
        buf[3] = ((timestamp >> 24) & 0xFF) as u8;
        buf[4] = ((timestamp >> 16) & 0xFF) as u8;
        buf[5] = ((timestamp >> 8) & 0xFF) as u8;
        buf[6] = ((timestamp >> 0) & 0xFF) as u8;
        buf[7] = 0; // Reset & Status
    }

    /// Send audio over UDP
    fn send(&self, send_buf: &mut [u8]) -> std::io::Result<usize> {
        self.write_message_header(&mut send_buf[..8]);
        let data: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(send_buf[8..].as_mut_ptr() as _, send_buf[8..].len() / 4) };
        let amt = self.buf.flush(data);
        if amt == 0 {
            println!("amt == 0");
            return Ok(0);
        }
        let i = 8 + amt * 4;
        self.sock.send_to(&send_buf[..i], self.dest)
    }


    pub fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf.accumulate(input_buffer)
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
                    crossbeam::channel::TryRecvError::Empty => {},
                    crossbeam::channel::TryRecvError::Disconnected => {
                        panic!("stop stream disconnected");
                    },
                }
            };
            stream.send(&mut buf[..]);
            std::thread::yield_now();
        }
    }
}