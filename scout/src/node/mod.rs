use std::sync::{Arc, Weak, RwLock};

#[derive(Debug)]
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

#[derive(Debug)]
pub enum NodeKind {
    Interface,
    Patchbay,
    Unit(AudioUnit),
}

#[derive(Debug)]
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
            .map(|i| IO::new(
                i,
                false,
                None,
                Arc::downgrade(&inst),
            ))
            .collect();
        inst.write().unwrap().outputs = (0..num_channels)
            .map(|i| IO::new(
                i,
                true,
                None,
                Arc::downgrade(&inst),
            ))
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

#[derive(Debug)]
pub struct IO {
    pub channel: u8,
    pub is_output: bool,
    pub input: Option<IOHandle>,
    pub node: Weak<RwLock<Node>>,
}

#[derive(Debug, Clone)]
pub struct IOHandle(Arc<RwLock<IO>>);

impl IOHandle {
    pub fn new(io: IO) -> Self {
        Self(Arc::new(RwLock::new(io)))
    }
}

impl PartialEq for IOHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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
        (&self.0.read().unwrap() as *const _ as u64).hash(state);
    }
}

impl IO {
    pub fn new(
        channel: u8,
        is_output: bool,
        input: Option<IOHandle>,
        node: Weak<RwLock<Node>>,
    ) -> IOHandle {
        IOHandle(Arc::new(RwLock::new(Self {
            channel,
            is_output,
            input,
            node,
        })))
    }

    pub fn successors(&self) -> Vec<(IOHandle, u32)> {
        let node = self.node.upgrade().unwrap();
        let node = node.read().unwrap();
        if self.is_output {
            // Can only route to a single input on another
            // piece of hardware or nothing at all.
            self.input.as_ref()
                .map_or(vec![], |v| vec![(v.clone(), 1)])
        } else {
            // Patchbay inputs can route to any output.
            // Inputs for other nodes are terminal.
            match &node.kind {
                NodeKind::Interface => vec![],
                NodeKind::Unit(u) => vec![],
                NodeKind::Patchbay => node.outputs.iter()
                    .map(|h| (h.clone(), 1))
                    .collect(),
            }
        }
    }
}
