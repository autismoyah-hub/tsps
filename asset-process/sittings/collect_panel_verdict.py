"""Collect a blind panel export directly from the owner's Downloads folder."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse
import json
from pathlib import Path
import re

parser = argparse.ArgumentParser()
parser.add_argument("asset")
parser.add_argument("--downloads", type=Path, default=Path.home() / "Downloads")
parser.add_argument("--out", type=Path, default=Path(_AP.get('VERDICTS_DIR')))
args = parser.parse_args()
if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", args.asset):
    parser.error("Expected a lowercase asset ID")
pattern = re.compile(re.escape(args.asset) + r"-verdict(?: \(\d+\))?\.json")
files = [p for p in args.downloads.iterdir() if pattern.fullmatch(p.name)]
if not files:
    parser.exit(1, "No saved verdict yet.\n")
records = []
for path in files:
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    record = {"asset": args.asset}
    if data.get("asset") != args.asset or data.get("preferred") not in {"A", "B", "Tie"}:
        parser.error(f"Incomplete or mismatched export in {path.name}")
    for stage in ("concept", "A", "B"):
        grade, reason = data.get(stage + "-grade"), data.get(stage + "-reason")
        if grade not in {"Rework", "Usable", "Strong"} or not isinstance(reason, str) or not reason.strip():
            parser.error(f"Missing grade or reason for {stage} in {path.name}")
        record.update({stage + "-grade": grade, stage + "-reason": reason})
    record["preferred"] = data["preferred"]
    records.append(record)
if any(record != records[0] for record in records):
    parser.error("Conflicting exports. Confirm which revision to record.")
target = args.out / (args.asset + "-blind-verdict.json")
args.out.mkdir(parents=True, exist_ok=True)
if target.exists():
    if json.loads(target.read_text(encoding="utf-8")) != records[0]:
        parser.error("[quoted words withheld]")
else:
    with target.open("x", encoding="utf-8") as stream:
        json.dump(records[0], stream, ensure_ascii=False, indent=2)
        stream.write("\n")
print(target.name)
