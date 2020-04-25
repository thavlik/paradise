struct InputStream {
    channel: usize,
    sock: std::net::UdpSocket,
    dest: std::net::SocketAddr,
    offset: std::time::Instant,
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

struct OutputStream {
    channel: usize,
    sock: std::net::UdpSocket,
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

impl OutputStream {
    fn new(port: usize, channel: usize) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        Ok(Self {
            channel,
            sock,
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        })
    }

    /// receive data over the network
    fn receive(&mut self) {
        // TODO: accumulate data into the buffer according to timestamp
        let mut buf = vec![0; 128];
        let (amt, src) = match self.sock.recv_from(&mut buf[..]) {
            Ok(value) => value,
            Err(e) => {
                error!("recv: {:?}", e);
                return;
            }
        };
        let hdr = &buf[..8];
        let timestamp = ((hdr[0] as u64) << 5) |
            ((hdr[1] as u64) << 4) |
            ((hdr[2] as u64) << 3) |
            ((hdr[3] as u64) << 2) |
            ((hdr[4] as u64) << 1) |
            ((hdr[5] as u64) << 0);
        let reset = (hdr[6] & 0x80) != 0;
        let status = hdr[6] & 0x80;
        // TODO: ensure data is aligned to 4 (?) byte address
        let data = &buf[8..];
        if data.len() % 4 != 0 {
            panic!("data buffer is not divisible by four")
        }
        let num_samples = data.len() / 4;
        let data: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
        // Copy data over into shared buffer
        let parity: usize = self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2;
        let mut buf = self.buf[parity].lock().unwrap();
        buf.extend_from_slice(data);
    }

    /// write data to the audio interface
    fn process(&self, buffer: &mut Vec<f32>) {
        let parity: usize = self.parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % 2;
        let mut buf = self.buf[parity].lock().unwrap();
        buffer.copy_from_slice(&buf[..]);
        buf.clear();
    }
}

impl InputStream {
    fn new(
        dest: std::net::SocketAddr,
        channel: usize,
        outbound_port: u16,
    ) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        Ok(Self {
            channel,
            sock,
            dest,
            offset: std::time::Instant::now(),
        })
    }

    fn send(&mut self) {
        let parity: usize = self.parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % 2;
        let mut buf = self.buf[parity].lock().unwrap();
        let data: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as _, num_samples) };
        buf.clear();
        //self.sock.send_to(, self.dest);
    }

    fn process(&mut self, buffer: &mut vst::buffer::AudioBuffer<f32>) {
        // buffer is received from the audio interface
        // send the data over the socket
        //match sock.send_to(&buffer[..], self.dest) {
        //    Ok(amt) => {},
        //    Err(e) => {},
        //}
        // TODO: serialize the buffer
        let timestamp = self.offset.elapsed().as_nanos();
    }
}
