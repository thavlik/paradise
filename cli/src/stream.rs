use std::net::SocketAddr;

use crate::{Result, TxStream, RxStream};

pub struct Channel {
    host: usize,
    num: usize,
}

pub enum Input {
    Channel(Channel),
    Remote(RxStream),
}

pub enum Output {
    Channel(Channel),
    Remote(TxStream),
}

pub struct Stream {
    source: Input,
    dest: Output,
}

impl Stream {
    pub fn new(source: Input, dest: Output) -> Self {
        Self { source, dest }
    }

    //pub fn from_yaml(doc: &str) -> Result<Self> {
    //    Ok(Stream{
    //        source: Signal::Channel(0),
    //        dest: Signal::Channel(0),
    //    })
    //}
}
