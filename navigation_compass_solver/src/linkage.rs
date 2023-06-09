#[derive(Clone)]
pub struct Linkage {
  pub inner: bool,
  pub middle: bool,
  pub outer: bool,
}

impl Linkage {
  pub fn new(mask: u8) -> Self {
    Self {
      inner: (mask & 0b100) > 0,
      middle: (mask & 0b010) > 0,
      outer: (mask & 0b001) > 0,
    }
  }

  pub fn to_u8(&self) -> u8 {
    (self.inner as u8) << 2 | (self.middle as u8) << 1 | self.outer as u8
  }
}

impl std::fmt::Debug for Linkage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:03b}", self.to_u8())
  }
}
