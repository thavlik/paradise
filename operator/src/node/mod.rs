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
    pub inputs: Vec<IOHandle>,
    pub outputs: Vec<IOHandle>,
}

impl Node {
    pub fn new(kind: NodeKind, num_channels: u8) -> Self {
        Self::make(
            kind,
            (0..num_channels).map(|i| IOHandle::new(IO::new(
                i,
                false,
                None,
            ))).collect(),
            (0..num_channels).map(|i| IOHandle::new(IO::new(
                i,
                true,
                None,
            ))).collect())
    }

    pub fn make(kind: NodeKind, inputs: Vec<IOHandle>, outputs: Vec<IOHandle>) -> Self {
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
    pub input: Option<IOHandle>,
}

#[derive(Clone)]
pub struct IOHandle(Rc<RefCell<IO>>);

impl IOHandle {
    pub fn new(io: IO) -> Self {
        Self(Rc::new(RefCell::new(io)))
    }
}

impl std::ops::Deref for IOHandle {
    type Target = Rc<RefCell<IO>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for IOHandle {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as _, other as _)
    }
}

impl Eq for IOHandle {}

impl std::hash::Hash for IOHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const _ as u64).hash(state);
    }
}

impl IO {
    pub fn new(
        channel: u8,
        is_output: bool,
        input: Option<IOHandle>,
    ) -> Self {
        Self {
            channel,
            is_output,
            input,
        }
    }

    pub fn successors(&self) -> Vec<(IOHandle, u32)> {
        vec![]
    }
}