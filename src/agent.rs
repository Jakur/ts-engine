use crate::action::{Decision, Action, Allowed};
use crate::card::Card;
use crate::country::Side;
use crate::state::GameState;

use rand::prelude::*;

pub struct Actors<A: Agent, B: Agent> {
    pub ussr_agent: A,
    pub us_agent: B,
}

impl<A, B> Actors<A, B>
where
    A: Agent,
    B: Agent,
{
    pub fn new(ussr_agent: A, us_agent: B) -> Actors<A, B> {
        Actors {
            ussr_agent,
            us_agent,
        }
    }
    pub fn get(&self, side: Side) -> &dyn Agent {
        match side {
            Side::USSR => &self.ussr_agent,
            Side::US => &self.us_agent,
            _ => unimplemented!(),
        }
    }
}

pub trait Agent {
    fn step(&self, state: &mut GameState, pending: &mut Vec<Decision>);
    fn side(&self) -> Side;
    /// Decides a country to act upon, or None if there is no legal option. Also
    /// includes a numerical evaluation of the position from the agent's perspective.
    fn decide_action(
        &self,
        state: &GameState,
        choices: &[usize],
        action: Action,
    ) -> (Option<usize>, f32);
    /// Picks a card among valid options for an action, either picking a card
    /// to play or else to discard. Also includes a numerical evaluation of the
    /// position from the agent's perspective.
    fn decide_card(
        &self,
        state: &GameState,
        cards: &[Card],
        china: bool,
        play: bool,
        can_pass: bool,
    ) -> (Option<Card>, f32);
    /// Returns just the evaluation of the given position
    fn get_eval(&self, state: &GameState) -> f32;
}

#[derive(Clone)]
pub struct DebugAgent<'a> {
    pub fav_action: Action,
    pub fav_card: Card,
    pub choices: &'a [usize],
}

impl<'a> DebugAgent<'a> {
    pub fn new(fav_action: Action, fav_card: Card, choices: &'a [usize]) -> Self {
        DebugAgent {
            fav_action,
            fav_card,
            choices,
        }
    }
}

impl<'a> Agent for DebugAgent<'a> {
    fn decide_action(&self, _s: &GameState, choices: &[usize], a: Action) -> (Option<usize>, f32) {
        use std::mem::discriminant;
        // Todo resolve first?
        let eval = if discriminant(&a) == discriminant(&self.fav_action) {
            1.0
        } else {
            0.0
        };
        for fav in self.choices.iter() {
            if choices.contains(fav) {
                return (Some(*fav), eval);
            }
        }
        let choice = if choices.len() != 0 {
            Some(choices[0])
        } else {
            None
        };
        (choice, eval)
    }
    fn decide_card(
        &self,
        _state: &GameState,
        _cards: &[Card],
        _china: bool,
        _play: bool,
        _can_pass: bool,
    ) -> (Option<Card>, f32) {
        todo!()
    }
    fn get_eval(&self, _state: &GameState) -> f32 {
        todo!()
    }
    fn step(&self, _: &mut GameState, _pending: &mut Vec<Decision>) { 
        unimplemented!() 
    }
    fn side(&self) -> Side {
        unimplemented!()
    }
}

pub struct RandAgent {}
impl RandAgent {
    pub fn new() -> Self {
        RandAgent {}
    }
}

impl Agent for RandAgent {
    fn decide_action(&self, _s: &GameState, choices: &[usize], _a: Action) -> (Option<usize>, f32) {
        if choices.len() == 0 {
            return (None, 0.0); // Todo detect this earlier?
        }
        let mut x = thread_rng();
        let choice = x.gen_range(0, choices.len());
        (Some(choices[choice]), x.gen())
    }
    fn decide_card(
        &self,
        _state: &GameState,
        hand: &[Card],
        china: bool,
        _play: bool,
        _can_pass: bool,
    ) -> (Option<Card>, f32) {
        let mut x = thread_rng();
        if hand.len() > 0 {
            let choice = if china {
                x.gen_range(0, hand.len() + 1)
            } else {
                x.gen_range(0, hand.len())
            };
            let card = if choice >= hand.len() {
                Card::The_China_Card
            } else {
                hand[choice]
            };
            (Some(card), x.gen())
        } else {
            let choices = &[None, Some(Card::The_China_Card)];
            // A player cannot be forced to play the China card
            if china {
                (choices[x.gen_range(0, 2)], x.gen())
            } else {
                (None, x.gen())
            }
        }
    }
    fn get_eval(&self, _state: &GameState) -> f32 {
        thread_rng().gen()
    }
    fn step(&self, state: &mut GameState, pending: &mut Vec<Decision>) { 
        let mut rng = thread_rng();
        if let Some(dec) = pending.last() {
            match dec.action {
                Action::Discard(_) => todo!(),
                Action::Event(c, ch) => {
                    assert!(ch.is_none()); // This should be caught before otherwise, I think
                    let e_options = c.e_choices(state).unwrap();
                    let choice = e_options.choose(&mut rng).unwrap_or(&0);
                    c.event(state, *choice, pending);
                },
                Action::IndependentReds => {
                    let e_options = Card::Independent_Reds.e_choices(state).unwrap();
                    let choice = e_options.choose(&mut rng).unwrap_or(&0);
                    Card::Independent_Reds.event(state, *choice, pending);
                },
                Action::Pass => {},
                _ => {
                    todo!()
                },
            }
        } else {
            let uses = state.card_uses();
            let select = uses.choose(&mut rng).unwrap();
            let d = Decision::use_card(self.side(), select.clone());
            match select {
                Action::Pass => todo!(),
                Action::Space(c) => state.resolve_card(d, *c),
                Action::Event(card, choice) => {
                    let choice = choice.unwrap(); // Safe from use_card()
                    card.event(state, choice, pending);
                }
                Action::PlayCard(_, _) => pending.push(d),
                _ => unreachable!(),
            }
            if let Action::Pass = select {
                todo!();
            }
        }

    }
    fn side(&self) -> Side {
        unimplemented!()
    }
}

fn all_legal_moves(agent: Side, state: &GameState, action: &Action) -> Vec<usize> {
    match action {
        // Action::PlayCard => {

        // }
        Action::ConductOps => {
            let mut vec = Vec::new();
            for x in [Action::StandardOps, Action::Coup(1, false), Action::Realignment].iter() {
                let mut d = Decision::new(agent, x.clone(), &[]);
                let allowed = state.standard_allowed(&d, &[]).unwrap(); // Todo always return Some
                d.allowed = Allowed::new_owned(allowed);
                vec.append(&mut d.tensor_indices(state))
            }
            vec
        },
        _ => todo!(),
    }
}
