#!/usr/bin/env python3.8

"""
In order for the solutions to pick up this module, the folder
containing this file must be in the PYTHONPATH.

The .env file in this folder adds it.
"""

import sys
import os
import glob

sys.path.append(
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)), os.pardir, "common", "Python"
    )
)

from aoc import *

__IMPORTED = False
if __name__ == "__main__" and not __IMPORTED:
    __IMPORTED = True
    day = f"{int(sys.argv[1]):02}"
    day_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), day)
    env_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".env")
    script = glob.glob(f"{day_dir}/*.py")[0]
    print(f"Running `{os.path.basename(script)}` for day {day}:")

    year_dir = os.path.dirname(os.path.abspath(__file__))
    # sys.path.append(year_dir)

    os.system(f"""export PYTHONPATH="${{PYTHON_PATH}}:{year_dir}"; {script}""")
