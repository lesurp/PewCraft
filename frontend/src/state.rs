use crate::api::{self, Endpoint};
use common::game::{Cell, Character, Class, GameDefinition, GameMap, GameState, Id, Team};
use common::io::{WireCreatedChar, WireCreatedGame, WireNewCharRequest, WireNewGameRequest};
use log::debug;

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

#[derive(Debug)]
pub enum ExpectedEvent {
    Char,
    SelectionVertical,
    SelectionHorizontal,
    Selection,
    None,
}

pub trait State {
    fn expected_event(&self) -> ExpectedEvent;
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
    fn expected_event(&self) -> ExpectedEvent {
        match self {
            GlobalState::CreateOrJoin(s) => s.expected_event(),
            GlobalState::SelectMap(_) => ExpectedEvent::SelectionHorizontal,
            GlobalState::WaitForGameCreation(_) => ExpectedEvent::None,
            GlobalState::CreateCharacter(s) => s.expected_event(),
            GlobalState::PlayGame(s) => s.expected_event(),
            GlobalState::Exit => ExpectedEvent::None,
        }
    }

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
            step: CreateCharacterStep::Name,
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
    fn expected_event(&self) -> ExpectedEvent {
        match self {
            CreateOrJoinState::Join(_) => ExpectedEvent::Char,
            CreateOrJoinState::Create(_) => ExpectedEvent::None,
        }
    }

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
            (CreateOrJoinState::Join(s), Event::Cancel) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Create(s))
            }
            /*
            (CreateOrJoinState::Join(s), Event::Confirm) => {
                let (global, join) = s.split();
                let login = &join.login;
                match login.len() {
                    10 => {
                        let joined_game = global.endpoint.join_game(login);
                        match joined_game {
                            None => GlobalState::CreateOrJoin(CreateOrJoinState::Create(
                                CreateOrJoinData::new(global, join),
                            )),
                            Some(game_info) => {
                                let map = global.game.maps.get(game_info.map).unwrap();
                                let create_character_state_data = CreateCharacterState {
                                    name: String::new(),
                                    class_index: 0,
                                    team_index: 0,
                                    position_index: 0,

                                    classes: global.game.classes.ids(),
                                    teams: map
                                        .teams
                                        .iter()
                                        .enumerate()
                                        .map(|(index, _)| Id::new(index))
                                        .collect(),
                                    map,
                                    game_id: game_info.game_id,
                                };
                                let state_data =
                                    StateData::new(global, create_character_state_data);
                                let create_character_state = CreateCharacterState::Team(state_data);
                                GlobalState::CreateCharacter(create_character_state)
                            }
                        }
                    }
                    21 => {
                        if login.chars().nth(10).unwrap() != '/' {
                            GlobalState::CreateOrJoin(CreateOrJoinState::Create(
                                CreateOrJoinData::new(global, join),
                            ))
                        } else {
                            let game_id = &login[..10];
                            let char_id = &login[11..];
                            let _joined_game =
                                global.endpoint.join_game_with_char(game_id, char_id);
                            unimplemented!()
                            //match joined_game
                        }
                    }
                    _ => GlobalState::CreateOrJoin(CreateOrJoinState::Create(
                        CreateOrJoinData::new(global, join),
                    )),
                }
            }
            */
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
    fn expected_event(&self) -> ExpectedEvent {
        todo!()
    }

    fn next(
        mut self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        match event {
            Event::Right => self.curr_id = (self.curr_id + 1) % self.map_ids.len(),
            Event::Left => self.curr_id = (self.curr_id - 1) % self.map_ids.len(),
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

impl State for CreateCharacterState {
    fn expected_event(&self) -> ExpectedEvent {
        todo!()
        /*
        match self {
            CreateCharacterState::Team(_) => ExpectedEvent::SelectionHorizontal,
            CreateCharacterState::Class(_) => ExpectedEvent::SelectionHorizontal,
            CreateCharacterState::Position(_) => ExpectedEvent::Selection,
            CreateCharacterState::Name(_) => ExpectedEvent::Char,
        }
        */
    }

    fn next(
        self,
        game_definition: &GameDefinition,
        endpoint: &Endpoint,
        event: Event,
    ) -> GlobalState {
        todo!()
        /*
            match (self, i) {
                // FIRST CHOOSE THE TEAM
                (CreateCharacterState::Team(mut s), Event::Right) => {
                    if s.curr().team_index == s.curr().teams.len() - 1 {
                        s.curr_mut().team_index = 0;
                    } else {
                        s.curr_mut().team_index += 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Team(s))
                }
                (CreateCharacterState::Team(mut s), Event::Left) => {
                    if s.curr_mut().team_index == 0 {
                        s.curr_mut().team_index = s.curr().teams.len() - 1;
                    } else {
                        s.curr_mut().team_index -= 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Team(s))
                }
                (CreateCharacterState::Team(s), Event::Confirm) => {
                    GlobalState::CreateCharacter(CreateCharacterState::Class(s))
                }

                // THEN THE CLASS
                (CreateCharacterState::Class(mut s), Event::Right) => {
                    if s.curr().class_index == s.curr().classes.len() - 1 {
                        s.curr_mut().class_index = 0;
                    } else {
                        s.curr_mut().class_index += 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Class(s))
                }
                (CreateCharacterState::Class(mut s), Event::Left) => {
                    if s.curr_mut().class_index == 0 {
                        s.curr_mut().class_index = s.curr().classes.len() - 1;
                    } else {
                        s.curr_mut().class_index -= 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Class(s))
                }
                (CreateCharacterState::Class(s), Event::Confirm) => {
                    GlobalState::CreateCharacter(CreateCharacterState::Position(s))
                }

                // THEN THE POSITION
                (CreateCharacterState::Position(mut s), Event::Right) => {
                    let positions = &s.curr().map.teams.get(s.curr().team_index).unwrap().1;
                    if s.curr().position_index == positions.len() - 1 {
                        s.curr_mut().position_index = 0;
                    } else {
                        s.curr_mut().position_index += 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Position(s))
                }
                (CreateCharacterState::Position(mut s), Event::Left) => {
                    let positions = &s.curr().map.teams.get(s.curr().team_index).unwrap().1;
                    if s.curr_mut().position_index == 0 {
                        s.curr_mut().position_index = positions.len() - 1;
                    } else {
                        s.curr_mut().position_index -= 1;
                    }
                    GlobalState::CreateCharacter(CreateCharacterState::Position(s))
                }
                (CreateCharacterState::Position(s), Event::Confirm) => {
                    GlobalState::CreateCharacter(CreateCharacterState::Name(s))
                }

                // THEN THE NAME
                (CreateCharacterState::Name(mut s), Event::PrintableString(string)) => {
                    s.curr_mut().name.push_str(&string);
                    GlobalState::CreateCharacter(CreateCharacterState::Name(s))
                }
                (CreateCharacterState::Name(mut s), Event::Backspace) => {
                    s.curr_mut().name.pop();
                    GlobalState::CreateCharacter(CreateCharacterState::Name(s))
                }
                (CreateCharacterState::Name(s), Event::Confirm) => {
                    debug!("Creating character with name {}", s.curr().name);
                    let (global, create_char) = s.split();
                    let name = create_char.name;
                    let class = *create_char.classes.get(create_char.class_index).unwrap();
                    let team = *create_char.teams.get(create_char.team_index).unwrap();
                    let position = *create_char
                        .map
                        .teams
                        .get(team.raw())
                        .unwrap()
                        .1
                        .get(create_char.position_index)
                        .unwrap();
                    let WireCreatedChar(login, id) = global.endpoint.create_char(
                        &create_char.game_id,
                        WireNewCharRequest {
                            name,
                            class,
                            team,
                            position,
                        },
                    );
                    GlobalState::WaitForGameCreation(WaitForGameCreationData::new(
                        global,
                        WaitForGameCreationState {
                            map: create_char.map,
                            game_id: create_char.game_id,
                            login,
                            id,
                        },
                    ))
                }
                unchanged => GlobalState::CreateCharacter(unchanged.0),
            }
        */
    }
}

#[derive(Debug)]
pub struct PlayGameState {
    pub cell: Id<Cell>,
    pub game_state: GameState,
    //pub map: &'a GameMap,
    pub map: Vec<Id<GameMap>>,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
    pub is_our_turn: bool,
}

impl State for PlayGameState {
    fn expected_event(&self) -> ExpectedEvent {
        if self.is_our_turn {
            unimplemented!()
        } else {
            ExpectedEvent::None
        }
    }

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
    pub map: Vec<Id<GameMap>>,
    //pub map: &'a GameMap,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
}
