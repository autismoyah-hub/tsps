"""Bounded numeric terrain observation audit; never launches a native client.

Adapted method: tools/revonx/audit_samples.py at
f3050eb2cf27396533efb6244d9de2851aa4c374,
"Author RevonX settlement and admit furnished native terrain atlas".
Original source blob: 806206e7576a8e479e142f255807f1299e401137.
Copyright (c) 2025 RS Mod
Copyright (c) 2026 Project Gielinor Rust contributors
Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.
THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
"""

import argparse
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import sys
import tempfile

MAX_BYTES = 1 << 20
OFFSETS = ((32, 32), (32, 96), (64, 64), (96, 32), (96, 96))
CORNER_FIELDS = ("local_x", "local_z", "world_x", "world_z",
                 "ground_height", "upper_height", "upper_flags")
SAMPLE_FIELDS = ("x", "z", "fx", "fz", "elevation")


class AuditError(ValueError):
    """Invalid or incomplete evidence, as distinct from measured mismatch."""


def integer(value, low=-(1 << 31), high=(1 << 31) - 1):
    if type(value) is not int or not low <= value <= high:
        raise AuditError("expected bounded integer")
    return value


def exact_keys(value, keys):
    if type(value) is not dict or set(value) != set(keys):
        raise AuditError("unexpected or missing source fields")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise AuditError("duplicate JSON key")
        result[key] = value
    return result


def read_input(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file():
        raise AuditError("input must be a regular, non-symlink file")
    with path.open("rb") as stream:
        data = stream.read(MAX_BYTES + 1)
    if len(data) > MAX_BYTES:
        raise AuditError("input exceeds 1 MiB")
    return data


def source_grid(data):
    source = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object)
    exact_keys(source, ("schema", "origin", "size", "surface", "heights", "bridges"))
    if source["schema"] != "terrain-audit-v1" or source["surface"] != "paint-se-nw":
        raise AuditError("unsupported schema or surface (only plain paint)")
    for key in ("origin", "size"):
        if type(source[key]) is not list or len(source[key]) != 2:
            raise AuditError("origin and size must be integer pairs")
    ox, oz = map(integer, source["origin"])
    width, depth = (integer(v, 1, 64) for v in source["size"])
    integer(ox + width)
    integer(oz + depth)
    heights = source["heights"]
    if type(heights) is not list or len(heights) != (width + 1) * (depth + 1):
        raise AuditError("incomplete x-major corner grid")
    for value in heights:
        if integer(value, 0, 2040) % 8:
            raise AuditError("heights must be multiples of eight")
    if type(source["bridges"]) is not list or len(source["bridges"]) > width * depth:
        raise AuditError("invalid bridge list")
    bridges, deck_corners = {}, {}
    for bridge in source["bridges"]:
        exact_keys(bridge, ("x", "z", "height"))
        x, z = integer(bridge["x"], 0, width - 1), integer(bridge["z"], 0, depth - 1)
        height = integer(bridge["height"], 0, 2040)
        if height % 8 or (x, z) in bridges:
            raise AuditError("duplicate bridge or invalid flat deck height")
        bridges[x, z] = height
        for dx, dz in ((0, 0), (0, 1), (1, 0), (1, 1)):
            key = (x + dx, z + dz)
            if height < heights[key[0] * (depth + 1) + key[1]]:
                raise AuditError("deck below ground")
            if key in deck_corners and deck_corners[key] != height:
                raise AuditError("inconsistent shared flat deck corner")
            deck_corners[key] = height
    return source, bridges, deck_corners


def csv_rows(data, required, optional=()):
    # DictReader otherwise accepts duplicate columns and silently drops data.
    reader = csv.DictReader(io.StringIO(data.decode("utf-8"), newline=""), strict=True)
    names = reader.fieldnames
    if (not names or len(set(names)) != len(names) or
            not set(required) <= set(names) or set(names) - set(required) - set(optional)):
        raise AuditError("unexpected, duplicate or missing CSV columns")
    for row in reader:
        if None in row or any(v is None for v in row.values()):
            raise AuditError("CSV row width mismatch")
        parsed = {}
        for key, value in row.items():
            if len(value) > 11 or re.fullmatch(r"-?[0-9]+", value) is None:
                raise AuditError("CSV fields must be bounded decimal integers")
            parsed[key] = integer(int(value))
        yield parsed


def triangle_elevation(heights, fx, fz):
    """SW/SE/NE/NW above-datum values -> signed native Y, truncating toward zero."""
    sw, se, ne, nw = heights
    if fx + fz <= 128:
        numerator = sw * (128 - fx - fz) + se * fx + nw * fz
    else:
        numerator = ne * (fx + fz - 128) + nw * (128 - fx) + se * (128 - fz)
    return -(numerator // 128)


def audit(source_bytes, corner_bytes, sample_bytes):
    source, bridges, decks = source_grid(source_bytes)
    width, depth = source["size"]
    ox, oz = source["origin"]
    heights = source["heights"]
    mismatches, seen_corners, seen_samples = [], set(), set()

    def compare(kind, key, actual, expected, tolerance=0):
        difference = abs(actual - expected)
        if difference > tolerance:
            mismatches.append(dict(kind=kind, key=list(key), expected=expected,
                                   actual=actual, difference=difference))
        return difference

    for row in csv_rows(corner_bytes, CORNER_FIELDS):
        x, z = row["local_x"], row["local_z"]
        key = (x, z)
        if not (0 <= x <= width and 0 <= z <= depth) or key in seen_corners:
            raise AuditError("duplicate or out-of-range corner")
        if (row["world_x"], row["world_z"]) != (ox + x, oz + z):
            raise AuditError("corner origin/axis mismatch")
        seen_corners.add(key)
        compare("ground_corner", key, row["ground_height"], heights[x * (depth + 1) + z])
        if key in decks:
            compare("deck_corner", key, row["upper_height"], decks[key])
        compare("upper_flags", key, row["upper_flags"], 2 if key in bridges else 0)
    if len(seen_corners) != (width + 1) * (depth + 1):
        raise AuditError("missing corner observations")

    maximum = 0
    for row in csv_rows(sample_bytes, SAMPLE_FIELDS, ("collision",)):
        x, z = row["x"] - ox, row["z"] - oz
        fx, fz = row["fx"], row["fz"]
        key = (x, z, fx, fz)
        if not (0 <= x < width and 0 <= z < depth) or (fx, fz) not in OFFSETS or key in seen_samples:
            raise AuditError("duplicate or unexpected surface sample")
        seen_samples.add(key)
        if (x, z) in bridges:
            expected = -bridges[x, z]
        else:
            corners = [heights[xx * (depth + 1) + zz] for xx, zz in
                       ((x, z), (x + 1, z), (x + 1, z + 1), (x, z + 1))]
            expected = triangle_elevation(corners, fx, fz)
        maximum = max(maximum, compare("surface", key, row["elevation"], expected, 1))
    if len(seen_samples) != width * depth * len(OFFSETS):
        raise AuditError("missing surface observations")
    mismatches.sort(key=lambda item: (item["kind"], item["key"]))
    return dict(schema="terrain-audit-report-v1", status="mismatch" if mismatches else "matched",
                scope="Numeric observations only; native execution and collision not established",
                input_sha256={name: hashlib.sha256(data).hexdigest() for name, data in
                              (("source", source_bytes), ("corners", corner_bytes), ("samples", sample_bytes))},
                origin=source["origin"], size=source["size"], surface_tolerance=1,
                counts=dict(ground_corners=len(seen_corners), deck_corners=len(decks),
                            flag_observations=len(seen_corners), surfaces=len(seen_samples)),
                max_surface_difference=maximum, mismatches=mismatches)


def write_report(path, report):
    """Publish complete bytes without replacing an existing output (including symlinks)."""
    path = Path(path)
    data = (json.dumps(report, sort_keys=True, indent=2) + "\n").encode("utf-8")
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".terrain-audit-", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, path)  # Atomic no-replace publication; fail closed if unsupported.
    finally:
        if temporary is not None:
            temporary.unlink()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for name in ("source", "corners", "samples", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args(argv)
    try:
        report = audit(*(read_input(path) for path in (args.source, args.corners, args.samples)))
        write_report(args.output, report)
    except (OSError, ValueError, UnicodeError, csv.Error, RecursionError) as error:
        print("terrain audit invalid: " + str(error), file=sys.stderr)
        return 2
    print("terrain audit " + report["status"] + ": " +
          str(len(report["mismatches"])) + " mismatches; numeric observations only")
    return 1 if report["mismatches"] else 0


if __name__ == "__main__":
    sys.exit(main())
