use crate::tile::{Direction, Tile, TileCollection};
use serde_derive::Serialize;
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RectangularBoard {
    #[serde(skip_serializing)]
    pub width: usize,

    #[serde(skip_serializing)]
    pub height: usize,

    pub board: Vec<Vec<bool>>,

    #[serde(skip_serializing)]
    counts: Vec<Vec<usize>>,
}

impl RectangularBoard {
    pub fn new(width: usize, height: usize) -> Self {
        let mut counts = vec![vec![0; width]; height];

        for row in counts.iter_mut() {
            row[0] = 1;
            row[width - 1] = 1;
        }

        for j in 0..width {
            counts[0][j] = 1;
            counts[height - 1][j] = 1;
        }

        RectangularBoard {
            width,
            height,
            board: vec![vec![false; width]; height],
            counts,
        }
    }

    /// Generates a new L-tetromino shaped board.
    ///
    /// This is a two step process - first we make an L shape
    /// with long side having length n, and then we replace each
    /// box with a scale^2 box.
    pub fn l_board(n: usize, scale: usize) -> Self {
        let mut board = RectangularBoard::new(n * scale, 2 * scale);

        for row in 0..scale {
            for col in scale..(n * scale) {
                board.board[row][col] = true;
            }
        }

        board
    }

    /// Generates a new T-tetromino shaped board.
    ///
    /// This is a two step process - first we make a T shape
    /// where the two tils have length n, and then we replace
    /// each box with a scale^2 box.
    pub fn t_board(n: usize, scale: usize) -> Self {
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

    /// What does it do?
    ///
    /// Details here.
    ///
    /// # Panics
    ///
    /// When does it panic?
    ///
    /// # Examples
    ///
    /// ```
    /// // Example code here
    /// ```
    fn mark(&mut self, p: Position) {
        for xp in (p.x - 1)..=(p.x + 1) {
            if xp == p.x {
                continue;
            }
            if self.is_valid(Position::from((xp, p.y))) {
                self.counts[xp as usize][p.y as usize] += 1;
            }
        }
        for yp in (p.y - 1)..=(p.y + 1) {
            if yp == p.y {
                continue;
            }
            if self.is_valid(Position::from((p.x, yp))) {
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
                if !(*col) {
                    return false;
                }
            }
        }
        true
    }

    pub fn place_tile(&self, tile_collection: &TileCollection) -> Vec<RectangularBoard> {
        let mut largest_count = None;
        let mut largest_position = None;

        // find the position with the highest count
        for j in 0..self.width {
            for i in 0..self.height {
                if !self.board[i][j] {
                    let count = self.counts[i][j];

                    // If our tile collection doesn't contain a 1x1 tile,
                    // then we've found a spot that cannot be tiled, so we're done
                    if !tile_collection.contains_single_tile() && count == 4 {
                        return Vec::new();
                    }

                    // keep track of the largest count we've found so far
                    if largest_count.is_none() || self.counts[i][j] > largest_count.unwrap() {
                        largest_count = Some(self.counts[i][j]);
                        largest_position = Some((i, j));
                    }
                }
            }
        }

        // Next, find all the tiles that fit at out best position
        let mut fitting_tiles = Vec::new();

        if let Some((i, j)) = largest_position {
            for tile in tile_collection.iter() {
                for start_index in 0..=tile.directions.len() {
                    if let Some(tp) =
                        self.tile_fits_at_position(tile, Position::from((i, j)), start_index)
                    {
                        // Really we should be using a HashSet for fitting_tiles, but it's annoying
                        // to hash a HashSet, so we just check for containment here instead
                        if !fitting_tiles.contains(&tp) {
                            fitting_tiles.push(tp);
                        }
                    }
                }
            }
        }

        // For each fitting tile we find, return the corresponding board
        fitting_tiles
            .into_iter()
            .map(|tp| {
                let mut child_board = self.clone();
                child_board.mark_tile_at_position(tp);
                child_board
            })
            .collect()
    }

    fn is_marked(&self, p: Position) -> bool {
        assert!(self.is_valid(p));

        self.board[p.x as usize][p.y as usize]
    }

    fn is_valid(&self, p: Position) -> bool {
        p.x >= 0 && (p.x as usize) < self.height && p.y >= 0 && (p.y as usize) < self.width
    }

    fn move_in_direction(&self, p: Position, direction: Direction) -> Position {
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
    fn tile_fits_at_position(
        &self,
        tile: &Tile,
        position: Position,
        start_index: usize,
    ) -> Option<TilePosition> {
        // make sure our start index isn't too large
        assert!(start_index <= tile.directions.len());

        let mut current_position = position;

        let valid_and_unmarked = |p: Position| self.is_valid(p) && !self.is_marked(p);

        if !valid_and_unmarked(current_position) {
            return None;
        }

        let mut covered = HashSet::new();
        covered.insert(current_position);

        // move backwards from start_index - 1
        for i in (0..start_index).rev() {
            current_position =
                self.move_in_direction(current_position, tile.directions[i].opposite());

            if !valid_and_unmarked(current_position) {
                return None;
            }
            covered.insert(current_position);
        }

        let mut current_position = position;

        // now move forwards after the start index
        for i in start_index..tile.directions.len() {
            current_position = self.move_in_direction(current_position, tile.directions[i]);

            if !valid_and_unmarked(current_position) {
                return None;
            }

            covered.insert(current_position);
        }

        Some(TilePosition::new(
            position,
            tile.clone(),
            start_index,
            covered,
        ))
    }

    fn mark_tile_at_position(&mut self, tp: TilePosition) {
        for position in tp.covered {
            self.mark(position);
        }
    }
}

impl fmt::Debug for RectangularBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut os = Vec::with_capacity((1 + self.width) * self.height);

        for i in 0..self.height {
            for j in 0..self.width {
                os.push(if self.board[i][j] { "x" } else { "*" });
            }
            os.push("\n");
        }

        write!(f, "{}", os.join(""))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Position {
    x: isize,
    y: isize,
}

impl From<(isize, isize)> for Position {
    fn from(p: (isize, isize)) -> Self {
        Position::new(p.0, p.1)
    }
}

impl From<(usize, usize)> for Position {
    fn from(p: (usize, usize)) -> Self {
        Position::new(p.0 as isize, p.1 as isize)
    }
}

impl Position {
    pub fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }
}

#[derive(Eq, Clone)]
struct TilePosition {
    position: Position,
    tile: Tile,
    start_index: usize,
    covered: HashSet<Position>,
}

impl TilePosition {
    pub fn new(
        position: Position,
        tile: Tile,
        start_index: usize,
        covered: HashSet<Position>,
    ) -> Self {
        TilePosition {
            covered,
            position,
            tile,
            start_index,
        }
    }
}

impl PartialEq for TilePosition {
    fn eq(&self, other: &TilePosition) -> bool {
        self.covered == other.covered
    }
}
