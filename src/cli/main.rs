use dcc_tiler::board::RectangularBoard;
use dcc_tiler::tile::{Tile, TileCollection};

use clap::{ArgEnum, Parser};

use dcc_tiler::render::render_single_tiling_from_vec;
use std::io::Result;
use tiler::Tiler;

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum BoardType {
    #[clap(name = "Rectangle")]
    Rectangle,
    #[clap(name = "LBoard")]
    LBoard,
    #[clap(name = "TBoard")]
    TBoard,
}

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum TileType {
    #[clap(name = "LTile")]
    LTile,
    #[clap(name = "TTile")]
    TTile,
    #[clap(name = "BoxTile")]
    BoxTile,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(help = "The size of the board to tile")]
    board_size: usize,

    #[clap(help = "The size of the tile")]
    tile_size: usize,

    #[clap(short, long, help = "The width of the board")]
    width: Option<usize>,

    #[clap(long, arg_enum, default_value_t = BoardType::LBoard, help = "The type of board to use")]
    board_type: BoardType,

    #[clap(
        long = "scale",
        default_value_t = 1,
        help = "The board scale ot use, if using an LBoard"
    )]
    board_scale: usize,

    #[clap(long, arg_enum, default_value_t = TileType::LTile, help = "The type of tile to use")]
    tile_type: TileType,

    #[clap(
        short,
        long,
        help = "Compute a single tiling",
        conflicts_with = "count",
        conflicts_with = "graph"
    )]
    single: bool,

    #[clap(
        short,
        long,
        help = "Render all tilings to a specified file in ZIP format",
        conflicts_with = "single",
        conflicts_with = "count",
        conflicts_with = "graph",
        conflicts_with = "scaling"
    )]
    all: Option<String>,

    #[clap(
        short,
        long,
        help = "Count all tilings",
        conflicts_with = "single",
        conflicts_with = "graph"
    )]
    count: bool,

    #[clap(
        short,
        long,
        help = "Compute the full tilings graph",
        conflicts_with = "count",
        conflicts_with = "single"
    )]
    graph: bool,

    #[clap(
        long,
        help = "Compute the tiling count for different value of the scale parameter",
        conflicts_with = "graph",
        conflicts_with = "count",
        conflicts_with = "single"
    )]
    scaling: bool,
}

mod tiler;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let board_width = cli.width.unwrap_or(cli.board_size);

    // Create a colletion of tiles based on the tile(s) specified by the user
    let tile = match cli.tile_type {
        TileType::LTile => Tile::l_tile(cli.tile_size),
        TileType::TTile => Tile::t_tile(cli.tile_size),
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

    if cli.scaling {
        // we deal with scaling separately to appease the borrow checker
        let mut board_scale: usize = 1;

        loop {
            let mut tiler = Tiler::new(
                tiles.clone(),
                make_board(cli.board_type, cli.board_size, board_width, board_scale),
            );
            println!("scale({}), {} tilings", board_scale, tiler.count_tilings());
            board_scale += 1;
        }
    } else {
        let board = make_board(cli.board_type, cli.board_size, board_width, cli.board_scale);
        let mut tiler = Tiler::new(tiles, board);

        if cli.count {
            // just do a quick tilings count - no need to generate the tiling graph
            println!("{} tilings found", tiler.count_tilings());
        } else if cli.single {
            let tiling = tiler.get_single_tiling(1000);

            if let Some(tiling) = tiling {
                println!("{}", render_single_tiling_from_vec(tiling.iter().collect()));
            } else {
                println!("No tilings found!");
            }
        } else if let Some(filename) = cli.all {
            tiler.render_all_tilings(&filename)?;
        } else if cli.graph {
            let board_graph = tiler.graph();

            {
                let board_graph = board_graph.read().unwrap();

                println!("{}", serde_json::to_string(&*board_graph).unwrap());
            }
        }
    }

    Ok(())
}
