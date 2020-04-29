use std::default::Default;
use std::{sync::{atomic::{AtomicPtr, Ordering::SeqCst}}, time::{SystemTime}};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::pool::Claim;

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
    pub uid: Uuid,
    pub kind: NodeKind,
    pub inputs: Vec<Box<IO>>,
    pub outputs: Vec<Box<IO>>,
}

impl Node {
    pub fn new(kind: NodeKind, num_channels: u8) -> Box<Self> {
        let mut inst = Box::new(Node {
            uid: Uuid::new_v4(),
            kind,
            inputs: vec![],
            outputs: vec![],
        });
        let inputs = (0..num_channels)
            .map(|i| Box::new(IO::new(
                Uuid::new_v4(),
                i,
                false,
                None,
                &*inst as _,
                Default::default(),
            )))
            .collect::<Vec<_>>();
        let outputs = (0..num_channels)
            .map(|i| Box::new(IO::new(
                Uuid::new_v4(),
                i,
                true,
                None,
                &*inst as _,
                Default::default(),
            )))
            .collect::<Vec<_>>();
        inst as _
    }

    pub fn make(uid: Uuid, kind: NodeKind, inputs: Vec<Box<IO>>, outputs: Vec<Box<IO>>) -> Box<Self> {
        Box::new(Self {
            uid,
            kind,
            inputs,
            outputs,
        })
    }
}

#[derive(Debug)]
pub struct IO {
    pub uid: Uuid,
    pub channel: u8,
    pub is_output: bool,
    pub input: Option<*const Self>,
    pub node: *const Node,
    pub claim: AtomicPtr<Claim>,
}

impl PartialEq for IO {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for IO {}

impl std::hash::Hash for IO {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state)
    }
}



impl std::ops::Drop for IO {
    fn drop(&mut self) {
        let claim = self.claim.load(SeqCst);
        if !std::ptr::eq(claim, std::ptr::null()) {
            unsafe {
                Box::from_raw(claim);
            }
        }
    }
}

impl IO {
    pub fn new(
        uid: Uuid,
        channel: u8,
        is_output: bool,
        input: Option<*const Self>,
        node: *const Node,
        claim: Option<Claim>,
    ) -> Box<Self> {
        Box::new(Self {
            uid,
            channel,
            is_output,
            input,
            node,
            claim: match claim {
                Some(claim) => AtomicPtr::new(Box::into_raw(Box::new(claim))),
                None => Default::default(),
            },
        })
    }

    pub fn successors(&self) -> Vec<(*const Self, u32)> {
        let result = {
            if self.is_output {
                // Can only route to a single input on another
                // piece of hardware or nothing at all.
                self.input.as_ref()
                    .map_or(vec![], |v| vec![(v.clone(), 1)])
            } else {
                // Patchbay inputs can route to any output.
                // Inputs for other nodes are terminal.
                unsafe {
                    match &(*self.node).kind {
                        NodeKind::Interface => vec![],
                        NodeKind::Unit(u) => vec![],
                        NodeKind::Patchbay => (*self.node).outputs.iter()
                            .map(|h| (&**h as _, 1))
                            .collect(),
                    }
                }
            }
        };
        result
    }
}
