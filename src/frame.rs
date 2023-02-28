use crate::{NUM_COLS, NUM_ROWS};

pub type Frame = [[i8; NUM_ROWS]; NUM_COLS];

pub fn new_frame() -> Frame {
    [[-1; NUM_ROWS]; NUM_COLS]
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
