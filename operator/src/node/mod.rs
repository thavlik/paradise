pub mod interface;
pub mod patchbay;

use std::sync::Arc;

pub enum Node {
    Interface(Arc<interface::Interface>),
    Patchbay(Arc<patchbay::Patchbay>),
}


#[derive(Clone)]
pub enum IO {
    InterfaceIO(Arc<interface::InterfaceIO>),
    PatchbayIO(Arc<patchbay::PatchbayIO>),
}

