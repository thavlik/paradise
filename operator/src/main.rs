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
    let mut iface = Interface::new(8);

    (0..4).map(|i| Patchbay::new(128));

    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}
