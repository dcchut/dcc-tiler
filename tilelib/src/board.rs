use crate::tile::{Tile, TilePosition, Position, Direction, TileCollection};
use std::collections::{HashSet, HashMap};
use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RectangularBoard {
    #[serde(skip_serializing)]
    pub width : usize,

    #[serde(skip_serializing)]
    pub height : usize,

    pub board : Vec<Vec<bool>>,

    #[serde(skip_serializing)]
    counts : Vec<Vec<usize>>,
}


impl RectangularBoard {
    pub fn new(width : usize, height : usize) -> Self {
        let mut counts = vec![vec![0 ; width] ; height ];

        for i in 0..height {
            counts[i][0] = 1;
            counts[i][width - 1] = 1;
        }
        for j in 0..width {
            counts[0][j] = 1;
            counts[height - 1][j] = 1;
        }


        RectangularBoard {
            width,
            height,
            board : vec![vec![false ; width] ; height],
            counts : counts,
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
            for col in ((n + 1) * scale)..((2 * n + 1) * scale) {
                board.board[row][col] = true;
            }
        }

        board
    }


    fn mark(&mut self, p: &Position) {
        for xp in (p.x-1)..=(p.x+1) {
            if xp == p.x {
                continue;
            }
            if self.is_valid(&Position::new(xp,p.y)) {
                self.counts[xp as usize][p.y as usize] += 1;
            }
        }
        for yp in (p.y-1)..=(p.y+1) {
            if yp == p.y {
                continue;
            }
            if self.is_valid(&Position::new(p.x, yp)) {
                self.counts[p.x as usize][yp as usize] += 1;
            }
        }


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


    pub fn place_tile(&self, tile_collection : &TileCollection) -> Vec<RectangularBoard> {
        let mut largest_count = None;
        let mut largest_position = None;

        // find the position with the highest count
        for j in 0..self.width {
            for i in 0..self.height {
                if !self.board[i][j] {
                    let count = self.counts[i][j];
                    // TODO : maybe deal with stupid case where we're allowed a 1x1 piece
                    // this spot cannot be tiled
                    if count == 4 {
                        return Vec::new();
                    }

                    if largest_count.is_none() || self.counts[i][j] > largest_count.unwrap() {
                        largest_count = Some(self.counts[i][j]);
                        largest_position = Some((i,j));
                    }
                }
            }
        }


        let mut fitting_tiles = Vec::new();

        if let Some((i,j)) = largest_position {
            for tile in tile_collection.tiles.iter() {
                for start_index in 0..=tile.directions.len() {
                    if let Some(tp) = self.tile_fits_at_position(tile, Position::new(i as i32, j as i32), start_index) {
                        // we really should be using a HashSet for fitting_tiles, but
                        // I haven't figured out how to put a TilePosition in a HashSet,
                        // so we just check for containment here instead
                        if !fitting_tiles.contains(&tp) {
                            fitting_tiles.push(tp);
                        }
                    }
                }
            }
        }

        fitting_tiles.into_iter().map(|tp| {
            let mut child_board = self.clone();
            child_board.mark_tile_at_position(tp);
            child_board
        }).collect()
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