"""Encode ONLY issue84's original Phase A rigid-label diagnostic; no cache writes.

Namespace values are explicit inputs, never allocation/reservation claims. The
full declared action policy is retained even though the current stager rejects it.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct

import native_codec
import synthetic_expectations as contract
from stage_synthetic_cache import checked_path

KINDS = ('model', 'skeleton', 'frames', 'sequence', 'location')


def namespace(value):
    if type(value) is not dict or set(value) != set(KINDS):
        raise ValueError('namespace requires exactly five kinds')
    if any(type(v) is not int or not 1 <= v <= 65534 for v in value.values()):
        raise ValueError('namespace requires integers1..65534 for this probe')
    return value


def write_json(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as f:
        json.dump(value, f, indent=2, sort_keys=True, allow_nan=False)
        f.write('\n')


def encode(observation, ids, output):
    """Validate everything before exclusive creation; retain partials on I/O failure."""
    contract.validate(observation)
    ids = namespace(ids)
    output = checked_path(output, missing=True)
    if output.exists() or not output.parent.is_dir():
        raise ValueError('output must be absent with existing parent')
    geometry = native_codec.model(observation['model'])
    action = copy.deepcopy(observation['action'])
    action.update(skeleton_id=ids['skeleton'], frame_group=ids['frames'])
    output.mkdir()
    write_json(output / 'partial.json', {'status': 'incomplete', 'namespace': ids})
    with (output / 'model.bin').open('xb') as f:
        f.write(geometry)
    # classic compiles all channels before writes; fresh owner directory only.
    native_codec.classic(action, output)
    location = b'\x07\x01' + struct.pack('>I', ids['model']) + b'\x18' + struct.pack('>H', ids['sequence']) + b'\x59\0'
    with (output / 'location.bin').open('xb') as f:
        f.write(location)
    names = dict(model='model.bin', skeleton=f"skeleton-{ids['skeleton']}.bin",
                 frame0='frame-0.bin', frame1='frame-1.bin', frame2='frame-2.bin',
                 sequence='sequence.bin', location='location.bin')
    inputs = {kind: {'file': name, 'sha256': hashlib.sha256((output / name).read_bytes()).hexdigest()}
              for kind, name in names.items()}
    write_json(output / 'allocation.json', {'schema': 1, 'namespace': ids, 'inputs': inputs})
    write_json(output / 'observation.json', observation)
    result = {'status': 'encoded only; native decoder not run', 'namespace_reserved': False,
              'cache_admitted': False, 'native_rendered': False, 'inputs': inputs}
    write_json(output / 'complete.json', result)
    return result


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--observation', required=True, type=Path)
    p.add_argument('--namespace', required=True, type=Path)
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args(argv)
    source = checked_path(a.observation)
    ids_path = checked_path(a.namespace)
    target = checked_path(a.output, missing=True)
    if target in source.parents or target in ids_path.parents:
        raise ValueError('output/source overlap')
    result = encode(contract.load(source), contract.load(ids_path), target)
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
