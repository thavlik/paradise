

struct InputStream {
    channel: usize,
    dest: std::net::SocketAddr,
}

struct OutputStream {
    source: std::net::SocketAddr,
    channel: usize,
}
