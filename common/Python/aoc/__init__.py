from typing import Any, Tuple

AOC_REBUILD_ENV_VAR: str = "AOC_REBUILD"


def _get_aoc_so(release_build: bool = True) -> str:
    from glob import glob
    from os.path import dirname, abspath, join, exists
    from os import pardir, environ
    from functools import reduce

    mod_path = dirname(abspath(__file__))
    up = lambda p: abspath(join(p, pardir))
    applyN = lambda func, start, N: reduce(lambda acc, _i: func(acc), range(N), start)

    crate = join(up(mod_path), "aoc-rs-bindings")
    build_type = "release" if release_build else "debug"
    target_dir = join(join(applyN(up, mod_path, 3), "target"), build_type)

    find_so = lambda: glob(f"{target_dir}/libaoc_rs.[a-z][a-z]*")

    # dylib, dll, or so
    if (
        exists(target_dir)
        and (path := find_so())
        and AOC_REBUILD_ENV_VAR not in environ
    ):
        return path[0]
    else:
        import subprocess

        p = subprocess.Popen(
            ["cargo", "build"] + ["--release"] if release_build else [], cwd=crate
        )
        p.wait()
        assert p.returncode == 0, "Failed to build `aoc-rs-bindings`."

        p = find_so()
        assert len(p) == 1, "Cannot find `libaoc_rs`."

        return p[0]


def _get_aoc_module():
    import importlib.util

    so = _get_aoc_so()
    if so.endswith(".dylib"):
        import pathlib, os

        renamed = pathlib.Path(so).with_suffix(".so")
        if not renamed.exists():
            os.symlink(so, renamed)

        so = renamed

    spec = importlib.util.spec_from_file_location("aoc_rs", so)
    aoc = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(aoc)

    return aoc


aoc_rs = _get_aoc_module()
from aoc_rs import Aoc


def _infer_year_and_day() -> Tuple[int, int]:
    from os.path import dirname, abspath, normpath
    from os import sep
    import sys

    exec_path = sys.argv[0]
    p = normpath(dirname(abspath(exec_path))).split(sep)
    day = p[-1]
    year = p[-2]

    if len(year) != 4 or len(day) != 2:
        raise Exception(f"Could not infer year and day from `{exec_path}`.")

    return (int(year), int(day))


def inp() -> str:
    """
    Infers the Year and Day from the file name of the file being
    run and grabs the input for it.
    """
    return Aoc(*_infer_year_and_day()).get_input().strip()


def p1(ans: Any) -> bool:
    """
    Infers the Year and Day from the file name of the file being
    run and submits P1 for it.
    """
    return Aoc(*_infer_year_and_day()).submit_p1(ans)


def p2(ans: Any) -> bool:
    """
    Infers the Year and Day from the file name of the file being
    run and submits P2 for it.
    """
    return Aoc(*_infer_year_and_day()).submit_p2(ans)


import itertools
from itertools import combinations, permutations
import functools
from functools import reduce as red
from math import prod
from typing import Any, Tuple, Generator, TypeVar, List

T = TypeVar("T")


def count(g: Generator[T, None, None]) -> int:
    return sum(1 for _ in g)
