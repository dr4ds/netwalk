use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

pub type Flag = u8;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Pos<T> {
  pub x: T,
  pub y: T,
}

impl<T> Pos<T> {
  pub fn new(x: T, y: T) -> Self {
    Self { x: x, y: y }
  }
}

impl<T: Add<Output = T>> Add for Pos<T> {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}

impl<T: AddAssign<T>> AddAssign for Pos<T> {
  fn add_assign(&mut self, other: Self) {
    self.x += other.x;
    self.y += other.y;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size<T> {
  pub width: T,
  pub height: T,
}

impl<T> Size<T> {
  pub fn new(width: T, height: T) -> Self {
    Self {
      width: width,
      height: height,
    }
  }
}

impl<T: Add<Output = T>> Add for Size<T> {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      width: self.width + other.width,
      height: self.height + other.height,
    }
  }
}

impl<T: AddAssign<T>> AddAssign for Size<T> {
  fn add_assign(&mut self, other: Self) {
    self.width += other.width;
    self.height += other.height;
  }
}

pub trait BitFlag {
  fn count_bits(&self) -> u8;
}

impl BitFlag for Flag {
  fn count_bits(&self) -> u8 {
    let mut n = *self;
    let mut c = 0;
    while n != 0u8 {
      if (n & 1) == 1 {
        c += 1;
      }
      n = n >> 1;
    }
    c
  }
}
