use super::direction::DIRECTIONS;
use super::util::{BitFlag, Flag, Pos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RotationDirection {
  Right,
  Left,
}

pub type TilePos = Pos<i32>;
pub type TileOffset = Pos<i32>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
  pub kind: TileKind,
  pub directions: Flag,
  pub neighbours: Flag,
  pub powered: bool,
}

impl Tile {
  pub fn new() -> Self {
    Self {
      kind: TileKind::default(),
      directions: 0,
      neighbours: 0,
      powered: false,
    }
  }

  pub fn rotate(&mut self, rd: RotationDirection, n: i32) {
    for _ in 0..n {
      let mut new = 0;
      for dir in DIRECTIONS.iter() {
        if (dir.flag & self.directions) > 0 {
          new |= match rd {
            RotationDirection::Left => dir.left,
            RotationDirection::Right => dir.right,
          };
        }
      }

      self.directions = new;
    }
  }

  pub fn connections(&self) -> u8 {
    self.directions.count_bits()
  }

  pub fn free_directions(&self) -> u8 {
    self.neighbours.count_bits()
  }

  // pub fn rand_free_dir(&self, rng: &mut GameRng) -> Option<Direction> {
  //   let n = self.free_directions() as usize;
  //   if n == 0 {
  //     return None;
  //   }

  //   let mut arr: Vec<Direction> = Vec::with_capacity(n);
  //   for d in DIRECTIONS.iter() {
  //     if (self.neighbours & d.flag) != 0 {
  //       arr.push(*d);
  //     }
  //   }

  //   let i = rng.gen_range(0, n);
  //   Some(arr[i])
  // }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileKind {
  UNDEFINED = 0,
  SERVER = 1,
  TERMINAL = 2,
  CONNECTOR = 3,
}

impl Default for TileKind {
  fn default() -> Self {
    Self::UNDEFINED
  }
}
