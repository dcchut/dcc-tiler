extern crate tilelib;

use num::{BigUint, One, Zero};
use tilelib::board::RectangularBoard;
use tilelib::graph::BoardGraph;
use tilelib::tile::{Tile, TileCollection};

use clap::{App, Arg};
use rand::Rng;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tilelib::render::render_single_tiling_from_vec;

#[macro_use]
extern crate clap;

pub struct Tiler {
    tiles: TileCollection,
    initial_board: RectangularBoard,
    graph: Option<Arc<RwLock<BoardGraph>>>,
}

impl Tiler {
    pub fn new(tiles: TileCollection, initial_board: RectangularBoard) -> Self {
        Tiler {
            tiles,
            initial_board,
            graph: None,
        }
    }

    pub fn count_tilings(&mut self) -> BigUint {
        // Use a boardgraph, if available.
        if self.graph.is_some() {
            self.count_tilings_from_graph()
        } else {
            self.count_tilings_quick()
        }
    }

    fn count_tilings_quick(&self) -> BigUint {
        // we keep the counter behind an Arc<RwLock<>>
        let mut counter = HashMap::new();
        counter.insert(self.initial_board.clone(), num::BigUint::one());
        let mut counter = Arc::new(RwLock::new(counter));

        // our working stack
        let mut stack = HashSet::new();
        stack.insert(self.initial_board.clone());

        let completed_board = Arc::new(RwLock::new(Vec::new()));

        while !stack.is_empty() {
            let handles = stack
                .par_iter()
                .map(|b| {
                    let current_count = &counter.read().unwrap()[&b];

                    let boards = b.place_tile(&self.tiles);

                    let mut next_boards = HashSet::new();
                    let mut completed_boards = HashSet::new();
                    let mut count_updates = HashMap::new();

                    for board in boards {
                        *count_updates
                            .entry(board.clone())
                            .or_insert(num::BigUint::zero()) += current_count;

                        if board.is_all_marked() {
                            completed_boards.insert(board);
                        } else {
                            next_boards.insert(board);
                        }
                    }

                    (next_boards, completed_boards, count_updates)
                })
                .collect::<Vec<_>>();

            let step_stack = Arc::new(RwLock::new(HashSet::new()));
            counter = Arc::new(RwLock::new(HashMap::new()));

            handles
                .into_par_iter()
                .for_each(|(next_boards, completed_boards, count_updates)| {
                    // extend the new stack
                    {
                        let mut stack_write = step_stack.write().unwrap();
                        stack_write.extend(next_boards);
                    }

                    // update all of the tiling counts
                    {
                        let mut counter_write = counter.write().unwrap();

                        // update the counts
                        for (board, count) in count_updates {
                            let entry = counter_write.entry(board).or_insert(num::BigUint::zero());
                            (*entry) += count;
                        }
                    }

                    // mark the completed board
                    for board in completed_boards {
                        // we obtain the lock on completed_board inside this for loop,
                        // because having a completed board occurs so infrequently
                        {
                            let mut completed_board_write = completed_board.write().unwrap();
                            completed_board_write.push(board);
                        }
                    }
                });

            // unwrap our stack
            stack = Arc::try_unwrap(step_stack).unwrap().into_inner().unwrap();
        }

        let completed_board = completed_board.read().unwrap();

        if let Some(board) = completed_board.last() {
            counter.read().unwrap()[board].clone()
        } else {
            num::BigUint::zero()
        }
    }

    fn count_tilings_from_graph(&self) -> BigUint {
        let graph = Arc::clone(self.graph.as_ref().unwrap());
        let g = graph.read().unwrap();

        // if the graph doesn't have any complete tilings,
        // then we don't have to do any work
        let complete_board_index = g.get_complete_index();

        if complete_board_index.is_none() {
            return BigUint::zero();
        }

        let mut count_map = HashMap::new();
        count_map.insert(0, BigUint::one());

        // now work through the stack
        let mut stack = HashSet::new();
        stack.insert(0);

        while !stack.is_empty() {
            let mut next_stack = HashSet::new();

            for board_index in stack {
                let c = count_map[&board_index].clone();

                if let Some(edges) = g.get_edges(board_index) {
                    for edge in edges {
                        let entry = count_map.entry(*edge).or_insert(BigUint::zero());
                        (*entry) += c.clone();

                        next_stack.insert(*edge);
                    }
                }
            }

            stack = next_stack;
        }

        if let Some(res) = count_map.remove(&complete_board_index.unwrap()) {
            res
        } else {
            BigUint::zero()
        }
    }

    #[allow(dead_code, clippy::map_entry)]
    fn generate_graph(&mut self) {
        let mut graph = BoardGraph::new();
        graph.add_node(self.initial_board.clone());

        let graph = Arc::new(RwLock::new(graph));

        let mut stack = vec![0];

        while !stack.is_empty() {
            let mut next_iteration = Vec::new();
            let mut board_map: HashMap<RectangularBoard, usize> = HashMap::new();

            for (board_index, child_boards) in stack
                .into_par_iter()
                .map(|board_index| {
                    let g = graph.read().unwrap();

                    // get the current board
                    (
                        board_index,
                        if let Some(board) = g.get_node(board_index) {
                            // now for each board, place a tile at some position,
                            board.place_tile(&self.tiles)
                        } else {
                            Vec::new()
                        },
                    )
                })
                .collect::<Vec<_>>()
            {
                // find / create the node id for this board
                let mut g = graph.write().unwrap();

                for board in child_boards {
                    let complete = board.is_all_marked();

                    // We don't want to use an entry here because it would mean
                    // having to clone our board every single time, even if the board
                    // was already in our hashmap
                    let child_index = if board_map.contains_key(&board) {
                        board_map[&board]
                    } else {
                        let index = g.add_node(board.clone());
                        board_map.insert(board, index);
                        index
                    };

                    g.add_edge(board_index, child_index);

                    if complete {
                        // mark this as a finished node in our graph
                        g.mark_node_as_complete(child_index);
                    } else {
                        next_iteration.push(child_index);
                    }
                }
            }

            stack = next_iteration;
        }
        self.graph = Some(graph);
    }

    pub fn graph(&mut self) -> Arc<RwLock<BoardGraph>> {
        // If the graph doesn't exist already, generate it
        if self.graph.is_none() {
            self.generate_graph();
        }

        // Now return a reference to the graph
        Arc::clone(self.graph.as_ref().unwrap())
    }

    pub fn get_single_tiling(&mut self, limit : usize) -> Option<Vec<RectangularBoard>> {
        let mut stack = vec![vec![self.initial_board.clone()]];
        let mut completed_tilings = Vec::new();

        while let Some(tvec) = stack.pop() {
            let current_board = tvec.last().unwrap();
            let fitting_tiles = current_board.place_tile(&self.tiles);

            for board in fitting_tiles {
                let is_all_marked = board.is_all_marked();

                let mut new_tvec = tvec.clone();
                new_tvec.push(board);

                if is_all_marked {
                    completed_tilings.push(new_tvec);
                } else {
                    stack.push(new_tvec);
                }
            }

            if completed_tilings.len() >= limit {
                break;
            }
        }

        if !completed_tilings.is_empty() {
            // Select a random solution from those already found
            let solution_index = rand::thread_rng().gen_range(0, completed_tilings.len());
            return Some(completed_tilings[solution_index].clone());
        }

        None
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum BoardType {
        Rectangle,
        LBoard,
        TBoard,
    }
}

arg_enum! {
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
        .arg(
            Arg::with_name("board_size")
                .help("The size of the board to tile")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .takes_value(true)
                .help("The (optional) width of the board"),
        )
        .arg(
            Arg::with_name("board_type")
                .help("The type of board to use")
                .possible_values(&BoardType::variants())
                .default_value("LBoard")
                .index(3),
        )
        .arg(
            Arg::with_name("board_scale")
                .help("The board scale to use, if using an LBoard")
                .long("scale")
                .default_value("1"),
        )
        .arg(
            Arg::with_name("tile_type")
                .help("The type of tile to use")
                .possible_values(&TileType::variants())
                .default_value("LTile")
                .index(4),
        )
        .arg(
            Arg::with_name("tile_size")
                .help("The size of the tile")
                .index(2)
                .required(true),
        )
        .arg(
            Arg::with_name("single")
                .short("s")
                .long("single")
                .help("Computes a single tiling")
                .conflicts_with("count")
                .conflicts_with("graph"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Counts all tilings")
                .conflicts_with("single")
                .conflicts_with("graph"),
        )
        .arg(
            Arg::with_name("graph")
                .short("g")
                .long("graph")
                .help("Computes the full tilings graph")
                .conflicts_with("count")
                .conflicts_with("single"),
        )
        .arg(
            Arg::with_name("scaling")
                .long("scaling")
                .help("Computes the tiling count for different values of the scale parameter")
                .conflicts_with("graph")
                .conflicts_with("count")
                .conflicts_with("single"),
        )
        .get_matches();

    let board_type =
        value_t!(matches.value_of("board_type"), BoardType).unwrap_or_else(|e| e.exit());
    let tile_type = value_t!(matches.value_of("tile_type"), TileType).unwrap_or_else(|e| e.exit());
    let board_size = value_t!(matches.value_of("board_size"), usize).unwrap_or_else(|e| e.exit());

    let board_width = if matches.is_present("width") {
        value_t!(matches.value_of("width"), usize).unwrap_or_else(|e| e.exit())
    } else {
        board_size
    };

    let tile_size = value_t!(matches.value_of("tile_size"), usize).unwrap_or_else(|e| e.exit());
    let board_scale = value_t!(matches.value_of("board_scale"), usize).unwrap_or_else(|e| e.exit());

    // Create a colletion of tiles based on the tile(s) specified by the user
    let tile = match tile_type {
        TileType::LTile => Tile::l_tile(tile_size),
        TileType::TTile => Tile::t_tile(tile_size),
    };

    let tiles = TileCollection::from(tile);

    // A closure to create a board based on specified options
    let make_board =
        |board_type: BoardType, board_size: usize, board_width: usize, board_scale: usize| {
            match board_type {
                BoardType::Rectangle => RectangularBoard::new(board_width, board_size),
                BoardType::LBoard => RectangularBoard::l_board(board_size, board_scale),
                BoardType::TBoard => RectangularBoard::t_board(board_size, board_scale),
            }
        };

    if matches.is_present("scaling") {
        // we deal with scaling separately to appease the borrow checker
        let mut board_scale: usize = 1;

        loop {
            let mut tiler = Tiler::new(
                tiles.clone(),
                make_board(board_type, board_size, board_width, board_scale),
            );
            println!("scale({}), {} tilings", board_scale, tiler.count_tilings());
            board_scale += 1;
        }
    } else {
        let board = make_board(board_type, board_size, board_width, board_scale);
        let mut tiler = Tiler::new(tiles, board);

        if matches.is_present("count") {
            // just do a quick tilings count
            dbg!(tiler.count_tilings());
        } else if matches.is_present("single") {
            let tiling = tiler.get_single_tiling(1000);

            if let Some(tiling) = tiling {
                println!("{}", render_single_tiling_from_vec(tiling));
            } else {
                println!("No tilings found!");
            }
        } else if matches.is_present("graph") {
            let board_graph = tiler.graph();

            {
                let board_graph = board_graph.read().unwrap();

                println!("{}", serde_json::to_string(&*board_graph).unwrap());
            }
        }
    }
}
