#[derive(Clone)]
pub struct Ring {
  pub current: i8,
  pub num: u8,
  pub direction: i8,
}

impl std::fmt::Debug for Ring {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.current)
  }
}

impl Ring {
  pub fn new(current: i8, num: u8, direction: i8) -> Self {
    Self {
      current,
      num,
      direction,
    }
  }

  pub fn rotate(&mut self) {
    self.current += self.direction * self.num as i8;
    self.current %= 6;
    if self.current < 0 {
      self.current += 6;
    }
  }
}
