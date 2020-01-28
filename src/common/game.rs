use super::board::Board;
use super::rng::{GameRng, GameSeed};
use std::fmt;

pub struct Game {
  pub rng: GameRng,
  pub board: Board,
}

impl Game {
  pub fn new(width: i32, height: i32, seed: GameSeed) -> Self {
    let mut rng = GameRng::from(seed);
    let mut board = Board::new(width, height, &mut rng);
    board.scramble(&mut rng);

    Self {
      rng: rng,
      board: board,
    }
  }
}

impl fmt::Debug for Game {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "seed: {:?}\nboard: {:?}",
      self.rng.seed().to_string(),
      self.board
    )
  }
}
