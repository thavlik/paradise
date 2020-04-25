pub struct TxStream {
    sock: std::net::UdpSocket,
    dest: std::net::SocketAddr,
    offset: std::time::Instant,
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

pub struct RxStream {
    sock: std::net::UdpSocket,
    parity: std::sync::atomic::AtomicUsize,
    clock: std::sync::atomic::AtomicU64,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

impl RxStream {
    pub fn new(port: usize) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        Ok(Self {
            sock,
            parity: std::default::Default::default(),
            clock: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        })
    }

    /// receive data over the network
    fn receive(&mut self) {
        let mut buf = vec![0; 128];
        let (_amt, _src) = match self.sock.recv_from(&mut buf[..]) {
            Ok(value) => value,
            Err(e) => {
                error!("recv: {:?}", e);
                return;
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
        let parity: usize = self.parity.load(std::sync::atomic::Ordering::SeqCst) % 2;
        let mut buf = self.buf[parity]
            .lock()
            .unwrap();
        let clock = self.clock.load(std::sync::atomic::Ordering::SeqCst);
        let delta = timestamp - clock;
        if delta < 0 {
            // Current timestamp is higher than incoming.
            // Discard this sample.
            warn!("discarding late sample");
            return;
        }
        let status = hdr[7];
        let data = &buf[8..];
        if data.len() % 4 != 0 {
            panic!("data buffer is not divisible by four")
        }
        let num_samples = data.len() / 4;
        let data: &[f32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as _, num_samples) };
        buf.extend_from_slice(data);
    }

    fn cycle(&mut self) -> usize {
        let parity: usize = self.parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let wrapped = parity % 2;
        if parity > 100_000_000 {
            // Wrap parity back to [0, 1] so there's no risk of overflow.
            // fetch_add returns the old value, so the current value will
            // (functionally) be the complement.
            self.parity.store(1 - wrapped, std::sync::atomic::Ordering::SeqCst);
        }
        wrapped
    }

    pub fn process(&mut self, output_buffer: &mut [f32]) {
        let mut buf = self.buf[self.cycle()].lock().unwrap();
        // Take only most recent samples
        let i = buf.len() - output_buffer.len();
        assert_eq!(buf[i..].len(), output_buffer.len());
        output_buffer.copy_from_slice(&buf[i..]);
        buf.clear();
    }
}

impl TxStream {
    pub fn new(
        dest: std::net::SocketAddr,
        outbound_port: u16,
    ) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        Ok(Self {
            sock,
            dest,
            offset: std::time::Instant::now(),
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        })
    }

    fn send(&mut self) -> std::io::Result<usize> {
        let parity: usize = self.parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let wrapped = parity % 2;
        if parity > 100_000_000 {
            // Wrap parity back to [0, 1] so there's no risk of overflow.
            // fetch_add returns the old value, so the current value will
            // (functionally) be the complement.
            self.parity.store(1 - wrapped, std::sync::atomic::Ordering::SeqCst);
        }
        let mut buf = self.buf[wrapped].lock().unwrap();
        let data: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as _, buf.len() * 4) };
        let timestamp = self.offset.elapsed().as_nanos();
        let mut send_buf = vec![
            ((timestamp >> 48) & 0xFF) as u8,
            ((timestamp >> 40) & 0xFF) as u8,
            ((timestamp >> 32) & 0xFF) as u8,
            ((timestamp >> 24) & 0xFF) as u8,
            ((timestamp >> 16) & 0xFF) as u8,
            ((timestamp >> 8) & 0xFF) as u8,
            ((timestamp >> 0) & 0xFF) as u8,
            0, // Reset & Status
        ];
        send_buf.extend_from_slice(data);
        match self.sock.send_to(&send_buf[..], self.dest) {
            Ok(amt) => {
                buf.clear();
                Ok(amt)
            },
            Err(e) => Err(e),
        }
    }

    fn cycle(&mut self) -> usize {
        let parity: usize = self.parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let wrapped = parity % 2;
        if parity > 100_000_000 {
            // Wrap parity back to [0, 1] so there's no risk of overflow.
            // fetch_add returns the old value, so the current value will
            // (functionally) be the complement.
            self.parity.store(1 - wrapped, std::sync::atomic::Ordering::SeqCst);
        }
        wrapped
    }

    pub fn process(&mut self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf[self.cycle()]
            .lock()
            .unwrap()
            .extend_from_slice(input_buffer);
    }
}

pub struct BidirectionalStream {
    tx: TxStream,
    rx: RxStream,
}


impl BidirectionalStream {
    pub fn process(&mut self, input_buffer: &[f32], output_buffer: &mut [f32]) {
        self.tx.process(input_buffer);
        self.rx.process(output_buffer);
    }
}
