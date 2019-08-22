# This script is used to generate the example images located in the img/ directory
# based on the README.md file

import re
import os

# This regex matches expressions of the form:
#  ![dcc_tiler_cli --single --scale 4 --board_type LBoard --tile_type TTile 3 1](img/LBoard_3_4_TTile_1.svg)
RE = re.compile(r"!\[dcc_tiler_cli --single (.*?)\]\(img\/(.*?)\.svg\)")

# This forms the base for the command that we eventually run
command = "cargo run --release -- --single {} > img/{}.svg"

with open("README.md", "r") as readme:
    for line in readme.readlines():
        # find all matches as above
        for (options, filename) in RE.findall(line):
            os.system(command.format(options, filename))
