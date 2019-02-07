use crate::tile::{Tile, TilePosition, Position, Direction};
use std::collections::{HashSet, HashMap};
use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RectangularBoard {
    #[serde(skip_serializing)]
    pub width : usize,

    #[serde(skip_serializing)]
    pub height : usize,

    pub board : Vec<Vec<bool>>,
}


impl RectangularBoard {
    pub fn new(width : usize, height : usize) -> Self {
        RectangularBoard {
            width,
            height,
            board : vec![vec![false ; width] ; height],
        }
    }

    pub fn disp(&self) -> String {
        let mut o = String::new();

        for row in &self.board {
            for col in row {
                o.push(if *col { ' ' } else { 'x' });
            }
            o.push('\n');
        }

        o
    }

    pub fn l_board(n : usize, scale : usize) -> Self {
        let mut board = RectangularBoard::new(n * scale, 2 * scale);

        for row in 0..scale {
            for col in scale..(n * scale) {
                board.board[row][col] = true;
            }
        }

        board
    }

    pub fn t_board(n : usize, scale: usize) -> Self {
        let mut board = RectangularBoard::new((2 * n + 1) * scale, 2 * scale);

        for row in 0..scale {
            for col in 0..(n * scale) {
                board.board[row][col] = true;
            }
            for col in ((n+1) * scale)..((2 * n + 1) * scale) {
                board.board[row][col] = true;
            }
        }

        board
    }


    fn mark(&mut self, p: &Position) {
        self.board[p.x as usize][p.y as usize] = true;
    }

    /// Determines whether the entire board is marked
    ///
    /// # Examples
    ///
    /// ```
    /// // Example code here
    /// ```
    pub fn is_all_marked(&self) -> bool {
        for row in self.board.iter() {
            for col in row.iter() {
                if *col == false {
                    return false;
                }
            }
        }
        true
    }

    // TODO: document
    pub fn get_unmarked_position(&self, tiles : &Vec<Tile>) -> Option<Position> {
        // TODO: refactor counts into a new matrix maybe, to avoid recalculating each time?
        let mut counts = HashMap::new();

        for (i, row) in self.board.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    // increment adjacent positions by 1
                    for p in vec![Position::new(i as i32 - 1, j as i32),
                                  Position::new(i as i32, j as i32 + 1),
                                  Position::new(i as i32, j as i32 - 1),
                                  Position::new(i as i32 + 1, j as i32)] {
                        if self.is_valid(&p) {
                            (*counts.entry(p).or_insert(0)) += 1;
                        }
                    }
                }
            }
        }

        let mut max_count = -1;
        let mut max_position = None;

        for j in 0..self.width {
            for i in 0..self.height {
                if self.board[i][j] == false {
                    let mut found_tile = false;

                    'outer: for tile in tiles {
                        for start_index in 0..=tile.directions.len() {
                            if let Some(_) = self.tile_fits_at_position(tile, Position::new(i as i32, j as i32), start_index) {
                                // found a fitting tile!
                                found_tile = true;
                                break 'outer;
                            }
                        }
                    }

                    // no tile found that could actually tile here!
                    if !found_tile {
                        return None;
                    }

                    let p = Position::new(i as i32, j as i32);
                    let count = *counts.entry(p.clone()).or_default();

                    if count == 3 {
                        return Some(p);
                    } else if count > max_count {
                        max_count = count;
                        max_position = Some(p);
                    }
                }
            }
        }

        max_position
    }


    fn is_marked(&self, p : &Position) -> bool {
        assert!(self.is_valid(p));

        self.board[p.x as usize][p.y as usize]
    }

    fn is_valid(&self, p : &Position) -> bool {
        p.x >= 0 && (p.x as usize) < self.height && p.y >= 0 && (p.y as usize) < self.width
    }


    fn move_in_direction(&self, p : &Position, direction : &Direction) -> Position {
        let mut row = p.x;
        let mut col = p.y;

        col += match direction {
            Direction::Left => -1,
            Direction::Right => 1,
            Direction::UpLeft => -1,
            Direction::UpRight => 1,
            Direction::DownLeft => -1,
            Direction::DownRight => 1,
            _ => 0,
        };
        row += match direction {
            Direction::Up => -1,
            Direction::Down => 1,
            Direction::UpLeft => -1,
            Direction::UpRight => -1,
            Direction::DownLeft => 1,
            Direction::DownRight => 1,
            _ => 0,
        };

        Position::new(row, col)
    }

    /// Tests whether the specified tile fits at the specified board position.
    /// If it does, then return Some(TilePosition)
    ///
    /// # Examples
    ///
    /// ```
    /// // Example code here
    /// ```
    pub fn tile_fits_at_position(&self, tile : &Tile, position : Position, start_index : usize) -> Option<TilePosition> {
        // make sure our start index isn't too large
        assert!(start_index <= tile.directions.len());

        let mut current_position = position;

        let valid_and_unmarked = |p : &Position| {
            self.is_valid(p) && !self.is_marked(p)
        };

        if !valid_and_unmarked(&current_position) {
            return None;
        }

        let mut covered = HashSet::new();
        covered.insert(current_position);

        // move backwards from start_index - 1
        for i in (0..start_index).rev() {
            current_position = self.move_in_direction(&current_position, &tile.directions[i].opposite());

            if !valid_and_unmarked(&current_position) {
                return None;
            }
            covered.insert(current_position);
        }

        let mut current_position = position;

        // now move forwards after the start index
        for i in start_index..tile.directions.len() {
            current_position = self.move_in_direction(&current_position, &tile.directions[i]);

            if !valid_and_unmarked(&current_position) {
                return None;
            }

            covered.insert(current_position);
        }

        Some(TilePosition::new(position, tile.clone(), start_index, covered))
    }

    pub fn mark_tile_at_position(&mut self, tp : TilePosition) {
        for position in tp.covered {
            self.mark(&position);
        }
    }
}