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

pub fn norad(state: &GameState) -> Vec<usize> {
    state
        .valid_countries()
        .iter()
        .enumerate()
        .filter_map(|(i, c)| if c.us > 0 { Some(i) } else { None })
        .collect()
}
