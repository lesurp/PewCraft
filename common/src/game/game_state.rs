use crate::game::character::Character;
use crate::game::effect::{EffectKind, Range, RangeKind, Target};
use crate::game::error::Error;
use crate::game::game_definition::GameDefinition;
use crate::game::id_map::{Id, IdMap};
use crate::game::map::{Cell, GameMap};
use crate::game::skill::Skill;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TurnState {
    MoveOrAction,
    ActionOnly,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    Movement(Id<Cell>),
    Skill(Id<Skill>, Id<Cell>),
    Pass,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub characters: IdMap<Character>,
    pub map: Id<GameMap>,
    pub turn_order: Vec<Id<Character>>,
    pub turn_state: TurnState,
}

impl GameState {
    pub fn new(g: &GameDefinition, characters: IdMap<Character>, map: Id<GameMap>) -> GameState {
        let mut gs = GameState {
            characters,
            map,
            turn_order: Vec::new(),
            turn_state: TurnState::MoveOrAction,
        };
        gs.new_turn(g);
        gs
    }

    fn new_turn(&mut self, g: &GameDefinition) {
        assert!(self.turn_order.is_empty());

        let mut turn_order = self.characters.iter().collect::<Vec<_>>();
        turn_order.sort_by_key(|(_, character)| {
            let class = g.classes.get(character.class).expect("Invalid class id");
            character.effective_swiftness(class)
        });
        self.turn_order = turn_order.iter().map(|(id, _)| **id).collect::<Vec<_>>();
    }

    pub fn player_to_play(&self) -> Id<Character> {
        self.turn_order
            .last()
            .cloned()
            .expect("Turn is finished and should be reset")
    }

    pub fn next_action(&mut self, g: &GameDefinition, ga: Action) -> Result<bool, Error> {
        let curr_char = self
            .characters
            .get(
                *self
                    .turn_order
                    .last()
                    .expect("Turn is finished and should be reset"),
            )
            .expect("Invalid character id")
            .clone();

        match (ga, self.turn_state) {
            (Action::Pass, _) => {
                self.turn_order.pop();
            }
            (Action::Skill(skill_id, cell_id), _) => {
                self.execute_skill(curr_char, g, skill_id, cell_id)?;
                self.turn_order.pop();
            }
            (Action::Movement(cell_id), TurnState::MoveOrAction) => {
                self.execute_move(curr_char, g, cell_id)?;
            }
            (Action::Movement(_), TurnState::ActionOnly) => return Err(Error::AlreadyMoved),
        }

        Ok(if self.turn_order.is_empty() {
            self.new_turn(g);
            true
        } else {
            false
        })
    }

    fn player_at(&self, cell_id: Id<Cell>) -> Option<(&Id<Character>, &Character)> {
        self.characters
            .iter()
            .find(|(_, character)| character.position == cell_id)
    }

    fn execute_skill(
        &mut self,
        curr_char: Character,
        g: &GameDefinition,
        skill_id: Id<Skill>,
        cell_id: Id<Cell>,
    ) -> Result<(), Error> {
        let skill = g.skills.get(skill_id).ok_or(Error::InvalidSkill)?;
        let target = self.player_at(cell_id);

        if !GameState::check_target(&curr_char, &target, skill.range.target) {
            return Err(Error::InvalidTarget);
        }

        if !self.check_range(g, curr_char.position, cell_id, skill.range) {
            return Err(Error::InvalidRange);
        }

        // TODO check LOS

        // FIXME: this is a borrow-checker workaround... but probably the only actual one..?
        let mut game_state_updates = HashSet::<(Id<Character>, i32)>::new();
        // TODO: compute if hit?
        if true {
            if let Some((id, target)) = target {
                for effect in &skill.effects {
                    let effect = g.effects.get(*effect).expect("Invalid effect id");
                    match &effect.kind {
                        // TODO
                        EffectKind::Buff(_) => unimplemented!(),

                        // TODO: add somewhere if skills can attack other cells than just the
                        // target
                        EffectKind::DirectDamage(direct_damage) => {
                            let damage = direct_damage
                                .damage
                                .compute_damage(&g.classes, &curr_char, target);
                            game_state_updates.insert((*id, damage));
                        }
                    }
                }
            }
        }

        // TODO stuff if dead
        for (id, damage) in game_state_updates {
            let character = self.characters.get_mut(id).expect("Invalid character id");
            character.current_health -= damage;
        }

        Ok(())
    }

    fn check_target(
        attacker: &Character,
        target_opt: &Option<(&Id<Character>, &Character)>,
        target_kind: Target,
    ) -> bool {
        if let Some((_, target)) = target_opt {
            match (target_kind, target.team == attacker.team) {
                (Target::Anything, _) | (Target::Anyone, _) => true,
                (Target::Enemy, is_same_team) => !is_same_team,
                (Target::Ally, is_same_team) => is_same_team,
            }
        } else {
            Target::Anything == target_kind
        }
    }

    fn check_range(
        &self,
        g: &GameDefinition,
        start: Id<Cell>,
        end: Id<Cell>,
        range: Range,
    ) -> bool {
        let map = g.maps.get(self.map).expect("Invalid game map id");

        match range.kind {
            RangeKind::Star => {
                let distance = map.distance(start, end);
                distance <= range.max && distance >= range.min
            }
            RangeKind::Cross => {
                let (sx, sy) = map.id_to_xy_i32(start);
                let (ex, ey) = map.id_to_xy_i32(end);

                let dx = sx - ex;
                let dy = sy - ey;

                let max = range.max as i32;
                let min = range.min as i32;

                dx == 0 && dy.abs() <= max && dy.abs() >= min
                    || dy == 0 && dx.abs() <= max && dx.abs() >= min
            }
            RangeKind::Square => {
                let (sx, sy) = map.id_to_xy_i32(start);
                let (ex, ey) = map.id_to_xy_i32(end);

                let dx = sx - ex;
                let dy = sy - ey;

                let max = range.max as i32;
                let min = range.min as i32;

                dx.abs() <= max && dx.abs() >= min && dy >= min && dy <= max
            }
        }
    }

    fn execute_move(
        &mut self,
        curr_char: Character,
        g: &GameDefinition,
        cell_id: Id<Cell>,
    ) -> Result<(), Error> {
        assert!(!self.turn_order.is_empty());

        let class = g.classes.get(curr_char.class).expect("Invalid class id");
        if g.maps
            .get(self.map)
            .expect("Invalid game map id")
            .can_move_to(
                curr_char.position,
                cell_id,
                curr_char.effective_swiftness(class),
            )
        {
            self.turn_state = TurnState::ActionOnly;
            Ok(())
        } else {
            Err(Error::MoveCellTooFar)
        }
    }
}
