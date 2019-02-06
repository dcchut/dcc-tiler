use std::collections::{HashSet,HashMap};
use std::sync::{Arc, RwLock};
use rayon::prelude::*;
use serde_derive::Serialize;
use clap::{Arg, App};

#[macro_use]
extern crate clap;

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


impl Direction {
    /// Returns the opposite of this direction
    ///
    /// # Examples
    ///
    /// ```
    /// // Example code here
    /// ```
    pub fn opposite(&self) -> Self {
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
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    directions : Vec<Direction>,
}

#[derive(Debug)]
enum Axis {
    Vertical,
    Horizontal,
}

impl Tile {
    pub fn new(directions : Vec<Direction>) -> Self {
        Tile {
            directions
        }
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
    /// let tile = Tile::l_tile(2);
    /// assert_eq!(tile, vec![Direction::Left, Direction::Up, Direction::Up]);
    /// ```
    pub fn l_tile(length : usize) -> Self {
        assert!(length > 0);

        let mut directions = vec![Direction::Left];

        for _ in 0..(length-1) {
            directions.push(Direction::Up);
        }

        Tile::new(directions)
    }

    pub fn t_tile(length : usize) -> Self {
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
    /// ```
    /// let l = Tile::new(vec![Direction::Left, Direction::Up, Direction::Left]);
    /// let q = Tile::new(vec![Direction::Up, Direction::Right, Direction::Up]);
    /// assert_eq!(l.rotate(), q);
    /// ```
    fn rotate(&self) -> Tile {
        let mut directions = Vec::new();

        for direction in &self.directions {
            directions.push(match direction {
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::UpLeft => Direction::UpRight,
                Direction::UpRight => Direction::DownRight,
                Direction::DownRight => Direction::DownLeft,
                Direction::DownLeft => Direction::UpLeft,
            });
        }
        Tile::new(directions)
    }

    /// Returns a reflected (about the specified axis) copy of this tile
    ///
    /// # Examples
    ///
    /// ```
    /// let tile = Tile::new(vec![Direction::Left, Direction::Up, Direction::Right]);
    /// let reflected_tile = tile.reflect(Axis::Vertical);
    /// assert_eq!(reflected_tile, Tile::new(vec![Direction::Left, Direction::Down, Direction::Right]));
    /// ```
    fn reflect(&self, axis : Axis) -> Tile {
        let do_reflect = |t: &Direction| {
            match axis {
                Axis::Vertical => {
                    match t {
                        Direction::Up => Direction::Down,
                        Direction::Down => Direction::Up,
                        Direction::UpLeft => Direction::DownLeft,
                        Direction::UpRight => Direction::DownRight,
                        Direction::DownLeft => Direction::UpLeft,
                        Direction::DownRight => Direction::UpRight,
                        x => x.clone(),
                    }
                },
                Axis::Horizontal => {
                    match t {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                        Direction::UpLeft => Direction::UpRight,
                        Direction::UpRight => Direction::UpLeft,
                        Direction::DownLeft => Direction::DownRight,
                        Direction::DownRight => Direction::DownLeft,
                        x => x.clone(),
                    }
                }
            }
        };

        Tile::new(self.directions.iter().map(do_reflect).collect())
    }
}

#[derive(Debug, Clone)]
pub struct TileCollection {
    tiles : Vec<Tile>,
}

impl TileCollection {
    pub fn new(tiles : Vec<Tile>) -> Self {
        TileCollection {
            tiles,
        }
    }
}

impl From<Tile> for TileCollection {
    fn from(tile: Tile) -> Self {
        /// Generates the orbit of this tile under the symmetry + rotate actions
        /// # Examples
        ///
        /// ```
        /// let tile = Tile::l_tile(3);
        /// assert_eq!(tile.symmetry_orbit().len(), 4);
        /// ```
        fn symmetry_orbit(tile : Tile) -> TileCollection {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RectangularBoard {
    width : usize,
    height : usize,
    board : Vec<Vec<bool>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position(i32, i32);

#[derive(Debug, Eq, Clone)]
pub struct TilePosition {
    position : Position,
    tile : Tile,
    start_index : usize,
    covered : HashSet<Position>
}

impl TilePosition {
    pub fn new(position : Position, tile : Tile, start_index : usize, covered : HashSet<Position>) -> Self {
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

    /*class TBoard(Board):
    def __init__(self, n):
        super().__init__(2 * n, n ** 2)

        for i in range(0, n):
            for j in range(0, n):
                self.board[n + i][j] = False
                self.board[n + i][n ** 2 - n + j] = False
                self.mark([n + i, j])
                self.mark([n + i, n ** 2 - n + j])
                */
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
        self.board[p.0 as usize][p.1 as usize] = true;
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

    pub fn get_unmarked_position(&self, tiles : &Vec<Tile>) -> Option<Position> {
        // for each valid position,
        let mut counts = HashMap::new();

        for (i, row) in self.board.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    // increment adjacent positions by 1
                    for p in vec![Position(i as i32 - 1, j as i32),
                    Position(i as i32, j as i32 + 1),
                    Position(i as i32, j as i32 - 1),
                    Position(i as i32 + 1, j as i32)] {
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
                            if let Some(_) = self.tile_fits_at_position(tile, Position(i as i32, j as i32), start_index) {
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

                    let p = Position(i as i32, j as i32);
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

        self.board[p.0 as usize][p.1 as usize]
    }

    fn is_valid(&self, p : &Position) -> bool {
        p.0 >= 0 && (p.0 as usize) < self.height && p.1 >= 0 && (p.1 as usize) < self.width
    }


    fn move_in_direction(&self, p : &Position, direction : &Direction) -> Position {
        let mut row = p.0;
        let mut col = p.1;

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

        Position(row, col)
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

#[derive(Debug, Serialize)]
pub struct BoardGraph {
    nodes_arena : Vec<RectangularBoard>,
    nodes_arena_index : usize,
    edges : HashMap<usize, HashSet<usize>>,
    complete_index : Option<usize>,
}

impl BoardGraph {
    pub fn new() -> Self {
        BoardGraph {
            nodes_arena : Vec::new(),
            nodes_arena_index : 0,
            edges : HashMap::new(),
            complete_index : None,
        }
    }

    pub fn add_node(&mut self, val : RectangularBoard) -> usize {
        self.nodes_arena.push(val);

        self.nodes_arena_index += 1;
        self.nodes_arena_index - 1
    }

    pub fn add_edge(&mut self, s : usize, t : usize) {
        assert!(s < self.nodes_arena_index && t < self.nodes_arena_index);

        self.edges.entry(s).or_insert_with(HashSet::new).insert(t);
    }
}

use simplesvg::{Fig, Svg, Attr, Color};
use rand::Rng;

fn render_single_tiling_from_vec(boards : Vec<RectangularBoard>) -> Fig {
    let mut tile_hashmap = HashMap::new();

    for i in (1..boards.len()).rev() {
        tile_hashmap.insert(boards[i].clone(), vec![boards[i-1].clone()]);
    }

    render_single_tiling(boards.last().unwrap(), &tile_hashmap)
}


fn render_single_tiling(board : &RectangularBoard, tile_hashmap : &HashMap<RectangularBoard, Vec<RectangularBoard>>) -> Fig {
    let gap_size = 0.0;
    let box_size = 50.0;
    let padding = 10.0;


    let colors = vec![Color(30,56,136),
                      Color(71, 115, 170),
                      Color(245, 230, 99),
                      Color(255, 173, 105),
                      Color(156, 56, 72),
                      //Color(95, 199, 227),
        Color(124,178,135),
        Color(251,219,136)
    ];

    let mut boxes = Vec::new();
    let mut color_index = 0;
    let mut current = board;

    while tile_hashmap.contains_key(current) {
        // choose a random source for this board state
        let next = rand::thread_rng().gen_range(0, tile_hashmap[current].len());
        let next_board = tile_hashmap.get(current).unwrap().get(next).unwrap();

        let mut tiled_positions = HashSet::new();

        // compute the tile that was placed here
        for y in 0..next_board.height {
            for x in 0..next_board.width {
                if next_board.board[y][x] ^ current.board[y][x] {
                    // we just tiled this position
                    tiled_positions.insert((x,y));
                }
            }
        }

        for (x,y) in tiled_positions.iter() {
            // draw the underlying box
            let rect = Fig::Rect((*x as f32) * (box_size + gap_size) + padding,
                                 (*y as f32) * (box_size + gap_size)+padding,
                                 box_size,
                                 box_size)
                .styled(Attr::default().fill(colors[color_index]));

            boxes.push(rect);


            enum Border {
                Left,
                Right,
                Top,
                Bottom
            };

            // helper function to construct our borders
            let border = |x: usize, y: usize, b : Border, gray : bool| {
                let xs = match b {
                    Border::Right => (x as f32 + 1.0) * (box_size + gap_size) + padding - gap_size,
                    _ => (x as f32) * (box_size + gap_size) + padding,
                };
                let ys = match b {
                    Border::Top => (y as f32 + 1.0) * (box_size + gap_size) + padding - gap_size,
                    _ => (y as f32) * (box_size + gap_size) + padding,
                };
                let xe = match b {
                    Border::Left => (x as f32) * (box_size + gap_size) + padding,
                    _ => (x as f32 + 1.0) * (box_size + gap_size) + padding - gap_size
                };
                let ye = match b {
                    Border::Bottom => (y as f32) * (box_size + gap_size) + padding,
                    _ => (y as f32 + 1.0) * (box_size + gap_size) + padding - gap_size,
                };

                let mut b = Fig::Line(xs, ys, xe, ye);
                b = b.styled(Attr::default().stroke(if gray { Color(211, 211, 211) } else { Color(0, 0, 0) }).stroke_width(0.5));

                b
            };

            // left border
            boxes.push(border(*x, *y, Border::Left, tiled_positions.contains(&(*x-1, *y))));
            // right border
            boxes.push(border(*x,*y, Border::Right, tiled_positions.contains(&(*x+1,*y))));
            // top border
            boxes.push(border(*x,*y,Border::Top, tiled_positions.contains(&(*x,*y+1))));
            // bottom border
            boxes.push(border(*x,*y,Border::Bottom, tiled_positions.contains(&(*x,*y-1))));
        }

        // increment the color index by 1
        color_index = (color_index + 1) % colors.len();


        current = next_board;
    }

    Fig::Multiple(boxes)
}

pub struct Tiler {
    tiles: TileCollection,
    board: RectangularBoard,
}

impl Tiler {
    pub fn new(tiles : TileCollection, board : RectangularBoard) -> Self {
        Tiler {
            tiles,
            board,
        }
    }
}

pub fn get_single_tiling(tiler : Tiler) -> Option<Vec<RectangularBoard>> {
    let mut stack = Vec::new();
    stack.push(vec![tiler.board.clone()]);

    while let Some(tvec) = stack.pop() {
        let current_board = tvec.last().unwrap();

        if let Some(p) = current_board.get_unmarked_position(&tiler.tiles.tiles) {
            let mut fitting_tiles = Vec::new();

            for tile in tiler.tiles.tiles.iter() {
                for start_index in 0..=tile.directions.len() {
                    if let Some(tile_position) = current_board.tile_fits_at_position(tile, p, start_index) {
                        if !fitting_tiles.contains(&tile_position) {
                            fitting_tiles.push(tile_position);
                        }
                    }
                }
            }

            for tp in fitting_tiles {
                let mut marked_board = current_board.clone();
                marked_board.mark_tile_at_position(tp);

                let is_all_marked = marked_board.is_all_marked();

                let mut new_tvec = tvec.clone();
                new_tvec.push(marked_board);

                if is_all_marked {
                    return Some(new_tvec);
                }

                stack.push(new_tvec);
            }
        }
    }

    None
}


pub fn count_tilings(tiler : Tiler) -> u64 {
    // at each stage, keep track of the counts
    let mut counter = HashMap::new();
    counter.insert(tiler.board.clone(), 1);

    let mut counter = Arc::new(RwLock::new(counter));

    let tiler_ref = Arc::new(RwLock::new(tiler.tiles.tiles.clone()));

    let mut stack = HashSet::new();
    stack.insert(tiler.board.clone());

    let mut completed_board = None;
    //let mut iterations = 0;

    while !stack.is_empty() {
        //iterations += 1;
        //dbg!(iterations);
/*
        for board in &stack {
            println!("{}", board.disp());
            dbg!(board.get_unmarked_position(&tiler_ref.read().unwrap()));
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input);
*/

        let handles = stack.par_iter().map(|b| {
            let current_tiler_ref = Arc::clone(&tiler_ref);
            let current_counter_ref = Arc::clone(&counter);

            let mut next_boards = HashSet::new();
            let mut completed_boards = HashSet::new();
            let mut count_updates = HashMap::new();

            if let Some(p) = b.get_unmarked_position(&current_tiler_ref.read().unwrap()) {
                let mut fitting_tiles = Vec::new();

                for tile in current_tiler_ref.read().unwrap().iter() {
                    for start_index in 0..=tile.directions.len() {
                        if let Some(tile_position) = b.tile_fits_at_position(tile, p, start_index) {
                            if !fitting_tiles.contains(&tile_position) {
                                fitting_tiles.push(tile_position);
                            }
                        }
                    }
                }

                for tp in fitting_tiles {
                    let mut marked_board = b.clone();
                    marked_board.mark_tile_at_position(tp);

                    // how many tilings does the previous state have?
                    let current_count = current_counter_ref.read().unwrap()[&b];

                    *count_updates.entry(marked_board.clone()).or_insert(0) += current_count;

                    if marked_board.is_all_marked() {
                        completed_boards.insert(marked_board);
                    } else {
                        next_boards.insert(marked_board);
                    }
                }
            }

            (next_boards, completed_boards, count_updates)
        }).collect::<Vec<_>>();

        stack = HashSet::new();
        counter = Arc::new(RwLock::new(HashMap::new()));

        for (next_boards, completed_boards, count_updates) in handles {
            stack.extend(next_boards);

            {
                let mut write = counter.write().unwrap();

                // update the counts
                for (board, cnt) in count_updates {
                    let entry = write.entry(board).or_insert(0);
                    (*entry) += cnt;
                }
            }

            for board in completed_boards {
                completed_board = Some(board);
            }
        }
    }

    if let Some(completed_board) = completed_board {
        counter.read().unwrap()[&completed_board]
    } else {
        0
    }
}

pub fn compute_boardgraph(tiler : Tiler) -> BoardGraph {
    let mut hashm = HashMap::new();
    let mut completed_boards = HashSet::new();

    let mut board_graph = BoardGraph::new();
    let mut board_graph_hashmap = HashMap::new();

    let board = tiler.board.clone();
    let tiles = tiler.tiles.clone();

    // add the starting board to our hashmap & graoh
    board_graph_hashmap.insert(board.clone(), board_graph.add_node(board.clone()));

    let mut stack = HashSet::new();
    stack.insert(board);

    while !stack.is_empty() {
        let handles : Vec<_> = stack.into_par_iter().map(|b| {
            let mut to_stack = Vec::new();
            let mut to_hash = HashMap::new();
            let mut to_completed = Vec::new();

            if let Some(p) = b.get_unmarked_position(&tiles.tiles) {
                let mut fitting_tiles: Vec<TilePosition> = Vec::new();

                for tile in &tiles.tiles {
                    for start_index in 0..=tile.directions.len() {
                        if let Some(tp) = b.tile_fits_at_position(tile, p, start_index) {
                            if !fitting_tiles.contains(&tp) {
                                fitting_tiles.push(tp);
                            }
                        }
                    }
                }

                let mut next_board = HashMap::new();

                // now for each fitting tile, mark our board with this tile & add it to the stack
                for tp in fitting_tiles {
                    let mut marked_board = b.clone();
                    marked_board.mark_tile_at_position(tp.clone());

                    next_board.entry(marked_board).or_insert_with(Vec::new).push(tp);
                }


                for (k, _) in next_board.into_iter() {
                    to_hash.entry(k.clone()).or_insert_with(Vec::new).push(b.clone());
                    if k.is_all_marked() {
                        to_completed.push(k);
                    } else {
                        to_stack.push(k);
                    }
                }
            }

            (to_stack, to_hash, to_completed)
        }).collect();

        // now merge the results back in
        stack = HashSet::new();

        for (to_stack, to_hash, to_completed) in handles {
            // merge everything in
            stack.extend(to_stack);
            completed_boards.extend(to_completed);

            // merge in hashm
            for (k,v) in to_hash {
                // k = target, v = sources?

                if !board_graph_hashmap.contains_key(&k) {
                    board_graph_hashmap.insert(k.clone(), board_graph.add_node(k.clone()));
                }

                let node = board_graph_hashmap[&k];

                // now insert an edge from v to k
                for p in &v {
                    board_graph.add_edge(board_graph_hashmap[p], node);
                }

                let entry = hashm.entry(k).or_insert_with(Vec::new);
                (*entry).extend(v);
            }
        }
    }

    for complete in completed_boards {
        board_graph.complete_index = Some(board_graph_hashmap[&complete]);
    }
    board_graph
}

arg_enum!{
    #[derive(Debug, Copy, Clone)]
    pub enum BoardType {
        Rectangle,
        LBoard,
        TBoard,
    }
}

arg_enum!{
    #[derive(Debug, Copy, Clone)]
    pub enum TileType {
        LTile,
        TTile
    }
}



fn main() {
    let matches = App::new("rs-tiler")
        .version("1.0")
        .author("Robert Usher")
        .about("Computes various tilings")
        .arg(Arg::with_name("board_size")
                 .help("The size of the board to tile")
                 .index(1)
                 .required(true))
        .arg(Arg::with_name("width")
                 .short("w")
                 .long("width")
                 .takes_value(true)
                 .help("The (optional) width of the board"))
        .arg(Arg::with_name("board_type")
                 .help("The type of board to use")
                 .possible_values(&BoardType::variants())
                 .default_value("LBoard")
                 .index(3))
        .arg(Arg::with_name("board_scale")
                 .help("The board scale to use, if using an LBoard")
                 .long("scale")
                 .default_value("1"))
        .arg(Arg::with_name("tile_type")
                 .help("The type of tile to use")
                 .possible_values(&TileType::variants())
                 .default_value("LTile")
                 .index(4))
        .arg(Arg::with_name("tile_size")
                 .help("The size of the tile")
                 .index(2)
                 .required(true))
        .arg(Arg::with_name("single")
                 .short("s")
                 .long("single")
                 .help("Computes a single tiling")
                 .conflicts_with("count")
                 .conflicts_with("graph"))
        .arg(Arg::with_name("count")
                 .short("c")
                 .long("count")
                 .help("Counts all tilings")
                 .conflicts_with("single")
                 .conflicts_with("graph"))
        .arg(Arg::with_name("graph")
                 .short("g")
                 .long("graph")
                 .help("Computes the full tilings graph")
                 .conflicts_with("count")
                 .conflicts_with("single"))
        .arg(Arg::with_name("scaling")
                 .long("scaling")
                 .help("Computes the tiling count for different values of the scale parameter")
                 .conflicts_with("graph")
                 .conflicts_with("count")
                 .conflicts_with("single"))
        .get_matches();

    let board_type = value_t!(matches.value_of("board_type"), BoardType).unwrap_or_else(|e| e.exit());
    let tile_type = value_t!(matches.value_of("tile_type"), TileType).unwrap_or_else(|e| e.exit());
    let board_size = value_t!(matches.value_of("board_size"),usize).unwrap_or_else(|e| e.exit());

    let board_width = if matches.is_present("width") {
        value_t!(matches.value_of("width"), usize).unwrap_or_else(|e| e.exit())
    } else {
        board_size
    };

    let tile_size = value_t!(matches.value_of("tile_size"), usize).unwrap_or_else(|e| e.exit());
    let board_scale = value_t!(matches.value_of("board_scale"), usize).unwrap_or_else(|e| e.exit());

    // first, get the tilecollection associated to the tile type specified by the user
    let tile = match tile_type {
        TileType::LTile => {
            Tile::l_tile(tile_size)
        },
        TileType::TTile => {
            Tile::t_tile(tile_size)
        },
    };

    let tiles = TileCollection::from(tile);

    // closure to create a board
    let make_board = |board_type : BoardType, board_size : usize, board_width : usize, board_scale : usize| {
        match board_type {
            BoardType::Rectangle => RectangularBoard::new(board_width, board_size),
            BoardType::LBoard => RectangularBoard::l_board(board_size, board_scale),
            BoardType::TBoard => RectangularBoard::t_board(board_size, board_scale),
        }
    };

    let board = make_board(board_type, board_size, board_width,board_scale);

    // now, do some stuff
    if matches.is_present("scaling") {
        let mut board_scale : usize = 1;

        loop {
            let tiler = Tiler::new(tiles.clone(), make_board(board_type, board_size, board_width,board_scale));
            println!("scale({}), {} tilings", board_scale, count_tilings(tiler));
            board_scale += 1;
        }
    } else if matches.is_present("count") {
        dbg!(count_tilings(Tiler::new(tiles, board)));
    } else if matches.is_present("single") {
        let tiler = Tiler::new(tiles, board);

        // render a single tiling
        let tiling = get_single_tiling(tiler);

        if let Some(tiling) = tiling {
            let w = 75 * tiling[0].width as u32;
            let h = 75 * tiling[0].height as u32;
            let figure = render_single_tiling_from_vec(tiling);

            println!("{}", Svg(vec![figure], w, h).to_string());
        } else {
            println!("No tilings found!");
        }
    } else if matches.is_present("graph") {
        let tiler = Tiler::new(tiles, board);

        // compute the entire boardgraph for this tiler
        let board_graph = compute_boardgraph(tiler);

        println!("{}", serde_json::to_string(&board_graph).unwrap());
    }

    /*
        let current_time = time::now().to_timespec().sec;

        let tiling_filename = format!("tiling_{}.html", current_time);
        fs::write(tiling_filename, Svg(vec![render_single_tiling(&complete, &hashm)], 50 * complete.width as u32, 50 * complete.height as u32).to_string()).expect("Unable to write tiling");

        //let tilegraph_filename = format!("graph_{}.json", current_time);
        //fs::write(tilegraph_filename, serde_json::to_string(&board_graph).unwrap()).expect("Unable to write tiling graph");
    */
}