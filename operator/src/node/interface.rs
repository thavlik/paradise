use std::sync::{
    Weak,
    Arc,
};

pub struct InterfaceIO {
    pub channel: u8,
    pub other: Option<super::IO>,
    pub is_output: bool,
}

impl InterfaceIO {
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

pub struct Interface {
    inputs: Vec<Arc<InterfaceIO>>,
    outputs: Vec<Arc<InterfaceIO>>,
}

impl Interface {
    pub fn new(inputs: Vec<Arc<InterfaceIO>>,
               outputs: Vec<Arc<InterfaceIO>>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }

    pub fn new_from_num_channels(num_channels: u8) -> Self {
        Interface::new(
            (0..num_channels).map(|i| Arc::new(InterfaceIO::new(
                i,
                false,
                None,
            ))).collect(),
            (0..num_channels).map(|i| Arc::new(InterfaceIO::new(
                i,
                true,
                None,
            ))).collect())
    }
}