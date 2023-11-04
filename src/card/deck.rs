use super::*;
use crate::state::TwilightRand;
use bitvec::prelude::*;
use once_cell::sync::Lazy;

type CardBits = BitArr!(for Card::AWACS as usize + 1, in u64, Lsb0);
static SCORING_CARDS: Lazy<CardSet> = Lazy::new(|| {
    CardSet::from_cards(&[
        Card::Asia_Scoring,
        Card::Europe_Scoring,
        Card::Middle_East_Scoring,
        Card::Africa_Scoring,
        Card::South_America_Scoring,
        Card::Central_America_Scoring,
        Card::Southeast_Asia_Scoring,
    ])
});
#[derive(Clone, Copy, Debug, Default)]
pub struct CardSet(CardBits);
impl CardSet {
    pub fn from_cards(cards: &[Card]) -> Self {
        let mut i = 0;
        let mut out = Self::empty();
        while i < cards.len() {
            let idx = cards[i] as usize;
            out.0.set(idx, true);
            i += 1;
        }
        out
    }
    pub const fn empty() -> Self {
        Self(CardBits::ZERO)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn pop_count(&self) -> usize {
        self.0.count_ones()
    }
    pub fn card_vec(&self) -> Vec<Card> {
        self.0.iter_ones().map(|x| Card::from_index(x)).collect()
    }
    pub fn iter_cards(&self) -> impl Iterator<Item = Card> + '_ {
        self.0.iter_ones().map(|x| Card::from_index(x))
    }
    pub fn iter_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter_ones()
    }
    pub fn push(&mut self, card: Card) {
        self.0.set(card as usize, true);
    }
    pub fn pop(&mut self, card: Card) {
        self.0.set(card as usize, false);
    }
    pub fn len(&self) -> usize {
        self.0.count_ones()
    }
    pub fn contains(&self, card: Card) -> bool {
        self.0[card as usize]
    }
}

impl std::ops::BitAnd for CardSet {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[derive(Clone, Default)]
pub struct Deck {
    us_hand: CardSet,
    ussr_hand: CardSet,
    discard_pile: CardSet,
    pending_discard: CardSet,
    draw_pile: CardSet,
    removed: CardSet,
    china: Side,
    china_up: bool,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck = Deck {
            china: Side::USSR,
            china_up: true,
            ..Default::default()
        };
        deck.add_early_war();
        deck
    }
    pub fn hand(&self, side: Side) -> &CardSet {
        match side {
            Side::US => &self.us_hand,
            Side::USSR => &self.ussr_hand,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn hand_mut(&mut self, side: Side) -> &mut CardSet {
        match side {
            Side::US => &mut self.us_hand,
            Side::USSR => &mut self.ussr_hand,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn end_turn_cleanup(&mut self) {
        self.china_up = true;
        // Todo draw cards
    }
    /// Returns a new CardSet holding all scoring cards in the side's hand.
    pub fn scoring_cards(&self, side: Side) -> CardSet {
        *self.hand(side) & *SCORING_CARDS
    }
    pub fn held_scoring(&self, side: Side) -> bool {
        !self.scoring_cards(side).is_empty()
    }
    /// Determines if the side has a position where the requirement to play
    /// scoring cards overwhelms other effect obligations, e.g. Bear Trap and
    /// Missile Envy.
    pub fn must_play_scoring(&self, side: Side, ar_left: i8) -> bool {
        self.scoring_cards(side).pop_count() >= (ar_left as usize)
    }
    /// Returns a vector of the cards with the highest ops value in the side's
    /// hand, excluding the China Card.
    pub fn highest_ops(&self, side: Side) -> CardSet {
        let mut hand = self.hand(side).card_vec();
        let max_val = hand.iter().map(|c| c.base_ops()).max();
        hand.retain(|c| c.base_ops() == max_val.unwrap_or(0));
        CardSet::from_cards(&hand)
    }
    pub fn draw_cards<T: TwilightRand>(&mut self, target: usize, rng: &mut T) {
        let mut side = Side::USSR;
        // Oscillating is relevant when reshuffles do occur
        while self.ussr_hand.len() < target && self.us_hand.len() < target {
            self.draw_to_hand(rng, side);
            side = side.opposite();
        }
        while self.ussr_hand.len() < target {
            self.draw_to_hand(rng, Side::USSR);
        }
        while self.us_hand.len() < target {
            self.draw_to_hand(rng, Side::US);
        }
    }
    /// Searches the pending discard pile for a played card and removes it.
    pub fn remove_card(&mut self, card: Card) {
        self.pending_discard.pop(card);
        self.removed.push(card);
    }
    pub fn pending_discard(&self) -> &CardSet {
        &self.pending_discard
    }
    pub fn discard_pile(&self) -> &CardSet {
        &self.discard_pile
    }
    pub fn flush_pending(&mut self) {
        self.discard_pile.0 |= self.pending_discard.0;
        self.pending_discard = CardSet::empty();
    }
    pub fn random_card<T: TwilightRand>(&self, side: Side, rng: &mut T) -> Option<Card> {
        rng.card_from_hand(self, side)
    }
    /// Draws the next card from the draw pile, reshuffling if necessary.
    pub fn draw_to_hand<T: TwilightRand>(&mut self, rng: &mut T, side: Side) {
        let card = rng.draw_card(self, side);
        self.hand_mut(side).push(card);
    }
    /// Returns a vector of cards which, if played by the given side, will cause
    /// the opponent's event to fire.
    pub fn opp_events_fire(&self, side: Side, state: &GameState) -> Vec<Card> {
        let hand = self.hand(side);
        let opp = side.opposite();
        hand.iter_cards()
            .filter(|c| c.side() == opp && c.can_event(state, side))
            .collect()
    }
    /// Returns a vector of cards that the given side can themselves event.
    pub fn can_event(&self, side: Side, state: &GameState) -> Vec<Card> {
        let hand = self.hand(side);
        let opp = side.opposite();
        hand.iter_cards()
            .filter(|c| c.side() != opp && c.can_event(state, side))
            .collect()
    }
    /// Returns cards that can be played for just ops, i.e. non-scoring cards of
    /// neutral or allied variety, or those opponent events that won't fire.
    pub fn can_play_ops(&self, side: Side, state: &GameState) -> Vec<Card> {
        let hand = self.hand(side);
        let opp = side.opposite();
        let mut vec: Vec<_> = hand
            .iter_cards()
            .filter(|c| !c.is_scoring() && (c.side() != opp || !c.can_event(state, side)))
            .collect();
        if self.china_available(side) {
            vec.push(Card::The_China_Card);
        }
        vec
    }
    pub fn play_china(&mut self) {
        self.china = self.china.opposite();
        self.china_up = false;
    }
    pub fn turn_china_up(&mut self) {
        self.china_up = true;
    }
    pub fn play_card(&mut self, side: Side, card: Card) -> Result<(), DeckError> {
        if let Card::The_China_Card = card {
            self.play_china();
            Ok(())
        } else {
            // Already ready to be discarded
            // if self.pending_discard.contains(&card) {
            //     return Ok(());
            // }
            match side {
                Side::US | Side::USSR => {
                    let hand = self.hand_mut(side);
                    hand.pop(card);
                    Ok(())
                }
                Side::Neutral => Err(DeckError::CannotFind),
            }
        }
    }
    pub fn our_man<T: TwilightRand>(&mut self, rng: &mut T) -> &[Card] {
        // Move the 5 cards to the end of the draw pile
        // let mut vec = Vec::with_capacity(5);
        // for _ in 0..5 {
        //     let card = rng.draw_card(self, Side::US);
        //     vec.push(card);
        // }
        // self.draw_pile.extend(vec.into_iter());
        // &self.draw_pile[self.draw_pile.len() - 5..]
        todo!()
    }
    pub fn draw_pile_mut(&mut self) -> &mut CardSet {
        &mut self.draw_pile
    }
    pub fn discard_draw(&mut self, card: Card) {
        self.draw_pile.pop(card);
        self.discard_pile.push(card);
    }
    pub fn china_available(&self, side: Side) -> bool {
        self.china == side && self.china_up
    }
    pub fn china(&self) -> Side {
        self.china
    }
    pub fn recover_card(&mut self, side: Side, card: Card) {
        // Todo figure out error handling
        self.hand_mut(side).push(card);
        self.discard_pile.pop(card);
    }
    pub fn reset_draw_pile(&mut self) {
        self.draw_pile.0 |= self.discard_pile.0;
        self.discard_pile = CardSet::empty();
    }
    pub fn reshuffle<T: TwilightRand>(&mut self, rng: &mut T) {
        rng.reshuffle(self);
    }
    pub fn add_early_war(&mut self) {
        for c_index in 1..Card::Formosan_Resolution as usize + 1 {
            let card = Card::from_index(c_index);
            if card == Card::The_China_Card {
                continue;
            }
            self.draw_pile.push(card);
        }
        for c_index in Card::Defectors as usize..Card::Che as usize {
            let card = Card::from_index(c_index);
            self.draw_pile.push(card);
        }
    }
    pub fn add_mid_war(&mut self) {
        // Todo the rest
        for c_index in Card::Formosan_Resolution as usize + 1..Card::total() {
            let card = Card::from_index(c_index);
            self.draw_pile.push(card);
        }
    }
    pub fn add_late_war(&mut self) {
        todo!()
    }
}

#[derive(Debug)]
pub enum DeckError {
    AlreadyContains,
    CannotFind,
    ChinaException,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DebugRand;
    // #[test]
    // fn test_our_man() {
    //     let mut rand = DebugRand::new_empty();
    //     // Extra cards that we don't want to hit
    //     rand.us_draw = vec![Card::De_Stalinization, Card::Duck_and_Cover];
    //     let vec = vec![
    //         Card::CIA_Created,
    //         Card::Defectors,
    //         Card::Decolonization,
    //         Card::Suez_Crisis,
    //         Card::Socialist_Governments,
    //     ];
    //     rand.us_draw.extend(vec.clone().into_iter());
    //     let mut deck = Deck::new();
    //     let cards = deck.our_man(&mut rand);
    //     assert_eq!(
    //         cards,
    //         &vec.clone().into_iter().rev().collect::<Vec<_>>()[..]
    //     );
    //     let indices: Vec<_> = cards.iter().map(|c| *c as usize).collect();
    //     for i in &indices[0..3] {
    //         deck.discard_draw(Card::from_index(*i));
    //     }
    //     // Assert discards worked
    //     assert_eq!(
    //         deck.discard_pile,
    //         vec[2..].iter().cloned().rev().collect::<Vec<_>>()
    //     );

    //     // Assert the other cards are still in the draw pile
    //     for i in &indices[3..] {
    //         let card = Card::from_index(*i);
    //         deck.draw_pile.contains(&card);
    //     }
    // }
}
