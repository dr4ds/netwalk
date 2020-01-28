use rand::{rngs::StdRng, Error, Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::string::ToString;

#[derive(Serialize, Deserialize)]
pub struct GameSeed([u8; 32]);

#[derive(Debug)]
pub struct InvalidSeedError;

impl GameSeed {
  pub fn new() -> Self {
    Self(rand::thread_rng().gen::<[u8; 32]>())
  }

  pub fn from_string(s: String) -> Result<Self, InvalidSeedError> {
    Self::from_str(s.as_str())
  }

  pub fn from_str(s: &str) -> Result<Self, InvalidSeedError> {
    let seed = match hex::decode(s.as_bytes()) {
      Ok(seed) => {
        if seed.len() != 32 {
          return Err(InvalidSeedError);
        }
        seed
      }
      Err(_) => return Err(InvalidSeedError),
    };

    Ok(Self::from_slice(&seed))
  }

  pub fn from_slice(s: &[u8]) -> Self {
    let mut seed = [0; 32];
    let bytes = &s[..32];
    seed.copy_from_slice(bytes);
    Self(seed)
  }
}

impl Default for GameSeed {
  fn default() -> Self {
    Self([0; 32])
  }
}

impl AsMut<[u8]> for GameSeed {
  fn as_mut(&mut self) -> &mut [u8] {
    &mut self.0
  }
}

impl fmt::Debug for GameSeed {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {:?})", self.0, self.to_string())
  }
}

impl ToString for GameSeed {
  fn to_string(&self) -> String {
    hex::encode(&self.0)
  }
}

pub struct GameRng(StdRng, GameSeed);

impl GameRng {
  pub fn seed(&self) -> &GameSeed {
    &self.1
  }
}

impl From<GameSeed> for GameRng {
  fn from(gs: GameSeed) -> Self {
    Self(StdRng::from_seed(gs.0), gs)
  }
}

impl RngCore for GameRng {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.0.next_u32()
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.0.next_u64()
  }

  #[inline(always)]
  fn fill_bytes(&mut self, dest: &mut [u8]) {
    self.0.fill_bytes(dest);
  }

  #[inline(always)]
  fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
    self.0.try_fill_bytes(dest)
  }
}
