use crate::game::effect::{Effect, Range};
use crate::game::id_map::Id;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub name: String,

    pub cost: i32,
    pub range: Range,
    pub precision: Option<f32>,
    pub effects: HashSet<Id<Effect>>,
}
