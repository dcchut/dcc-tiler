use dcc_tiler::board::RectangularBoard;
use dcc_tiler::graph::BoardGraph;
use dcc_tiler::tile::TileCollection;
use num::{BigUint, One, Zero};

use rand::Rng;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use dcc_tiler::render::render_single_tiling_from_vec;
use std::io::{Result, Write};

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
                            .or_insert_with(num::BigUint::zero) += current_count;

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
                            let entry = counter_write
                                .entry(board)
                                .or_insert_with(num::BigUint::zero);
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
                        let entry = count_map.entry(*edge).or_insert_with(BigUint::zero);
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

    // Maybe change String to Into<PathBuf>?
    pub fn render_all_tilings(&mut self, output_filename: &str) -> Result<()> {
        let graph = self.graph();
        let graph = graph.read().expect("Unable to read graph");
        let mut tiling_counter = 0;

        let path = std::path::Path::new(output_filename);
        let file = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);

        if let Some(complete) = graph.get_complete_index() {
            let board = graph.get_node(complete).unwrap();

            let mut stack = vec![(complete, vec![board])];

            while !stack.is_empty() {
                let (index, boards) = stack.pop().unwrap();

                if index == 0 {
                    // render this tiling
                    let tiling = render_single_tiling_from_vec(boards);

                    // filename for this tiling
                    let tiling_filename = tiling_counter.to_string() + ".svg";

                    zip.start_file(tiling_filename, Default::default())?;
                    zip.write_all(tiling.as_bytes())?;

                    tiling_counter += 1;
                } else {
                    for e in graph.get_rev_edges(index).unwrap() {
                        let mut new_boards = boards.clone();
                        new_boards.push(graph.get_node(*e).unwrap());

                        stack.push((*e, new_boards));
                    }
                }
            }
        }

        let _ = zip.finish()?;

        Ok(())
    }

    pub fn get_single_tiling(&mut self, limit: usize) -> Option<Vec<RectangularBoard>> {
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
