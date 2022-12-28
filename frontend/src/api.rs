use common::game::{GameDefinition, GameState};
use common::io::{WireCreatedChar, WireCreatedGame, WireNewCharRequest, WireNewGameRequest};
use log::{debug, info};
use reqwest::{blocking::Client, Url};
use std::fmt;

pub struct Endpoint {
    url: Url,
    client: Client,
}

impl Endpoint {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        info!("API endpoint: {}", url.as_ref());
        Endpoint {
            url: Url::parse(url.as_ref()).unwrap(),
            client: Client::new(),
        }
    }

    pub fn load_game(&self) -> GameDefinition {
        self.client
            .get(self.url.join("game").unwrap())
            .send()
            .unwrap()
            .json()
            .unwrap()
    }

    pub fn create_game(&self, request: WireNewGameRequest) -> WireCreatedGame {
        debug!("Creating game with request: {:?}", request);
        self.client
            .post(self.url.join("new_game").unwrap())
            .json(&request)
            .send()
            .unwrap()
            .json()
            .unwrap()
    }

    pub fn join_game_with_char<S: AsRef<str>>(
        &self,
        _game_id: S,
        _char_id: S,
    ) -> Option<WireCreatedGame> {
        unimplemented!()
        /*
        debug!("Creating char with request: {:?}", request);
        self.client
            .post(self.url.join(game_id.as_ref()).unwrap())
            .json(&request)
            .send()
            .unwrap()
            .json()
            .unwrap()
            */
    }

    pub fn create_char<S: AsRef<str>>(
        &self,
        game_id: S,
        request: WireNewCharRequest,
    ) -> WireCreatedChar {
        debug!("Creating char with request: {:?}", request);
        self.client
            .post(self.url.join(game_id.as_ref()).unwrap())
            .json(&request)
            .send()
            .unwrap()
            .json()
            .unwrap()
    }

    pub fn game_state<S: AsRef<str>>(&self, game_id: S) -> Option<GameState> {
        self.client
            .get(self.url.join(game_id.as_ref()).unwrap())
            .send()
            .unwrap()
            .json()
            .unwrap()
    }
}

impl fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Endpoint {{ url: {:?}, client: <hidden> }}", self.url)
    }
}
