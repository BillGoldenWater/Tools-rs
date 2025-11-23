use navigation_compass_solver::{
    action::Action, linkage::Linkage,
    navigation_compass::NavigationCompass, ring::Ring,
};
use wasm_bindgen::prelude::wasm_bindgen;

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

    let result = navigation_compass
        .try_solve(linkages.into_iter().map(Linkage::new).collect());

    result.map(|it| {
        it.into_iter()
            .map(|it| match it {
                Action::Rotate(linkage) => linkage.to_u8(),
            })
            .collect::<Vec<_>>()
    })
}
