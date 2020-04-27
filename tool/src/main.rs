use std::io::prelude::*;

fn main_udp() {
    let port: u16 = 30001;
    println!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port);
    let sock = std::net::UdpSocket::bind(&addr).unwrap();
    const BUFFER_SIZE: usize = 256_000;
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    let send_addrs: Vec<_> = (30005..30006).map(|port| {
        std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), port))
    }).collect();

    loop {
        let (amt, src) = match sock.recv_from(&mut buf[..]) {
            Ok(value) => value,
            Err(e) => {
                println!("recv_from: {:?}", e);
                std::thread::yield_now();
                continue
            },
        };
        let data = &buf[8..amt];
        let all_zero = data.iter().all(|v| *v == 0);
        //println!("active={} {} => {:?}", !all_zero, &addr, send_addrs);
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

fn main_tcp() {
    const BUFFER_SIZE: usize = 256_000;
    let mut recv_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let port: u16 = 30001;
    println!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&addr).unwrap();
    let mut stream: Option<(std::net::TcpStream, std::net::SocketAddr)> = None;
    loop {
        std::thread::yield_now();
        match listener.accept() {
            Ok((incoming, addr)) => {
                // Existing connection is closed when stream is dropped
                incoming.set_nonblocking(true).unwrap();
                stream = Some((incoming, addr));
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => {},
                _ => {
                    println!("listener.accept(): {:?}", e);
                },
            }
        };
        if stream.is_none() {
            continue;
        }
        match stream.as_mut() {
            Some((stream, addr)) => {
                let amt = match stream.read(&mut recv_buf[..]) {
                    Ok(amt) => amt,
                    Err(e) => {
                        println!("read: {:?}", e);
                        continue;
                    },
                };
                println!("echo {:?}", &recv_buf[..amt]);
                match stream.write(&recv_buf[..amt]) {
                    Ok(amt) => {},
                    Err(e) => {
                        println!("write: {:?}", e);
                        continue;
                    },
                }
            },
            None => {
                println!("rx: no stream");
            },
        }
    }
}

fn main() {
    main_udp()
}

