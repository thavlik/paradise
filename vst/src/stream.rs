use tokio::runtime;

pub struct TxStream {
    sock: std::net::UdpSocket,
    dest: std::net::SocketAddr,
    clock: std::time::Instant,
    parity: std::sync::atomic::AtomicUsize,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

async fn tx_entry(stream: std::sync::Weak<TxStream>) {
    loop {
        match stream.upgrade() {
            Some(stream) => {
                stream.send();
            },
            None => {
                return;
            },
        };
        std::thread::yield_now();
    }
}

pub struct RxStream {
    sock: std::net::UdpSocket,
    parity: std::sync::atomic::AtomicUsize,
    clock: std::sync::atomic::AtomicU64,
    buf: [Box<std::sync::Mutex<Vec<f32>>>; 2],
}

async fn rx_entry(stream: std::sync::Weak<RxStream>) {
    const RECEIVE_BUFFER_SIZE: usize = 256_000;
    let mut buf: [u8; RECEIVE_BUFFER_SIZE] =  [0; RECEIVE_BUFFER_SIZE];
    loop {
        match stream.upgrade() {
            Some(stream) => {
                stream.receive(&mut buf[..]);
            },
            None => {
                return;
            }
        };
        std::thread::yield_now();
    }
}

impl RxStream {
    pub fn new(port: usize, rt: &runtime::Runtime) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = std::net::UdpSocket::bind(&addr)?;
        let stream = std::sync::Arc::new(Self {
            sock,
            parity: std::default::Default::default(),
            clock: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        });
        rt.spawn(rx_entry(std::sync::Arc::downgrade(&stream)));
        Ok(stream)
    }

    /// Receive data over the network. A thread is supposed
    /// to call this repeatedly to ensure the socket is
    /// quickly synchronized with the output buffer.
    fn receive(&self, buf: &mut [u8]) {
        let (_amt, _src) = match self.sock.recv_from(buf) {
            Ok(value) => value,
            Err(e) => {
                error!("recv_from: {:?}", e);
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

    fn cycle(&self) -> usize {
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

    pub fn process(&self, output_buffer: &mut [f32]) {
        let mut buf = self.buf[self.cycle()]
            .lock()
            .unwrap();
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
        rt: &runtime::Runtime,
    ) -> std::io::Result<std::sync::Arc<Self>> {
        let addr = format!("0.0.0.0:{}", outbound_port);
        let sock = std::net::UdpSocket::bind(addr)?;
        let stream = std::sync::Arc::new(Self {
            sock,
            dest,
            clock: std::time::Instant::now(),
            parity: std::default::Default::default(),
            buf: [
                Box::new(std::sync::Mutex::new(Vec::new())),
                Box::new(std::sync::Mutex::new(Vec::new()))
            ],
        });
        rt.spawn(tx_entry(std::sync::Arc::downgrade(&stream)));
        Ok(stream)
    }

    /// Send audio over UDP
    fn send(&self) -> std::io::Result<usize> {
        let timestamp = self.clock.elapsed().as_nanos();
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
        {
            let mut buf = self.buf[self.cycle()].lock().unwrap();
            if buf.len() == 0 {
                // Don't send empty messages
                return Ok(0);
            }
            let data: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as _, buf.len() * 4) };
            send_buf.extend_from_slice(data);
            buf.clear();
        }
        self.sock.send_to(&send_buf[..], self.dest)
    }

    fn cycle(&self) -> usize {
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

    pub fn process(&self, input_buffer: &[f32]) {
        // Accumulate the samples in the send buffer
        self.buf[self.cycle()]
            .lock()
            .unwrap()
            .extend_from_slice(input_buffer);
    }
}
