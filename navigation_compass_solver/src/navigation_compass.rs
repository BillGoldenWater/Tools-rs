use itertools::Itertools;

use crate::{action::Action, linkage::Linkage, ring::Ring};

#[derive(Debug, Clone)]
pub struct NavigationCompass {
  inner: Ring,
  middle: Ring,
  outer: Ring,
}

macro_rules! gen_rotate {
  ($fn_name:ident, $part:ident) => {
    fn $fn_name(&mut self) {
      self.$part.rotate();
    }
  };
}

impl NavigationCompass {
  pub fn new(inner: Ring, middle: Ring, outer: Ring) -> Self {
    Self {
      inner,
      middle,
      outer,
    }
  }

  pub fn try_solve(&self, available_linkages: Vec<Linkage>) -> Option<Vec<Action>> {
    for combination_num in 1..=100 {
      let result = available_linkages
        .iter()
        .cloned()
        .combinations_with_replacement(combination_num)
        .map(|it| it.into_iter().map(Action::Rotate).collect_vec())
        .find(|it| self.do_actions(it).is_solved());

      if let Some(result) = result {
        return Some(result);
      }
    }

    None
  }

  fn is_solved(&self) -> bool {
    self.inner.current == 0 && self.middle.current == 0 && self.outer.current == 0
  }

  fn do_actions(&self, actions: &Vec<Action>) -> NavigationCompass {
    let mut result = self.clone();

    for action in actions {
      match action {
        Action::Rotate(Linkage {
          inner,
          middle,
          outer,
        }) => {
          if *inner {
            result.rotate_inner();
          }
          if *middle {
            result.rotate_middle();
          }
          if *outer {
            result.rotate_outer();
          }
        }
      }
    }

    result
  }

  gen_rotate!(rotate_inner, inner);
  gen_rotate!(rotate_middle, middle);
  gen_rotate!(rotate_outer, outer);
}
