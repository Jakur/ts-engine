use crate::country::Side;

use super::{Card, NUM_CARDS};

#[rustfmt::skip]
const EVENT_VALUE: [f32; NUM_CARDS] = [
    0.0,
    2.0,
    2.0,
    2.0,
    3.0, // Duck
    2.1,
    0.0,
    3.2,
    2.4, // Fidel
    2.0,
    1.8,
    2.5,
    1.3,
    1.6,
    1.7, // Comecon
    2.1,
    2.9,
    2.6,
    2.1, // CNS
    1.4,
    0.8,
    1.4,
    1.3,
    5.5, // Marshall
    2.0,
    2.5,
    1.5,
    3.3, // Japan
    3.1,
    2.3,
    4.5,
    5.0,
    1.2,
    6.0, // Destal
    2.1,
    1.4,
    5.5, // Brush War
    2.0,
    2.0,
    1.5,
    2.0,
    2.2,
    2.7, // Quagmire
    3.1,
    3.1,
    0.1, // Summit
    1.9,
    4.1,
    1.8,
    2.2, // Missile
    3.1,
    2.7,
    1.7,
    2.5,
    2.1,
    1.4,
    4.0,
    5.8, // ABM
    1.8,
    2.5,
    0.9,
    3.0,
    1.5,
    4.2,
    2.1,
    2.1,
    3.5, // Puppet
    5.0,
    2.5,
    1.1,
    2.1,
    2.3,
    1.9,
    1.7,
    4.9, // VOA
    2.9,
    3.3,
    2.5, // Ask not
    2.1,
    2.0,
    2.3,
    2.0,
    2.9,
    0.6, // Iron Lady
    1.1,
    4.0, // Star Wars
    1.8,
    5.0, // Reformer
    2.1,
    4.5,
    3.3,
    1.6,
    2.7,
    1.9,
    3.0,
    1.1,
    3.9,
    1.2,
    4.4, // Ames
    3.3,
    2.0, // Wargames todo
    2.7,
    1.8,
    1.7,
    0.6,
    2.0,
    2.7,
    3.6,
    2.4,
    1.5,
    2.0
];

pub fn event_value(card: Card, side: Side) -> f32 {
    let base = EVENT_VALUE[card as usize];
    let card_side = card.side();
    if let Side::Neutral = card_side {
        return base;
    }
    if card_side == side {
        base
    } else {
        card.base_ops() as f32 - base
    }
}
