use crate::api;
use log::debug;
use pewcraft_common::game::{Cell, Character, Class, GameDefinition, GameMap, GameState, Id, Team};
use pewcraft_common::io::{
    WireCreatedChar, WireCreatedGame, WireNewCharRequest, WireNewGameRequest,
};

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
    type RootState: State<RootState = Self::RootState>;
    fn expected_event(&self) -> ExpectedEvent;
    fn next(self, i: Event) -> Self::RootState;
}

#[derive(Debug)]
pub struct StateData<Prev, Curr>(Prev, Curr);
impl<P, C> StateData<P, C> {
    pub fn new(p: P, c: C) -> Self {
        StateData(p, c)
    }

    pub fn prev(&self) -> &P {
        &self.0
    }

    pub fn curr(&self) -> &C {
        &self.1
    }

    fn curr_mut(&mut self) -> &mut C {
        &mut self.1
    }

    fn split(self) -> (P, C) {
        (self.0, self.1)
    }
}

#[derive(Debug)]
pub enum GlobalState<'a> {
    CreateOrJoin(CreateOrJoinState<'a>),
    SelectMap(SelectMapData<'a>),
    WaitForGameCreation(WaitForGameCreationData<'a>),
    CreateCharacter(CreateCharacterState<'a>),
    PlayGame(PlayGameState<'a>),
    Exit,
}

#[derive(Debug)]
pub struct GlobalStateData<'a> {
    pub game: &'a GameDefinition,
    endpoint: &'a api::Endpoint,
}

impl<'a> State for GlobalState<'a> {
    type RootState = Self;

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

    fn next(self, i: Event) -> Self::RootState {
        match (self, i) {
            (_, Event::Exit) => GlobalState::Exit,
            unchanged @ (_, Event::Other) => unchanged.0,

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
                let created_game = s.prev().endpoint.create_game(request);
                let map = s.prev().game.maps.get(map_id).unwrap();
                GlobalState::join_game(created_game, map, s)
            }

            (GlobalState::CreateCharacter(c), i) => c.next(i),
            unchanged @ (GlobalState::WaitForGameCreation(_), _) => unchanged.0,

            unchanged @ (_, Event::Timeout) => unchanged.0,
            (s, i) => {
                panic!("Input: {:?}\nState: {:?}", i, s);
            }
        }
    }
}

impl<'a> GlobalState<'a> {
    pub fn get_game_id(&self) -> Option<String> {
        match self {
            GlobalState::CreateOrJoin(_) => None,
            GlobalState::SelectMap(_) => None,

            GlobalState::CreateCharacter(c) => match c {
                CreateCharacterState::Team(c)
                | CreateCharacterState::Class(c)
                | CreateCharacterState::Position(c)
                | CreateCharacterState::Name(c) => Some(c.curr().game_id.clone()),
            },

            GlobalState::WaitForGameCreation(c) => {
                Some(format!("{}/{}", c.curr().game_id, c.curr().login))
            }
            GlobalState::PlayGame(play) => match play {
                PlayGameState::OurTurn(c) | PlayGameState::NotOurTurn(c) => {
                    Some(format!("{}/{}", c.curr().game_id, c.curr().login))
                }
            },
            GlobalState::Exit => unreachable!(),
        }
    }

    pub fn exit(&self) -> bool {
        matches!(self, GlobalState::Exit)
    }

    pub fn new(game: &'a GameDefinition, endpoint: &'a api::Endpoint) -> Self {
        let global_state_data = GlobalStateData { game, endpoint };
        GlobalState::CreateOrJoin(CreateOrJoinState::Create(CreateOrJoinData::new(
            global_state_data,
            CreateOrJoinDataImpl {
                login: String::new(),
            },
        )))
    }

    pub fn join_game(
        created_game: WireCreatedGame,
        map: &'a GameMap,
        s: SelectMapData<'a>,
    ) -> GlobalState<'a> {
        let create_character_state_data = CreateCharacterStateDataImpl {
            name: String::new(),
            class_index: 0,
            team_index: 0,
            position_index: 0,

            classes: s.prev().game.classes.ids(),
            teams: map
                .teams
                .iter()
                .enumerate()
                .map(|(index, _)| Id::new(index))
                .collect(),
            map,
            game_id: created_game.game_id,
        };

        let state_data = StateData::new(s.split().0, create_character_state_data);
        let create_character_state = CreateCharacterState::Team(state_data);
        GlobalState::CreateCharacter(create_character_state)
    }
}

#[derive(Debug)]
pub struct CreateOrJoinDataImpl {
    pub login: String,
}

#[derive(Debug)]
pub enum CreateOrJoinState<'a> {
    Create(CreateOrJoinData<'a>),
    Join(CreateOrJoinData<'a>),
}
pub type CreateOrJoinData<'a> = StateData<GlobalStateData<'a>, CreateOrJoinDataImpl>;

impl<'a> State for CreateOrJoinState<'a> {
    type RootState = GlobalState<'a>;

    fn expected_event(&self) -> ExpectedEvent {
        match self {
            CreateOrJoinState::Join(_) => ExpectedEvent::Char,
            CreateOrJoinState::Create(_) => ExpectedEvent::None,
        }
    }

    fn next(self, i: Event) -> Self::RootState {
        match (self, i) {
            (CreateOrJoinState::Join(s), Event::Right)
            | (CreateOrJoinState::Join(s), Event::Up)
            | (CreateOrJoinState::Join(s), Event::Down)
            | (CreateOrJoinState::Join(s), Event::Left) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Create(s))
            }
            (CreateOrJoinState::Join(mut s), Event::PrintableString(string)) => {
                s.curr_mut().login.push_str(&string);
                GlobalState::CreateOrJoin(CreateOrJoinState::Join(s))
            }
            (CreateOrJoinState::Join(s), Event::Cancel) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Create(s))
            }
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
                                let create_character_state_data = CreateCharacterStateDataImpl {
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
                            let joined_game = global.endpoint.join_game_with_char(game_id, char_id);
                            unimplemented!()
                            //match joined_game
                        }
                    }
                    _ => GlobalState::CreateOrJoin(CreateOrJoinState::Create(
                        CreateOrJoinData::new(global, join),
                    )),
                }
            }

            (CreateOrJoinState::Create(s), Event::Right)
            | (CreateOrJoinState::Create(s), Event::Up)
            | (CreateOrJoinState::Create(s), Event::Down)
            | (CreateOrJoinState::Create(s), Event::Left) => {
                GlobalState::CreateOrJoin(CreateOrJoinState::Join(s))
            }
            (CreateOrJoinState::Create(s), Event::Confirm) => {
                let global = s.split().0;
                let global_state_data = GlobalStateData {
                    game: global.game,
                    endpoint: global.endpoint,
                };
                let select_map_state_data = SelectMapDataImpl {
                    map_ids: global_state_data.game.maps.ids(),
                    curr_id: 0,
                };

                GlobalState::SelectMap(SelectMapData::new(global_state_data, select_map_state_data))
            }

            unchanged => GlobalState::CreateOrJoin(unchanged.0),
        }
    }
}

#[derive(Debug)]
pub struct SelectMapDataImpl {
    pub map_ids: Vec<Id<GameMap>>,
    pub curr_id: usize,
}
pub type SelectMapData<'a> = StateData<GlobalStateData<'a>, SelectMapDataImpl>;

#[derive(Debug)]
pub enum CreateCharacterState<'a> {
    Class(CreateCharacterStateData<'a>),
    Team(CreateCharacterStateData<'a>),
    Position(CreateCharacterStateData<'a>),
    Name(CreateCharacterStateData<'a>),
}
impl<'a> State for CreateCharacterState<'a> {
    type RootState = GlobalState<'a>;

    fn expected_event(&self) -> ExpectedEvent {
        match self {
            CreateCharacterState::Team(_) => ExpectedEvent::SelectionHorizontal,
            CreateCharacterState::Class(_) => ExpectedEvent::SelectionHorizontal,
            CreateCharacterState::Position(_) => ExpectedEvent::Selection,
            CreateCharacterState::Name(_) => ExpectedEvent::Char,
        }
    }

    fn next(self, i: Event) -> Self::RootState {
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
                    WaitForGameCreationDataImpl {
                        map: create_char.map,
                        game_id: create_char.game_id,
                        login,
                        id,
                    },
                ))
            }
            unchanged => GlobalState::CreateCharacter(unchanged.0),
        }
    }
}
#[derive(Debug)]
pub struct CreateCharacterStateDataImpl<'a> {
    pub name: String,
    pub class_index: usize,
    pub team_index: usize,
    pub position_index: usize,

    pub classes: Vec<Id<Class>>,
    pub teams: Vec<Id<Team>>,
    pub map: &'a GameMap,
    pub game_id: String,
}
pub type CreateCharacterStateData<'a> =
    StateData<GlobalStateData<'a>, CreateCharacterStateDataImpl<'a>>;

#[derive(Debug)]
pub enum PlayGameState<'a> {
    OurTurn(PlayGameStateData<'a>),
    NotOurTurn(PlayGameStateData<'a>),
}
impl<'a> State for PlayGameState<'a> {
    type RootState = GlobalState<'a>;

    fn expected_event(&self) -> ExpectedEvent {
        match self {
            PlayGameState::NotOurTurn(_) => ExpectedEvent::None,
            PlayGameState::OurTurn(_) => unimplemented!(),
        }
    }

    fn next(self, i: Event) -> Self::RootState {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct PlayGameStateDataImpl<'a> {
    pub cell: Id<Cell>,
    pub game_state: GameState,
    pub map: &'a GameMap,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
}
pub type PlayGameStateData<'a> = StateData<GlobalStateData<'a>, PlayGameStateDataImpl<'a>>;

#[derive(Debug)]
pub struct WaitForGameCreationDataImpl<'a> {
    pub map: &'a GameMap,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
}
pub type WaitForGameCreationData<'a> =
    StateData<GlobalStateData<'a>, WaitForGameCreationDataImpl<'a>>;
