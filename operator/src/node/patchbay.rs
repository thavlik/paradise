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
   pub inputs: Vec<Arc<PatchbayIO>>,
   pub outputs: Vec<Arc<PatchbayIO>>,
}

impl Patchbay {
    pub fn new(num_channels: u8) -> Self {
        Self::make(
            (0..num_channels).map(|i| Arc::new(PatchbayIO::new(
                i,
                false,
                None,
            ))).collect(),
            (0..num_channels).map(|i| Arc::new(PatchbayIO::new(
                i,
                true,
                None,
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