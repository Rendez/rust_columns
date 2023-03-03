use std::time::Duration;

use crate::{
    block::{Block, BlockKind},
    frame::{Drawable, Frame},
    pit::Heap,
    timer::Timer,
    Vec2, NUM_COLS, NUM_ROWS, STARTING_X, STARTING_Y,
};
use rand::{distributions::Uniform, thread_rng, Rng};

type Shaft = [Block; 3];

#[derive(Debug)]
pub struct Column {
    shaft: Shaft,
    x: usize,
    y: usize,
    dropping: bool,
    move_timer: Timer,
}

impl Column {
    pub const MOVE_MILLIS: u64 = 1000;

    pub fn new() -> Self {
        let blocks = thread_rng()
            .sample_iter(Uniform::<u8>::new_inclusive(1, 6))
            .take(3)
            .map(|index| -> Block {
                let kind = match index {
                    1 => Some(BlockKind::Blue),
                    2 => Some(BlockKind::Yellow),
                    3 => Some(BlockKind::Green),
                    4 => Some(BlockKind::Red),
                    5 => Some(BlockKind::Cyan),
                    6 => Some(BlockKind::Magenta),
                    _ => None,
                };
                Block::new(kind)
            })
            .collect::<Vec<Block>>();

        Self::from([blocks[0], blocks[1], blocks[2]])
    }

    pub fn from(shaft: Shaft) -> Self {
        Self {
            shaft,
            ..Column::default()
        }
    }

    pub fn cycle(&mut self) {
        if self.dropping {
            self.shaft.rotate_right(1);
        }
    }

    pub fn move_down(&mut self, heap: &Heap) {
        if !self.detect_hit_downwards(heap) {
            self.y += 1;
        }
    }

    pub fn move_left(&mut self, heap: &Heap) {
        if !self.detect_hit_leftwards(heap) {
            self.x -= 1;
        }
    }

    pub fn move_right(&mut self, heap: &Heap) {
        if !self.detect_hit_rightwards(heap) {
            self.x += 1;
        }
    }

    pub fn detect_landing(&mut self, heap: &mut Heap, delta: Duration) -> Option<Vec<Vec2>> {
        if self.detect_hit_downwards(heap) {
            // reached the bottom of the pit or there is a upcoming hit with an existing block
            let mut move_timer_copy = self.move_timer;
            move_timer_copy.update(delta);
            // we will be ready when the timer finishes, to give the player
            // the chance to cycle the column before we have fully landed
            if move_timer_copy.ready {
                // now that we have landed, we copy the blocks into our matrix of blocks
                self.dropping = false;
                // transfer shaft block to heap of blocks
                let mut origins = Vec::new();
                for (i, block) in self.shaft.into_iter().rev().enumerate() {
                    if i > self.y {
                        // y points to the base block, if any above it are out of the matrix, stop transfer to teh heap.
                        break;
                    }
                    let origin = Vec2::xy(self.x, self.y - i);
                    heap[origin.x][origin.y] = block;
                    origins.push(origin);
                }
                return Some(origins);
            }
        }
        None
    }

    pub fn update(&mut self, heap: &Heap, delta: Duration) -> bool {
        self.move_timer.update(delta);
        if self.move_timer.ready {
            self.move_timer.reset();
            self.move_down(heap);
        }
        self.dropping
    }

    fn detect_hit_downwards(&self, heap: &Heap) -> bool {
        self.dropping && (self.y == NUM_ROWS - 1 || !heap[self.x][self.y + 1].empty())
    }

    fn detect_hit_leftwards(&self, heap: &Heap) -> bool {
        self.dropping && (self.x == 0 || !heap[self.x - 1][self.y].empty())
    }

    fn detect_hit_rightwards(&self, heap: &Heap) -> bool {
        self.dropping && (self.x == NUM_COLS - 1 || !heap[self.x + 1][self.y].empty())
    }
}

impl Default for Column {
    fn default() -> Self {
        Self {
            shaft: [Block::default(), Block::default(), Block::default()],
            x: STARTING_X,
            y: STARTING_Y,
            dropping: true,
            move_timer: Timer::from_millis(Column::MOVE_MILLIS),
        }
    }
}

impl Drawable for Column {
    fn draw(&self, frame: &mut Frame) {
        // Since it's already transfered to the heap of blocks,
        // we do not want to draw it on top unless it's still moving
        if self.dropping {
            for (i, block) in self.shaft.iter().rev().enumerate() {
                if i > self.y {
                    // since it starts at y=0, do not draw the first two blocks as they would have negative y's
                    break;
                }
                frame[self.x][self.y - i] = block.numbered();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{
        block::{Block, BlockKind},
        column::Column,
        pit::{Heap, Pit},
        Vec2, NUM_ROWS, STARTING_X, STARTING_Y,
    };

    const DELTA: Duration = Duration::from_millis(Column::MOVE_MILLIS);

    #[test]
    fn test_new() {
        let col = Column::new();

        assert_eq!(col.x, STARTING_X);
        assert_eq!(col.y, STARTING_Y);
        assert!(col.dropping);

        const MAX_COUNT: u8 = 5;
        let mut count_cmp = MAX_COUNT;
        loop {
            let shaft_cmp = &Column::new().shaft;
            let different = col
                .shaft
                .iter()
                .enumerate()
                .any(|(i, block)| block != &shaft_cmp[i]);

            if different {
                break;
            }
            count_cmp -= 1;

            assert!(
                count_cmp > 0,
                "two columns instances were equal after {MAX_COUNT} comparisons"
            );
        }
    }

    #[test]
    fn test_cycle() {
        let mut col = Column::new();
        let shaft_copy = col.shaft;
        col.cycle();
        let shaft = col.shaft;

        assert_eq!(shaft_copy[0], shaft[1]);
        assert_eq!(shaft_copy[1], shaft[2]);
        assert_eq!(shaft_copy[2], shaft[0]);
    }

    #[test]
    fn test_update() {
        let heap = Pit::new_heap(None);
        let mut col = Column::new();

        col.update(&heap, Duration::from_millis(Column::MOVE_MILLIS - 1));
        assert_eq!(col.y, 0);
        col.update(&heap, Duration::from_millis(1));
        assert_eq!(col.y, 1);
    }

    #[test]
    fn test_landing_on_heap() {
        let mut heap: Heap = Pit::new_heap(None);
        let mut col = Column::new();

        assert_eq!(col.detect_landing(&mut heap, DELTA), None);

        heap[STARTING_X][STARTING_Y + 1] = Block::new(Some(BlockKind::Blue));

        assert_eq!(
            col.detect_landing(&mut heap, DELTA),
            Some(vec![Vec2::xy(STARTING_X, STARTING_Y)])
        );
    }

    #[test]
    fn test_landing_reached_bottom() {
        let mut heap: Heap = Pit::new_heap(None);
        let mut col = Column::new();

        assert_eq!(col.detect_landing(&mut heap, DELTA), None);

        for _ in 1..NUM_ROWS {
            col.move_down(&heap);
        }

        assert_eq!(
            col.detect_landing(&mut heap, DELTA),
            Some(vec![
                Vec2::xy(STARTING_X, NUM_ROWS - 1),
                Vec2::xy(STARTING_X, NUM_ROWS - 2),
                Vec2::xy(STARTING_X, NUM_ROWS - 3)
            ])
        );
    }
}
