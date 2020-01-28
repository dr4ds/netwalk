use super::tile::TileOffset;
use super::util::Flag;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectionKind {
  UP = 1,
  RIGHT = 2,
  DOWN = 4,
  LEFT = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Direction {
  pub kind: DirectionKind,
  pub flag: Flag,
  pub opposite: Flag,
  pub offset: TileOffset,
  pub right: Flag,
  pub left: Flag,
}

impl Default for DirectionKind {
  fn default() -> Self {
    Self::UP
  }
}

pub static DIRECTIONS: [Direction; 4] = [
  Direction {
    kind: DirectionKind::UP,
    flag: 1,
    opposite: 4,
    right: 2,
    left: 8,
    offset: TileOffset { x: 0, y: -1 },
  },
  Direction {
    kind: DirectionKind::RIGHT,
    flag: 2,
    opposite: 8,
    right: 4,
    left: 1,
    offset: TileOffset { x: 1, y: 0 },
  },
  Direction {
    kind: DirectionKind::DOWN,
    flag: 4,
    opposite: 1,
    right: 8,
    left: 2,
    offset: TileOffset { x: 0, y: 1 },
  },
  Direction {
    kind: DirectionKind::LEFT,
    flag: 8,
    opposite: 2,
    right: 1,
    left: 4,
    offset: TileOffset { x: -1, y: 0 },
  },
];
