use pathfinding::prelude::{absdiff, astar};
use std::sync::{
    Arc,
};

mod node;

use node::{
    Node,
    IO,
    interface::{
        Interface,
        InterfaceIO,
    },
    patchbay::{
        Patchbay,
        PatchbayIO,
    },
};



fn main() {


    let inputs = (0..128).map(|i| Arc::new(InterfaceIO::new(
        i,
        false,
        None,
    ))).collect();
    let outputs = (0..128).map(|i| Arc::new(InterfaceIO::new(
        i,
        true,
        None,
    ))).collect();
    let iface = Interface::new(inputs, outputs);

    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}
