fn main() {
    let port: u16 = 30001;
    println!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port);
    let sock = std::net::UdpSocket::bind(&addr).unwrap();
    const BUFFER_SIZE: usize = 256_000;
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    loop {
        let (amt, _src) = match sock.recv_from(&mut buf[..]) {
            Ok(value) => value,
            Err(e) => {
                println!("recv_from: {:?}", e);
                continue
            }
        };
        println!("received {} bytes: {:?}", amt, &buf[..amt]);
        std::thread::yield_now();
    }
}
