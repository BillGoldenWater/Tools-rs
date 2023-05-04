use crate::linkage::Linkage;

#[derive(Clone)]
pub enum Action {
  Rotate(Linkage),
}

impl std::fmt::Debug for Action {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Action::Rotate(linkage) => linkage.fmt(f),
    }
  }
}
