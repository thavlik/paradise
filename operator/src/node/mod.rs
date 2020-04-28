pub struct Node {
    pub inputs: Vec<Box<IO>>,
    pub outputs: Vec<Box<IO>>,
}

impl Node {
    pub fn new(num_channels: u8) -> Self {
        Self::make(
            (0..num_channels).map(|i| Box::new(IO::new(
                i,
                false,
                None,
            ))).collect(),
            (0..num_channels).map(|i| Box::new(IO::new(
                i,
                true,
                None,
            ))).collect())
    }

    pub fn make(inputs: Vec<Box<IO>>, outputs: Vec<Box<IO>>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }
}

#[derive(Clone)]
pub struct IO {
    pub channel: u8,
    pub is_output: bool,
    pub other: Option<Box<IO>>,
}

impl IO {
    pub fn new(
        channel: u8,
        is_output: bool,
        other: Option<Box<IO>>,
    ) -> Self {
        Self {
            channel,
            is_output,
            other,
        }
    }
}