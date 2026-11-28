"""Synthetic observation fixtures; no cache, native process, art or stock map inputs."""
import copy
import csv
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("terrain_audit", Path(__file__).with_name("terrain_audit.py"))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def encode_csv(fields, rows):
    stream = io.StringIO(newline="")
    writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return stream.getvalue().encode()


def fixture():
    source = dict(schema="terrain-audit-v1", origin=[100, -200], size=[2, 2],
                  surface="paint-se-nw", heights=[0, 128, 128, 0, 0, 0, 0, 0, 0],
                  bridges=[dict(x=1, z=1, height=160)])
    # Ground is x-major. The deck is flat; linked-cell flag is set only at (1,1).
    corners = []
    for x in range(3):
        for z in range(3):
            corners.append(dict(local_x=x, local_z=z, world_x=100 + x, world_z=-200 + z,
                                ground_height=source["heights"][x * 3 + z],
                                upper_height=160 if x >= 1 and z >= 1 else 240,
                                upper_flags=2 if (x, z) == (1, 1) else 0))
    # Independently hand-calculated values: NW-only hill, west-high slope,
    # zero plane, then a constant deck. Do not call the implementation for expectations.
    expected = { (0, 0): [-32, -96, -64, -32, -32],
                 (0, 1): [-96, -96, -64, -32, -32],
                 (1, 0): [0, 0, 0, 0, 0],
                 (1, 1): [-160, -160, -160, -160, -160] }
    points = [(32, 32), (32, 96), (64, 64), (96, 32), (96, 96)]
    samples = [dict(x=100 + x, z=-200 + z, fx=fx, fz=fz, elevation=value)
               for (x, z), values in expected.items()
               for (fx, fz), value in zip(points, values)]
    return source, corners, samples


def observations(source, corners, samples):
    return (json.dumps(source).encode(), encode_csv(audit.CORNER_FIELDS, corners),
            encode_csv(audit.SAMPLE_FIELDS, samples))


class TerrainAuditTests(unittest.TestCase):
    def test_complete_asymmetric_both_halves_diagonal_and_flat_bridge(self):
        report = audit.audit(*observations(*fixture()))
        self.assertEqual(report["status"], "matched")
        self.assertEqual(report["counts"], dict(ground_corners=9, deck_corners=4,
                                                flag_observations=9, surfaces=20))
        self.assertEqual(report["max_surface_difference"], 0)
        self.assertEqual(report["mismatches"], [])

    def test_bilinear_value_is_rejected_as_surface_observation(self):
        source, corners, samples = fixture()
        samples[0]["elevation"] = -24  # Bilinear NW-only hill at quarter tile, not -32.
        report = audit.audit(*observations(source, corners, samples))
        self.assertEqual(report["status"], "mismatch")
        self.assertEqual(report["mismatches"], [dict(kind="surface", key=[0, 0, 32, 32],
                                                     expected=-32, actual=-24, difference=8)])

    def test_surface_tolerance_one_unit_but_corners_exact(self):
        source, corners, samples = fixture()
        samples[0]["elevation"] = -31
        samples[-1]["elevation"] = -161
        self.assertEqual(audit.audit(*observations(source, corners, samples))["status"], "matched")
        samples[0]["elevation"] = -30
        self.assertEqual(audit.audit(*observations(source, corners, samples))["status"], "mismatch")
        samples[0]["elevation"] = -32
        corners[0]["ground_height"] = 1
        self.assertEqual(audit.audit(*observations(source, corners, samples))["mismatches"][0]["kind"],
                         "ground_corner")

    def test_non_square_grid_without_bridge_and_parameterized_origin(self):
        source = dict(schema="terrain-audit-v1", origin=[-30, 70], size=[1, 2],
                      surface="paint-se-nw", heights=[64] * 6, bridges=[])
        corners = [dict(local_x=x, local_z=z, world_x=-30 + x, world_z=70 + z,
                        ground_height=64, upper_height=304, upper_flags=0)
                   for x in range(2) for z in range(3)]
        samples = [dict(x=-30, z=70 + z, fx=fx, fz=fz, elevation=-64)
                   for z in range(2) for fx, fz in audit.OFFSETS]
        report = audit.audit(*observations(source, corners, samples))
        self.assertEqual(report["status"], "matched")
        self.assertEqual(report["counts"]["surfaces"], 10)

    def test_missing_duplicate_and_extra_evidence_never_pass(self):
        original = fixture()
        for table_index in (1, 2):
            for mutation in ("empty", "missing", "duplicate", "extra"):
                with self.subTest(table=table_index, mutation=mutation):
                    inputs = copy.deepcopy(original)
                    rows = inputs[table_index]
                    if mutation == "empty":
                        rows.clear()
                    elif mutation == "missing":
                        rows.pop()
                    elif mutation == "duplicate":
                        rows.append(copy.deepcopy(rows[0]))
                    else:
                        rows.append(copy.deepcopy(rows[0]))
                        rows[-1]["local_x" if table_index == 1 else "x"] = 999
                    with self.assertRaises(audit.AuditError):
                        audit.audit(*observations(*inputs))

    def test_wrong_origin_axis_offset_and_bridge_observations(self):
        source, corners, samples = fixture()
        corners[0]["world_x"] += 1
        with self.assertRaises(audit.AuditError):
            audit.audit(*observations(source, corners, samples))
        source, corners, samples = fixture()
        samples[0]["fx"] = 128
        with self.assertRaises(audit.AuditError):
            audit.audit(*observations(source, corners, samples))
        source, corners, samples = fixture()
        corners[-1]["upper_height"] = 168
        corners[4]["upper_flags"] = 0
        report = audit.audit(*observations(source, corners, samples))
        self.assertEqual({m["kind"] for m in report["mismatches"]}, {"deck_corner", "upper_flags"})

    def test_source_corner_tamper_compares_source_and_samples(self):
        source, corners, samples = fixture()
        source["heights"][1] = 136
        report = audit.audit(*observations(source, corners, samples))
        self.assertEqual(report["status"], "mismatch")
        self.assertTrue(any(m["kind"] == "ground_corner" for m in report["mismatches"]))
        self.assertTrue(any(m["kind"] == "surface" for m in report["mismatches"]))

    def test_invalid_schema_dimensions_and_heights(self):
        mutations = (("schema", "other"), ("surface", "bilinear"), ("size", [0, 2]),
                     ("size", [65, 2]), ("size", [True, 2]), ("origin", [1.0, 2]),
                     ("origin", [2147483647, 0]), ("heights", []),
                     ("heights", [7] * 9), ("heights", [2048] * 9),
                     ("heights", [False] * 9), ("bridges", None))
        for key, value in mutations:
            with self.subTest(key=key, value=value):
                source, corners, samples = fixture()
                source[key] = value
                with self.assertRaises(audit.AuditError):
                    audit.audit(*observations(source, corners, samples))

    def test_reject_sloped_duplicate_below_ground_or_conflicting_decks(self):
        invalid = [[dict(x=1, z=1, heights=[160, 160, 168, 160])],
                   [dict(x=1, z=1, height=160)] * 2,
                   [dict(x=0, z=0, height=64)],
                   [dict(x=1, z=0, height=160), dict(x=1, z=1, height=168)]]
        for bridges in invalid:
            with self.subTest(bridges=bridges):
                source, corners, samples = fixture()
                source["bridges"] = bridges
                with self.assertRaises(audit.AuditError):
                    audit.audit(*observations(source, corners, samples))

    def test_malformed_json_and_csv(self):
        source, corners, samples = observations(*fixture())
        for data in (b'{}', b'[]', b'{', source.replace(b'"schema":', b'"schema": 1, "schema":')):
            with self.subTest(data=data[:20]), self.assertRaises(ValueError):
                audit.audit(data, corners, samples)
        malformed = (b'', corners.replace(b'local_x', b'world_x', 1),
                     corners.replace(b'100,', b'100.5,', 1),
                     corners + b'0,0,100,-200,0,240\n', corners + b'0,0,100,-200,0,240,0,7\n',
                     corners.replace(b'100,', b'"unterminated,', 1))
        for data in malformed:
            with self.subTest(data=data[:30]), self.assertRaises((ValueError, csv.Error)):
                audit.audit(source, data, samples)

    def test_optional_collision_is_parsed_but_not_certified(self):
        source, corners, samples = fixture()
        for row in samples:
            row["collision"] = 123
        report = audit.audit(json.dumps(source).encode(), encode_csv(audit.CORNER_FIELDS, corners),
                             encode_csv((*audit.SAMPLE_FIELDS, "collision"), samples))
        self.assertEqual(report["status"], "matched")
        self.assertIn("collision not established", report["scope"])

    def test_deterministic_report_preserves_inputs_and_refuses_overwrite(self):
        inputs = observations(*fixture())
        before = copy.deepcopy(inputs)
        with tempfile.TemporaryDirectory() as folder:
            first, second = Path(folder) / "first.json", Path(folder) / "second.json"
            audit.write_report(first, audit.audit(*inputs))
            audit.write_report(second, audit.audit(*inputs))
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with self.assertRaises(FileExistsError):
                audit.write_report(first, {"wrong": True})
            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(sorted(p.name for p in Path(folder).iterdir()), ["first.json", "second.json"])
        self.assertEqual(inputs, before)

    def test_bounded_file_read(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "oversized.csv"
            path.write_bytes(b"x" * (audit.MAX_BYTES + 1))
            with self.assertRaises(audit.AuditError):
                audit.read_input(path)

    def test_actual_cli_success_mismatch_invalid_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            paths = [root / n for n in ("source.json", "corners.csv", "samples.csv")]
            for path, data in zip(paths, observations(*fixture())):
                path.write_bytes(data)
            before = [p.read_bytes() for p in paths]
            output = root / "audit.json"
            args = [sys.executable, str(Path(audit.__file__)), "--source", str(paths[0]),
                    "--corners", str(paths[1]), "--samples", str(paths[2]), "--output", str(output)]

            def call():
                return subprocess.run(args, capture_output=True, text=True)

            result = call()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("terrain audit matched: 0 mismatches", result.stdout)
            report = output.read_bytes()
            self.assertEqual(call().returncode, 2)
            self.assertEqual(output.read_bytes(), report)
            self.assertEqual([p.read_bytes() for p in paths], before)
            output.unlink()
            source, corners, samples = fixture()
            samples[0]["elevation"] = -24
            paths[2].write_bytes(observations(source, corners, samples)[2])
            self.assertEqual(call().returncode, 1)
            self.assertEqual(json.loads(output.read_bytes())["status"], "mismatch")
            output.unlink()
            paths[1].write_bytes(b"local_x,local_z\n")
            self.assertEqual(call().returncode, 2)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
