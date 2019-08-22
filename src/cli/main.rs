use dcc_tiler::board::RectangularBoard;
use dcc_tiler::tile::{Tile, TileCollection};

use clap::{App, Arg};

use dcc_tiler::render::render_single_tiling_from_vec;
use std::io::Result;
use tiler::Tiler;

#[macro_use]
extern crate clap;

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
        TTile,
        BoxTile,
    }
}

mod tiler;

fn main() -> Result<()> {
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
                .long("board_type"),
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
                .long("tile_type"),
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
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Renders all tilings to specified file in ZIP format")
                .conflicts_with("single")
                .conflicts_with("count")
                .conflicts_with("graph")
                .conflicts_with("scaling")
                .takes_value(true)
                .required(false),
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
        TileType::BoxTile => Tile::box_tile(),
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
            // just do a quick tilings count - no need to generate the tiling graph
            println!("{} tilings found", tiler.count_tilings());
        } else if matches.is_present("single") {
            let tiling = tiler.get_single_tiling(1000);

            if let Some(tiling) = tiling {
                println!("{}", render_single_tiling_from_vec(tiling.iter().collect()));
            } else {
                println!("No tilings found!");
            }
        } else if matches.is_present("all") {
            let filename = value_t!(matches.value_of("all"), String).unwrap_or_else(|e| e.exit());
            tiler.render_all_tilings(&filename)?;
        } else if matches.is_present("graph") {
            let board_graph = tiler.graph();

            {
                let board_graph = board_graph.read().unwrap();

                println!("{}", serde_json::to_string(&*board_graph).unwrap());
            }
        }
    }

    Ok(())
}
