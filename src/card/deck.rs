use super::*;
use crate::state::TwilightRand;

#[derive(Clone)]
pub struct Deck {
    us_hand: Vec<Card>,
    ussr_hand: Vec<Card>,
    discard_pile: Vec<Card>,
    pending_discard: Vec<Card>,
    draw_pile: Vec<Card>,
    removed: Vec<Card>,
    china: Side,
    china_up: bool,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck = Deck {
            us_hand: Vec::new(),
            ussr_hand: Vec::new(),
            discard_pile: Vec::new(),
            pending_discard: Vec::new(),
            draw_pile: Vec::new(),
            removed: Vec::new(),
            china: Side::USSR,
            china_up: true,
        };
        deck.add_early_war();
        deck
    }
    pub fn hand(&self, side: Side) -> &Vec<Card> {
        match side {
            Side::US => &self.us_hand,
            Side::USSR => &self.ussr_hand,
            Side::Neutral => unimplemented!(),
        }
    }
    pub fn hand_mut(&mut self, side: Side) -> &mut Vec<Card> {
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
    /// Returns a new vector holding all scoring cards in the side's hand.
    pub fn scoring_cards(&self, side: Side) -> Vec<Card> {
        self.hand(side)
            .iter()
            .copied()
            .filter(|c| c.is_scoring())
            .collect()
    }
    pub fn held_scoring(&self, side: Side) -> bool {
        let hand = self.hand(side);
        hand.iter().any(|x| x.is_scoring())
    }
    /// Determines if the side has a position where the requirement to play
    /// scoring cards overwhelms other effect obligations, e.g. Bear Trap and
    /// Missile Envy.
    pub fn must_play_scoring(&self, side: Side, ar_left: i8) -> bool {
        let scoring_count =
            self.hand(side)
                .iter()
                .fold(0, |acc, x| if x.is_scoring() { acc + 1 } else { acc });
        scoring_count >= ar_left
    }
    /// Returns a vector of the cards with the highest ops value in the side's
    /// hand, excluding the China Card.
    pub fn highest_ops(&self, side: Side) -> Vec<Card> {
        let hand = self.hand(side);
        let max_val = hand.iter().map(|c| c.base_ops()).max();
        match max_val {
            Some(max) => hand
                .iter()
                .copied()
                .filter(|c| c.base_ops() == max)
                .collect(),
            None => Vec::new(),
        }
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
    pub fn remove_card(&mut self, card: Card) -> Result<(), DeckError> {
        let found = self.pending_discard.iter().position(|&c| c == card);
        if let Some(i) = found {
            let c = self.pending_discard.swap_remove(i);
            self.removed.push(c);
            Ok(())
        } else {
            Err(DeckError::CannotFind)
        }
    }
    pub fn pending_discard(&self) -> &Vec<Card> {
        &self.pending_discard
    }
    pub fn flush_pending(&mut self) {
        self.discard_pile.append(&mut self.pending_discard);
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
        hand.iter()
            .copied()
            .filter(|c| c.side() == opp && c.can_event(state, side))
            .collect()
    }
    /// Returns a vector of cards that the given side can themselves event.
    pub fn can_event(&self, side: Side, state: &GameState) -> Vec<Card> {
        let hand = self.hand(side);
        let opp = side.opposite();
        hand.iter()
            .copied()
            .filter(|c| c.side() != opp && c.can_event(state, side))
            .collect()
    }
    /// Returns cards that can be played for just ops, i.e. non-scoring cards of
    /// neutral or allied variety, or those opponent events that won't fire.
    pub fn can_play_ops(&self, side: Side, state: &GameState) -> Vec<Card> {
        let hand = self.hand(side);
        let opp = side.opposite();
        let mut vec: Vec<_> = hand
            .iter()
            .copied()
            .filter(|c| !c.is_scoring() && (c.side() != opp || !c.can_event(state, side)))
            .collect();
        if self.china_available(side) {
            vec.push(Card::The_China_Card);
        }
        vec
    }
    pub fn pop_draw_pile(&mut self) -> Option<Card> {
        self.draw_pile.pop()
    }
    pub fn us_hand_mut(&mut self) -> &mut Vec<Card> {
        &mut self.us_hand
    }
    pub fn us_hand(&self) -> &Vec<Card> {
        &self.us_hand
    }
    pub fn ussr_hand_mut(&mut self) -> &mut Vec<Card> {
        &mut self.ussr_hand
    }
    pub fn ussr_hand(&self) -> &Vec<Card> {
        &self.ussr_hand
    }
    pub fn discard_pile(&self) -> &Vec<Card> {
        &self.discard_pile
    }
    pub fn discard_pile_mut(&mut self) -> &mut Vec<Card> {
        &mut self.discard_pile
    }
    pub fn draw_pile(&self) -> &Vec<Card> {
        &self.draw_pile
    }
    pub fn draw_pile_mut(&mut self) -> &mut Vec<Card> {
        &mut self.draw_pile
    }
    pub fn removed(&self) -> &Vec<Card> {
        &self.removed
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
            if self.pending_discard.contains(&card) {
                return Ok(());
            }
            match side {
                Side::US | Side::USSR => {
                    let hand = self.hand_mut(side);
                    let index = hand
                        .iter()
                        .position(|&c| c == card)
                        .ok_or(DeckError::CannotFind)?;
                    let card = hand.swap_remove(index);
                    self.pending_discard.push(card);
                    Ok(())
                }
                Side::Neutral => Err(DeckError::CannotFind),
            }
        }
    }
    pub fn our_man<T: TwilightRand>(&mut self, rng: &mut T) -> &[Card] {
        // Move the 5 cards to the end of the draw pile
        let mut vec = Vec::with_capacity(5);
        for _ in 0..5 {
            let card = rng.draw_card(self, Side::US);
            vec.push(card);
        }
        self.draw_pile.extend(vec.into_iter());
        &self.draw_pile[self.draw_pile.len() - 5..]
    }
    pub fn discard_draw(&mut self, card: Card) {
        let (index, _) = self
            .draw_pile
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, c)| **c == card)
            .expect("Found card");
        let card = self.draw_pile.swap_remove(index);
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
        let index = self.discard_pile.iter().copied().position(|c| c == card);
        self.discard_pile.swap_remove(index.unwrap());
        let hand = self.hand_mut(side);
        hand.push(card);
    }
    pub fn reset_draw_pile(&mut self) {
        self.draw_pile.append(&mut self.discard_pile);
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
    #[test]
    fn test_our_man() {
        let mut rand = DebugRand::new_empty();
        // Extra cards that we don't want to hit
        rand.us_draw = vec![Card::De_Stalinization, Card::Duck_and_Cover];
        let vec = vec![
            Card::CIA_Created,
            Card::Defectors,
            Card::Decolonization,
            Card::Suez_Crisis,
            Card::Socialist_Governments,
        ];
        rand.us_draw.extend(vec.clone().into_iter());
        let mut deck = Deck::new();
        let cards = deck.our_man(&mut rand);
        assert_eq!(
            cards,
            &vec.clone().into_iter().rev().collect::<Vec<_>>()[..]
        );
        let indices: Vec<_> = cards.iter().map(|c| *c as usize).collect();
        for i in &indices[0..3] {
            deck.discard_draw(Card::from_index(*i));
        }
        // Assert discards worked
        assert_eq!(
            deck.discard_pile,
            vec[2..].iter().cloned().rev().collect::<Vec<_>>()
        );

        // Assert the other cards are still in the draw pile
        for i in &indices[3..] {
            let card = Card::from_index(*i);
            deck.draw_pile.contains(&card);
        }
    }
}
