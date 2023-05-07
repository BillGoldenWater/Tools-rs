use wasm_bindgen::prelude::*;

use navigation_compass_solver::action::Action;
use navigation_compass_solver::linkage::Linkage;
use navigation_compass_solver::navigation_compass::NavigationCompass;
use navigation_compass_solver::ring::Ring;

#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn navigation_compass_solve(
  ic: i8,
  in_n: u8,
  id: i8,

  mc: i8,
  mn: u8,
  md: i8,

  oc: i8,
  on: u8,
  od: i8,

  linkages: Vec<u8>,
) -> Option<Vec<u8>> {
  let navigation_compass = NavigationCompass::new(
    Ring::new(ic, in_n, id),
    Ring::new(mc, mn, md),
    Ring::new(oc, on, od),
  );

  let result = navigation_compass.try_solve(linkages.into_iter().map(Linkage::new).collect());

  result.map(|it| {
    it.into_iter()
      .map(|it| match it {
        Action::Rotate(linkage) => linkage.to_u8(),
      })
      .collect::<Vec<_>>()
  })
}

#[wasm_bindgen]
pub fn wasm_init() {
  console_error_panic_hook::set_once();

  let log_level = if cfg!(debug_assertions) {
    log::Level::Debug
  } else {
    log::Level::Info
  };
  let result = console_log::init_with_level(log_level);
  if result.is_err() {
    web_sys::console::warn_1(&JsValue::from_str("failed to set logger"))
  }
}
