pub mod block;
pub mod board;
pub mod column;
pub mod frame;
pub mod pit;
pub mod renderer;
pub mod terminal;
pub mod timer;

const NUM_COLS: usize = 6;
const NUM_ROWS: usize = 13;
const PIT_STARTING_X: usize = 10;
const WIDTH: usize = NUM_COLS + PIT_STARTING_X;
const STARTING_X: usize = 2;
const STARTING_Y: usize = 0;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    fn xy(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
