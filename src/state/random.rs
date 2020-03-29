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
    /// Draws a new card for the given side from the draw pile, adding it to that
    /// player's hand.
    fn draw_card(&mut self, deck: &mut Deck, side: Side);
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
    fn draw_card(&mut self, deck: &mut Deck, side: Side) {
        match deck.pop_draw_pile() {
            Some(c) => deck.hand_mut(side).push(c),
            None => {
                self.reshuffle(deck);
                self.draw_card(deck, side);
            }
        }
    }
}

#[derive(Clone)]
pub struct DebugRand {
    pub us_rolls: Vec<i8>,
    pub ussr_rolls: Vec<i8>,
    pub discards: Vec<Option<Card>>,
    pub us_draw: Vec<Card>,
    pub ussr_draw: Vec<Card>,
}

impl DebugRand {
    pub fn new_empty() -> Self {
        Self::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }
    pub fn new(us_rolls: Vec<i8>, ussr_rolls: Vec<i8>, discards: Vec<Option<Card>>, 
        us_draw: Vec<Card>, ussr_draw: Vec<Card>) -> Self {
            DebugRand {us_rolls, ussr_rolls, discards, us_draw, ussr_draw}
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
        todo!()
    }
    fn draw_card(&mut self, deck: &mut Deck, side: Side) {
        let card = match side {
            Side::US => self.us_draw.pop(),
            Side::USSR => self.ussr_draw.pop(),
            _ => None,
        };
        if let Some(card) = card {
            if card == Card::The_China_Card {
                return
            }
            let index = deck.draw_pile().iter().position(|&c| c == card).unwrap();
            deck.draw_pile_mut().swap_remove(index);
            deck.hand_mut(side).push(card);
        } else {
            // We've drawn all the known cards we care about, so just draw 
            // non-scoring cards for now
            let index = deck.draw_pile().iter().position(|c| !c.is_scoring()).unwrap();
            let card = deck.draw_pile_mut().swap_remove(index);
            deck.hand_mut(side).push(card);
        }
    }
}