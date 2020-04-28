pub type ClassId = u64;

pub struct AudioUnit {
    class_id: ClassId,
}

impl AudioUnit {
    pub fn new(class_id: ClassId) -> Self {
        Self {
            class_id
        }
    }
}

pub enum NodeKind {
    Interface,
    Patchbay,
    Unit(AudioUnit),
}

pub struct Node {
    pub kind: NodeKind,
    pub inputs: Vec<Box<IO>>,
    pub outputs: Vec<Box<IO>>,
}

impl Node {
    pub fn new(kind: NodeKind, num_channels: u8) -> Self {
        Self::make(
            kind,
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

    pub fn make(kind: NodeKind, inputs: Vec<Box<IO>>, outputs: Vec<Box<IO>>) -> Self {
        Self {
            kind,
            inputs,
            outputs,
        }
    }
}

#[derive(Clone)]
pub struct IO {
    pub channel: u8,
    pub is_output: bool,
    pub input: Option<Box<IO>>,
}

impl IO {
    pub fn new(
        channel: u8,
        is_output: bool,
        input: Option<Box<IO>>,
    ) -> Self {
        Self {
            channel,
            is_output,
            input,
        }
    }
}