use ::tui::buffer::Buffer;
use ::tui::layout::Rect;
use ::tui::widgets::Widget;

use common::game::{Cell, Character, GameMap, Id, IdMap};
use log::debug;

struct CellCorner(char, char, char, char);
impl CellCorner {
    pub fn from_map_size(x: u16, y: u16, width: usize, height: usize) -> Self {
        let max_x = width as u16 - 1;
        let max_y = height as u16 - 1;

        let tl = match (x, y) {
            (0, 0) => '╔',
            (0, _) => '╠',
            (_, 0) => '╦',
            (_, _) => '╬',
        };

        let tr = if x == max_x {
            if y == 0 {
                '╗'
            } else {
                '╣'
            }
        } else if y == 0 {
            '╦'
        } else {
            '╬'
        };

        let bl = if x == 0 {
            if y == max_y {
                '╚'
            } else {
                '╠'
            }
        } else if y == max_y {
            '╩'
        } else {
            '╬'
        };

        let br = if x == max_x {
            if y == max_y {
                '╝'
            } else {
                '╣'
            }
        } else if y == max_y {
            '╩'
        } else {
            '╬'
        };

        CellCorner(tl, tr, bl, br)
    }
}

/*
impl<'a> Display for FormatCell<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
}

impl<'a> Display for FormatMap<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let map = &self.0;
        let ref_pos = &self.2;

        let mut id = Id::new(0);
        let mut out = String::new();
        for cell in map.data.iter() {
            let (x, y) = map.id_to_xy(id);
            let (x, y) = (x as u16, y as u16);
            debug!("Offset of curr cell: {}/{}", x, y);

            // convert our map coordinates to the screen coordinates (top left corner of each cell)
            let pos = (ref_pos.0 + 14 * x as u16, ref_pos.1 + 6 * y as u16);
            // get the character in the cell (if any)
            let character = self.character(id);
            let corners = CellCorner::from_map_size(x, y, map.width, map.height);

            out = format!(
                "{}{}{}",
                out,
                Goto(pos.0, pos.1),
                FormatCell(cell, character, corners, pos),
            );

            id = Id::new(id.raw() + 1);
        }

        write!(f, "{}", out)
    }
}

impl<'a> Display for FormatMapSelection<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut line = 0;
        let map = &self.0;
        let pos = &self.1;
        let mut nl = move || {
            line += 1;
            Goto(pos.0, pos.1 + line)
        };

        // TODO: add nb of char per team!
        write!(
            f,
            "Name: {}{}Width: {}{}Height: {}{}Max number of teams: {}",
            map.name,
            nl(),
            map.width,
            nl(),
            map.height,
            nl(),
            map.teams.len(),
        )
    }
}
*/

pub(super) struct FormatMap<'a>(
    pub(super) &'a GameMap,
    pub(super) Option<&'a IdMap<Character>>,
);
impl<'a> FormatMap<'a> {
    fn character(&self, id: Id<Cell>) -> Option<&'a Character> {
        match self.1 {
            None => None,
            Some(characters) => characters
                .iter()
                .find(|(_, c)| c.position == id)
                .map(|(_, c)| c),
        }
    }
}

impl<'a> Widget for FormatMap<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let map = &self.0;
        let cell_width = area.width / map.width as u16;
        let cell_height = area.height / map.height as u16;

        assert!(
            cell_width > 2 && cell_height > 2,
            "Need at least a 3*3 cell size! Your screen is too small and/or the map too big..."
        );

        let mut id = Id::new(0);
        for cell in map.data.iter() {
            let (x, y) = map.id_to_xy(id);
            let (x, y) = (x as u16, y as u16);
            let corners = CellCorner::from_map_size(x, y, map.width, map.height);
            debug!("Offset of curr cell: {}/{}", x, y);

            let cell_rect = Rect::new(x * cell_width, y * cell_height, cell_width, cell_height);
            // get the character in the cell (if any)
            let character = self.character(id);

            FormatCell(cell, character, corners).draw(cell_rect, buf);

            id = Id::new(id.raw() + 1);
        }
    }
}

struct FormatCell<'a>(&'a Cell, Option<&'a Character>, CellCorner);
impl<'a> Widget for FormatCell<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let cell = self.0;
        let character = self.1;
        let corners = &self.2;

        for x in 1..area.width {
            let x = x + area.left();
            buf.get_mut(x, area.top()).set_char('═');
            buf.get_mut(x, area.bottom()).set_char('═');
        }

        for y in 1..area.height {
            let y = y + area.top();
            buf.get_mut(area.left(), y).set_char('║');
            buf.get_mut(area.right(), y).set_char('║');
        }

        buf.get_mut(area.left(), area.top()).set_char(corners.0);
        buf.get_mut(area.right(), area.top()).set_char(corners.1);
        buf.get_mut(area.left(), area.bottom()).set_char(corners.2);
        buf.get_mut(area.right(), area.bottom()).set_char(corners.3);
    }
}
