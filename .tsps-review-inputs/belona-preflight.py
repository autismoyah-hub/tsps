#!/usr/bin/env python3
"""Read-only Belona source/map preflight; never selects or contacts a provider."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
SOURCES = {'before': '247ed387d6a631a00440904a00aa8e1a2e44a962', 'after': '88b5e5793039c1f246c2a4096206664e50c51003'}
MANIFEST_HASH = {'before': 'a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305', 'after': 'f2797ca4c62cd14e4c039f766d17163d49f4a28ed1548e3af87616aafa4e7206'}
PLAN_HASH = {'before': 'e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf', 'after': '7b141df4d1b9966654795c6fcff01efc11c0c153bbf7806519df1ca60560157e'}

def checked_file(name, digest):
    data = (ROOT / name).read_bytes()
    if hashlib.sha256(data).hexdigest() != digest:
        raise ValueError(f'wrong hash: {name}')
    return json.loads(data)

def git(*args):
    return subprocess.check_output(['git', *args], stderr=subprocess.PIPE)

def main():
    inputs = {}
    for label, source in SOURCES.items():
        manifest = checked_file(label + '-manifest.json', MANIFEST_HASH[label])
        plan = checked_file(label + '-held-plan.json', PLAN_HASH[label])
        if manifest['source'] != source or plan['source'] != source or plan['executionHold'] != 'HELD_NOT_SELECTED_CASE_PROFILE_REVIEW_REQUIRED':
            raise ValueError('source/hold mismatch')
        if plan['timeout'] != 1800 or plan['attempts'] != 1 or plan['cacheId'] != 2746:
            raise ValueError('runtime profile mismatch')
        expected = manifest['expectedInputs']
        blobs = manifest['sourceBlobs']
        if set(expected) != set(blobs) or len(blobs) != (4912 if label == 'before' else 4913):
            raise ValueError('manifest keyset mismatch')
        tree = {}
        for row in git('ls-tree', '-rz', '-r', '--full-tree', source).split(b'\0'):
            if not row:
                continue
            meta, name = row.split(b'\t', 1)
            mode, kind, oid = meta.decode().split()
            name = name.decode()
            if name.startswith('server/') or name in ('.yarnrc.yml', 'package.json', 'yarn.lock', 'scripts/run-with-env.mjs'):
                if kind != 'blob' or mode not in ('100644', '100755'):
                    raise ValueError('unexpected source mode')
                tree[name] = oid
        if tree != blobs:
            raise ValueError('git tree/manifest mismatch')
        # Recompute Git blob and SHA256 values without unpacking or writing product files.
        names = list(sorted(blobs))
        process = subprocess.run(['git', 'cat-file', '--batch'], input=('\n'.join(blobs[n] for n in names)+'\n').encode(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        data, offset = process.stdout, 0
        for name in names:
            end = data.index(b'\n', offset)
            oid, kind, size = data[offset:end].decode().split()
            offset = end + 1
            body = data[offset:offset+int(size)]
            offset += int(size)
            if data[offset:offset+1] != b'\n': raise ValueError('batch framing')
            offset += 1
            if oid != blobs[name] or kind != 'blob' or hashlib.sha256(body).hexdigest() != expected[name]:
                raise ValueError('blob mismatch: ' + name)
        if offset != len(data): raise ValueError('batch trailing data')
        inputs[label] = blobs
    delta = sorted(set(inputs['after']) ^ set(inputs['before']))
    if delta != ['server/plugins/npcs/Belona.plugin.js'] or any(inputs['before'][p] != inputs['after'][p] for p in inputs['before']):
        raise ValueError('before/after not plugin-only')
    if inputs['after']['server/data/definitions/shops.json'] != '939ecfa4c83e15145c125e1264fadc679528829f':
        raise ValueError('generated output mismatch')
    print(json.dumps({'stage':'SOURCE_PREFLIGHT_ONLY','sources':SOURCES,'rows':{k:len(v) for k,v in inputs.items()},'selection':False,'providerContact':False},sort_keys=True))

if __name__ == '__main__':
    main()
