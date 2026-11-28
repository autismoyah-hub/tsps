"""Author-safe RS3 rebuild sheet on the tools-hd-gate model: pictures and aggregate numbers, no vertex lists.

usage: rs3sheet.py KIND ID OUTDIR      KIND: item (expands to ground + worn forms), item-ground, item-worn,
                                        item-female, npc, model
Material numbers on the sheet are build-951 material ids; swatches are their diffuse images.
"""
import sys, argparse, json
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import rs3asset
import hdsheet
rs3asset.install(hdsheet)

if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('kind', choices=['item'] + rs3asset.KINDS)
    ap.add_argument('id', type=int)
    ap.add_argument('out', type=Path)
    a = ap.parse_args()
    if a.kind == 'item':
        import rs3mesh
        for k in ('item-ground', 'item-worn', 'item-female'):
            try:
                if rs3mesh.definition(k if k != 'item-ground' else 'item', a.id)['models']:
                    hdsheet.make(rs3asset.REV, k, a.id, a.out/k)
            except KeyError:
                pass
    else:
        hdsheet.make(rs3asset.REV, a.kind, a.id, a.out)
