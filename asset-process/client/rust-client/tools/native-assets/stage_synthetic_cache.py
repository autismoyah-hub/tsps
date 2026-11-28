"""Bounded private synthetic-cache staging; stdlib only, plan is the default.

No native decoder/admission claim. No map/archive-5 injection. Three independent
copies are made ONLY with --stage and a separately authorized copy grant.
Source revision policy defaults to strict; server-low16 is an explicit CRC- and
header-checked opt-in. Output defaults to full32; server-low16 explicitly emits
two-byte trailers while retaining full u32 metadata versions. Neither policy
changes the native client's JavaStrict reader or grants cache admission.
Default plan also reads/decodes source references and config groups: authentic
plan use needs the separate authentic-read/collision grant, not just copy GO.
Authored Content-package/floor namespace collisions are OUT of this offline
tool's scope. Separate owner collision/admission approval is required before
authentic staging; no cache-only result allocates or admits any namespace.
Changed references are re-encoded uncompressed (type 0); existing names survive,
but new group/file name hashes are 0 (ID-only lookup, no promised name lookup).
Unsupported flags, zero-file groups, suffix-less groups and reference trailers
are refused; no unsupported semantics are silently normalized. Later maintained
Rust cross-read and client serving/admission remain separate unproved gates.
Cache file bytes are fsynced before completion; directories are fsynced where
stdlib supports it. Windows directory/crash durability remains OS-dependent.
Allocation JSON schema 1: namespace {model,skeleton,frames,sequence,location},
inputs {model,skeleton,frame0,frame1,frame2,sequence,location}, each input
{file: simple basename, sha256: lowercase hex}. Explicit synthetic allocations
in tests confer no reservation in a real cache. Plan reads references directly.

Format evidence: maintained rust-client/src/{cache,disk_cache}.rs; historical
stage_live_cache.rs was reviewed as source only (not executed or imported).
This is independently authored tooling, not a copy of that historical helper.
"""

import argparse
import bz2
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import struct
import sys
import zlib

U24 = 0xFFFFFF
MAX_STORED = U24
MAX_DECODED = 64 * 1024 * 1024
MAX_INDEX = 16 * 1024 * 1024
MAX_REFERENCE = 2 * 1024 * 1024
MAX_REFERENCE_ENTRIES = 1000000
MAX_FILES = 65535
MAX_INPUT = 1024 * 1024
COPY_CAP = 1024 * 1024 * 1024
GROWTH_CAP = 16 * 1024 * 1024
DISK_FLOOR = 20 * 1024 * 1024 * 1024
KINDS = ('model', 'skeleton', 'frames', 'sequence', 'location')
INPUTS = ('model', 'skeleton', 'frame0', 'frame1', 'frame2', 'sequence', 'location')


def require(ok, why):
    if not ok:
        raise ValueError(why)


def integer(value, maximum, name):
    require(type(value) is int and 0 <= value <= maximum, name + ' out of bounds')
    return value


def u24(value):
    return integer(value, U24, 'u24').to_bytes(3, 'big')


def crc(data):
    return zlib.crc32(data) & 0xFFFFFFFF


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def checked_path(path, *, missing=False, directory=False):
    """Reject all symlink/reparse ancestors, including a missing target's parent."""
    path = Path(os.path.abspath(path))
    for item in (path, *path.parents):
        try:
            info = item.lstat()
        except FileNotFoundError:
            require(missing, 'missing path: ' + str(item))
            continue
        require(not stat.S_ISLNK(info.st_mode) and
                not getattr(info, 'st_file_attributes', 0) & 0x400,
                'symlink/reparse path: ' + str(item))
    if not missing:
        info = path.stat()
        require(stat.S_ISDIR(info.st_mode) if directory else stat.S_ISREG(info.st_mode),
                'wrong path type: ' + str(path))
        if not directory:
            require(info.st_nlink == 1, 'hard-linked source refused')
    return path.resolve()


def related(a, b):
    a, b = os.path.normcase(str(a)), os.path.normcase(str(b))
    return os.path.commonpath((a, b)) in (a, b) if Path(a).drive == Path(b).drive else False


def read_small(path, limit):
    path = checked_path(path)
    require(path.stat().st_size <= limit, 'file length bound')
    with path.open('rb') as f:
        data = f.read(limit + 1)
    require(len(data) <= limit, 'file grew beyond bound')
    return data


class Reader:
    def __init__(self, data):
        self.data, self.pos = data, 0

    def number(self, n):
        require(self.pos + n <= len(self.data), 'truncated table')
        value = int.from_bytes(self.data[self.pos:self.pos + n], 'big')
        self.pos += n
        return value

    def count(self, protocol):
        if protocol < 7:
            return self.number(2)
        require(self.pos < len(self.data), 'truncated smart')
        return self.number(4) & 0x7FFFFFFF if self.data[self.pos] & 0x80 else self.number(2)

    def end(self):
        require(self.pos == len(self.data), 'table trailing bytes')


def decode_container(data, decoded_limit=MAX_DECODED):
    integer(decoded_limit, MAX_DECODED, 'decoded limit')
    require(5 <= len(data) <= MAX_STORED, 'stored container bound')
    kind, stored = data[0], int.from_bytes(data[1:5], 'big')
    require(kind in (0, 1, 2), 'unsupported compression/encryption')
    header = 5 if kind == 0 else 9
    require(stored <= MAX_STORED - header and len(data) == header + stored,
            'container declared length/trailing data')
    if kind == 0:
        require(stored <= decoded_limit, 'decoded length bound before allocation')
        return data[5:]
    require(len(data) >= 9, 'compressed header')
    expected = int.from_bytes(data[5:9], 'big')
    require(expected <= decoded_limit, 'decoded length bound before allocation')
    payload = data[9:]
    if kind == 2:
        obj = zlib.decompressobj(31)
        result = obj.decompress(payload, expected + 1)
        require(obj.eof and not obj.unused_data and not obj.unconsumed_tail,
                'gzip incomplete/trailing/overlong data')
    else:
        # JS5 bzip2 commonly omits BZh1. Both forms are explicitly bounded.
        obj = bz2.BZ2Decompressor()
        result = obj.decompress(payload if payload.startswith(b'BZh') else b'BZh1' + payload,
                                max_length=expected + 1)
        require(obj.eof and not obj.unused_data, 'bzip2 incomplete/trailing/overlong data')
    require(len(result) == expected, 'decompressed length mismatch')
    return result


def container(payload):
    require(len(payload) <= MAX_STORED - 9, 'new payload bound')
    return b'\0' + struct.pack('>I', len(payload)) + payload


def validate_group(raw, info):
    # Full-width first, only if BOTH CRC and revision match. No u16 truncation.
    for width in (4, 2):
        if len(raw) >= width + 5 and int.from_bytes(raw[-width:], 'big') == info['version']:
            body = raw[:-width]
            if crc(body) == info['crc']:
                return decode_container(body)
    raise ValueError('group CRC/full revision mismatch')


def revision_policies(source, output):
    require(source in ('strict', 'server-low16'), 'unknown source revision policy')
    require(output in ('full32', 'server-low16'), 'unknown output revision policy')
    return hashlib.sha256(json.dumps(dict(source=source, output=output),
                                     sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def validate_revision_container(raw, info, policy):
    """Explicit source/output representation check, never a client policy change."""
    require(policy in ('strict', 'server-low16'), 'unknown source revision policy')
    require(7 <= len(raw) <= MAX_STORED, 'versioned container bound')
    require(raw[0] in (0, 1, 2), 'unsupported compression/encryption')
    header = 5 if raw[0] == 0 else 9
    require(len(raw) >= header + 2, 'truncated versioned container header')
    end = header + int.from_bytes(raw[1:5], 'big')
    width = len(raw) - end
    require(end >= header and width in (2, 4), 'exact container/body/trailer accounting')
    version = integer(info['version'], 0xFFFFFFFF, 'full reference version')
    expected_crc = integer(info['crc'], 0xFFFFFFFF, 'reference CRC')
    stored = int.from_bytes(raw[end:], 'big')
    expected = version & 0xffff if width == 2 and policy == 'server-low16' else version
    require(stored == expected, 'group CRC/full revision mismatch' if policy == 'strict' else 'source revision policy mismatch')
    body = raw[:end]
    actual_crc = crc(body)
    require(actual_crc == expected_crc, 'group CRC/full revision mismatch' if policy == 'strict' else 'source container CRC mismatch')
    observation = dict(requested_source_policy=policy, trailer_bytes=width,
                       matched_rule='full32' if width == 4 else ('unsigned16-low16' if policy == 'server-low16' else 'unsigned16-exact'),
                       full_reference_version=version, stored_revision=stored,
                       reference_crc=expected_crc, actual_crc=actual_crc, container_bytes=len(body),
                       raw_sha256=hashlib.sha256(raw).hexdigest(), container_sha256=hashlib.sha256(body).hexdigest())
    return body, observation


def read_source_group(raw, info, policy='strict'):
    # Header accounting + CRC + selected revision rule precede decompression.
    body, observation = validate_revision_container(raw, info, policy)
    # The strict validator itself is deliberately unchanged. There is no
    # automatic retry under another policy after its refusal.
    packed = validate_group(raw, info) if policy == 'strict' else decode_container(body)
    observation['packed_bytes'] = len(packed)
    return packed, observation


def parse_reference(payload):
    require(len(payload) <= MAX_REFERENCE, 'reference bound')
    # Every group/file ID consumes at least two bytes in protocols5/6/7.
    entry_limit = min(len(payload) // 2, MAX_REFERENCE_ENTRIES)
    r = Reader(payload)
    protocol = r.number(1)
    require(protocol in (5, 6, 7), 'unsupported reference protocol')
    version = r.number(4) if protocol >= 6 else 0
    flags = r.number(1)
    require(flags in (0, 1, 4, 5), 'unsupported reference metadata flags')
    n = r.count(protocol)
    # At least one file per supported group: IDs, CRC/version, file count and
    # optional lengths/names must all fit before allocating group dictionaries.
    group_minimum = 14 + (8 if flags & 4 else 0) + (8 if flags & 1 else 0)
    require(n <= entry_limit and n <= (len(payload) - r.pos) // group_minimum, 'group count bound')
    groups, previous = {}, 0
    for i in range(n):
        delta = r.count(protocol)
        require(i == 0 or delta > 0, 'duplicate group')
        previous += delta
        integer(previous, (MAX_INDEX // 6) - 1, 'group/index id')
        groups[previous] = {'files': {}, 'crc': 0, 'version': 0, 'name': 0}
    if flags & 1:
        for g in groups.values():
            g['name'] = r.number(4)
    for g in groups.values():
        g['crc'] = r.number(4)
    if flags & 4:
        for g in groups.values():
            g['lengths'] = (r.number(4), r.number(4))
    for g in groups.values():
        g['version'] = r.number(4)
    counts = [r.count(protocol) for _ in groups]
    require(n + sum(counts) <= entry_limit and all(1 <= x <= MAX_FILES for x in counts),
            'file count bound')
    file_minimum = 6 if flags & 1 else 2
    require(sum(counts) <= (len(payload) - r.pos) // file_minimum, 'truncated file counts')
    for g, count in zip(groups.values(), counts):
        previous = 0
        for i in range(count):
            delta = r.count(protocol)
            require(i == 0 or delta > 0, 'duplicate file')
            previous += delta
            integer(previous, 0x7FFFFFFF if protocol == 7 else 65535, 'file id')
            g['files'][previous] = 0
    if flags & 1:
        for g in groups.values():
            for fid in g['files']:
                g['files'][fid] = r.number(4)
    r.end()
    return dict(protocol=protocol, version=version, flags=flags, groups=groups)


def encode_reference(ref):
    p = ref['protocol']
    require(p in (5, 6, 7), 'unsupported reference protocol')
    flags = integer(ref['flags'], 255, 'reference flags')
    require(flags in (0, 1, 4, 5), 'unsupported reference metadata flags')
    def count(n):
        integer(n, 0x7FFFFFFF if p == 7 else 65535, 'reference count/id delta')
        return struct.pack('>I', n | 0x80000000) if p == 7 and n >= 32768 else struct.pack('>H', n)
    groups = sorted(ref['groups'].items())
    require(len(groups) + sum(len(g['files']) for _, g in groups) <= MAX_REFERENCE_ENTRIES and
            all(1 <= len(g['files']) <= MAX_FILES for _, g in groups), 'encoded reference count bound')
    out = bytearray([p])
    if p >= 6:
        out += struct.pack('>I', integer(ref['version'], 0xFFFFFFFF, 'reference version'))
    out += bytes([flags]) + count(len(groups))
    previous = 0
    for gid, _ in groups:
        out += count(gid - previous)
        previous = gid
    if flags & 1:
        for _, g in groups:
            out += struct.pack('>I', g['name'])
    for _, g in groups:
        out += struct.pack('>I', g['crc'])
    if flags & 4:
        for _, g in groups:
            pair = g.get('lengths')
            require(type(pair) in (tuple, list) and len(pair) == 2, 'reference length pair')
            out += struct.pack('>II', *(integer(v, 0xFFFFFFFF, 'reference length') for v in pair))
    for _, g in groups:
        out += struct.pack('>I', g['version'])
    for _, g in groups:
        out += count(len(g['files']))
    for _, g in groups:
        previous = 0
        for fid in sorted(g['files']):
            out += count(fid - previous)
            previous = fid
    if flags & 1:
        for _, g in groups:
            for fid in sorted(g['files']):
                out += struct.pack('>I', g['files'][fid])
    require(len(out) <= MAX_REFERENCE, 'encoded reference bound')
    return bytes(out)


def split_files(data, ids):
    n = len(ids)
    require(1 <= n <= MAX_FILES and len(data) <= MAX_DECODED, 'split bounds')
    if n == 1:
        return {ids[0]: data}
    require(data and data[-1] > 0, 'missing chunk table')
    chunks = data[-1]
    start = len(data) - 1 - chunks * n * 4
    require(start >= 0, 'chunk table length')
    table = Reader(data[start:-1])
    sizes = [0] * n
    # First pass validates ALL lengths and total before allocating output buffers.
    for _ in range(chunks):
        running = 0
        for i in range(n):
            value = table.number(4)
            running += value if value < 0x80000000 else value - 0x100000000
            require(0 <= running <= start, 'negative/oversized chunk')
            sizes[i] += running
    require(sum(sizes) == start, 'chunk coverage mismatch')
    files = [bytearray() for _ in ids]
    table.pos, pos = 0, 0
    for _ in range(chunks):
        running = 0
        for i in range(n):
            value = table.number(4)
            running += value if value < 0x80000000 else value - 0x100000000
            files[i] += data[pos:pos + running]
            pos += running
    return dict(zip(ids, map(bytes, files)))


def join_files(files):
    require(1 <= len(files) <= MAX_FILES, 'join file count')
    values = [files[k] for k in sorted(files)]
    total = sum(map(len, values)) + (4 * len(values) + 1 if len(values) > 1 else 0)
    require(total <= MAX_STORED - 9, 'joined data bound before allocation')
    if len(values) == 1:
        return values[0]
    previous, table = 0, bytearray()
    for value in values:
        table += struct.pack('>i', len(value) - previous)
        previous = len(value)
    return b''.join(values) + table + b'\1'


def inventory(root):
    root = checked_path(root, directory=True)
    files = {}
    for p in root.iterdir():
        if p.name == 'main_file_cache.dat2' or re.fullmatch(r'main_file_cache\.idx(?:0|[1-9][0-9]{0,2})', p.name):
            if '.idx' in p.name:
                require(int(p.name.split('.idx')[1]) <= 255, 'archive id bound')
            checked_path(p)
            info = p.stat()
            require('.idx' not in p.name or info.st_size <= MAX_INDEX and info.st_size % 6 == 0,
                    'index length bound/alignment')
            files[p.name] = (info.st_size, info.st_mtime_ns, info.st_dev, info.st_ino)
    require('main_file_cache.dat2' in files, 'missing dat2')
    require(files['main_file_cache.dat2'][0] <= COPY_CAP, 'source dat2 bound')
    return dict(sorted(files.items()))


class Cache:
    def __init__(self, root):
        self.root = checked_path(root, directory=True)
        self.snapshot = inventory(self.root)

    def entry(self, archive, group):
        integer(archive, 255, 'archive')
        integer(group, MAX_INDEX // 6 - 1, 'index group')
        p = self.root / ('main_file_cache.idx' + str(archive))
        if p.name not in self.snapshot or (group + 1) * 6 > self.snapshot[p.name][0]:
            return None
        with p.open('rb') as f:
            f.seek(group * 6)
            entry = f.read(6)
        require(len(entry) == 6, 'index changed/truncated')
        size, first = int.from_bytes(entry[:3], 'big'), int.from_bytes(entry[3:], 'big')
        if size == first == 0:
            return None
        require(0 < size <= MAX_STORED and first > 0, 'index entry size/pointer')
        return size, first

    def raw(self, archive, group):
        entry = self.entry(archive, group)
        if entry is None:
            return None
        length, sector = entry
        width, payload = (8, 512) if group <= 65535 else (10, 510)
        require((length + payload - 1) // payload <= 65536, 'sector chunk bound')
        result, seen, chunk = bytearray(), set(), 0
        with (self.root / 'main_file_cache.dat2').open('rb') as f:
            while len(result) < length:
                require(0 < sector <= U24 and sector not in seen, 'sector pointer/cycle')
                seen.add(sector)
                need = min(payload, length - len(result))
                require(sector * 520 + width + need <= self.snapshot['main_file_cache.dat2'][0],
                        'sector outside data')
                f.seek(sector * 520)
                header = f.read(width)
                gid = int.from_bytes(header[:width - 6], 'big')
                number = int.from_bytes(header[width - 6:width - 4], 'big')
                nxt = int.from_bytes(header[width - 4:width - 1], 'big')
                require(gid == group and number == chunk and header[-1] == archive,
                        'sector ownership/group/chunk/archive')
                block = f.read(need)
                require(len(block) == need, 'truncated sector')
                result += block
                require(nxt != 0 if len(result) < length else nxt == 0, 'sector terminal link')
                sector, chunk = nxt, chunk + 1
        return bytes(result)

    def reference(self, archive):
        raw = self.raw(255, archive)
        if raw is None:
            # Never synthesize a table which would omit unrelated indexed groups.
            index = self.root / ('main_file_cache.idx' + str(archive))
            if index.name in self.snapshot:
                with index.open('rb') as f:
                    for block in iter(lambda: f.read(6144), b''):
                        require(not any(block), 'missing reference with existing archive groups')
            return dict(protocol=6, version=0, flags=0, groups={})
        return parse_reference(decode_container(raw, MAX_REFERENCE))


def append_group(root, archive, group, raw):
    integer(archive, 255, 'append archive')
    require(5 <= len(raw) <= MAX_STORED, 'append length')
    integer(group, MAX_INDEX // 6 - 1, 'append group/index bound')
    path = root / 'main_file_cache.dat2'
    width, payload = (8, 512) if group <= 65535 else (10, 510)
    start = max(1, (path.stat().st_size + 519) // 520)
    count = (len(raw) + payload - 1) // payload
    require(count <= 65536 and start + count - 1 <= U24, 'append sector/u24 bound')
    with path.open('r+b') as f:
        old_end = f.seek(0, 2)
        f.write(bytes(start * 520 - old_end))
        for chunk in range(count):
            nxt = start + chunk + 1 if chunk + 1 < count else 0
            header = group.to_bytes(width - 6, 'big') + struct.pack('>H', chunk) + u24(nxt) + bytes([archive])
            block = raw[chunk * payload:(chunk + 1) * payload]
            f.write(header + block + bytes(payload - len(block)))
        f.flush()
        os.fsync(f.fileno())
    index = root / ('main_file_cache.idx' + str(archive))
    with index.open('r+b' if index.exists() else 'xb') as f:
        f.seek(group * 6)
        f.write(u24(len(raw)) + u24(start))
        f.flush()
        os.fsync(f.fileno())


def unique_object(pairs):
    obj = {}
    for key, value in pairs:
        require(key not in obj, 'duplicate JSON key')
        obj[key] = value
    return obj


def inputs_from(root, allocation):
    allocation_bytes = read_small(allocation, 65536)
    doc = json.loads(allocation_bytes, object_pairs_hook=unique_object,
                     parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))
    require(type(doc) is dict and set(doc) == {'schema', 'namespace', 'inputs'} and
            type(doc['schema']) is int and doc['schema'] == 1,
            'allocation schema/keys')
    ns = doc['namespace']
    require(type(ns) is dict and type(doc['inputs']) is dict and
            set(ns) == set(KINDS) and set(doc['inputs']) == set(INPUTS), 'allocation kind coverage')
    for name, value in ns.items():
        integer(value, 65535, name)  # Server JS5 Store::container takes u16 group.
    payloads = {}
    for name in INPUTS:
        item = doc['inputs'][name]
        require(type(item) is dict and set(item) == {'file', 'sha256'} and type(item['file']) is str and
                re.fullmatch(r'[A-Za-z0-9_-]+\.bin', item['file']) and
                type(item['sha256']) is str and re.fullmatch(r'[0-9a-f]{64}', item['sha256']),
                'input identity schema')
        raw = read_small(root / item['file'], MAX_INPUT)
        require(raw and hashlib.sha256(raw).hexdigest() == item['sha256'], 'input SHA/empty')
        payloads[name] = raw
    for i in range(3):
        require(len(payloads['frame' + str(i)]) >= 3 and
                int.from_bytes(payloads['frame' + str(i)][:2], 'big') == ns['skeleton'],
                'frame/skeleton mismatch')
    seq = Reader(payloads['sequence'])
    require(seq.number(1) == 1 and seq.number(2) == 3, 'synthetic sequence three-pose contract')
    lengths = [seq.number(2) for _ in range(3)]
    require(all(length > 0 for length in lengths), 'zero pose duration')
    require([seq.number(2) for _ in range(3)] == [0, 1, 2] and
            [seq.number(2) for _ in range(3)] == [ns['frames']] * 3,
            'sequence/frame namespace mismatch')
    tail = payloads['sequence'][seq.pos:]
    # Existing minimal fixtures retain default policy. The explicit form is
    # ONLY the independently declared Phase A action; never strip its policy.
    if tail != b'\0':
        require(lengths == [50, 75, 100] and tail == bytes.fromhex('05 05 08 01 09 00 0a 00 0b 02 00'),
                'synthetic explicit action policy/durations')
    seq.number(len(tail))
    seq.end()
    require(ns['sequence'] < 65535 and payloads['location'] ==
            b'\7\1' + struct.pack('>I', ns['model']) + b'\x18' +
            struct.pack('>H', ns['sequence']) + b'\x59\0',
            'location/model/sequence synthetic contract (op7/op24/op89)')
    # Native model/skeleton/definition semantics remain a separate decoder gate.
    return doc, payloads, hashlib.sha256(allocation_bytes).hexdigest()


def changes(cache, namespace, payloads, source_revision_policy='strict', output_revision_policy='full32',
            source_observations=None):
    revision_policies(source_revision_policy, output_revision_policy)
    refs = {a: cache.reference(a) for a in (0, 1, 2, 7)}
    pending = {}
    for archive, key, files in ((7, 'model', {0: payloads['model']}),
                                (1, 'skeleton', {0: payloads['skeleton']}),
                                (0, 'frames', {i: payloads['frame' + str(i)] for i in range(3)})):
        gid = namespace[key]
        require(gid not in refs[archive]['groups'] and cache.entry(archive, gid) is None,
                'source group collision')
        pending[archive, gid] = (files, None)
    for gid, key in ((6, 'location'), (12, 'sequence')):
        ref = refs[2]['groups'].get(gid)
        raw = cache.raw(2, gid)
        require((ref is None) == (raw is None), 'config reference/disk mismatch')
        if ref is None:
            files = {}
        else:
            packed, observation = read_source_group(raw, ref, source_revision_policy)
            if refs[2]['flags'] & 4:
                container_length = (5 if raw[0] == 0 else 9) + int.from_bytes(raw[1:5], 'big')
                require(ref['lengths'] == (container_length, len(packed)), 'source config length pair mismatch')
            files = split_files(packed, sorted(ref['files']))
            if source_observations is not None:
                source_observations.append(dict(archive=2, group=gid, **observation))
        require(namespace[key] not in files, 'source definition collision')
        files[namespace[key]] = payloads[key]
        pending[2, gid] = (files, ref)
    writes = {}
    for (archive, gid), (files, old) in sorted(pending.items()):
        version = 1 if old is None else integer(old['version'] + 1, 0xFFFFFFFF, 'group version overflow')
        packed = join_files(files)
        body = container(packed)
        meta = dict(crc=crc(body), version=version, name=0 if old is None else old['name'],
                    files={fid: old['files'].get(fid, 0) if old else 0 for fid in sorted(files)})
        if refs[archive]['flags'] & 4:
            meta['lengths'] = (len(body), len(packed))
        refs[archive]['groups'][gid] = meta
        trailer = struct.pack('>I', version) if output_revision_policy == 'full32' else struct.pack('>H', version & 0xffff)
        writes[archive, gid] = body + trailer
    for archive, ref in refs.items():
        if ref['protocol'] >= 6:
            ref['version'] = integer(ref['version'] + 1, 0xFFFFFFFF, 'reference version overflow')
        writes[255, archive] = container(encode_reference(ref))
    return writes, refs


def write_json_new(path, data):
    with path.open('x', encoding='utf-8') as f:
        json.dump(data, f, sort_keys=True, indent=2)
        f.write('\n')
        f.flush()
        os.fsync(f.fileno())


def prepare(game, js5, input_root, allocation, output, growth_limit=GROWTH_CAP,
            source_revision_policy='strict', output_revision_policy='full32'):
    policy_sha256 = revision_policies(source_revision_policy, output_revision_policy)
    integer(growth_limit, GROWTH_CAP, 'growth limit')
    game, js5 = Cache(game), Cache(js5)
    input_root = checked_path(input_root, directory=True)
    allocation = checked_path(allocation)
    output = checked_path(output, missing=True)
    require(not output.exists(), 'output already exists; partials are preserved')
    require(output.parent.is_dir(), 'output parent must exist')
    for source in (game.root, js5.root, input_root, allocation):
        require(not related(output, source), 'output/source overlap/ancestor alias')
    require(not related(game.root, js5.root), 'game/JS5 roots must be independent')
    doc, payloads, allocation_sha = inputs_from(input_root, allocation)
    observations = dict(game=[], js5=[])
    game_writes, game_refs = changes(game, doc['namespace'], payloads, source_revision_policy,
                                    output_revision_policy, observations['game'])
    js5_writes, js5_refs = changes(js5, doc['namespace'], payloads, source_revision_policy,
                                  output_revision_policy, observations['js5'])
    sources = ((game, game_writes, game_refs), (js5, js5_writes, js5_refs), (js5, js5_writes, js5_refs))
    copied = sum(sum(v[0] for v in cache.snapshot.values()) for cache, _, _ in sources)
    growth = 1024 * 1024  # reserve bounded receipts/partial ledger, not only cache sectors
    for cache, writes, _ in sources:
        growth += 520 + sum(((len(raw) + (512 if gid <= 65535 else 510) - 1) //
                             (512 if gid <= 65535 else 510)) * 520 for (_, gid), raw in writes.items())
        for archive in {a for a, _ in writes}:
            needed = (max(g for a, g in writes if a == archive) + 1) * 6
            growth += max(0, needed - cache.snapshot.get('main_file_cache.idx' + str(archive), (0,))[0])
    require(growth <= growth_limit and copied + growth <= COPY_CAP, 'copy/growth budget')
    return dict(game=game, js5=js5, sources=sources, output=output, doc=doc,
                allocation_sha256=allocation_sha, growth_limit=growth_limit,
                allocation=allocation, input_root=input_root, copied=copied, growth=growth,
                source_revision_policy=source_revision_policy, output_revision_policy=output_revision_policy,
                revision_policy_sha256=policy_sha256, source_revision_observations=observations)


def revision_receipt(plan):
    return dict(source_revision_policy=plan['source_revision_policy'],
                output_revision_policy=plan['output_revision_policy'],
                revision_policy_sha256=plan['revision_policy_sha256'],
                output_trailer_bytes=4 if plan['output_revision_policy'] == 'full32' else 2,
                full_reference_versions_preserved=True,
                source_revision_observations=plan['source_revision_observations'],
                low16_limitation='Two-byte revisions cannot authenticate the high16 bits; CRC binds container bytes, not full-width freshness.',
                native_client_disk_policy='JavaStrict unchanged; high-version server-low16 output requires verified JS5 recovery, not offline acceptance')


def summary(plan):
    return dict(**revision_receipt(plan), mode='read-only plan', namespace=plan['doc']['namespace'], real_allocation_reserved=False,
                authentic_plan_requires_read_collision_grant=True,
                authored_package_namespace_checked=False, owner_collision_admission_approval_required=True,
                reference_encoding='changed references type0 uncompressed; CRC/master pairs change',
                new_name_hashes='0; ID lookup only, name lookup not promised',
                allocation_sha256=plan['allocation_sha256'], input_identities=plan['doc']['inputs'],
                source_copy_bytes=plan['copied'], growth_upper_bound=plan['growth'],
                target_children=['game', 'js5', 'client'], native_admission=False,
                source_snapshots={name: cache.snapshot for name, cache in
                                  (('game', plan['game']), ('js5', plan['js5']))})


def stage(plan):
    # Re-evaluate collisions/identities immediately before any output mutation.
    fresh = prepare(plan['game'].root, plan['js5'].root, plan['input_root'],
                    plan['allocation'], plan['output'], plan['growth_limit'],
                    source_revision_policy=plan['source_revision_policy'],
                    output_revision_policy=plan['output_revision_policy'])
    require(summary(plan) == summary(fresh) and plan['doc'] == fresh['doc'], 'source/allocation changed')
    require(shutil.disk_usage(fresh['output'].parent).free >= DISK_FLOOR + fresh['copied'] + fresh['growth'],
            'disk floor/budget')
    output = fresh['output']
    output.mkdir()  # exclusive, no retry into a partial directory
    write_json_new(output / 'partial.json', dict(status='incomplete', plan=summary(fresh)))
    copies = []
    source_hashes = {}
    for name, (cache, writes, refs) in zip(('game', 'js5', 'client'), fresh['sources']):
        require(inventory(cache.root) == cache.snapshot, 'source metadata changed before copy')
        child = output / name
        child.mkdir()
        hashes = {}
        for filename, identity in cache.snapshot.items():
            source, target = cache.root / filename, child / filename
            h = hashlib.sha256()
            with source.open('rb') as src, target.open('xb') as dst:
                for block in iter(lambda: src.read(1024 * 1024), b''):
                    require(shutil.disk_usage(output).free >= DISK_FLOOR + len(block), 'disk below floor during copy')
                    h.update(block)
                    dst.write(block)
                dst.flush()
                os.fsync(dst.fileno())
            require(target.stat().st_size == identity[0] and digest(target) == h.hexdigest() == digest(source),
                    'independent copy SHA/size mismatch')
            hashes[filename] = h.hexdigest()
        require(inventory(cache.root) == cache.snapshot, 'source metadata changed after copy')
        if cache.root in source_hashes:
            require(source_hashes[cache.root] == hashes, 'source changed between independent copies')
        source_hashes[cache.root] = hashes
        for (archive, gid), raw in sorted(writes.items()):
            require(shutil.disk_usage(output).free >= DISK_FLOOR + fresh['growth'],
                    'disk floor before append')
            append_group(child, archive, gid, raw)
        check = Cache(child)
        for (archive, gid), raw in writes.items():
            require(check.raw(archive, gid) == raw, 'staged sector readback mismatch')
            if archive != 255:
                policy = 'strict' if fresh['output_revision_policy'] == 'full32' else 'server-low16'
                _, observed = validate_revision_container(raw, refs[archive]['groups'][gid], policy)
                require(observed['trailer_bytes'] == (4 if policy == 'strict' else 2), 'staged output revision width mismatch')
        # Master pairs are derived from exact on-disk reference bytes, no suffix.
        master = {}
        for archive in refs:
            body = check.raw(255, archive)
            parsed = parse_reference(decode_container(body))
            require(parsed == refs[archive], 'reference readback mismatch')
            master[archive] = [crc(body), parsed['version']]
        copies.append(dict(name=name, source_sha256=hashes, master_pairs=master,
                           staged_sha256={p.name: digest(p) for p in child.iterdir()}))
        require(shutil.disk_usage(output).free >= DISK_FLOOR, 'disk below floor')
    for cache in (fresh['game'], fresh['js5']):
        require(inventory(cache.root) == cache.snapshot, 'source changed before completion')
        require(all(digest(cache.root / name) == sha for name, sha in source_hashes[cache.root].items()),
                'source SHA changed before completion')
    actual = sum(p.stat().st_size for p in output.rglob('*') if p.is_file())
    result = dict(**revision_receipt(fresh), status='staged Python readback only', copies=copies, native_admission=False,
                  allocation_sha256=fresh['allocation_sha256'], input_identities=fresh['doc']['inputs'],
                  source_unchanged_metadata=True, cache_bytes=actual)
    result.update(authored_package_namespace_checked=False, owner_collision_admission_approval_required=True,
                  reference_encoding='type0 uncompressed', new_name_hashes=0,
                  cache_files_fsynced_before_completion=True,
                  directory_durability='fsync before publication where supported; Windows OS-dependent')
    complete_size = len((json.dumps(result, sort_keys=True, indent=2) + '\n').encode('utf-8'))
    require(actual + 2 * complete_size <= COPY_CAP and
            actual + 2 * complete_size - fresh['copied'] <= fresh['growth_limit'], 'final storage bound')
    # A write/fsync failure leaves only a pending receipt, never complete.json.
    pending = output / 'completion-ready.json'
    write_json_new(pending, result)
    sync_staged_files_and_directories(output)
    require(not (output / 'complete.json').exists(), 'completion path conflict')
    if os.name == 'nt':
        os.rename(pending, output / 'complete.json')  # Windows refuses replacement.
    else:
        # Atomic exclusive publication of OWN immutable receipt, never a cache
        # file or original. Preserve the pending name too; no unlink/cleanup.
        os.link(pending, output / 'complete.json')
    return result


def sync_staged_files_and_directories(output):
    """Final barrier covers every copied index/data file, including unchanged ones."""
    for name in ('game', 'js5', 'client'):
        child = output / name
        for path in child.iterdir():
            with path.open('r+b') as f:
                f.flush()
                os.fsync(f.fileno())
        sync_directory(child)
    sync_directory(output)


def sync_directory(path):
    if os.name != 'nt':
        fd = os.open(path, os.O_RDONLY | getattr(os, 'O_DIRECTORY', 0))
        try:
            os.fsync(fd)
        finally:
            os.close(fd)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ('game', 'js5', 'input', 'allocation', 'output'):
        parser.add_argument('--' + flag, required=True, type=Path)
    parser.add_argument('--growth-limit-bytes', type=int, default=GROWTH_CAP)
    parser.add_argument('--source-revision-policy', choices=('strict', 'server-low16'), default='strict')
    parser.add_argument('--output-revision-policy', choices=('full32', 'server-low16'), default='full32')
    parser.add_argument('--stage', action='store_true', help='mutates NEW private output; needs separate copy grant')
    args = parser.parse_args(argv)
    plan = prepare(args.game, args.js5, args.input, args.allocation, args.output, args.growth_limit_bytes,
                   source_revision_policy=args.source_revision_policy,
                   output_revision_policy=args.output_revision_policy)
    print(json.dumps(stage(plan) if args.stage else summary(plan), sort_keys=True, indent=2))


if __name__ == '__main__':
    sys.dont_write_bytecode = True
    try:
        main()
    except (ValueError, OSError, EOFError, zlib.error) as error:
        print('ERROR: ' + str(error), file=sys.stderr)
        sys.exit(1)
