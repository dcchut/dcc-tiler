# dcc-tiler

### Basic tile terminology

There are currently two types of tiles supported, which are explained below.

#### `LTile`
An `LTile` of size `n` is the L-tetronimo with `n + 1` blocks.  For example
an`LTile` of size 3 is:

![dcc_ltiler_cli --single --board_type LBoard --tile_type LTile 3 3](img/LTile_3.svg)

while an `LTile` of size 5 is:

![dcc_ltiler_cli --single --board_type LBoard --tile_type LTile 5 5](img/LTile_5.svg)

#### `TTile`
A `TTile` of size `n` is the T-tetronimo with `2(n+1)` blocks.  For example, a `TTile` of size 1 is:

![dcc_ltiler_cli --single --board_type TBoard --tile_type TTile 1 1](img/TTile_1.svg)

while a `TTile` of size 2 is:

![dcc_ltiler_cli --single --board_type TBoard --tile_type TTile 2 2](img/TTile_2.svg)

### Basic board terminology

There are currently three supported boards: `Rectangle`, `LBoard`, and `TBoard`.  

#### `LBoard` and `TBoard`

There are two parameters that affect the shape/size of an L/T board: `board_size` and `board_scale`.
With these parameters, a tile (either L or T) of size `board_size` is created, and then each
box in this tile is replaced by `board_scale ** 2` boxes.

For example, an `LBoard` with size 4 and scale 1 looks like:

![dcc_ltiler_cli --single --scale 1 --board_type LBoard --tile_type BoxTile 4 0](img/LBoard_4_1.svg)

while bumping the scale up to 2 results in:

![dcc_ltiler_cli --single --scale 2 --board_type LBoard --tile_type BoxTile 4 0](img/LBoard_4_2.svg)

### Counting tilings of an LBoard by LTiles

The following command counts the number of tilings of an LBoard of size 2 by LTile's of size 2,
with scale parameter `x`:

`dcc_tiler_cli --count --scale x --board_type LBoard --tile_type LTile 2 2`

This results in the following tiling counts as `x` varies:

| `x` | Tilings           |
|-----|-------------------|
| 1   | 1                 |
| 2   | 1                 |
| 3   | 4                 |
| 4   | 409               |
| 5   | 108388            |
| 6   | 104574902         |
| 7   | 608850350072      |
| 8   | 19464993703121249 |

which interestingly is an integer sequence not appearing in OEIS.

### Counting tilings of a TBoard by TTiles

The command here is:

`dcc_tiler_cli --count --scale x --board_type TBoard --tile_type TTile 1 1`

*Exercise:* Show that if `x > 1` and `x % 4 != 0` then there are no such tilings!

This results in the following tiling counts as `x` varies:

| `x` | Tilings   |
|-----|-----------|
| 1   | 1         |
| 4   | 54        |
| 8   | 655302180 |
| 12  | ?         |

### Counting tilings of an LBoard by TTiles

Many combinations are possible.  An example is:

`dcc_tiler_cli --count --scale 4 --board_type LBoard --tile_type TTile 3 1`

which counts 54 tilings.  An example of such a tiling is:

![dcc_tiler_cli --single --scale 4 --board_type LBoard --tile_type TTile 3 1](img/LBoard_3_4_TTile_1.svg)







 