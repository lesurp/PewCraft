use crate::game::{Action, Cell, Character, Class, GameMap, GameState, Id, Team};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireAction(pub Action);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireNewCharRequest {
    pub name: String,
    pub class: Id<Class>,
    pub team: Id<Team>,
    pub position: Id<Cell>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireCreatedGame {
    pub game_id: String,
    pub map: Id<GameMap>,
    pub team_size: usize,
    pub players: Vec<WireNewCharRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireCreatedChar(pub String, pub Id<Character>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireNewGameRequest {
    pub map: Id<GameMap>,
    pub team_size: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireRunningGame(pub GameState);
