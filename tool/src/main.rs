fn main() {
    let port: u16 = 30001;
    println!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port);
    let sock = std::net::UdpSocket::bind(&addr).unwrap();
    const BUFFER_SIZE: usize = 256_000;
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    let send_addrs: Vec<_> = (30000..30001).map(|port| {
        std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), port))
    }).collect();

    loop {
        let (amt, src) = match sock.recv_from(&mut buf[..]) {
            Ok(value) => value,
            Err(e) => {
                println!("recv_from: {:?}", e);
                std::thread::yield_now();
                continue
            }
        };
        let data = &buf[8..amt];
        let all_zero = data.iter().all(|v| *v == 0);
        println!("active={}, sending to {:?}", !all_zero, send_addrs);
        send_addrs.iter()
            .for_each(|addr| {
                match sock.send_to(&buf[..amt], &addr) {
                    Ok(_) => return,
                    Err(e) => {
                        println!("send_to {:?}: {:?}", addr, e);
                        return;
                    },
                }
            });
        std::thread::yield_now();
    }
}
