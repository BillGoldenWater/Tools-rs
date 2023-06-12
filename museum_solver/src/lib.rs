use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MemberInfo {
  pub name: String,
  pub attr: Attribute,
}

impl MemberInfo {
  pub fn new(name: impl Into<String>, attr: Attribute) -> Self {
    Self {
      name: name.into(),
      attr,
    }
  }
}

impl Display for MemberInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name.as_str())
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Attribute {
  pub time: i64,
  pub value: i64,
  pub popularity: i64,
}

impl Attribute {
  pub fn new(time: i64, value: i64, popularity: i64) -> Self {
    Self {
      time,
      value,
      popularity,
    }
  }

  pub fn mul_by(mut self, rhs: i64) -> Self {
    self.time *= rhs;
    self.value *= rhs;
    self.popularity *= rhs;

    self
  }
}

impl Add for Attribute {
  type Output = Attribute;

  fn add(mut self, rhs: Self) -> Self::Output {
    self += rhs;
    self
  }
}

impl AddAssign for Attribute {
  fn add_assign(&mut self, rhs: Self) {
    self.time += rhs.time;
    self.value += rhs.value;
    self.popularity += rhs.popularity;
  }
}

impl Sub for Attribute {
  type Output = Self;

  fn sub(mut self, rhs: Self) -> Self::Output {
    self -= rhs;
    self
  }
}

impl SubAssign for Attribute {
  fn sub_assign(&mut self, rhs: Self) {
    self.time -= rhs.time;
    self.value -= rhs.value;
    self.popularity -= rhs.popularity;
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Zone {
  pub name: String,
  pub base: Attribute,
  pub sub_level: Attribute,
  pub require: Attribute,
}

impl Zone {
  pub fn new(
    name: impl Into<String>,
    base: Attribute,
    sub_level: Attribute,
    require: Attribute,
  ) -> Self {
    Self {
      name: name.into(),
      base,
      sub_level,
      require,
    }
  }

  pub fn calc(&self, member: &[MemberInfo]) -> CalcResult {
    assert!(!member.is_empty() && member.len() <= 3);

    let mut req = self.require.clone() - self.base.clone() - self.sub_level.clone().mul_by(10);
    let mut require: u64 = 0;
    let mut overflow: u64 = 0;

    for info in member {
      req -= info.attr.clone();
    }

    if req.time >= 0 {
      require += req.time as u64;
    } else {
      overflow += req.time.unsigned_abs();
    }
    if req.value >= 0 {
      require += req.value as u64;
    } else {
      overflow += req.value.unsigned_abs();
    }
    if req.popularity >= 0 {
      require += req.popularity as u64;
    } else {
      overflow += req.popularity.unsigned_abs();
    }

    CalcResult::new(require, overflow)
  }

  pub fn calc_detail(&self, member: &[MemberInfo]) -> Attribute {
    let member_sum: Attribute = member
      .iter()
      .map(|it| &it.attr)
      .cloned()
      .fold(Attribute::new(0, 0, 0), |acc, it| acc + it);

    member_sum + self.base.clone() + self.sub_level.clone().mul_by(10)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CalcResult {
  pub require: u64,
  pub overflow: u64,
}

impl Add for CalcResult {
  type Output = Self;

  fn add(mut self, rhs: Self) -> Self::Output {
    self += rhs;
    self
  }
}

impl AddAssign for CalcResult {
  fn add_assign(&mut self, rhs: Self) {
    self.require += rhs.require;
    self.overflow += rhs.overflow;
  }
}

impl CalcResult {
  pub fn new(require: u64, overflow: u64) -> Self {
    Self { require, overflow }
  }
}

type SolveResult = (CalcResult, HashMap<Zone, Vec<MemberInfo>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SolveState {
  members: Vec<MemberInfo>,
  zones: Vec<Zone>,
}

impl SolveState {
  pub fn can_next(&self) -> bool {
    self.members.len() >= 6 && self.zones.len() >= 2
  }

  pub fn current_zone(&self) -> &Zone {
    &self.zones[0]
  }

  pub fn member_combinations(&self) -> Vec<Vec<MemberInfo>> {
    self.members.iter().cloned().combinations(3).collect_vec()
  }

  pub fn next(&self, members: &[MemberInfo]) -> Self {
    Self {
      members: self
        .members
        .iter()
        .filter(|it| !members.contains(it))
        .cloned()
        .sorted()
        .collect_vec(),
      zones: self.zones.iter().skip(1).cloned().collect_vec(),
    }
  }
}

pub fn solve(members: Vec<MemberInfo>, zones: Vec<Zone>) -> Option<SolveResult> {
  if members.len() < 3 || zones.is_empty() {
    return None;
  }

  let mut cache = HashMap::new();

  Some(solve_inner(SolveState { members, zones }, &mut cache))
}

fn solve_inner(state: SolveState, cache: &mut HashMap<SolveState, SolveResult>) -> SolveResult {
  if let Some(solve_result) = cache.get(&state) {
    return solve_result.clone();
  }

  let combinations = state.member_combinations();

  let mut min_result = (
    CalcResult::new(u64::MAX, u64::MAX),
    HashMap::<Zone, Vec<MemberInfo>>::new(),
  );

  for members in combinations {
    let mut result = state.current_zone().calc(&members);
    if result >= min_result.0 {
      continue;
    }

    let mut zones = HashMap::with_capacity(1);

    if state.can_next() {
      let next = solve_inner(state.next(&members), cache);

      result += next.0;
      zones = next.1;
    }

    if min_result.0 > result {
      zones.insert(state.current_zone().clone(), members);
      min_result = (result, zones)
    }
  }

  cache.insert(state, min_result.clone());

  min_result
}
