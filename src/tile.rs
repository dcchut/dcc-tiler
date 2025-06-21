use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    Vertical,
    Horizontal,
}

impl Direction {
    /// Returns the opposite of this direction
    ///
    /// # Examples
    ///
    /// ```
    /// // Example code here
    /// ```
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::UpLeft => Direction::DownRight,
            Direction::DownRight => Direction::UpLeft,
            Direction::UpRight => Direction::DownLeft,
            Direction::DownLeft => Direction::UpRight,
        }
    }

    pub fn rotate(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::UpLeft => Direction::UpRight,
            Direction::UpRight => Direction::DownRight,
            Direction::DownRight => Direction::DownLeft,
            Direction::DownLeft => Direction::UpLeft,
        }
    }

    pub fn reflect(self, axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => match self {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::UpLeft => Direction::DownLeft,
                Direction::UpRight => Direction::DownRight,
                Direction::DownLeft => Direction::UpLeft,
                Direction::DownRight => Direction::UpRight,
                x => x,
            },
            Axis::Vertical => match self {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
                Direction::UpLeft => Direction::UpRight,
                Direction::UpRight => Direction::UpLeft,
                Direction::DownLeft => Direction::DownRight,
                Direction::DownRight => Direction::DownLeft,
                x => x,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    pub directions: Vec<Direction>,
}

impl Tile {
    pub fn new(directions: Vec<Direction>) -> Self {
        Tile { directions }
    }

    /// Returns an L-shaped tile consisting of n + 1 blocks
    ///
    /// # Panics
    ///
    /// Will panic if length = 0
    ///
    /// # Examples
    ///
    /// ```
    /// use dcc_tiler::tile::{Tile, Direction};
    ///
    /// let tile = Tile::l_tile(2);
    /// assert_eq!(tile.directions, vec![Direction::Left, Direction::Up]);
    /// ```
    pub fn l_tile(length: usize) -> Self {
        assert!(length > 0);

        let mut directions = vec![Direction::Left];

        for _ in 0..(length - 1) {
            directions.push(Direction::Up);
        }

        Tile::new(directions)
    }

    pub fn box_tile() -> Self {
        Tile::new(Vec::new())
    }

    pub fn t_tile(length: usize) -> Self {
        assert!(length > 0);

        let mut directions = Vec::new();

        for _ in 0..length {
            directions.push(Direction::Right);
        }
        directions.push(Direction::Up);
        directions.push(Direction::DownRight);

        for _ in 0..(length - 1) {
            directions.push(Direction::Right);
        }

        Tile::new(directions)
    }

    /// Returns a rotated (by 90 degrees clockwise) copy of this tile.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dcc_tiler::tile::{Tile, Direction};
    ///
    /// let l = Tile::new(vec![Direction::Left, Direction::Up, Direction::Left]);
    /// let q = Tile::new(vec![Direction::Up, Direction::Right, Direction::Up]);
    /// assert_eq!(l.rotate(), q);
    /// ```
    pub fn rotate(&self) -> Tile {
        Tile::new(self.directions.iter().map(|d| d.rotate()).collect())
    }

    /// Returns a reflected (about the specified axis) copy of this tile
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dcc_tiler::tile::{Tile, Direction, Axis};
    ///
    /// let tile = Tile::new(vec![Direction::Left, Direction::Up, Direction::Right]);
    /// // Reflect our tile about a vertical line
    /// let reflected_tile = tile.reflect(Axis::Vertical);
    ///
    /// assert_eq!(reflected_tile, Tile::new(vec![Direction::Right, Direction::Up, Direction::Left]));
    /// ```
    pub fn reflect(&self, axis: Axis) -> Tile {
        Tile::new(self.directions.iter().map(|d| d.reflect(axis)).collect())
    }
}

#[derive(Debug, Clone)]
pub struct TileCollection {
    tiles: Vec<Tile>,
    contains_single_tile: bool,
}

impl TileCollection {
    pub fn new(tiles: Vec<Tile>) -> Self {
        TileCollection {
            contains_single_tile: tiles.iter().any(|b| b.directions.is_empty()),
            tiles,
        }
    }

    pub fn contains_single_tile(&self) -> bool {
        self.contains_single_tile
    }

    pub fn iter<'b>(&'b self) -> Box<dyn Iterator<Item = &'b Tile> + 'b> {
        Box::new(self.tiles.iter())
    }
}

impl From<Tile> for TileCollection {
    fn from(tile: Tile) -> Self {
        /// Generates the orbit of this tile under the symmetry + rotate actions
        fn symmetry_orbit(tile: Tile) -> TileCollection {
            let mut orbit = HashSet::new();

            // our starting set of directions
            orbit.insert(tile);

            loop {
                // in each iteration, we check whether our directions set
                // increased.  If it didn't, then we've got the entire orbit
                let current_size = orbit.len();

                let mut to_insert = Vec::new();

                for directions in &orbit {
                    // apply the rotate function
                    to_insert.push(directions.rotate());
                    // apply the two axis reflections
                    to_insert.push(directions.reflect(Axis::Horizontal));
                    to_insert.push(directions.reflect(Axis::Vertical));
                }

                orbit.extend(to_insert);

                if orbit.len() == current_size {
                    break;
                }
            }

            TileCollection::new(orbit.into_iter().collect())
        }
        symmetry_orbit(tile)
    }
}
