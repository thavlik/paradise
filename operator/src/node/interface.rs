use std::sync::{
    Weak,
    Arc,
    Mutex,
};

use super::IO;

pub struct InterfaceIO {
    pub channel: u8,
    pub other: Mutex<Option<IO>>,
    pub is_output: bool,
}

impl InterfaceIO {
    pub fn new(
        channel: u8,
        is_output: bool,
        other: Mutex<Option<IO>>,
    ) -> Self {
        Self {
            channel,
            is_output,
            other,
        }
    }

    pub fn set_other(&self, other: Option<IO>) {
        *self.other.lock().unwrap() = other;
    }
}

pub struct Interface {
    pub inputs: Vec<Arc<InterfaceIO>>,
    pub outputs: Vec<Arc<InterfaceIO>>,
}

impl Interface {
    pub fn new(num_channels: u8) -> Self {
        Self::make(
            (0..num_channels).map(|i| Arc::new(InterfaceIO::new(
                i,
                false,
                Mutex::new(None),
            ))).collect(),
            (0..num_channels).map(|i| Arc::new(InterfaceIO::new(
                i,
                true,
                Mutex::new(None),
            ))).collect())
    }

    pub fn make(inputs: Vec<Arc<InterfaceIO>>,
               outputs: Vec<Arc<InterfaceIO>>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }
}