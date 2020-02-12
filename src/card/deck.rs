use super::*;

#[derive(Clone)]
pub struct Deck {
    us_hand: Vec<Card>,
    ussr_hand: Vec<Card>,
    discard_pile: Vec<Card>,
    draw_pile: Vec<Card>,
    removed: Vec<Card>,
    china: Side,
    china_up: bool,
}

impl Deck {
    pub fn new() -> Self {
        Deck {
            us_hand: Vec::new(),
            ussr_hand: Vec::new(),
            discard_pile: Vec::new(),
            draw_pile: Vec::new(),
            removed: Vec::new(),
            china: Side::USSR,
            china_up: true,
        }
    }
    pub fn hand(&self, side: Side) -> &Vec<Card> {
        match side {
            Side::US => &self.us_hand,
            Side::USSR => &self.ussr_hand,
            Side::Neutral => unimplemented!(),
        }
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
    pub fn draw_cards(&mut self, target: usize) {
        let mut pick_ussr = true;
        // Oscillating is relevant when reshuffles do occur
        while self.ussr_hand.len() < target && self.us_hand.len() < target {
            let next_card = self.draw_card();
            if pick_ussr {
                self.ussr_hand.push(next_card);
            } else {
                self.us_hand.push(next_card);
            }
            pick_ussr = !pick_ussr;
        }
        while self.ussr_hand.len() < target {
            let c = self.draw_card();
            self.ussr_hand.push(c);
        }
        while self.us_hand.len() < target {
            let c = self.draw_card();
            self.us_hand.push(c);
        }
    }
    /// Searches the discard pile for a played card and removes it.
    pub fn remove_card(&mut self, card: Card) -> bool {
        let found = self.discard_pile.iter().rposition(|&c| c == card);
        if let Some(i) = found {
            let c = self.discard_pile.remove(i); // Should be fast since i should be near the end
            self.removed.push(c);
            true
        } else {
            false
        }
    }
    pub fn random_card(&self, side: Side) -> Option<&Card> {
        let hand = self.hand(side);
        let mut rng = thread_rng();
        hand.choose(&mut rng)
    }
    /// Draws the next card from the draw pile, reshuffling if necessary.
    fn draw_card(&mut self) -> Card {
        match self.draw_pile.pop() {
            Some(c) => c,
            None => {
                self.reshuffle();
                self.draw_card()
            }
        }
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
    pub fn draw_pile(&self) -> &Vec<Card> {
        &self.draw_pile
    }
    pub fn removed(&self) -> &Vec<Card> {
        &self.removed
    }
    pub fn play_china(&mut self) {
        self.china = self.china.opposite();
        self.china_up = false;
    }
    pub fn play_card(&mut self, side: Side, card: Card) {
        let hand = match side {
            Side::US => &mut self.us_hand,
            Side::USSR => &mut self.ussr_hand,
            Side::Neutral => unimplemented!(),
        };
        if let Card::The_China_Card = card {
            self.play_china();
        } else {
            let index = hand
                .iter()
                .position(|&c| c == card)
                .expect("Valid card in hand");
            let card = hand.swap_remove(index);
            self.discard_pile.push(card);
        }
    }
    pub fn china_available(&self, side: Side) -> bool {
        self.china == side && self.china_up
    }
    pub fn china(&self) -> Side {
        self.china
    }
    fn reshuffle(&mut self) {
        let mut rng = thread_rng();
        self.discard_pile.shuffle(&mut rng);
        self.draw_pile.append(&mut self.discard_pile);
    }
}
