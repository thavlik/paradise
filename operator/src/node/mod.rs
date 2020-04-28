use std::cell::RefCell;
use std::rc::Rc;

pub struct AudioUnit {
    class_name: String,
}

impl AudioUnit {
    pub fn new(class_name: String) -> Self {
        Self {
            class_name
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
    pub inputs: Vec<Rc<RefCell<IO>>>,
    pub outputs: Vec<Rc<RefCell<IO>>>,
}

impl Node {
    pub fn new(kind: NodeKind, num_channels: u8) -> Self {
        Self::make(
            kind,
            (0..num_channels).map(|i| Rc::new(RefCell::new(IO::new(
                i,
                false,
                None,
            )))).collect(),
            (0..num_channels).map(|i| Rc::new(RefCell::new(IO::new(
                i,
                true,
                None,
            )))).collect())
    }

    pub fn make(kind: NodeKind, inputs: Vec<Rc<RefCell<IO>>>, outputs: Vec<Rc<RefCell<IO>>>) -> Self {
        Self {
            kind,
            inputs,
            outputs,
        }
    }
}

pub struct IO {
    pub channel: u8,
    pub is_output: bool,
    pub input: Option<Rc<RefCell<IO>>>,
}

impl IO {
    pub fn new(
        channel: u8,
        is_output: bool,
        input: Option<Rc<RefCell<IO>>>,
    ) -> Self {
        Self {
            channel,
            is_output,
            input,
        }
    }
}