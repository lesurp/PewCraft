use crate::game::class::Class;
use crate::game::effect::Buff;
use crate::game::error::Error;
use crate::game::game_definition::GameDefinition;
use crate::game::id_map::{Id, IdMap, IdMapBuilder};
use crate::game::map::Cell;
use crate::game::map::{GameMap, Team};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CharacterMapBuilder<'a> {
    builder: IdMapBuilder<Character>,
    empty_starting_cells: Vec<(usize, Vec<Id<Cell>>)>,
    game_definition: &'a GameDefinition,
}

impl<'a> CharacterMapBuilder<'a> {
    pub fn new(game_definition: &'a GameDefinition, map_id: Id<GameMap>, team_size: usize) -> Self {
        let empty_starting_cells = game_definition
            .maps
            .get(map_id)
            .expect("Invalid map id")
            .teams
            .iter()
            .map(|team| (team_size, team.1.clone()))
            .collect();

        CharacterMapBuilder {
            builder: IdMapBuilder::new(),
            empty_starting_cells,
            game_definition,
        }
    }

    pub fn add(&mut self, c: Character) -> Result<Id<Character>, Error> {
        let (spots_left, cells_left) = &mut self
            .empty_starting_cells
            .get_mut(c.team.raw())
            .expect("Wrong team id was assigned to the character");

        if self.game_definition.classes.get(c.class).is_none() {
            return Err(Error::InvalidCharacterClass);
        }

        if c.name.trim().is_empty() {
            return Err(Error::InvalidCharacterName);
        }

        // Team is full
        if *spots_left == 0 {
            return Err(Error::TeamFull);
        }

        // Available cells left
        if !cells_left.remove_item(&c.position).is_some() {
            return Err(Error::InvalidStartingCell);
        }

        *spots_left -= 1;
        Ok(self.builder.add(c))
    }

    pub fn can_build(&self) -> bool {
        for (spots_left, _) in &self.empty_starting_cells {
            if *spots_left > 0 {
                return false;
            }
        }
        true
    }

    pub fn build(self) -> IdMap<Character> {
        for (spots_left, _) in &self.empty_starting_cells {
            debug_assert!(*spots_left == 0);
        }
        self.builder.build()
    }

    pub fn try_build(self) -> Result<IdMap<Character>, Self> {
        for (spots_left, _) in &self.empty_starting_cells {
            if *spots_left > 0 {
                return Err(self);
            }
        }
        Ok(self.builder.build())
    }
}

// TODO: store the applier of the buff's stats here!
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuffInstance(Buff, ());

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub class: Id<Class>,

    pub current_health: i32,
    pub current_mana: i32,
    pub position: Id<Cell>,
    pub buffs: Vec<BuffInstance>,
    pub team: Id<Team>,
}

impl Character {
    pub fn new<S: Into<String>>(
        id: Id<Class>,
        position: Id<Cell>,
        class: &Class,
        name: S,
        team: Id<Team>,
    ) -> Character {
        Character {
            name: name.into(),
            class: id,
            current_health: class.health,
            current_mana: class.mana,
            position,
            buffs: Vec::new(),
            team,
        }
    }

    // TODO: look at current debuffs to return the "effective" stats
    pub fn effective_health(&self, class: &Class) -> i32 {
        class.health
    }

    pub fn effective_mana(&self, class: &Class) -> i32 {
        class.mana
    }

    pub fn effective_swiftness(&self, class: &Class) -> i32 {
        class.swiftness
    }

    // PHYSICAL STUFF
    pub fn effective_strength(&self, class: &Class) -> i32 {
        class.strength
    }

    pub fn effective_dexterity(&self, class: &Class) -> i32 {
        class.dexterity
    }

    pub fn effective_armor(&self, class: &Class) -> i32 {
        class.armor
    }

    // MAGICAL STUFF
    pub fn effective_intelligence(&self, class: &Class) -> i32 {
        class.intelligence
    }

    pub fn effective_concentration(&self, class: &Class) -> i32 {
        class.concentration
    }

    pub fn effective_willpower(&self, class: &Class) -> i32 {
        class.willpower
    }
}
