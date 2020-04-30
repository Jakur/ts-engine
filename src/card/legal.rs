use super::*;

pub fn muslim_rev(state: &GameState) -> Vec<usize> {
    country::MIDDLE_EAST
        .iter()
        .copied()
        .filter_map(|x| {
            if state.countries[x].has_influence(Side::US) {
                Some(x)
            } else {
                None
            }
        })
        .collect()
}
