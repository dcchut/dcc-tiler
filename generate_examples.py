# This script is used to generate the example images located in the img/ directory
# This is done by scanning the README.md file for images

import re
import os

# This regex matches expressions of the form:
RE = re.compile(r"!\[dcc_tiler_cli --single (.*?)\]\(img\/(.*?)\.svg\)")

# This forms the base for the command that we eventually run
command = "cargo run --release -- --single {} > img/{}.svg"

with open("README.md", "r") as readme:
    for line in readme.readlines():
        # find all matches as above
        for (options, filename) in RE.findall(line):
            os.system(command.format(options, filename))
