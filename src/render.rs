use crate::board::RectangularBoard;
use rand::Rng;
use simplesvg::{Attr, Color, Fig, Svg};
use std::collections::{HashMap, HashSet};

pub fn render_single_tiling_from_vec(boards: Vec<RectangularBoard>) -> String {
    let mut tile_hashmap = HashMap::new();

    for i in (1..boards.len()).rev() {
        tile_hashmap.insert(boards[i].clone(), vec![boards[i - 1].clone()]);
    }

    render_single_tiling(boards.last().unwrap(), &tile_hashmap)
}

pub fn render_single_tiling<S: ::std::hash::BuildHasher>(
    board: &RectangularBoard,
    tile_hashmap: &HashMap<RectangularBoard, Vec<RectangularBoard>, S>,
) -> String {
    // TODO: maybe remove gap_size now that we've implemented borders
    let gap_size = 0.0;
    let box_size = 50.0;
    let padding = 10.0;

    // TODO: make these configurable
    let colors = vec![
        Color(30, 56, 136),
        Color(71, 115, 170),
        Color(245, 230, 99),
        Color(255, 173, 105),
        Color(156, 56, 72),
        //Color(95, 199, 227),
        Color(124, 178, 135),
        Color(251, 219, 136),
    ];

    let mut boxes = Vec::new();

    // choose a random initial colour
    // we do this so that when you render a single tile, it won't always be the first colour in the colors vector
    let mut color_index = rand::thread_rng().gen_range(0, colors.len());
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
                    tiled_positions.insert((x, y));
                }
            }
        }

        for (x, y) in tiled_positions.iter() {
            // draw the underlying box
            let rect = Fig::Rect(
                (*x as f32) * (box_size + gap_size) + padding,
                (*y as f32) * (box_size + gap_size) + padding,
                box_size,
                box_size,
            )
            .styled(Attr::default().fill(colors[color_index]));

            boxes.push(rect);

            enum Border {
                Left,
                Right,
                Top,
                Bottom,
            };

            // helper function to construct our borders
            let border = |x: usize, y: usize, b: Border, gray: bool| {
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
                    _ => (x as f32 + 1.0) * (box_size + gap_size) + padding - gap_size,
                };
                let ye = match b {
                    Border::Bottom => (y as f32) * (box_size + gap_size) + padding,
                    _ => (y as f32 + 1.0) * (box_size + gap_size) + padding - gap_size,
                };

                let mut b = Fig::Line(xs, ys, xe, ye);
                b = b.styled(
                    Attr::default()
                        .stroke(if gray {
                            Color(211, 211, 211)
                        } else {
                            Color(0, 0, 0)
                        })
                        .stroke_width(0.5),
                );

                b
            };

            // left border
            boxes.push(border(
                *x,
                *y,
                Border::Left,
                tiled_positions.contains(&(*x - 1, *y)),
            ));
            // right border
            boxes.push(border(
                *x,
                *y,
                Border::Right,
                tiled_positions.contains(&(*x + 1, *y)),
            ));
            // top border
            boxes.push(border(
                *x,
                *y,
                Border::Top,
                tiled_positions.contains(&(*x, *y + 1)),
            ));
            // bottom border
            boxes.push(border(
                *x,
                *y,
                Border::Bottom,
                tiled_positions.contains(&(*x, *y - 1)),
            ));
        }

        // increment the color index by 1
        color_index = (color_index + 1) % colors.len();

        current = next_board;
    }

    Svg(
        vec![Fig::Multiple(boxes)],
        (50 * board.width) as u32 + 2 * (padding as u32),
        (50 * board.height) as u32 + 2 * (padding as u32),
    )
    .to_string()
}
