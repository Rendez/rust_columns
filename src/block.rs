#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockKind {
    Blue,
    Yellow,
    Green,
    Red,
    Cyan,
    Magenta,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Block {
    kind: Option<BlockKind>,
    pub exploding: bool,
}

impl Block {
    pub fn new(kind: Option<BlockKind>) -> Self {
        Self {
            kind,
            exploding: false,
        }
    }

    pub fn numbered(&self) -> i8 {
        if self.exploding {
            return 6;
        }
        match self.kind {
            Some(kind) => kind as i8,
            None => -1,
        }
    }

    pub fn update(&mut self, kind: Option<BlockKind>) {
        self.kind = kind;
    }

    pub fn empty(&self) -> bool {
        self.kind.is_none()
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        assert!(Block::default().empty());
    }
}
