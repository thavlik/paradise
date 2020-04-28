use pathfinding::prelude::{absdiff, astar};

mod node;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Pos(i32, i32);

impl Pos {
    fn successors(&self) -> Vec<(Pos, u32)> {
        let &Pos(x, y) = self;
        vec![Pos(x+1,y+2), Pos(x+1,y-2), Pos(x-1,y+2), Pos(x-1,y-2),
             Pos(x+2,y+1), Pos(x+2,y-1), Pos(x-2,y+1), Pos(x-2,y-1)]
            .into_iter().map(|p| (p, 1)).collect()
    }
}

static GOAL: Pos = Pos(4, 6);

fn main() {
    let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
                       |p| *p == GOAL);
    assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}
