use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use crate::card::{Card, Deck};
use crate::country::Side;

/// A trait representing the nondeterminism involved in Twilight Struggle. It is
/// generic so we can abstract across games where we the program have control
/// over the randomness and also games where the randomness is provided from an
/// external source, e.g. a server.
pub trait TwilightRand {
    /// Rolls a six-sided die for the given side and returns the result.
    fn roll(&mut self, side: Side) -> i8;
    /// Returns a random card from the given side's hand, or None if and only if
    /// that side's hand is empty.
    fn card_from_hand(&mut self, deck: &Deck, side: Side) -> Option<Card>;
    /// Reshuffles the discard pile of the deck into the draw pile. Cards will
    /// be drawn from the end of the draw pile vector, as in a stack. 
    fn reshuffle(&mut self, deck: &mut Deck);
}

#[derive(Clone)]
pub struct InternalRand {
    rng: SmallRng
}

impl InternalRand {
    pub fn new_entropy() -> Self {
        InternalRand { rng: SmallRng::from_entropy() }
    }
    pub fn new_seeded(seed: u64) -> Self {
        InternalRand {rng: SmallRng::seed_from_u64(seed) }
    }
}

impl TwilightRand for InternalRand {
    fn roll(&mut self, _side: Side) -> i8 {
        self.rng.gen_range(1, 7)
    }
    fn card_from_hand(&mut self, deck: &Deck, side: Side) -> Option<Card> {
        let hand = deck.hand(side);
        if hand.len() == 0 {
            None
        } else {
            let i = self.rng.gen_range(0, hand.len());
            Some(hand[i])
        }
    }
    fn reshuffle(&mut self, deck: &mut Deck) {
        use rand::seq::SliceRandom;
        deck.discard_pile_mut().shuffle(&mut self.rng);
        deck.reset_draw_pile();
    }
}

#[derive(Clone)]
pub struct DebugRand {
    pub us_rolls: Vec<i8>,
    pub ussr_rolls: Vec<i8>,
    pub discards: Vec<Option<Card>>,
    pub shuffle_order: Vec<Vec<Card>>
}

impl DebugRand {
    pub fn new_empty() -> Self {
        DebugRand {us_rolls: Vec::new(), ussr_rolls: Vec::new(), 
            discards: Vec::new(), shuffle_order: Vec::new()}
    }
}

impl TwilightRand for DebugRand {
    fn roll(&mut self, side: Side) -> i8 {
        match side {
            Side::US => self.us_rolls.pop().unwrap(),
            Side::USSR => self.ussr_rolls.pop().unwrap(),
            _ => unimplemented!()
        }
    }
    fn card_from_hand(&mut self, deck: &Deck, side: Side) -> Option<Card> {
        let card = self.discards.pop().unwrap();
        if let Some(card) = card {
            let find = deck.hand(side).iter().find(|c| **c == card);
            assert!(find.is_some());
        } else {
            assert!(deck.hand(side).is_empty());
        }
        card
    }
    fn reshuffle(&mut self, deck: &mut Deck) {
        let mut order = self.shuffle_order.pop().unwrap();
        deck.draw_pile_mut().clear();
        deck.discard_pile_mut().clear();
        deck.draw_pile_mut().append(&mut order);
    }
}