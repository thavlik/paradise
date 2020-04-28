mod interface;
mod patchbay;

use std::sync::Arc;

pub enum Node {
    Interface(Arc<interface::Interface>),
    Patchbay(Arc<patchbay::Patchbay>),
}

pub enum IO {
    InterfaceIO(Arc<interface::InterfaceIO>),
    PatchbayIO(Arc<patchbay::PatchbayIO>),
}

pub trait NodeTrait {
    fn inputs(&self) -> Vec<IO>;

    fn outputs(&self) -> Vec<IO>;
}

