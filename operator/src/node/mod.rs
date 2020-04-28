use std::sync::{Arc, Weak, RwLock};

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

pub type NodeHandle = Arc<RwLock<Node>>;

impl Node {
    pub fn new(kind: NodeKind, num_channels: u8) -> NodeHandle {
        let mut inst = Arc::new(RwLock::new(Node {
            kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }));
        inst.write().unwrap().inputs = (0..num_channels)
            .map(|i| IOHandle::new(IO::new(
                i,
                false,
                None,
                Arc::downgrade(&inst),
            )))
            .collect();
        inst.write().unwrap().outputs = (0..num_channels)
            .map(|i| IOHandle::new(IO::new(
                i,
                true,
                None,
                Arc::downgrade(&inst),
            )))
            .collect();
        inst as _
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
    pub node: Weak<RwLock<Node>>,
}

#[derive(Clone)]
pub struct IOHandle(Arc<RwLock<IO>>);

impl IOHandle {
    pub fn new(io: IO) -> Self {
        Self(Arc::new(RwLock::new(io)))
    }
}

impl PartialEq for IOHandle {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as _, other as _)
    }
}

impl Eq for IOHandle {}

impl std::ops::Deref for IOHandle {
    type Target = Arc<RwLock<IO>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
        node: Weak<RwLock<Node>>,
    ) -> Self {
        Self {
            channel,
            is_output,
            input,
            node,
        }
    }

    pub fn successors(&self) -> Vec<(IOHandle, u32)> {
        let node = self.node.upgrade().unwrap();
        let node = node.read().unwrap();
        match self.is_output {
            true => node.inputs.iter()
                .map(|h| (h.clone(), 1))
                .collect(),
            false => node.outputs.iter()
                .map(|h| (h.clone(), 1))
                .collect(),
        }
    }
}
