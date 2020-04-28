use std::sync::{
    Weak,
    Arc,
    atomic::AtomicBool,
};
use super::IO;

pub struct PatchbayIO {
    pub channel: u8,
    pub is_output: bool,
    pub other: Option<IO>,
}

impl PatchbayIO {
    pub fn new(
        channel: u8,
        is_output: bool,
        other: Option<IO>,
    ) -> Self {
        Self {
            channel,
            is_output,
            other,
        }
    }
}


pub struct Patchbay {
    inputs: Vec<Arc<PatchbayIO>>,
    outputs: Vec<Arc<PatchbayIO>>,
}

impl Patchbay {
    pub fn new(num_channels: u8, input_conn: Vec<(u8, IO)>, output_conn: Vec<(u8, IO)>) -> Self {
        Self::make(
            (0..num_channels).map(|i| Arc::new(PatchbayIO::new(
                i,
                false,
                match input_conn.iter()
                    .find(|(ch, _)| *ch == i) {
                    Some((_, v)) => Some(v.clone()),
                    None => None,
                },
            ))).collect(),
            (0..num_channels).map(|i| Arc::new(PatchbayIO::new(
                i,
                true,
                match output_conn.iter()
                    .find(|(ch, _)| *ch == i) {
                    Some((_, v)) => Some(v.clone()),
                    None => None,
                },
            ))).collect())
    }

    pub fn make(inputs: Vec<Arc<PatchbayIO>>,
               outputs: Vec<Arc<PatchbayIO>>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }
}