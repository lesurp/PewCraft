use crate::game::error::Error;
use crate::game::id_map::Id;
use log::debug;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};
use std::ops::Index;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Team(pub String, pub Vec<Id<Cell>>);

impl Id<Cell> {
    fn invalid() -> Id<Cell> {
        Id::new(std::usize::MAX)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CellAttibute {
    None,
    Wall,
    Hole,
    Water,
    // etc.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cell {
    height: i32,
    attribute: CellAttibute,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameMap {
    pub name: String,
    pub data: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    pub teams: Vec<Team>,
}

// Used internally for the A* path computation
#[derive(Eq)]
struct Node {
    real_cost: u32,
    heuristic: u32,
    curr_cell: Id<Cell>,
}

impl Node {
    fn cost(&self) -> u32 {
        self.real_cost + self.heuristic
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost().cmp(&other.cost())
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost() == other.cost()
    }
}

impl GameMap {
    pub fn check_validity(&self) -> Result<(), Error> {
        if self.height * self.width != self.data.len() {
            return Err(Error::InvalidMapSize);
        }

        let mut starting_cells = HashSet::new();
        for Team(_, cells) in &self.teams {
            for cell in cells {
                if !starting_cells.insert(cell) {
                    return Err(Error::OverlappingStartingCells);
                }
            }
        }

        debug!(
            "Map named `{}` checked properly (size {}x{})",
            self.name, self.width, self.height
        );
        Ok(())
    }

    pub fn xy_to_id(&self, x: usize, y: usize) -> Id<Cell> {
        Id::new(y * self.width + x)
    }

    pub fn id_to_xy(&self, id: Id<Cell>) -> (usize, usize) {
        let x = id.raw() % self.width;
        let y = (id.raw() - x) / self.width;
        (x, y)
    }

    pub fn id_to_xy_i32(&self, id: Id<Cell>) -> (i32, i32) {
        let x = id.raw() % self.width;
        let y = (id.raw() - x) / self.width;
        (x as i32, y as i32)
    }

    fn surrounding_cells(&self, c: Id<Cell>) -> [Id<Cell>; 4] {
        let (x, y) = self.id_to_xy(c);

        // NOTE: those values WILL underflow for the first row
        // however those ids are unsigned, meaning that they will be invalidated when using
        // is_valid_cell anyway
        [
            if x + 1 < self.width {
                self.xy_to_id(x + 1, y)
            } else {
                Id::invalid()
            },
            if y + 1 < self.height {
                self.xy_to_id(x, y + 1)
            } else {
                Id::invalid()
            },
            if x > 0 {
                self.xy_to_id(x - 1, y)
            } else {
                Id::invalid()
            },
            if y > 0 {
                self.xy_to_id(x, y - 1)
            } else {
                Id::invalid()
            },
        ]
    }

    fn is_valid_cell(&self, c: Id<Cell>) -> bool {
        self.data.len() > c.raw()
    }

    pub fn distance(&self, a: Id<Cell>, b: Id<Cell>) -> u32 {
        let (ax, ay) = self.id_to_xy(a);
        let (bx, by) = self.id_to_xy(b);
        ((ax as i32 - bx as i32).abs() + (ay as i32 - by as i32).abs()) as u32
    }

    pub fn can_move_to(&self, start: Id<Cell>, end: Id<Cell>, swiftness: i32) -> bool {
        let mut nodes = BinaryHeap::new();
        nodes.push(Reverse(Node {
            real_cost: 0,
            heuristic: self.distance(start, end),
            curr_cell: start,
        }));

        loop {
            let n = if let Some(n) = nodes.pop() {
                n.0
            } else {
                // No path possible (should never happen though...)
                return false;
            };

            // our two exit conditions (arrived, or too long)
            if swiftness < n.cost() as i32 {
                return false;
            } else if end == n.curr_cell {
                return true;
            }

            let prev_cell = &self[n.curr_cell];
            for neighbor in self.surrounding_cells(n.curr_cell).iter() {
                let neighbor = *neighbor;
                if !self.is_valid_cell(neighbor) {
                    continue;
                }

                let next_cell = &self[neighbor];

                let additional_cost =
                    std::cmp::max(0, next_cell.height - prev_cell.height) as u32 + 1;

                nodes.push(Reverse(Node {
                    real_cost: n.real_cost + additional_cost,
                    heuristic: self.distance(neighbor, end),
                    curr_cell: neighbor,
                }));
            }
        }
    }
}

impl Index<Id<Cell>> for GameMap {
    type Output = Cell;

    fn index(&self, id: Id<Cell>) -> &Self::Output {
        &self.data[id.raw()]
    }
}

#[cfg(test)]
mod test {
    use super::{Cell, CellAttibute, Error, GameMap, Team};
    use crate::game::id_map::Id;

    #[test]
    fn test_xy_to_id() {
        let w = 4;
        let h = 6;
        let map = GameMap {
            name: "".to_owned(),
            width: w,
            height: h,
            data: Vec::new(),
            teams: Default::default(),
        };

        let x = 1;
        let y = 3;

        let cell_id = map.xy_to_id(x, y);

        assert_eq!(cell_id.raw(), 13);
    }

    #[test]
    fn test_id_to_xy() {
        let w = 4;
        let h = 6;
        let map = GameMap {
            name: "".to_owned(),
            width: w,
            height: h,
            data: Vec::new(),
            teams: Default::default(),
        };

        let cell_id = Id::new(22);
        let (x, y) = map.id_to_xy(cell_id);

        assert_eq!(x, 2);
        assert_eq!(y, 5);
    }

    #[test]
    fn test_id_consistency() {
        let w = 4;
        let h = 6;
        let map = GameMap {
            name: "".to_owned(),
            width: w,
            height: h,
            data: Vec::new(),
            teams: Default::default(),
        };

        let cell_id = Id::new(22);
        let (x, y) = map.id_to_xy(cell_id);

        assert_eq!(cell_id, map.xy_to_id(x, y));
    }

    #[test]
    fn test_validity_size_ok() {
        let width = 5;
        let height = 5;
        let map = GameMap {
            name: "".to_owned(),
            width,
            height,
            data: vec![
                Cell {
                    height: 0,
                    attribute: CellAttibute::None,
                };
                25
            ],
            teams: Default::default(),
        };

        assert!(matches!(map.check_validity(), Ok(())));
    }

    #[test]
    fn test_validity_team_ok() {
        let width = 5;
        let height = 5;

        let cells_first = vec![Id::new(0), Id::new(1)];
        let cells_second = vec![Id::new(2), Id::new(3)];
        let teams = vec![
            Team("first".to_owned(), cells_first),
            Team("second".to_owned(), cells_second),
        ];

        let map = GameMap {
            name: "".to_owned(),
            width,
            height,
            data: vec![
                Cell {
                    height: 0,
                    attribute: CellAttibute::None,
                };
                25
            ],
            teams,
        };

        assert!(matches!(map.check_validity(), Ok(())));
    }

    #[test]
    fn test_validity_size_nok() {
        let width = 5;
        let height = 2;
        let map = GameMap {
            name: "".to_owned(),
            width,
            height,
            data: vec![
                Cell {
                    height: 0,
                    attribute: CellAttibute::None,
                };
                20
            ],
            teams: Default::default(),
        };

        assert!(matches!(map.check_validity(), Err(Error::InvalidMapSize)));
    }

    #[test]
    fn test_validity_team_nok() {
        let width = 5;
        let height = 5;

        let cells_first = vec![Id::new(0), Id::new(1)];
        let cells_second = vec![Id::new(10), Id::new(20), Id::new(0)];
        let teams = vec![
            Team("first".to_owned(), cells_first),
            Team("second".to_owned(), cells_second),
        ];

        let map = GameMap {
            name: "".to_owned(),
            width,
            height,
            data: vec![
                Cell {
                    height: 0,
                    attribute: CellAttibute::None,
                };
                25
            ],
            teams,
        };

        assert!(matches!(
            map.check_validity(),
            Err(Error::OverlappingStartingCells)
        ));
    }

    #[test]
    fn test_a_star_basic() {
        let width = 5;
        let height = 5;
        let map = GameMap {
            name: "".to_owned(),
            width,
            height,
            data: vec![
                Cell {
                    height: 0,
                    attribute: CellAttibute::None,
                };
                25
            ],
            teams: Default::default(),
        };

        let start = Id::new(0);
        let end = Id::new(4);
        assert!(map.can_move_to(start, end, 4));
        assert!(!map.can_move_to(start, end, 3));
    }
}
