use crate::state::{
    CreateCharacterState, CreateOrJoinState, GlobalState, PlayGameState, SelectMapData,
};
use crate::tui_impl::map::FormatMap;
use common::game::GameDefinition;
use log::{debug, info};
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::widgets::{Block, Borders, Widget};
use tui::Frame;

const SELECT_MAP_BLOCK_TITLE: &str = "Select map";
const CREATE_CHAR_BLOCK_TITLE: &str = "Create your character";

trait Reduceable<'a> {
    fn reduce(self) -> Text<'a>;
}

impl<'a, const K: usize> Reduceable<'a> for [Text<'a>; K] {
    fn reduce(self) -> Text<'a> {
        self.into_iter()
            .reduce(|mut text, add| {
                text.extend(add);
                text
            })
            .unwrap()
    }
}

pub struct Renderer<'a, 'b, 'c, B: tui::backend::Backend> {
    f: &'a mut Frame<'c, B>,
    s: &'a GlobalState<'b>,
    g: &'a GameDefinition,
    chunks: Vec<tui::layout::Rect>,
}

impl<'a, 'b, 'c, B: tui::backend::Backend> Renderer<'a, 'b, 'c, B> {
    pub fn render(f: &'a mut Frame<'c, B>, s: &'a GlobalState<'b>, g: &'a GameDefinition) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.size());

        Renderer { f, s, g, chunks }.render_impl();
    }

    // TODO: what was this for again?
    fn invert_text() -> Style {
        //let style = Style::default();
        //let fg = style.fg.unwrap();
        //let bg = style.bg.unwrap();
        //style.bg(fg).fg(bg)
        Style::default()
    }

    fn render_impl(self) {
        match self.s {
            GlobalState::CreateOrJoin(create_or_join) => {
                self.create_or_join(create_or_join);
            }
            GlobalState::CreateCharacter(create_character) => {
                self.create_character(create_character);
            }
            //GlobalState::JoinedGame(joined_game) => self.render_joined_game(joined_game),
            GlobalState::SelectMap(select_map) => {
                self.select_map(select_map);
            }
            GlobalState::PlayGame(play_game) => {
                self.play_game(play_game);
            }
            // TODO
            GlobalState::WaitForGameCreation(game_state) => {
                let full_login =
                    format!("{}/{}", game_state.curr().game_id, game_state.curr().login);
                self.f.render_widget(
                    Block::default()
                        .title(format!(
                            "Waiting for other players | Character login: {}",
                            full_login
                        ))
                        .borders(Borders::ALL),
                    self.chunks[1],
                );
            }
            GlobalState::Exit => panic!("Should not try to render when in the 'Exit' state"),
        };
    }

    fn create_or_join(self, create_or_join: &CreateOrJoinState) {
        let text = match create_or_join {
            CreateOrJoinState::Create(s) => [
                Text::raw("    "),
                Text::styled("CREATE", Self::invert_text()),
                Text::raw("\n or "),
                Text::raw("JOIN"),
                Text::raw("Login: "),
                Text::raw(&s.curr().login),
            ],
            CreateOrJoinState::Join(s) => [
                Text::raw("    "),
                Text::raw("CREATE"),
                Text::raw("\n or "),
                Text::styled("JOIN", Self::invert_text()),
                Text::raw("Login: "),
                Text::raw(&s.curr().login),
            ],
        };

        self.f.render_widget(
            Paragraph::new(text.reduce())
                .block(
                    Block::default()
                        .title(CREATE_CHAR_BLOCK_TITLE)
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Left),
            self.chunks[1],
        );
    }

    fn create_character(self, create_character: &CreateCharacterState) {
        let map = match create_character {
            CreateCharacterState::Class(s) => {
                let curr_id = s.curr().class_index;
                let class_ids = &s.curr().classes;
                let class = self
                    .g
                    .classes
                    .get(*class_ids.get(curr_id).unwrap())
                    .unwrap();
                let text = [
                    Text::styled(
                        format!("    {} / {}", curr_id + 1, class_ids.len()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Text::raw("\n    Name:         "),
                    Text::raw(&class.name),
                    Text::raw("\n    Description:\n    "),
                    Text::raw(&class.description),
                ];

                self.f.render_widget(
                    Paragraph::new(text.reduce())
                        .block(
                            Block::default()
                                .title(CREATE_CHAR_BLOCK_TITLE)
                                .borders(Borders::ALL),
                        )
                        .alignment(Alignment::Left),
                    self.chunks[1],
                );
                s.curr().map
            }
            CreateCharacterState::Name(s) => {
                let text = [Text::raw(format!(
                    "    Now type your name: {}",
                    s.curr().name
                ))];
                self.f.render_widget(
                    Paragraph::new(text.reduce())
                        .block(
                            Block::default()
                                .title(CREATE_CHAR_BLOCK_TITLE)
                                .borders(Borders::ALL),
                        )
                        .alignment(Alignment::Left),
                    self.chunks[1],
                );
                s.curr().map
            }
            CreateCharacterState::Team(s) => {
                let curr_id = s.curr().team_index;
                let team_ids = &s.curr().teams;
                let team = s
                    .curr()
                    .map
                    .teams
                    .get(team_ids.get(curr_id).unwrap().raw())
                    .unwrap();
                let text = [
                    Text::styled(
                        format!("    {} / {}", curr_id + 1, team_ids.len()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Text::raw("\n    Name:         "),
                    Text::raw(&team.0),
                    Text::raw("\n    TODO store the nb of players, and what classes are already taken etc. OR EVEN BETTER<, SHOW THEM ON THE MAP!")
                ];

                self.f.render_widget(
                    Paragraph::new(text.reduce())
                        .block(
                            Block::default()
                                .title(CREATE_CHAR_BLOCK_TITLE)
                                .borders(Borders::ALL),
                        )
                        .alignment(Alignment::Left),
                    self.chunks[1],
                );
                s.curr().map
            }
            CreateCharacterState::Position(s) => {
                let curr_id = s.curr().position_index;
                let positions = &s.curr().map.teams.get(s.curr().team_index).unwrap().1;
                let position = positions.get(curr_id).unwrap();

                let (x, y) = s.curr().map.id_to_xy(*position);
                let text = [
                    Text::styled(
                        format!("    {} / {}", curr_id + 1, positions.len()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Text::raw("\n    Initial position:         "),
                    Text::raw(format!("X: {}", x)),
                    Text::raw(format!("Y: {}", y)),
                ];

                self.f.render_widget(
                    Paragraph::new(text.reduce())
                        .block(
                            Block::default()
                                .title(CREATE_CHAR_BLOCK_TITLE)
                                .borders(Borders::ALL),
                        )
                        .alignment(Alignment::Left),
                    self.chunks[1],
                );
                s.curr().map
            }
        };
        self.f.render_widget(FormatMap(map, None), self.chunks[0]);
    }

    fn select_map(self, s: &SelectMapData) {
        let map_ids = &s.curr().map_ids;
        let curr_id = s.curr().curr_id;
        let map = self.g.maps.get(*map_ids.get(curr_id).unwrap()).unwrap();
        self.f.render_widget(FormatMap(map, None), self.chunks[0]);

        let text = [
            Text::styled(
                format!("    {} / {}", curr_id + 1, map_ids.len()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Text::raw("\n    Name:      "),
            Text::raw(&map.name),
            Text::raw("\n    Width:     "),
            Text::raw(format!("{}", map.width)),
            Text::raw("\n    Height:    "),
            Text::raw(format!("{}", map.height)),
            Text::raw("\n    Max teams: "),
            Text::raw(format!("{}", map.teams.len())),
        ];

        self.f.render_widget(
            Paragraph::new(text.reduce())
                .block(
                    Block::default()
                        .title(SELECT_MAP_BLOCK_TITLE)
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Left),
            self.chunks[1],
        );
    }

    fn play_game(self, s: &PlayGameState) {
        match s {
            PlayGameState::NotOurTurn(s) | PlayGameState::OurTurn(s) => {
                let map = s.prev().game.maps.get(s.curr().game_state.map).unwrap();
                self.f.render_widget(FormatMap(map, None), self.chunks[0]);
            }
        }
        self.f.render_widget(
            Block::default()
                .title("PLAYING THE GAME ASODUHASUOB")
                .borders(Borders::ALL),
            self.chunks[1],
        );
    }
}
