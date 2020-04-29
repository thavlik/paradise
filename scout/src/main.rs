use pathfinding::prelude::{astar};
use std::ops::Deref;
use std::sync::{
    Arc,
    Mutex,
    RwLock,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

mod node;
mod pool;

#[cfg(test)]
mod test;

use node::{
    Node,
    NodeKind,
    IO,
    AudioUnit,
};

fn main() {
    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}

fn reserve(source_uid: Uuid, dest_uid: Uuid) -> Result<Vec<Uuid>, ()> {
    Ok(vec![])
}

fn reserve_on_patchbay(patchbays: &mut Vec<Node>) { //-> (Rc<RefCell<IO>>, Rc<RefCell<IO>>) {
}