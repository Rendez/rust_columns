use std::cmp::min;
use std::time::Duration;

use crate::{
    block::{Block, BlockKind},
    column::Column,
    frame::{Drawable, Frame},
    timer::Timer,
    Vec2, NUM_COLS, NUM_ROWS,
};

pub type Heap = [[Block; NUM_ROWS]; NUM_COLS];

#[derive(Debug)]
pub enum CardinalAxis {
    Default,
    NxS,
    ExW,
    NExSW,
    NWxSE,
}

impl Default for CardinalAxis {
    fn default() -> Self {
        CardinalAxis::Default
    }
}

impl Iterator for CardinalAxis {
    type Item = CardinalAxis;

    fn next(&mut self) -> Option<Self::Item> {
        use CardinalAxis::*;
        match *self {
            Default => {
                *self = NxS;
                Some(NxS)
            }
            NxS => {
                *self = ExW;
                Some(ExW)
            }
            ExW => {
                *self = NExSW;
                Some(NExSW)
            }
            NExSW => {
                *self = NWxSE;
                Some(NWxSE)
            }
            NWxSE => None,
        }
    }
    // fn turn(&self) -> Option<Self> {
    //     use CardinalAxis::*;
    //     match *self {
    //         NxS => Some(ExW),
    //         ExW => Some(NExSW),
    //         NExSW => Some(NWxSE),
    //         NWxSE => None,
    //     }
    // }
}

#[derive(Debug, PartialEq)]
enum PitStage {
    Stable,
    Matching,
    Collecting,
    Dropping,
}

// impl Default for PitStage {
//     fn default() -> Self {
//         Self::Matching
//     }
// }

pub struct PitState {
    stage: PitStage,
    move_timer: Timer,
    times: u8,
}

impl Default for PitState {
    fn default() -> Self {
        Self {
            stage: PitStage::Stable,
            move_timer: Timer::from_millis(Self::MOVE_MILLIS),
            times: 0,
        }
    }
}

impl PitState {
    const MOVE_MILLIS: u64 = 1000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_dropping_at<const R: usize, const C: usize>(
        &self,
        heap: &mut [[Block; R]; C],
        origins: &mut [Vec2],
    ) -> bool {
        let mut something_dropped = false;
        // drop all active blocks one step if they have a slot for that
        for origin in origins.iter_mut() {
            if origin.y < R - 1 && heap[origin.x][origin.y + 1].empty() {
                // the slot below is empty, let's drop it we can drop one level!
                // and let's update things accordingly in the heap
                let new_item = heap[origin.x][origin.y].to_owned();
                heap[origin.x][origin.y].update(None);
                origin.y += 1;
                heap[origin.x][origin.y] = new_item;
                something_dropped = true;
            }
        }

        something_dropped
    }

    /*
     *
     */
    pub fn collect_dropping_at<const R: usize, const C: usize>(
        &self,
        heap: &[[Block; R]; C],
        origins: &[Vec2],
    ) -> Vec<Vec2> {
        let mut items = Vec::new();

        for origin in origins {
            for y in (0..origin.y).rev() {
                if heap[origin.x][y].empty() {
                    break;
                }
                items.push(Vec2::xy(origin.x, y));
            }
        }

        // sort by highest 'y' points first, so we don't run into troubles when updating next...
        items.sort_unstable_by(|a, b| b.y.cmp(&a.y));

        items
    }

    pub fn collect_matching_at<const R: usize, const C: usize>(
        &self,
        heap: &[[Block; R]; C],
        origins: &[Vec2],
    ) -> Vec<Vec2> {
        let mut items = Vec::new();
        let mut cache = [[false; R]; C];

        for origin in origins {
            for item in self.matching_at(heap, origin) {
                if !cache[item.x][item.y] {
                    cache[item.x][item.y] = true;
                    items.push(item);
                }
            }
        }

        items
    }

    fn matching_at<const R: usize, const C: usize>(
        &self,
        heap: &[[Block; R]; C],
        origin: &Vec2,
    ) -> Vec<Vec2> {
        let mut items = Vec::new();
        let origin_item = heap[origin.x][origin.y];

        if origin_item.empty() {
            return items;
        }

        let cardinal_axis = CardinalAxis::default();

        for axis in cardinal_axis {
            // CardinalAxis.iter()
            let mut matches: Vec<Vec2> = Vec::new();

            match axis {
                CardinalAxis::Default => (),
                CardinalAxis::NxS => {
                    // north (N)
                    for y in (0..origin.y).rev() {
                        if heap[origin.x][y] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x, y));
                    }
                    // south (S)
                    for y in (origin.y + 1)..R {
                        if heap[origin.x][y] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x, y));
                    }
                }
                CardinalAxis::ExW => {
                    // west (W)
                    for x in (0..origin.x).rev() {
                        if heap[x][origin.y] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(x, origin.y));
                    }
                    // east (E)
                    #[allow(clippy::needless_range_loop)]
                    for x in (origin.x + 1)..C {
                        if heap[x][origin.y] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(x, origin.y));
                    }
                }
                CardinalAxis::NExSW => {
                    // northeast (NE)
                    for i in 1..min(C - origin.x, origin.y + 1) {
                        if heap[origin.x + i][origin.y - i] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x + i, origin.y - i));
                    }
                    // southwest (SW)
                    for i in 1..min(R - origin.y, origin.x + 1) {
                        if heap[origin.x - i][origin.y + i] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x - i, origin.y + i));
                    }
                }
                CardinalAxis::NWxSE => {
                    // northwest (NW)
                    for i in 1..=min(origin.x, origin.y) {
                        if heap[origin.x - i][origin.y - i] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x - i, origin.y - i));
                    }
                    // southeast (SE)
                    for i in 1..min(C - origin.x, R - origin.y) {
                        if heap[origin.x + i][origin.y + i] != origin_item {
                            break;
                        }
                        matches.push(Vec2::xy(origin.x + i, origin.y + i));
                    }
                }
            }
            if matches.len() >= 2 {
                items.append(&mut matches);
            }
        }

        if !items.is_empty() {
            items.push(origin.to_owned());
        }

        items
    }
}

pub struct Pit {
    pub heap: Heap,
    state: PitState,
    active_origins: Vec<Vec2>,
}

impl Default for Pit {
    fn default() -> Self {
        Self {
            heap: Self::new_heap(None),
            active_origins: Vec::new(),
            state: PitState::new(),
        }
    }
}

impl Pit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_heap<const R: usize, const C: usize>(
        block_kind: Option<BlockKind>,
    ) -> [[Block; R]; C] {
        [[Block::new(block_kind); R]; C]
    }

    pub fn update(&mut self, column: &mut Column, delta: Duration) {
        use PitStage::*;

        match &self.state.stage {
            Stable => {
                self.state.move_timer.update(delta);

                if self.state.move_timer.ready {
                    if let Some(origins) = column.landing(&mut self.heap) {
                        self.active_origins = origins;
                        self.state.stage = Matching;
                    } else {
                        self.state.move_timer.reset();
                    }
                }
            }
            Matching => {
                self.active_origins = self
                    .state
                    .collect_matching_at(&self.heap, &self.active_origins);

                self.state.stage = if self.active_origins.is_empty() {
                    Stable
                } else {
                    Collecting
                };
            }
            Collecting => {
                if self.state.times == 3 {
                    self.state.times = 0;

                    for item in self.active_origins.iter() {
                        self.heap[item.x][item.y].exploding = false;
                        self.heap[item.x][item.y].update(None);
                    }

                    self.active_origins = self
                        .state
                        .collect_dropping_at(&self.heap, &self.active_origins);

                    self.state.stage = if self.active_origins.is_empty() {
                        Stable
                    } else {
                        Dropping
                    };
                } else {
                    self.state.move_timer.update(delta);

                    if self.state.move_timer.ready {
                        self.state.move_timer.reset();
                        self.state.times += 1;

                        if !self.active_origins.is_empty() {
                            let exploding = self.state.times % 2 != 0;

                            for item in self.active_origins.iter() {
                                self.heap[item.x][item.y].exploding = exploding;
                            }
                        }
                    }
                }
            }
            Dropping => {
                self.state.move_timer.update(delta);

                if self.state.move_timer.ready {
                    self.state.move_timer.reset();

                    if !self
                        .state
                        .update_dropping_at(&mut self.heap, &mut self.active_origins)
                    {
                        self.state.stage = Matching;
                    }
                }
            }
        }
    }

    pub fn topped_up(&self) -> bool {
        self.stable() && self.heap.iter().any(|c| !c[0].empty())
    }

    pub fn stable(&self) -> bool {
        self.state.stage == PitStage::Stable
    }
}

impl Drawable for Pit {
    fn draw(&self, frame: &mut Frame) {
        for (x, cols) in self.heap.iter().enumerate() {
            for (y, block) in cols.iter().enumerate() {
                frame[x][y] = block.numbered();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::block::BlockKind;

    type Heap = [[Block; 3]; 3];

    mod test_stage_transition {
        use super::*;

        const DELTA: Duration = Duration::from_millis(0);

        #[test]
        fn test_update_stable_stage() {
            let mut pit = Pit::new();

            assert!(pit.stable());

            let mut col = Column::from([
                Block::default(),
                Block::new(Some(BlockKind::Blue)),
                Block::new(Some(BlockKind::Blue)),
            ]);
            for _ in 1..NUM_ROWS {
                col.move_down(&pit.heap);
            }

            pit.update(&mut col, DELTA);
            pit.update(&mut col, DELTA);

            assert!(pit.stable());
        }

        #[test]
        fn test_update_matching_stage() {
            let mut pit = Pit::new();

            assert!(pit.stable());

            let mut col = Column::from([
                Block::new(Some(BlockKind::Blue)),
                Block::new(Some(BlockKind::Blue)),
                Block::new(Some(BlockKind::Blue)),
            ]);
            for _ in 1..NUM_ROWS {
                col.move_down(&pit.heap);
            }
            pit.update(&mut col, DELTA);

            assert!(!pit.stable());
        }
    }

    mod test_collect_matching {
        use super::*;

        #[test]
        fn test_collect_matching_empty() {
            //
            // ┌─┬─┬─┐
            // │▒│ │ │
            // ├─┼─┤─┤
            // │▒│ │ │
            // ├─┼─┼─┤
            // │ │ │ │
            // └─┴─┴─┘
            //
            let pit_state = PitState::new();
            let mut heap: [[Block; 3]; 3] = Pit::new_heap(None);
            let origins = [Vec2::xy(0, 0), Vec2::xy(0, 1)];
            for origin in origins {
                heap[origin.x][origin.y] = Block::new(Some(BlockKind::Blue));
            }
            let items = pit_state.collect_matching_at(&heap, &origins);

            assert!(items.is_empty());
        }

        #[test]
        fn test_collect_matching_north_south() {
            // ┌─┬─┬─┐
            // │▒│ │ │
            // ├─┼─┤─┤
            // │▒│ │ │
            // ├─┼─┼─┤
            // │▒│ │ │
            // └─┴─┴─┘
            let assert_items = |pit_state: &PitState, heap: &Heap, origins: &[Vec2]| {
                let items = pit_state.collect_matching_at(heap, origins);

                assert_eq!(items.len(), 3);

                for origin in origins {
                    assert!(items.contains(origin));
                    assert!(items.contains(&Vec2::xy(origin.x, (origin.y + 1) % 3)));
                    assert!(items.contains(&Vec2::xy(origin.x, (origin.y + 2) % 3)));
                }
            };
            let pit_state = PitState::new();
            let mut heap = Pit::new_heap(None);
            let origins = [Vec2::xy(0, 0), Vec2::xy(0, 1), Vec2::xy(0, 2)];
            for origin in origins {
                heap[origin.x][origin.y] = Block::new(Some(BlockKind::Blue));
            }

            assert_items(&pit_state, &heap, &origins);
        }

        #[test]
        fn test_collect_matching_east_west() {
            // ┌─┬─┬─┐
            // │ │ │ │
            // ├─┼─┤─┤
            // │▒│▒│▒│
            // ├─┼─┼─┤
            // │ │ │ │
            // └─┴─┴─┘
            let assert_items = |pit_state: &PitState, heap: &Heap, origins: &[Vec2]| {
                let items = pit_state.collect_matching_at(heap, origins);

                assert_eq!(items.len(), 3);

                for origin in origins {
                    assert!(items.contains(origin));
                    assert!(items.contains(&Vec2::xy((origin.x + 1) % 3, origin.y)));
                    assert!(items.contains(&Vec2::xy((origin.x + 2) % 3, origin.y)));
                }
            };
            let pit_state = PitState::new();
            let mut heap = Pit::new_heap(None);
            let origins = [Vec2::xy(0, 1), Vec2::xy(1, 1), Vec2::xy(2, 1)];
            for origin in origins {
                heap[origin.x][origin.y] = Block::new(Some(BlockKind::Blue));
            }

            assert_items(&pit_state, &heap, &origins);
        }

        #[test]
        fn test_collect_matching_north_east() {
            // ┌─┬─┬─┐
            // │▒│ │ │
            // ├─┼─┤─┤
            // │ │▒│ │
            // ├─┼─┼─┤
            // │ │ │▒│
            // └─┴─┴─┘
            let assert_items = |pit_state: &PitState, heap: &Heap, origins: &[Vec2]| {
                let items = pit_state.collect_matching_at(heap, origins);

                assert_eq!(items.len(), 3);

                for origin in origins {
                    assert!(items.contains(origin));
                    assert!(items.contains(&Vec2::xy((origin.x + 1) % 3, (origin.y + 1) % 3)));
                    assert!(items.contains(&Vec2::xy((origin.x + 2) % 3, (origin.y + 2) % 3)));
                }
            };
            let pit_state = PitState::new();
            let mut heap = Pit::new_heap(None);
            let origins = [Vec2::xy(0, 0), Vec2::xy(1, 1), Vec2::xy(2, 2)];
            for origin in origins {
                heap[origin.x][origin.y] = Block::new(Some(BlockKind::Blue));
            }

            assert_items(&pit_state, &heap, &origins);
        }

        #[test]
        fn test_collect_matching_south_west() {
            //
            // ┌─┬─┬─┐
            // │ │ │▒│
            // ├─┼─┤─┤
            // │ │▒│ │
            // ├─┼─┼─┤
            // │▒│ │ │
            // └─┴─┴─┘
            //
            let assert_items = |pit_state: &PitState, heap: &Heap, origins: &[Vec2]| {
                let items = pit_state.collect_matching_at(heap, origins);

                assert_eq!(items.len(), 3);

                for origin in origins {
                    assert!(items.contains(origin));
                    assert!(items.contains(&Vec2::xy((origin.x + 1) % 3, (origin.y + 2) % 3)));
                    assert!(items.contains(&Vec2::xy((origin.x + 2) % 3, (origin.y + 1) % 3)));
                }
            };
            let pit_state = PitState::new();
            let mut heap = Pit::new_heap(None);
            let origins = [Vec2::xy(2, 0), Vec2::xy(1, 1), Vec2::xy(0, 2)];
            for origin in origins {
                heap[origin.x][origin.y] = Block::new(Some(BlockKind::Blue));
            }

            assert_items(&pit_state, &heap, &origins);
        }

        #[test]
        fn test_collect_matching_all_directions() {
            #[rustfmt::skip]
        let assert_items_for_matches = |
            pit_state: &PitState,
            heap: &Heap,
            origins: &[Vec2],
            matches: &[Vec2; 6]
        | -> Vec<Vec2> {
            let items = pit_state.collect_matching_at(heap, origins);

            for origin in origins {
                assert!(items.contains(origin));
            }
            for item in matches {
                assert!(items.contains(item));
            }

            items
        };

            let pit_state = PitState::new();
            let heap = Pit::new_heap(Some(BlockKind::Magenta));
            let origins = [Vec2::xy(0, 0), Vec2::xy(2, 2)];
            let matches = [
                // ┌─┬─┬─┐
                // │▓│▒│▒│
                // ├─┼─┤─┤
                // │▒│▒│░│
                // ├─┼─┼─┤
                // │▒│░│▒│
                // └─┴─┴─┘
                [
                    Vec2::xy(1, 0),
                    Vec2::xy(2, 0),
                    Vec2::xy(0, 1),
                    Vec2::xy(0, 2),
                    Vec2::xy(1, 1),
                    Vec2::xy(2, 2),
                ],
                // ┌─┬─┬─┐
                // │▒│░│▒│
                // ├─┼─┤─┤
                // │░│▒│▒│
                // ├─┼─┼─┤
                // │▒│▒│▓│
                // └─┴─┴─┘
                [
                    Vec2::xy(0, 0),
                    Vec2::xy(2, 0),
                    Vec2::xy(1, 1),
                    Vec2::xy(2, 2),
                    Vec2::xy(0, 2),
                    Vec2::xy(1, 2),
                ],
            ];

            let items = assert_items_for_matches(&pit_state, &heap, &origins[..1], &matches[0]);
            assert_eq!(items.len(), 1 + &matches[0].len());

            let items = assert_items_for_matches(&pit_state, &heap, &origins[1..], &matches[1]);
            assert_eq!(items.len(), 1 + &matches[0].len());

            let items = pit_state.collect_matching_at(&heap, &origins);
            assert_eq!(items.len(), 9);
        }
    }

    mod test_collect_dropping {
        use super::*;

        fn create_and_populate_heap_for_dropping() -> (Heap, [Vec2; 5], [Vec2; 2]) {
            let mut heap: [[Block; 3]; 3] = Pit::new_heap(None);
            let origins = [
                Vec2::xy(0, 0),
                Vec2::xy(0, 1),
                Vec2::xy(1, 1),
                Vec2::xy(2, 1),
                Vec2::xy(2, 2),
            ];
            let magenta = [Vec2::xy(1, 0), Vec2::xy(2, 0)];
            let blue = [Vec2::xy(0, 2), Vec2::xy(1, 2)];
            let dropping = [Vec2::xy(1, 0), Vec2::xy(2, 0)];

            for origin in magenta {
                heap[origin.x][origin.y].update(Some(BlockKind::Magenta));
            }
            for origin in blue {
                heap[origin.x][origin.y].update(Some(BlockKind::Blue));
            }
            (heap, origins, dropping)
        }

        #[test]
        fn test_collect_dropping_at() {
            // ┌─┬─┬─┐
            // │*│░│░│  ░ = Magenta
            // ├─┼─┤─┤  ▒ = Blue
            // │*│*│*│  * = Empty (previously exploding)
            // ├─┼─┼─┤
            // │▒│▒│*│
            // └─┴─┴─┘
            let (heap, origins, dropping) = create_and_populate_heap_for_dropping();
            let items = PitState::new().collect_dropping_at(&heap, &origins);

            for item in dropping {
                assert!(items.contains(&item));
            }
        }

        #[test]
        fn test_update_dropping_at() {
            // ┌─┬─┬─┐
            // │*│░│░│  ░ = Magenta
            // ├─┼─┤─┤  ▒ = Blue
            // │*│*│*│  * = Empty (previously exploding)
            // ├─┼─┼─┤
            // │▒│▒│*│
            // └─┴─┴─┘
            let (mut heap, _, mut dropping) = create_and_populate_heap_for_dropping();
            let pit_state = PitState::new();
            let mut drop_times = 0;

            loop {
                if !pit_state.update_dropping_at(&mut heap, &mut dropping) {
                    break;
                }
                drop_times += 1;
            }

            assert_eq!(drop_times, 2);
            // ┌─┬─┬─┐
            // │ │ │ │  ░ = Magenta
            // ├─┼─┤─┤  ▒ = Blue
            // │ │░│ │
            // ├─┼─┼─┤
            // │▒│▒│░│
            // └─┴─┴─┘
            // empty blocks
            assert!(&heap[0][0].empty());
            assert!(&heap[1][0].empty());
            assert!(&heap[2][0].empty());
            assert!(&heap[0][1].empty());
            assert!(&heap[2][1].empty());
            // non-empty blocks
            assert!(!&heap[1][1].empty());
            assert!(!&heap[0][2].empty());
            assert!(!&heap[1][2].empty());
            assert!(!&heap[2][2].empty());
        }
    }
}
