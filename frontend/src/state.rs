use crate::api::{self, Endpoint};
use common::game::{Cell, Character, Class, GameDefinition, GameMap, GameState, Id, Team};
use common::io::{WireCreatedChar, WireCreatedGame, WireNewCharRequest, WireNewGameRequest};
use log::debug;

fn wrap_inc(n: usize, max_n: usize) -> usize {
    (n + 1) % max_n
}

fn wrap_dec(n: usize, max_n: usize) -> usize {
    match n {
        0 => max_n - 1,
        n => n - 1,
    }
}

#[derive(Debug)]
pub enum Event {
    Timeout,
    PrintableString(String),
    Exit,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Cancel,
    Confirm,
    Other,
}

pub trait State {
    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState;
}

#[derive(Debug)]
pub enum GlobalState {
    CreateOrJoin(CreateOrJoinState),
    SelectMap(SelectMapState),
    WaitForGameCreation(WaitForGameCreationState),
    CreateCharacter(CreateCharacterState),
    PlayGame(PlayGameState),
    Exit,
}

impl State for GlobalState {
    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match (self, event) {
            (_, Event::Exit) => GlobalState::Exit,
            (state, Event::Other) | (state, Event::Timeout) => state,
            (GlobalState::CreateOrJoin(create_or_join), event) => {
                create_or_join.next(game_definition, endpoint, event)
            }
            (GlobalState::SelectMap(s), event) => s.next(game_definition, endpoint, event),
            (GlobalState::CreateCharacter(c), event) => c.next(game_definition, endpoint, event),
            (s, i) => unimplemented!("Input: {i:?}\nState: {s:?}"),
            /*
            /* SelectMap */
            (GlobalState::SelectMap(mut s), Event::Right) => {
                if s.curr().curr_id == s.curr().map_ids.len() - 1 {
                    s.curr_mut().curr_id = 0
                } else {
                    s.curr_mut().curr_id += 1;
                }
                GlobalState::SelectMap(s)
            }
            (GlobalState::SelectMap(mut s), Event::Left) => {
                if s.curr().curr_id == 0 {
                    s.curr_mut().curr_id = s.curr().map_ids.len() - 1;
                } else {
                    s.curr_mut().curr_id -= 1;
                }
                GlobalState::SelectMap(s)
            }
            (GlobalState::SelectMap(s), Event::Confirm) => {
                let map_id = *s.curr().map_ids.get(s.curr().curr_id).unwrap();
                // TODO hardcoded team size
                // TODO this can fail :)
                let request = WireNewGameRequest {
                    map: map_id,
                    team_size: 2,
                };
                let created_game = s.endpoint.create_game(request);
                let map = s.game.maps.get(map_id).unwrap();
                GlobalState::join_game(created_game, map, s)
            }

            (GlobalState::CreateCharacter(c), i) => c.next(i),
            unchanged @ (GlobalState::WaitForGameCreation(_), _) => unchanged.0,

            */
        }
    }
}

impl GlobalState {
    pub fn get_game_id(&self) -> Option<&str> {
        match self {
            GlobalState::CreateOrJoin(_) => None,
            GlobalState::SelectMap(_) => None,

            /*
            GlobalState::CreateCharacter(c) => match c {
                CreateCharacterState::Team(c)
                | CreateCharacterState::Class(c)
                | CreateCharacterState::Position(c)
                | CreateCharacterState::Name(c) => Some(&c.curr().game_id),
            },

            GlobalState::WaitForGameCreation(c) => {
                Some(format!("{}/{}", c.curr().game_id, c.curr().login))
            }
            GlobalState::PlayGame(play) => match play {
                PlayGameState::OurTurn(c) | PlayGameState::NotOurTurn(c) => {
                    Some(format!("{}/{}", c.curr().game_id, c.curr().login))
                }
            },
            */
            GlobalState::Exit => unreachable!(),
            _ => todo!(),
        }
    }

    pub fn exit(&self) -> bool {
        matches!(self, GlobalState::Exit)
    }

    pub fn new() -> Self {
        GlobalState::CreateOrJoin(CreateOrJoinState::Create(CreateOrJoinData {
            login: String::new(),
        }))
    }

    pub fn join_game(
        created_game: WireCreatedGame,
        game_definition: &GameDefinition,
        map: Id<GameMap>,
    ) -> GlobalState {
        let create_character_state = CreateCharacterState {
            step: CreateCharacterStep::Team,
            name: String::new(),
            class_index: 0,
            team_index: 0,
            position_index: 0,

            classes: game_definition.classes.ids(),
            teams: game_definition
                .maps
                .get(map)
                .unwrap()
                .teams
                .iter()
                .enumerate()
                .map(|(index, _)| Id::new(index))
                .collect(),
            map,
            game_id: created_game.game_id,
        };

        GlobalState::CreateCharacter(create_character_state)
    }
}

#[derive(Debug)]
pub struct CreateOrJoinData {
    pub login: String,
}

#[derive(Debug)]
pub enum CreateOrJoinState {
    Create(CreateOrJoinData),
    Join(CreateOrJoinData),
}

impl State for CreateOrJoinState {
    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match (self, event) {
            (CreateOrJoinState::Join(s), Event::Right)
            | (CreateOrJoinState::Join(s), Event::Up)
            | (CreateOrJoinState::Join(s), Event::Down)
            | (CreateOrJoinState::Join(s), Event::Left) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Create(s))
            }
            (CreateOrJoinState::Join(mut s), Event::PrintableString(string)) => {
                s.login.push_str(&string);
                GlobalState::CreateOrJoin(CreateOrJoinState::Join(s))
            }
            (CreateOrJoinState::Join(mut s), Event::Backspace) => {
                s.login.pop();
                GlobalState::CreateOrJoin(CreateOrJoinState::Join(s))
            }
            (CreateOrJoinState::Join(s), Event::Cancel) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Create(s))
            }
            (CreateOrJoinState::Join(s), Event::Confirm) => {
                match s.login.len() {
                    10 => {
                        let joined_game = endpoint.game_state(&s.login);
                        match joined_game {
                            None => GlobalState::CreateOrJoin(CreateOrJoinState::Join(s)),
                            Some(game_info) => {
                                let map = game_definition.maps.get(game_info.map).unwrap();
                                let create_character = CreateCharacterState {
                                    name: String::new(),
                                    step: CreateCharacterStep::Team,
                                    class_index: 0,
                                    team_index: 0,
                                    position_index: 0,

                                    classes: game_definition.classes.ids(),
                                    teams: map
                                        .teams
                                        .iter()
                                        .enumerate()
                                        .map(|(index, _)| Id::new(index))
                                        .collect(),
                                    map: game_info.map,
                                    game_id: s.login,
                                };
                                GlobalState::CreateCharacter(create_character)
                            }
                        }
                    }
                    21 => {
                        if s.login.chars().nth(10).unwrap() != '/' {
                            return GlobalState::CreateOrJoin(CreateOrJoinState::Join(s));
                        }
                        let game_id = &s.login[..10];
                        let char_id = &s.login[11..];
                        let _joined_game = endpoint.join_game_with_char(game_id, char_id);
                        unimplemented!()
                        //match joined_game
                    }
                    _ => GlobalState::CreateOrJoin(CreateOrJoinState::Join(s)),
                }
            }
            (CreateOrJoinState::Create(s), Event::Right)
            | (CreateOrJoinState::Create(s), Event::Up)
            | (CreateOrJoinState::Create(s), Event::Down)
            | (CreateOrJoinState::Create(s), Event::Left) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Join(s))
            }
            (CreateOrJoinState::Create(_s), Event::Confirm) => {
                GlobalState::SelectMap(SelectMapState::new(game_definition))
            }

            unchanged => GlobalState::CreateOrJoin(unchanged.0),
        }
    }
}

#[derive(Debug)]
pub struct SelectMapState {
    pub map_ids: Vec<Id<GameMap>>,
    pub curr_id: usize,
}

impl SelectMapState {
    fn new(game_definition: &GameDefinition) -> SelectMapState {
        SelectMapState {
            map_ids: game_definition.maps.ids(),
            curr_id: 0,
        }
    }
}

impl State for SelectMapState {
    fn next(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match event {
            Event::Right => self.curr_id = wrap_inc(self.curr_id, self.map_ids.len()),
            Event::Left => self.curr_id = wrap_dec(self.curr_id, self.map_ids.len()),
            Event::Confirm => {
                let map_id = *self.map_ids.get(self.curr_id).unwrap();
                // TODO hardcoded team size
                let request = WireNewGameRequest {
                    map: map_id,
                    team_size: 2,
                };
                // TODO this can fail :)
                let created_game = endpoint.create_game(request);
                return GlobalState::join_game(created_game, game_definition, map_id);
            }
            _ => {}
        }
        GlobalState::SelectMap(self)
    }
}

#[derive(Debug)]
pub enum CreateCharacterStep {
    Class,
    Team,
    Position,
    Name,
}

#[derive(Debug)]
pub struct CreateCharacterState {
    pub name: String,
    pub class_index: usize,
    pub team_index: usize,
    pub position_index: usize,

    pub classes: Vec<Id<Class>>,
    pub teams: Vec<Id<Team>>,
    //pub map: &'a GameMap,
    pub map: Id<GameMap>,
    pub game_id: String,
    pub step: CreateCharacterStep,
}

impl CreateCharacterState {
    fn handle_event_name(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match event {
            Event::PrintableString(string) => {
                self.name.push_str(&string);
                GlobalState::CreateCharacter(self)
            }
            Event::Backspace => {
                self.name.pop();
                GlobalState::CreateCharacter(self)
            }
            Event::Confirm => {
                debug!("Creating character with name {}", self.name);
                let name = self.name;
                let class = *self.classes.get(self.class_index).unwrap();
                let team = *self.teams.get(self.team_index).unwrap();
                let position = *game_definition
                    .maps
                    .get(self.map)
                    .unwrap()
                    .teams
                    .get(team.raw())
                    .unwrap()
                    .1
                    .get(self.position_index)
                    .unwrap();
                let WireCreatedChar(login, id) = endpoint.create_char(
                    &self.game_id,
                    WireNewCharRequest {
                        name,
                        class,
                        team,
                        position,
                    },
                );
                GlobalState::WaitForGameCreation(WaitForGameCreationState {
                    map: self.map,
                    game_id: self.game_id,
                    login,
                    id,
                })
            }
            _ => GlobalState::CreateCharacter(self),
        }
    }

    fn handle_event_team(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match event {
            Event::Right => {
                self.team_index = wrap_inc(self.team_index, self.teams.len());
            }
            Event::Left => {
                self.team_index = wrap_dec(self.team_index, self.teams.len());
            }
            Event::Confirm => {
                self.step = CreateCharacterStep::Class;
            }
            _ => {}
        }
        GlobalState::CreateCharacter(self)
    }

    fn handle_event_class(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match event {
            Event::Right => {
                self.class_index = wrap_inc(self.class_index, self.classes.len());
            }
            Event::Left => {
                self.class_index = wrap_dec(self.class_index, self.classes.len());
            }
            Event::Confirm => {
                self.step = CreateCharacterStep::Position;
            }
            _ => {}
        }
        GlobalState::CreateCharacter(self)
    }

    fn handle_event_position(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        let positions = &game_definition
            .maps
            .get(self.map)
            .unwrap()
            .teams
            .get(self.team_index)
            .unwrap()
            .1;
        match event {
            Event::Right => {
                self.position_index = wrap_inc(self.position_index, positions.len());
            }
            Event::Left => {
                self.position_index = wrap_dec(self.position_index, positions.len());
            }
            Event::Confirm => {
                self.step = CreateCharacterStep::Name;
            }
            _ => {}
        }
        GlobalState::CreateCharacter(self)
    }
}

impl State for CreateCharacterState {
    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match self.step {
            CreateCharacterStep::Class => self.handle_event_class(game_definition, endpoint, event),
            CreateCharacterStep::Team => self.handle_event_team(game_definition, endpoint, event),
            CreateCharacterStep::Position => {
                self.handle_event_position(game_definition, endpoint, event)
            }
            CreateCharacterStep::Name => self.handle_event_name(game_definition, endpoint, event),
        }
    }
}

#[derive(Debug)]
pub struct PlayGameState {
    pub cell: Id<Cell>,
    pub game_state: GameState,
    //pub map: &'a GameMap,
    pub map: Id<GameMap>,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
    pub is_our_turn: bool,
}

impl State for PlayGameState {
    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct WaitForGameCreationState {
    pub map: Id<GameMap>,
    //pub map: &'a GameMap,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
}
