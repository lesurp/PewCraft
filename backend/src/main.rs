#![feature(proc_macro_hygiene, decl_macro)]

use lazy_static::lazy_static;
use pewcraft_common::{
    game::{Character, CharacterMapBuilder, GameDefinition, GameMap, GameState, Id},
    io::{
        WireAction, WireCreatedChar, WireCreatedGame, WireGetGame, WireNewCharRequest,
        WireNewGameRequest,
    },
};
use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::{get, post, routes, State};
use rocket_contrib::json::Json;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref GAME: GameDefinition = game_definition_loader::load("./data");
}

mod error;
mod game_definition_loader;

type ServerRunningGames = Mutex<HashMap<String, ServerRunningGame>>;
type ServerBuiltGames = Mutex<HashMap<String, ServerBuiltGame>>;

struct ServerRunningGame {
    game_state: GameState,
    // Users "login" with a randomly generated string...
    login_to_character_id: HashMap<String, Id<Character>>,
}

struct ServerBuiltGame {
    // Users "login" with a randomly generated string...
    login_to_character_id: HashMap<String, Id<Character>>,
    character_map_builder: CharacterMapBuilder<'static>,
    map: Id<GameMap>,
    team_size: usize,
}

#[post("/new_game", data = "<new_game>")]
fn create_game(
    builders: State<ServerBuiltGames>,
    new_game: Json<WireNewGameRequest>,
) -> Json<WireCreatedGame> {
    let s = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>();
    builders.lock().unwrap().insert(
        s.clone(),
        ServerBuiltGame {
            login_to_character_id: Default::default(),
            character_map_builder: CharacterMapBuilder::new(
                &GAME,
                new_game.map,
                new_game.team_size,
            ),
            map: new_game.map,
            team_size: new_game.team_size,
        },
    );
    Json(WireCreatedGame {
        game_id: s,
        map: new_game.map,
        team_size: new_game.team_size,
    })
}

#[get("/<game>")]
fn get_game(
    games: State<ServerRunningGames>,
    builders: State<ServerBuiltGames>,
    game: String,
) -> Json<WireGetGame> {
    let builders = builders.lock().unwrap();
    if let Some(builder) = builders.get(&game) {
        return Json(WireGetGame::BeingCreated(builder.map, builder.team_size));
    }

    let games = games.lock().unwrap();
    if let Some(game) = games.get(&game) {
        return Json(WireGetGame::Running(game.game_state.clone()));
    }

    Json(WireGetGame::None)
}

#[post("/<game>", data = "<new_character>")]
fn create_character(
    games: State<ServerRunningGames>,
    builders: State<ServerBuiltGames>,
    game: String,
    new_character: Json<WireNewCharRequest>,
) -> Result<Json<WireCreatedChar>, ()> {
    let mut builders = builders.lock().unwrap();

    // TODO wrong game id
    let builder = builders.get_mut(&game).ok_or(())?;

    // TODO wrong class id
    let class = GAME.classes.get(new_character.class).ok_or(())?;
    let req = new_character.into_inner();
    let c = Character {
        class: req.class,
        team: req.team,
        position: req.position,
        current_health: class.health,
        current_mana: class.mana,
        buffs: Default::default(),
        name: req.name,
    };
    // TODO many things...
    let character_id = builder.character_map_builder.add(c).map_err(|_| ())?;

    // Player's "login" after successful character creation
    let s = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>();
    builder
        .login_to_character_id
        .insert(s.clone(), character_id);

    // If game is ready to be start, consume the builder
    if builder.character_map_builder.can_build() {
        let builder = builders.remove(&game).unwrap();
        let character_map = builder.character_map_builder.build();
        let login_to_character_id = builder.login_to_character_id;
        let map = builder.map;
        games.lock().unwrap().insert(
            game,
            ServerRunningGame {
                login_to_character_id,
                game_state: GameState::new(&GAME, character_map, map),
            },
        );
    }

    Ok(Json(WireCreatedChar(s, character_id)))
}

#[post("/<game>/<login>", data = "<action>")]
fn character_action(
    games: State<ServerRunningGames>,
    game: String,
    login: String,
    action: Json<WireAction>,
) -> Result<(), ()> {
    let mut games = games.lock().unwrap();
    // TODO wrong game id
    let game = games.get_mut(&game).ok_or(())?;
    // TODO invalid player
    let character_id = game.login_to_character_id.get(&login).ok_or(())?;

    let curr_char_id = game.game_state.player_to_play();

    if curr_char_id != *character_id {
        // TODO wrong character
        return Err(());
    }

    // TODO
    game.game_state
        .next_action(&GAME, (action.0).0)
        .map_err(|_| ())?;

    // TODO
    Ok(())
}

#[get("/game")]
fn load_game() -> Json<GameDefinition> {
    Json(GAME.clone())
}
#[get("/<game>")]
fn game_state(
    games: State<ServerRunningGames>,
    game: String,
) -> Result<Json<Option<GameState>>, ()> {
    let games = games.lock().unwrap();
    let game_state = games.get(&game).map(|g| g.game_state.clone());
    Ok(Json(game_state))
}

fn main() {
    env_logger::init();
    let games_running: ServerRunningGames = Default::default();
    let game_builders: ServerBuiltGames = Default::default();
    {
        // Force the lazy_static initialization before starting the server
        // If some issues arises during the deserialization, I wanna see it right away...
        let _ = GAME.classes.ids();
    }
    rocket::ignite()
        .manage(game_builders)
        .manage(games_running)
        .mount(
            "/",
            routes![
                create_game,
                create_character,
                character_action,
                load_game,
                game_state
            ],
        )
        .launch();
}
