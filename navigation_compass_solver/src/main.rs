mod action;
mod linkage;
mod navigation_compass;
mod ring;

use itertools::Itertools;
use linkage::Linkage;
use ring::Ring;

use crate::navigation_compass::NavigationCompass;

#[derive(Debug, argh::FromArgs)]
/// A tool for solve Navigation Compass puzzle in Honkai: Star Rail
struct Args {
  /// inner ring current value (count from zero, clockwise)
  #[argh(option)]
  ic: i8,
  /// inner ring number per rotate
  #[argh(option, long = "in")]
  r#in: u8,
  /// inner ring direction per rotate clockwise: 1, anticlockwise: -1
  #[argh(option)]
  id: i8,

  /// middle ring, same as above
  #[argh(option)]
  mc: i8,
  /// middle ring, same as above
  #[argh(option)]
  mn: u8,
  /// middle ring, same as above
  #[argh(option)]
  md: i8,

  /// outer ring, same as above
  #[argh(option)]
  oc: i8,
  /// outer ring, same as above
  #[argh(option)]
  on: u8,
  /// outer ring, same as above
  #[argh(option)]
  od: i8,

  /// linkage info for a action, three digits for inner, middle and outer.
  /// eg. inner and outer: 101, inner only: 100.
  /// repeat this argument for multiple action, eg. -l 101 -l 001 -l 100.
  #[argh(option, short = 'l')]
  linkage: Vec<String>,
}

fn parse_linkages(raw: Vec<String>) -> Vec<Linkage> {
  raw
    .into_iter()
    .map(|it| {
      if it.chars().count() != 3 {
        println!("failed to parse linkage, wrong length: {}", it);
        std::process::exit(-1);
      }

      if it.chars().any(|it| it != '0' && it != '1') {
        println!("failed to parse linkage, contain unknown character: {}", it);
        std::process::exit(-1);
      }

      let linkage =
        it.chars().rev().enumerate().fold(
          0,
          |acc, (idx, val)| {
            if val == '1' {
              acc | (1 << idx)
            } else {
              acc
            }
          },
        );

      Linkage::new(linkage)
    })
    .collect_vec()
}

fn main() {
  let args: Args = argh::from_env();

  let navigation_compass = NavigationCompass::new(
    Ring::new(args.ic, args.r#in, args.id),
    Ring::new(args.mc, args.mn, args.md),
    Ring::new(args.oc, args.on, args.od),
  );
  // NavigationCompass::new(Ring::new(4, 1, -1), Ring::new(0, 3, -1), Ring::new(0, 3, 1));

  let result = navigation_compass.try_solve(parse_linkages(args.linkage));

  if let Some(result) = result {
    result
      .into_iter()
      .enumerate()
      .for_each(|(idx, val)| println!("step {}: {val:?}", idx + 1))
  } else {
    println!("failed to solve in 100 step");
  }
}
