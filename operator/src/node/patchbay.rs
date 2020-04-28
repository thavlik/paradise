use std::sync::{
    Weak,
    Arc,
    atomic::AtomicBool,
};

pub struct PatchbayIO {
    pub channel: u8,
    pub is_output: bool,
    pub other: Option<super::IO>,
}

impl PatchbayIO {
    pub fn new(
        channel: u8,
        is_output: bool,
        other: Option<super::IO>,
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
    pub fn new(inputs: Vec<Arc<PatchbayIO>>,
               outputs: Vec<Arc<PatchbayIO>>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }

    pub fn new_from_num_channels(num_channels: u8) -> Self {
        Patchbay::new(
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
}