use pathfinding::prelude::astar;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;
use node::{AudioUnit, Node, NodeKind, IO};

pub mod rk;

mod node;
mod pool;

#[cfg(test)]
mod test;


fn main() {
    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}
