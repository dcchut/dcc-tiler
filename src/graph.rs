use crate::board::RectangularBoard;
use serde_derive::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Default)]
pub struct BoardGraph {
    // The nodes in our graph are boards - we store there here inside a vec
    //// so that we dont have Rc<RefCell<..>> all over the place
    nodes_arena: Vec<RectangularBoard>,

    #[serde(skip_serializing)]
    nodes_arena_index: usize,

    // An edge in our graph indicates that it is possible to get from one board state
    // to another by placing down a tile.
    edges: HashMap<usize, HashSet<usize>>,

    // This hashmap keeps track of edges "going backwards"
    rev_edges: HashMap<usize, HashSet<usize>>,

    // If a complete tiling of an initial board (index 0 in nodes_arena)
    complete_indices: HashSet<usize>,
}

impl BoardGraph {
    pub fn new() -> Self {
        BoardGraph {
            nodes_arena: Vec::new(),
            nodes_arena_index: 0,
            edges: HashMap::new(),
            rev_edges: HashMap::new(),
            complete_indices: HashSet::new(),
        }
    }

    #[allow(clippy::never_loop)]
    pub fn get_complete_index(&self) -> Option<usize> {
        for index in &self.complete_indices {
            return Some(*index);
        }
        None
    }

    pub fn mark_node_as_complete(&mut self, i: usize) {
        self.complete_indices.insert(i);
    }

    pub fn find_node(&self, v: &RectangularBoard) -> Option<usize> {
        for (i, node) in self.nodes_arena.iter().enumerate() {
            if node == v {
                return Some(i);
            }
        }
        None
    }

    pub fn get_edges(&self, i: usize) -> Option<&HashSet<usize>> {
        self.edges.get(&i)
    }

    pub fn get_rev_edges(&self, i: usize) -> Option<&HashSet<usize>> {
        self.rev_edges.get(&i)
    }

    pub fn get_node(&self, i: usize) -> Option<&RectangularBoard> {
        self.nodes_arena.get(i)
    }

    pub fn add_node(&mut self, v: RectangularBoard) -> usize {
        self.nodes_arena.push(v);

        self.nodes_arena_index += 1;
        self.nodes_arena_index - 1
    }

    pub fn add_edge(&mut self, s: usize, t: usize) {
        assert!(s < self.nodes_arena_index && t < self.nodes_arena_index);

        self.edges.entry(s).or_insert_with(HashSet::new).insert(t);
        self.rev_edges
            .entry(t)
            .or_insert_with(HashSet::new)
            .insert(s);
    }
}
