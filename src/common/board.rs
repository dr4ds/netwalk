use super::direction::Direction;
use super::direction::DIRECTIONS;
use super::rng::GameRng;
use super::tile::{RotationDirection, Tile, TileKind, TilePos};
use super::util::{BitFlag, Flag, Size};
use rand::Rng;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
  size: Size<i32>,
  tiles: Vec<Tile>,
  tiles_to_visit: Vec<TilePos>,
  root: TilePos,
  start: Instant,
}

impl Board {
  pub fn new(width: i32, height: i32, rng: &mut GameRng) -> Self {
    let mut board = Self {
      size: Size::new(width, height),
      tiles: Vec::with_capacity((width * height) as usize),
      tiles_to_visit: Vec::new(),
      root: TilePos::new(rng.gen_range(0, width), rng.gen_range(0, height)),
      start: Instant::now(),
    };

    board.init_tiles();
    board.generate_tree(rng);
    board.set_tiles_kinds();

    board
  }

  pub fn start_timer(&mut self) {
    self.start = Instant::now();
  }

  pub fn get_start_time(&self) -> Instant {
    self.start
  }

  fn init_tiles(&mut self) {
    for x in 0..self.size.width {
      for y in 0..self.size.height {
        let mut tile = Tile::new();

        for dir in DIRECTIONS.iter() {
          let mut tile_pos = TilePos::new(y, x);
          tile_pos += dir.offset;

          if self.in_bounds(&tile_pos) {
            tile.neighbours |= dir.flag;
          }
        }
        self.tiles.push(tile);
      }
    }
  }

  fn set_tiles_kinds(&mut self) {
    for x in 0..self.size.width {
      for y in 0..self.size.height {
        let mut tile = self.get_tile_mut(&TilePos::new(x, y));
        if tile.connections() == 1 {
          tile.kind = TileKind::TERMINAL;
        } else {
          tile.kind = TileKind::CONNECTOR;
        }
      }
    }

    let pos = self.root;
    self.get_tile_mut(&pos).kind = TileKind::SERVER;
  }

  fn solve_walk(&mut self, pos: TilePos) {
    let tile = self.get_tile_mut(&pos);
    tile.powered = true;
    let tile_dirs = tile.directions;

    for dir in DIRECTIONS.iter() {
      if (dir.flag & tile_dirs) != 0 {
        let neighbour_pos = pos + dir.offset;
        if !self.in_bounds(&neighbour_pos) {
          continue;
        }
        let neighbour_tile = self.get_tile_mut(&neighbour_pos);
        if (dir.opposite & neighbour_tile.directions) != 0 && !neighbour_tile.powered {
          self.solve_walk(neighbour_pos);
        }
      }
    }
  }

  pub fn is_solved(&mut self) -> bool {
    for tile in &mut self.tiles {
      tile.powered = false;
    }

    self.solve_walk(self.root);

    for tile in &self.tiles {
      if tile.kind == TileKind::TERMINAL && !tile.powered {
        return false;
      }
    }

    true
  }

  pub fn get_directions(&self) -> Vec<Flag> {
    self.tiles.iter().map(|v| v.directions).collect()
  }

  pub fn get_neighbours(&self) -> Vec<Flag> {
    self.tiles.iter().map(|v| v.neighbours).collect()
  }

  fn in_bounds(&self, pos: &TilePos) -> bool {
    pos.x >= 0 && pos.y >= 0 && pos.x < self.size.width && pos.y < self.size.height
  }

  pub fn get_root(&self) -> TilePos {
    self.root
  }

  pub fn set_tile(&mut self, pos: &TilePos, tile: Tile) {
    self.tiles[(pos.x + pos.y * self.size.width) as usize] = tile
  }

  pub fn get_tile(&self, pos: &TilePos) -> Tile {
    self.tiles[(pos.x + pos.y * self.size.width) as usize]
  }

  pub fn get_tile_mut(&mut self, pos: &TilePos) -> &mut Tile {
    &mut self.tiles[(pos.x + pos.y * self.size.width) as usize]
  }

  fn visit_tile(&mut self, tile_pos: &TilePos, flag: Flag) {
    let mut tile = self.get_tile(&tile_pos);
    if tile.directions == 0 {
      self.tiles_to_visit.push(*tile_pos);
    }

    tile.directions |= flag;
    self.set_tile(&tile_pos, tile);

    for dir in DIRECTIONS.iter() {
      let mut tp = *tile_pos;
      tp += dir.offset;

      if self.in_bounds(&tp) {
        let tile = self.get_tile_mut(&tp);
        tile.neighbours &= !dir.opposite;
      }
    }
  }

  pub fn rotate_tile(&mut self, pos: &TilePos, dir: RotationDirection) -> Flag {
    if !self.in_bounds(pos) {
      return 0;
    }

    let tile = self.get_tile_mut(pos);
    let old = tile.directions;
    tile.rotate(dir, 1);

    if tile.directions == old {
      return 0;
    }

    tile.directions
  }

  pub fn scramble(&mut self, rng: &mut GameRng) {
    for tile in &mut self.tiles {
      tile.rotate(RotationDirection::Right, rng.gen_range(0, 3));
    }
  }

  fn rand_dir(&self, pos: &TilePos, rng: &mut GameRng) -> Option<Direction> {
    let tile = self.get_tile(pos);

    let n = tile.neighbours.count_bits();
    if n == 0 {
      return None;
    }

    let mut arr: Vec<Direction> = Vec::new();
    for d in DIRECTIONS.iter() {
      if (tile.neighbours & d.flag) != 0 {
        arr.push(*d);
      }
    }

    let i = rng.gen_range(0, arr.len());
    Some(arr[i])
  }

  pub fn get_size(&self) -> Size<i32> {
    self.size
  }

  fn generate_tree(&mut self, rng: &mut GameRng) {
    let rp = self.root;
    self.visit_tile(&rp, 0);

    while self.tiles_to_visit.len() > 0 {
      let n = rng.gen_range(0, self.tiles_to_visit.len());
      let mut tile_pos = self.tiles_to_visit[n];

      if let Some(dir) = self.rand_dir(&tile_pos, rng) {
        self.visit_tile(&tile_pos, dir.flag);
        tile_pos += dir.offset;

        self.visit_tile(&tile_pos, dir.opposite);
      }

      if self.get_tile(&tile_pos).neighbours.count_bits() <= 1 {
        self.tiles_to_visit.remove(n);
      }
    }
  }
}
