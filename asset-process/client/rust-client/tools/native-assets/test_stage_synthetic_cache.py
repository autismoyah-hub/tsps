"""Tiny original fixtures, independent reader: no writer helpers used as oracle.

All roots are per-test temp directories; <=16 MiB on disk per test. These tests
do not open authentic caches and do not establish maintained native readback.
"""
import bz2
import contextlib
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest import mock
import zlib

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('stager', Path(__file__).with_name('stage_synthetic_cache.py'))
stager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stager)


# Independent fixture construction/reading below uses only stdlib primitives.
def box(payload):
    return b'\0' + len(payload).to_bytes(4, 'big') + payload


def ref_bytes(groups, protocol=6, version=10, named=False, lengths=False):
    def count(n):
        return (n | 0x80000000).to_bytes(4, 'big') if protocol == 7 and n >= 32768 else n.to_bytes(2, 'big')
    out = bytes([protocol]) + (version.to_bytes(4, 'big') if protocol > 5 else b'') + bytes([int(named) | (4 if lengths else 0)])
    out += count(len(groups))
    last = 0
    for gid in sorted(groups):
        out += count(gid - last)
        last = gid
    if named:
        out += b''.join(struct.pack('>I', g.get('name', 123)) for _, g in sorted(groups.items()))
    out += b''.join(struct.pack('>I', zlib.crc32(g['body']) & 0xFFFFFFFF) for _, g in sorted(groups.items()))
    if lengths:
        out += b''.join(struct.pack('>II', *g.get('lengths', (len(g['body']), len(g['body']) - 5)))
                        for _, g in sorted(groups.items()))
    out += b''.join(struct.pack('>I', g.get('version', 7)) for _, g in sorted(groups.items()))
    out += b''.join(count(len(g['files'])) for _, g in sorted(groups.items()))
    for _, g in sorted(groups.items()):
        last = 0
        for fid in sorted(g['files']):
            out += count(fid - last)
            last = fid
    if named:
        out += b''.join(struct.pack('>I', 1000 + fid) for _, g in sorted(groups.items()) for fid in sorted(g['files']))
    return out


def fixture_cache(root, groups, protocols=None, ref_version=10, reference_flags=None):
    root.mkdir()
    protocols = protocols or {}
    data = bytearray(520)
    indexes = {}
    def put(archive, gid, raw):
        large = gid >= 65536
        capacity = 510 if large else 512
        start = len(data) // 520
        for chunk, offset in enumerate(range(0, len(raw), capacity)):
            number = len(data) // 520
            more = offset + capacity < len(raw)
            header = gid.to_bytes(4 if large else 2, 'big') + chunk.to_bytes(2, 'big')
            header += (number + 1 if more else 0).to_bytes(3, 'big') + bytes([archive])
            piece = raw[offset:offset + capacity]
            data.extend(header + piece + bytes(capacity - len(piece)))
        index = indexes.setdefault(archive, bytearray())
        index.extend(bytes(max(0, (gid + 1) * 6 - len(index))))
        index[gid * 6:gid * 6 + 6] = len(raw).to_bytes(3, 'big') + start.to_bytes(3, 'big')
    for archive, entries in sorted(groups.items()):
        for gid, g in sorted(entries.items()):
            put(archive, gid, g['body'] + g.get('disk_version', g.get('version', 7)).to_bytes(g.get('width', 4), 'big'))
        protocol = protocols.get(archive, 6)
        flags = 1 if reference_flags is None else reference_flags.get(archive, 1)
        put(255, archive, box(ref_bytes(entries, protocol, version=ref_version,
                                       named=bool(flags & 1), lengths=bool(flags & 4))))
    (root / 'main_file_cache.dat2').write_bytes(data)
    for archive, index in indexes.items():
        (root / ('main_file_cache.idx' + str(archive))).write_bytes(index)


def independent_raw(root, archive, gid):
    index = (root / ('main_file_cache.idx' + str(archive))).read_bytes()
    size = int.from_bytes(index[gid * 6:gid * 6 + 3], 'big')
    sector = int.from_bytes(index[gid * 6 + 3:gid * 6 + 6], 'big')
    data = (root / 'main_file_cache.dat2').read_bytes()
    output = bytearray()
    seen, chunk = set(), 0
    while len(output) < size:
        assert sector not in seen and sector > 0
        seen.add(sector)
        n = 4 if gid > 65535 else 2
        offset = sector * 520
        assert int.from_bytes(data[offset:offset + n], 'big') == gid
        assert int.from_bytes(data[offset + n:offset + n + 2], 'big') == chunk
        following = int.from_bytes(data[offset + n + 2:offset + n + 5], 'big')
        assert data[offset + n + 5] == archive
        count = min(514 - n, size - len(output))
        output += data[offset + n + 6:offset + n + 6 + count]
        sector, chunk = following, chunk + 1
    assert not sector and len(output) == size
    return bytes(output)


def independent_reference(body):
    assert body[0] == 0 and int.from_bytes(body[1:5], 'big') == len(body) - 5
    payload = body[5:]
    pos = 0
    def number(n):
        nonlocal pos
        x = int.from_bytes(payload[pos:pos + n], 'big')
        assert pos + n <= len(payload)
        pos += n
        return x
    protocol = number(1)
    version = number(4) if protocol >= 6 else 0
    flags = number(1)
    assert flags in (0, 1, 4, 5)
    def count():
        return number(4) & 0x7FFFFFFF if protocol == 7 and payload[pos] & 128 else number(2)
    n = count()
    groups, gid = {}, 0
    for _ in range(n):
        gid += count()
        groups[gid] = {}
    if flags & 1:
        for g in groups.values():
            g['name'] = number(4)
    for g in groups.values():
        g['crc'] = number(4)
    if flags & 4:
        for g in groups.values():
            g['lengths'] = (number(4), number(4))
    for g in groups.values():
        g['version'] = number(4)
    counts = [count() for _ in groups]
    for g, n in zip(groups.values(), counts):
        fid, files = 0, {}
        for _ in range(n):
            fid += count()
            files[fid] = 0
        g['files'] = files
    if flags & 1:
        for g in groups.values():
            for fid in g['files']:
                g['files'][fid] = number(4)
    assert pos == len(payload)
    return version, groups


def independent_files(raw, meta):
    body = raw[:-4]
    assert zlib.crc32(body) & 0xFFFFFFFF == meta['crc']
    assert int.from_bytes(raw[-4:], 'big') == meta['version']
    assert body[0] == 0 and int.from_bytes(body[1:5], 'big') == len(body) - 5
    data, ids = body[5:], sorted(meta['files'])
    if len(ids) == 1:
        return {ids[0]: data}
    # Writer promises one deterministic chunk, independently checked here.
    assert data[-1] == 1
    tail = len(data) - 1 - len(ids) * 4
    running, offset, result = 0, 0, {}
    for i, fid in enumerate(ids):
        running += struct.unpack_from('>i', data, tail + i * 4)[0]
        assert running >= 0
        result[fid] = data[offset:offset + running]
        offset += running
    assert offset == tail
    return result


class StagerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='synthetic-stager-test-')
        self.root = Path(self.tmp.name)
        # Literal asymmetric file lengths, including negative signed length delta.
        self.loc = b'LONG-LOCATION' + b'L' + struct.pack('>ii', 13, -12) + b'\1'
        self.seq = b'SEQ' + b'OTHERSEQ' + struct.pack('>ii', 3, 5) + b'\1'
        self.groups = {2: {6: dict(body=box(self.loc), files=[2, 8]),
                           12: dict(body=box(self.seq), files=[1, 9])},
                       7: {3: dict(body=box(b'UNRELATED-MODEL'), files=[0])},
                       0: {}, 1: {}, 9: {2: dict(body=box(b'UNTOUCHED-ARCHIVE'), files=[0])}}
        self.game, self.js5 = self.root / 'game-source', self.root / 'js5-source'
        fixture_cache(self.game, self.groups)
        fixture_cache(self.js5, self.groups)
        self.inputs = self.root / 'inputs'
        self.inputs.mkdir()
        self.ns = dict(model=20, skeleton=21, frames=22, sequence=23, location=24)
        data = dict(model=b'ORIGINAL-SYNTHETIC-MODEL', skeleton=b'ORIGINAL-SYNTHETIC-LABELS',
                    frame0=b'\0\x15\1pose0', frame1=b'\0\x15\1pose1', frame2=b'\0\x15\1pose2',
                    sequence=b'\1\0\3' + struct.pack('>9H', 2, 3, 5, 0, 1, 2, 22, 22, 22) + b'\0',
                    location=b'\7\1\0\0\0\x14\x18\0\x17\x59\0')
        self.doc = dict(schema=1, namespace=self.ns, inputs={})
        for name, raw in data.items():
            (self.inputs / (name + '.bin')).write_bytes(raw)
            self.doc['inputs'][name] = dict(file=name + '.bin', sha256=hashlib.sha256(raw).hexdigest())
        self.allocation = self.root / 'allocation.json'
        self.save_doc()
        self.output = self.root / 'new-stage'

    def tearDown(self):
        size = sum(p.stat().st_size for p in self.root.rglob('*') if p.is_file())
        self.assertLessEqual(size, 16 * 1024 * 1024, 'test disk budget')
        self.tmp.cleanup()

    def save_doc(self):
        self.allocation.write_text(json.dumps(self.doc))

    def sync_location(self):
        raw = b'\7\1' + self.ns['model'].to_bytes(4, 'big') + b'\x18' + self.ns['sequence'].to_bytes(2, 'big') + b'\x59\0'
        (self.inputs / 'location.bin').write_bytes(raw)
        self.doc['inputs']['location']['sha256'] = hashlib.sha256(raw).hexdigest()
        self.save_doc()

    def plan(self, **policies):
        return stager.prepare(self.game, self.js5, self.inputs, self.allocation, self.output, **policies)

    def run_stage(self, **policies):
        # Tests still exercise the production floor; fake only a too-small disk.
        return stager.stage(self.plan(**policies))

    def legacy_sources(self, version=65536):
        groups = {a: {gid: dict(g) for gid, g in entries.items()} for a, entries in self.groups.items()}
        for group in groups[2].values():
            group.update(version=version, width=2, disk_version=version & 0xffff)
        self.game, self.js5 = self.root/'legacy-game', self.root/'legacy-js5'
        for root in (self.game, self.js5):
            fixture_cache(root, groups, reference_flags={a: 4 for a in groups})

    def test_revision_defaults_and_explicit_legacy_source(self):
        default = self.plan()
        self.assertEqual(stager.summary(default)['source_revision_policy'], 'strict')
        self.assertEqual(stager.summary(default)['output_revision_policy'], 'full32')
        self.legacy_sources(65537)
        with self.assertRaisesRegex(ValueError, 'CRC/full revision'):
            self.plan()
        with self.assertRaisesRegex(ValueError, 'CRC/full revision'):
            self.plan(output_revision_policy='server-low16')  # No implicit input fallback.
        plan = self.plan(source_revision_policy='server-low16')
        self.assertEqual(plan['output_revision_policy'], 'full32')
        for _, writes, refs in plan['sources']:
            self.assertEqual(writes[2,6][-4:], struct.pack('>I',65538))
            self.assertEqual(refs[2]['groups'][6]['version'],65538)
        self.assertFalse(self.output.exists())
        for entries in plan['source_revision_observations'].values():
            self.assertEqual(len(entries), 2)
            for observed in entries:
                self.assertEqual(observed['full_reference_version'], 65537)
                self.assertEqual(observed['stored_revision'], 1)
                self.assertEqual(observed['reference_crc'], observed['actual_crc'])
                self.assertEqual(observed['matched_rule'], 'unsigned16-low16')
                self.assertEqual(observed['trailer_bytes'], 2)

    def test_revision_guards_precede_decompression_and_keep_alias_limit(self):
        body = box(b'ABCD')
        info = dict(crc=zlib.crc32(body)&0xffffffff, version=65543)
        raw = body+b'\0\7'
        self.assertEqual(stager.read_source_group(raw, info, 'server-low16')[0], b'ABCD')
        # Identical bytes+CRC+low16 cannot authenticate different high16 bits.
        alias = dict(info, version=131079)
        self.assertEqual(stager.read_source_group(raw, alias, 'server-low16')[1]['full_reference_version'], 131079)
        with self.assertRaisesRegex(ValueError, 'CRC/full revision'):
            stager.read_source_group(raw, info)
        wrong_crc = bytearray(raw); wrong_crc[5] ^= 1
        invalid = [bytes(wrong_crc), raw[:-1]+b'\x08', raw+b'X', raw[:-1], body,
                   body+struct.pack('>I', 131079)]
        for malformed in invalid:
            with self.subTest(malformed=malformed.hex()), \
                    mock.patch.object(stager, 'decode_container', side_effect=AssertionError('no decompression')):
                with self.assertRaises(ValueError):
                    stager.read_source_group(malformed, info, 'server-low16')
        with self.assertRaisesRegex(ValueError, 'unknown source'):
            stager.read_source_group(raw, info, 'fallback')

    def test_exact_compressed_header_accounting_before_decompression(self):
        payload = b'original synthetic compressed source'
        compressed = gzip.compress(payload, mtime=0)
        body = b'\2'+struct.pack('>II',len(compressed),len(payload))+compressed
        info = dict(crc=zlib.crc32(body)&0xffffffff, version=65537)
        self.assertEqual(stager.read_source_group(body+b'\0\1',info,'server-low16')[0], payload)
        for malformed in (body+b'\0\1X',body[:-1]+b'\0\1',
                          b'\2'+struct.pack('>I',len(compressed)+1)+body[5:]+b'\0\1'):
            with mock.patch.object(stager,'decode_container',side_effect=AssertionError('header/CRC before decode')):
                with self.assertRaises(ValueError):
                    stager.read_source_group(malformed,info,'server-low16')

    def test_server_low16_output_full_metadata_and_revalidation_receipt(self):
        self.legacy_sources(65536)
        before={str(p):p.read_bytes() for root in (self.game,self.js5) for p in root.iterdir()}
        policies=dict(source_revision_policy='server-low16',output_revision_policy='server-low16')
        plan=self.plan(**policies)
        original_prepare=stager.prepare
        with mock.patch.object(stager,'prepare',wraps=original_prepare) as fresh:
            result=stager.stage(plan)
        self.assertEqual(fresh.call_args.kwargs,policies)
        self.assertEqual(result['source_revision_policy'],'server-low16')
        self.assertEqual(result['output_revision_policy'],'server-low16')
        self.assertEqual(result['revision_policy_sha256'],plan['revision_policy_sha256'])
        self.assertEqual(result['output_trailer_bytes'],2)
        self.assertTrue(result['full_reference_versions_preserved'])
        self.assertFalse(result['native_admission'])
        for child in ('game','js5','client'):
            root=self.output/child
            for archive,gids in ((0,[22]),(1,[21]),(2,[6,12]),(7,[20])):
                ref=independent_raw(root,255,archive)
                version,groups=independent_reference(ref)
                self.assertEqual(version,11)
                for gid in gids:
                    raw=independent_raw(root,archive,gid); body=raw[:-2]; meta=groups[gid]
                    self.assertEqual(len(body),5+int.from_bytes(body[1:5],'big'))
                    self.assertEqual(zlib.crc32(body)&0xffffffff,meta['crc'])
                    self.assertEqual(int.from_bytes(raw[-2:],'big'),meta['version']&0xffff)
                    self.assertEqual(meta['lengths'],(len(body),len(body)-5))
                    if archive==2:
                        self.assertEqual(meta['version'],65537)
                        with self.assertRaisesRegex(ValueError,'CRC/full revision'):
                            stager.validate_group(raw,meta)  # JavaStrict is not relaxed.
                receipt=next(c for c in result['copies'] if c['name']==child)
                self.assertEqual(receipt['master_pairs'][archive],[zlib.crc32(ref)&0xffffffff,11])
        self.assertTrue(all(Path(p).read_bytes()==raw for p,raw in before.items()))

    def test_revision_rollover_overflow_and_policy_identity(self):
        self.legacy_sources(65535)
        plan=self.plan(source_revision_policy='server-low16',output_revision_policy='server-low16')
        for cache,writes,refs in plan['sources']:
            self.assertEqual(refs[2]['groups'][6]['version'],65536)
            self.assertEqual(writes[2,6][-2:],b'\0\0')
        plan['output_revision_policy']='full32'
        with self.assertRaisesRegex(ValueError,'source/allocation changed'):
            stager.stage(plan)
        self.assertFalse(self.output.exists())
        for policies in (dict(source_revision_policy='automatic'),dict(output_revision_policy='truncate')):
            with self.assertRaises(ValueError): self.plan(**policies)
        # Checked full u32 increment fails even though a low16 trailer could wrap.
        groups={a:{gid:dict(g) for gid,g in entries.items()} for a,entries in self.groups.items()}
        groups[2][6].update(version=0xffffffff,width=2,disk_version=65535)
        self.game=self.root/'legacy-overflow'
        fixture_cache(self.game,groups)
        with self.assertRaisesRegex(ValueError,'version overflow'):
            self.plan(source_revision_policy='server-low16',output_revision_policy='server-low16')

    def test_revision_policy_cli_requires_explicit_choices(self):
        self.legacy_sources(65537)
        args=[]
        for name,value in (('game',self.game),('js5',self.js5),('input',self.inputs),
                           ('allocation',self.allocation),('output',self.output)):
            args += ['--'+name,str(value)]
        with self.assertRaisesRegex(ValueError,'CRC/full revision'), contextlib.redirect_stdout(io.StringIO()):
            stager.main(args)
        capture=io.StringIO()
        with contextlib.redirect_stdout(capture):
            stager.main(args+['--source-revision-policy','server-low16','--output-revision-policy','server-low16'])
        receipt=json.loads(capture.getvalue())
        self.assertEqual(receipt['source_revision_policy'],'server-low16')
        self.assertEqual(receipt['output_revision_policy'],'server-low16')
        self.assertFalse(self.output.exists())
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            stager.main(args+['--output-revision-policy','automatic'])
        self.assertEqual(error.exception.code,2)

    def test_selected_output_width_is_checked_after_sector_readback(self):
        plan=self.plan(output_revision_policy='server-low16')
        bad=dict(plan); sources=[]
        for cache,writes,refs in plan['sources']:
            changed=dict(writes)
            changed[0,22]=changed[0,22][:-2]+struct.pack('>I',refs[0]['groups'][22]['version'])
            sources.append((cache,changed,refs))
        bad['sources']=tuple(sources)
        # Inject valid CRC/full4 bytes under a selected two-byte output policy.
        # Raw equality alone is insufficient; representation must match too.
        with mock.patch.object(stager,'prepare',return_value=bad):
            with self.assertRaisesRegex(ValueError,'output revision width mismatch'):
                stager.stage(plan)
        self.assertTrue((self.output/'partial.json').exists())
        self.assertFalse((self.output/'complete.json').exists())

    def test_observed_legacy_source_mutation_is_refused_before_copy(self):
        self.legacy_sources(65537)
        plan=self.plan(source_revision_policy='server-low16',output_revision_policy='server-low16')
        entry=(self.game/'main_file_cache.idx2').read_bytes()[36:42]
        sector=int.from_bytes(entry[3:],'big')
        path=self.game/'main_file_cache.dat2'; stat=path.stat()
        raw=bytearray(path.read_bytes());raw[sector*520+8+5]^=1
        path.write_bytes(raw);os.utime(path,ns=(stat.st_atime_ns,stat.st_mtime_ns))
        with self.assertRaisesRegex(ValueError,'CRC mismatch'):
            stager.stage(plan)
        self.assertFalse(self.output.exists())

    def test_default_plan_no_writes_and_direct_collisions(self):
        plan = self.plan()
        self.assertFalse(self.output.exists())
        self.assertFalse(stager.summary(plan)['native_admission'])
        self.doc['namespace']['model'] = 3
        self.sync_location()
        with self.assertRaisesRegex(ValueError, 'collision'):
            self.plan()
        # Orphan disk entry absent from reference still blocks allocation.
        self.doc['namespace']['model'] = 20
        self.sync_location()
        p = self.js5 / 'main_file_cache.idx7'
        raw = bytearray(p.read_bytes()); raw.extend(bytes(21 * 6 - len(raw)))
        raw[120:126] = raw[18:24]; p.write_bytes(raw)
        with self.assertRaisesRegex(ValueError, 'collision'):
            self.plan()

    def test_stage_independent_readback_preserves_files_and_originals(self):
        before = {str(p): p.read_bytes() for root in (self.game, self.js5) for p in root.iterdir()}
        result = self.run_stage()
        self.assertFalse(result['native_admission'])
        for child in ('game', 'js5', 'client'):
            root = self.output / child
            _, refs = independent_reference(independent_raw(root, 255, 2))
            self.assertEqual(independent_files(independent_raw(root, 2, 6), refs[6]),
                             {2: b'LONG-LOCATION', 8: b'L', 24: (self.inputs / 'location.bin').read_bytes()})
            self.assertEqual(independent_files(independent_raw(root, 2, 12), refs[12]),
                             {1: b'SEQ', 9: b'OTHERSEQ', 23: (self.inputs / 'sequence.bin').read_bytes()})
            self.assertEqual(refs[6]['files'][2], 1002)
            self.assertEqual(refs[6]['name'], 123)
            self.assertEqual(independent_raw(root, 7, 3), independent_raw(self.game, 7, 3))
            self.assertEqual((root / 'main_file_cache.idx9').read_bytes(),
                             (self.game / 'main_file_cache.idx9').read_bytes())
            self.assertEqual(independent_raw(root, 255, 9), independent_raw(self.game, 255, 9))
            original = (self.game / 'main_file_cache.dat2').read_bytes()
            self.assertEqual((root / 'main_file_cache.dat2').read_bytes()[:len(original)], original)
            for a in (0, 1, 2, 7):
                raw = independent_raw(root, 255, a)
                version, _ = independent_reference(raw)
                expected = [zlib.crc32(raw) & 0xFFFFFFFF, version]
                receipt = next(c for c in result['copies'] if c['name'] == child)
                self.assertEqual(receipt['master_pairs'][a], expected)
                self.assertEqual(version, 11)
            # Every original index byte except explicitly changed entries survives.
            modified = {0: {22}, 1: {21}, 2: {6, 12}, 7: {20}, 255: {0, 1, 2, 7}}
            for original_index in self.game.glob('*.idx*'):
                a = int(original_index.name.split('idx')[1])
                old, new = original_index.read_bytes(), (root / original_index.name).read_bytes()
                for offset in range(0, len(old), 6):
                    if offset // 6 not in modified.get(a, set()):
                        self.assertEqual(new[offset:offset + 6], old[offset:offset + 6])
        self.assertTrue(all(Path(p).read_bytes() == raw for p, raw in before.items()))
        inos = [(self.output / n / 'main_file_cache.dat2').stat().st_ino for n in ('game', 'js5', 'client')]
        self.assertEqual(len(set(inos)), 3)

    def test_group_collision_in_each_source(self):
        for key, value in (('location', 2), ('sequence', 1)):
            old = self.doc['namespace'][key]; self.doc['namespace'][key] = value; self.sync_location()
            with self.assertRaisesRegex(ValueError, 'collision'):
                self.plan()
            self.doc['namespace'][key] = old; self.sync_location()

    def test_no_clobber_overlap_reparse_and_hardlink(self):
        self.output.mkdir()
        with self.assertRaisesRegex(ValueError, 'already exists'):
            self.plan()
        self.output = self.game / 'nested'
        with self.assertRaisesRegex(ValueError, 'overlap'):
            self.plan()
        self.output = self.root / 'inputs' / 'nested'
        with self.assertRaisesRegex(ValueError, 'overlap'):
            self.plan()
        target = self.root / 'hardlink.bin'
        os.link(self.inputs / 'model.bin', target)
        with self.assertRaisesRegex(ValueError, 'hard-linked'):
            stager.read_small(target, 100)
        # Windows reparse attribute is checked without needing symlink privilege.
        info = mock.Mock(st_mode=stat_regular(), st_file_attributes=0x400)
        with mock.patch.object(Path, 'lstat', return_value=info):
            with self.assertRaisesRegex(ValueError, 'reparse'):
                stager.checked_path(self.inputs)

    def test_failure_preserves_partial_no_complete(self):
        plan = self.plan()
        with mock.patch.object(stager, 'append_group', side_effect=OSError('injected failure')):
            with self.assertRaisesRegex(OSError, 'injected failure'):
                stager.stage(plan)
        self.assertTrue((self.output / 'partial.json').is_file())
        self.assertFalse((self.output / 'complete.json').exists())
        with self.assertRaisesRegex(ValueError, 'already exists'):
            stager.stage(plan)

    def test_disk_floor_source_change_and_growth_limits(self):
        plan = self.plan()
        disk = mock.Mock(free=19 * 1024**3)
        with mock.patch.object(stager.shutil, 'disk_usage', return_value=disk):
            with self.assertRaisesRegex(ValueError, 'disk floor'):
                stager.stage(plan)
        self.assertFalse(self.output.exists())
        with self.assertRaisesRegex(ValueError, 'budget'):
            stager.prepare(self.game, self.js5, self.inputs, self.allocation, self.output, 1)
        (self.game / 'main_file_cache.dat2').write_bytes((self.game / 'main_file_cache.dat2').read_bytes() + b'x')
        with self.assertRaisesRegex(ValueError, 'changed'):
            stager.stage(plan)

    def test_input_keys_hashes_and_action_namespace(self):
        original = self.allocation.read_bytes()
        for mutate, text in ((lambda d: d.update(xtea=[0, 0, 0, 0]), 'schema'),
                             (lambda d: d['inputs'].pop('frame2'), 'coverage'),
                             (lambda d: d['inputs']['model'].update(file='../other.bin'), 'identity'),
                             (lambda d: d['inputs']['model'].update(sha256='0' * 64), 'SHA'),
                             (lambda d: d['namespace'].update(skeleton=99), 'skeleton'),
                             (lambda d: d['namespace'].update(frames=99), 'namespace'),
                             (lambda d: d['namespace'].update(model=True), 'bounds')):
            self.doc = json.loads(original); mutate(self.doc); self.save_doc()
            with self.assertRaisesRegex(ValueError, text):
                self.plan()
        self.allocation.write_text('{"schema":1,"schema":1}')
        with self.assertRaisesRegex(ValueError, 'duplicate JSON'):
            self.plan()

    def test_u24_and_index_length_bounds(self):
        self.assertEqual(stager.u24(0xFFFFFF), b'\xff\xff\xff')
        for n in (-1, 0x1000000, True):
            with self.assertRaises(ValueError):
                stager.u24(n)
        p = self.game / 'main_file_cache.idx7'; p.write_bytes(p.read_bytes() + b'x')
        with self.assertRaisesRegex(ValueError, 'index length'):
            self.plan()

    def test_protocols_names_bigsmart_and_trailing(self):
        for protocol in (5, 6, 7):
            gid = 65536 if protocol == 7 else 65535
            raw = ref_bytes({gid: dict(body=box(b'x'), files=[0, 40000])}, protocol, named=True)
            parsed = stager.parse_reference(raw)
            self.assertEqual(stager.encode_reference(parsed), raw)
            self.assertIn(gid, parsed['groups'])
            with self.assertRaisesRegex(ValueError, 'trailing'):
                stager.parse_reference(raw + b'x')
        # Literal one-group protocol-7, 4-byte bigsmart group id 65536.
        # Explicit literal without names to avoid relying on fixture encoder.
        literal = bytes.fromhex('070000000100000180010000000000000000000100010000')
        self.assertIn(65536, stager.parse_reference(literal)['groups'])

    def test_unsupported_flags_counts_duplicates(self):
        for flags in (2, 3, 6, 7, 8, 9, 12, 13, 16, 255):
            with self.assertRaisesRegex(ValueError, 'metadata'):
                stager.parse_reference(b'\6\0\0\0\0' + bytes([flags]) + b'\0\0')
        for raw in (b'\7\0\0\0\0\0\x80\x10\0\0',
                    b'\6\0\0\0\0\0\0\2\0\0\0\0' + bytes(32)):
            with self.assertRaises(ValueError):
                stager.parse_reference(raw)

    def test_crc_full_revision_legacy_and_overflow(self):
        body = box(b'asymmetric')
        info = dict(crc=zlib.crc32(body) & 0xFFFFFFFF, version=65537)
        self.assertEqual(stager.validate_group(body + b'\0\1\0\1', info), b'asymmetric')
        with self.assertRaises(ValueError):
            stager.validate_group(body + b'\0\1', info)
        info['version'] = 17
        self.assertEqual(stager.validate_group(body + b'\0\x11', info), b'asymmetric')
        self.assertEqual(stager.validate_group(body + b'\0\0\0\x11', info), b'asymmetric')
        for corrupt in (body[:-1] + b'X' + b'\0\x11', body + b'\0\x12'):
            with self.assertRaises(ValueError):
                stager.validate_group(corrupt, info)
        with self.assertRaises(ValueError):
            stager.integer(0x100000000, 0xFFFFFFFF, 'version overflow')

    def test_literal_length_references_and_names_are_independent_bits(self):
        literals = [
            bytes.fromhex('06 00000001 04 0001 0003 11223344 00000009 00000004 00000007 0001 0000'),
            bytes.fromhex('07 00000001 05 0001 0003 aabbccdd 11223344 00000011 0000000c '
                          '00000007 0002 0000 0001 01020304 05060708'),
        ]
        for original in literals:
            for protocol in (5, 6, 7):
                raw = bytes([protocol]) + (original[1:5] if protocol >= 6 else b'') + original[5:]
                with self.subTest(flags=original[5], protocol=protocol):
                    parsed = stager.parse_reference(raw)
                    self.assertEqual(stager.encode_reference(parsed), raw)
                    _, observed = independent_reference(box(raw))
                    group = parsed['groups'][3]
                    if original[5] == 4:
                        self.assertEqual(group['lengths'], (9, 4))
                        self.assertEqual(group['name'], 0)
                        self.assertEqual(group['files'], {0: 0})
                        self.assertNotIn('name', observed[3])
                    else:
                        self.assertEqual(group['lengths'], (17, 12))
                        self.assertEqual(group['name'], 0xaabbccdd)
                        self.assertEqual(group['files'], {0: 0x01020304, 1: 0x05060708})
                    self.assertEqual(observed[3]['lengths'], group['lengths'])

    def test_malformed_length_pairs_and_reference_resource_bounds(self):
        literal = bytes.fromhex('06 00000001 04 0001 0003 11223344 00000009 00000004 00000007 0001 0000')
        for cut in range(len(literal)):
            with self.subTest(cut=cut), self.assertRaises(ValueError):
                stager.parse_reference(literal[:cut])
        with self.assertRaisesRegex(ValueError, 'trailing'):
            stager.parse_reference(literal + b'x')
        for pair in (None, (9,), (9, 4, 0), (-1, 4), (9, 0x100000000), (True, 4)):
            parsed = stager.parse_reference(literal)
            parsed['groups'][3]['lengths'] = pair
            with self.subTest(pair=pair), self.assertRaises(ValueError):
                stager.encode_reference(parsed)
        with self.assertRaisesRegex(ValueError, 'reference bound'):
            stager.parse_reference(bytes(stager.MAX_REFERENCE + 1))
        parsed = stager.parse_reference(literal)
        with mock.patch.object(stager, 'MAX_REFERENCE', 16):
            with self.assertRaisesRegex(ValueError, 'encoded reference bound'):
                stager.encode_reference(parsed)
        with mock.patch.object(stager, 'MAX_REFERENCE_ENTRIES', 1):
            with self.assertRaisesRegex(ValueError, 'file count bound'):
                stager.parse_reference(literal)

    def test_reference_limit_precedes_decompression(self):
        cache = object.__new__(stager.Cache)
        # Correct stored-length framing, oversized expanded declaration: the
        # decompressor must never run just to discover the reference limit.
        cache.raw = mock.Mock(return_value=b'\2' + struct.pack('>II', 1, stager.MAX_REFERENCE + 1) + b'x')
        with mock.patch.object(stager.zlib, 'decompressobj', side_effect=AssertionError('must not decompress')):
            with self.assertRaisesRegex(ValueError, 'decoded length bound'):
                cache.reference(7)
        with self.assertRaisesRegex(ValueError, 'decoded limit'):
            stager.decode_container(box(b'x'), stager.MAX_DECODED + 1)

    def test_combined_reference_entries_above_old_200k_limit(self):
        # Original reference data only: four groups with independent ascending
        # 50001-file tables. No sector cache or producer encoder constructs it.
        groups = {gid: dict(body=box(b'x'), files=range(50001)) for gid in range(4)}
        raw = ref_bytes(groups, protocol=7, lengths=True)
        self.assertLess(len(raw), 2 << 20)
        parsed = stager.parse_reference(raw)
        self.assertEqual(sum(1 + len(g['files']) for g in parsed['groups'].values()), 200008)
        self.assertEqual(parsed['groups'][3]['files'][50000], 0)
        self.assertEqual(stager.encode_reference(parsed), raw)

    def test_length_flags_stage_preserves_compressed_metadata_and_full4_output(self):
        for flags in (4, 5):
            groups = {a: {gid: dict(g) for gid, g in entries.items()} for a, entries in self.groups.items()}
            original = b'UNRELATED-MODEL'
            compressed = gzip.compress(original, mtime=0)
            compressed_body = b'\2' + struct.pack('>II', len(compressed), len(original)) + compressed
            groups[7][3].update(body=compressed_body, lengths=(len(compressed_body), len(original)))
            self.game, self.js5 = self.root / f'game-{flags}', self.root / f'js5-{flags}'
            for root in (self.game, self.js5):
                fixture_cache(root, groups, reference_flags={a: flags for a in groups})
            before = {str(p): p.read_bytes() for root in (self.game, self.js5) for p in root.iterdir()}
            self.output = self.root / f'stage-{flags}'
            result = self.run_stage()
            for child in ('game', 'js5', 'client'):
                root = self.output / child
                for archive, changed in ((0, [22]), (1, [21]), (2, [6, 12]), (7, [20])):
                    reference = independent_raw(root, 255, archive)
                    self.assertEqual(reference[10], flags)
                    version, entries = independent_reference(reference)
                    for gid in changed:
                        raw = independent_raw(root, archive, gid)
                        body = raw[:-4]
                        meta = entries[gid]
                        self.assertEqual(int.from_bytes(raw[-4:], 'big'), meta['version'])
                        self.assertEqual(zlib.crc32(body) & 0xffffffff, meta['crc'])
                        self.assertEqual(meta['lengths'], (len(body), len(body) - 5))
                        self.assertNotEqual(meta['lengths'][0], len(raw))
                    receipt = next(c for c in result['copies'] if c['name'] == child)
                    self.assertEqual(receipt['master_pairs'][archive], [zlib.crc32(reference) & 0xffffffff, version])
                _, models = independent_reference(independent_raw(root, 255, 7))
                self.assertEqual(models[3]['lengths'], (len(compressed_body), len(original)))
                self.assertEqual(independent_raw(root, 7, 3), independent_raw(self.game, 7, 3))
                _, definitions = independent_reference(independent_raw(root, 255, 2))
                self.assertEqual(independent_files(independent_raw(root, 2, 6), definitions[6]),
                                 {2: b'LONG-LOCATION', 8: b'L', 24: (self.inputs/'location.bin').read_bytes()})
                if flags == 5:
                    self.assertEqual(definitions[6]['files'][2], 1002)
                    self.assertEqual(definitions[6]['name'], 123)
                else:
                    self.assertNotIn('name', definitions[6])
                    self.assertEqual(definitions[6]['files'][2], 0)
            self.assertTrue(all(Path(path).read_bytes() == raw for path, raw in before.items()))

    def test_source_size_mismatch_and_high_legacy_revision_still_refuse(self):
        for mode in ('length-first', 'length-second', 'revision'):
            groups = {a: {gid: dict(g) for gid, g in entries.items()} for a, entries in self.groups.items()}
            if mode.startswith('length'):
                body = groups[2][6]['body']
                groups[2][6]['lengths'] = (len(body) + (mode == 'length-first'),
                                          len(body) - 5 + (mode == 'length-second'))
            else:
                groups[2][6].update(version=65537, width=2, disk_version=1)
            self.game = self.root / ('bad-' + mode)
            fixture_cache(self.game, groups, reference_flags={a: 4 for a in groups})
            with self.assertRaisesRegex(ValueError, 'length pair mismatch' if mode.startswith('length') else 'CRC/full revision'):
                self.plan()
            self.assertFalse(self.output.exists())

    def test_bounded_gzip_bzip2_lengths_trailing_and_bomb(self):
        payload = b'original-synthetic-' * 40
        for kind, packed in ((2, gzip.compress(payload, mtime=0)), (1, bz2.compress(payload, compresslevel=1)),
                             (1, bz2.compress(payload, compresslevel=1)[4:])):
            def wrapped(data, expected=len(payload)):
                return bytes([kind]) + struct.pack('>II', len(data), expected) + data
            self.assertEqual(stager.decode_container(wrapped(packed)), payload)
            for data, size in ((packed + b'trailing', len(payload)), (packed, len(payload) - 1),
                               (packed + packed, len(payload)),
                               (packed, len(payload) + 1), (packed[:-2], len(payload)),
                               (packed, stager.MAX_DECODED + 1)):
                with self.assertRaises((ValueError, EOFError, OSError, zlib.error)):
                    stager.decode_container(wrapped(data, size))
        bomb = gzip.compress(b'Z' * 100000, mtime=0)
        with self.assertRaises(ValueError):
            stager.decode_container(b'\2' + struct.pack('>II', len(bomb), 2) + bomb)
        for raw in (b'\0\xff\xff\xff\xff', b'\3\0\0\0\0', box(b'x') + b'x'):
            with self.assertRaises(ValueError):
                stager.decode_container(raw)

    def test_signed_chunk_tables_and_malformed_coverage(self):
        self.assertEqual(stager.split_files(self.loc, [2, 8]), {2: b'LONG-LOCATION', 8: b'L'})
        two_chunks = b'abXcYYY' + struct.pack('>iiii', 2, -1, 1, 2) + b'\2'
        self.assertEqual(stager.split_files(two_chunks, [4, 7]), {4: b'abc', 7: b'XYYY'})
        for raw in (b'', b'abc\0', b'x' + struct.pack('>ii', 2, -3) + b'\1',
                    b'x' + struct.pack('>ii', 0x7FFFFFFF, 0) + b'\1'):
            with self.assertRaises(ValueError):
                stager.split_files(raw, [1, 2])

    def test_sector_boundary_handcrafted_and_append_reader(self):
        root = self.root / 'boundary'
        for gid in (65535, 65536):
            for size in (510, 511, 512, 513, 1024):
                target = root.with_name('boundary-' + str(gid) + '-' + str(size))
                payload = bytes((i * 7 + 3) % 256 for i in range(size))
                fixture_cache(target, {7: {gid: dict(body=box(payload), files=[0])}}, {7: 7})
                expected = box(payload) + b'\0\0\0\7'
                self.assertEqual(stager.Cache(target).raw(7, gid), expected)
                # Independently inspect writer output at the exact header boundary.
                stager.append_group(target, 7, gid, expected)
                self.assertEqual(independent_raw(target, 7, gid), expected)

    def test_corrupt_sector_headers_pointer_cycle_and_truncation(self):
        fixture = self.root / 'broken'; fixture_cache(fixture, {7: {3: dict(body=box(b'x' * 800), files=[0])}})
        datafile = fixture / 'main_file_cache.dat2'; original = datafile.read_bytes()
        index = (fixture / 'main_file_cache.idx7').read_bytes()
        start = int.from_bytes(index[21:24], 'big') * 520
        for offset in (0, 2, 7):
            bad = bytearray(original); bad[start + offset] ^= 1; datafile.write_bytes(bad)
            with self.assertRaises(ValueError):
                stager.Cache(fixture).raw(7, 3)
        bad = bytearray(original); bad[start + 4:start + 7] = (start // 520).to_bytes(3, 'big'); datafile.write_bytes(bad)
        with self.assertRaisesRegex(ValueError, 'cycle'):
            stager.Cache(fixture).raw(7, 3)
        datafile.write_bytes(original[:start + 12])
        with self.assertRaisesRegex(ValueError, 'outside'):
            stager.Cache(fixture).raw(7, 3)

    def test_staged_bytes_deterministic_across_fresh_outputs(self):
        self.run_stage()
        first = {str(p.relative_to(self.output)): p.read_bytes()
                 for p in self.output.rglob('main_file_cache.*')}
        self.output = self.root / 'second-stage'
        self.run_stage()
        second = {str(p.relative_to(self.output)): p.read_bytes()
                  for p in self.output.rglob('main_file_cache.*')}
        self.assertEqual(first, second)

    def test_protocol5_and7_stage_with_extended_unrelated_group(self):
        for protocol in (5, 7):
            game, js5 = self.root / ('p-game' + str(protocol)), self.root / ('p-js5' + str(protocol))
            groups = {a: dict(g) for a, g in self.groups.items()}
            if protocol == 7:
                groups[7][65536] = dict(body=box(b'ORIGINAL-EXTENDED'), files=[0])
            fixture_cache(game, groups, dict.fromkeys(groups, protocol))
            fixture_cache(js5, groups, dict.fromkeys(groups, protocol))
            self.game, self.js5, self.output = game, js5, self.root / ('p-out' + str(protocol))
            self.run_stage()
            for a in (0, 1, 2, 7):
                raw = independent_raw(self.output / 'client', 255, a)
                version, table = independent_reference(raw)
                self.assertEqual(version, 0 if protocol == 5 else 11)
                if a == 7 and protocol == 7:
                    self.assertIn(65536, table)
                    self.assertEqual(independent_raw(self.output / 'client', 7, 65536),
                                     independent_raw(game, 7, 65536))

    def test_source_config_crc_tamper_rejected_before_output(self):
        index = (self.game / 'main_file_cache.idx2').read_bytes()
        sector = int.from_bytes(index[39:42], 'big')
        p = self.game / 'main_file_cache.dat2'
        raw = bytearray(p.read_bytes()); raw[sector * 520 + 8 + 5] ^= 1; p.write_bytes(raw)
        with self.assertRaisesRegex(ValueError, 'CRC'):
            self.plan()
        self.assertFalse(self.output.exists())

    def test_whole_container_input_bound_checked_before_decode(self):
        packed = gzip.compress(b'x' * 20, mtime=0)
        with mock.patch.object(stager, 'MAX_STORED', 10):
            with self.assertRaisesRegex(ValueError, 'stored container bound'):
                stager.decode_container(b'\2' + struct.pack('>II', len(packed), 20) + packed)

    def test_valid_partial_last_sector_preserves_prefix(self):
        p = self.game / 'main_file_cache.idx255'
        entry = p.read_bytes()[9 * 6:10 * 6]
        size, sector = int.from_bytes(entry[:3], 'big'), int.from_bytes(entry[3:], 'big')
        dat = self.game / 'main_file_cache.dat2'
        original = dat.read_bytes()[:sector * 520 + 8 + size]
        self.assertNotEqual(len(original) % 520, 0)
        dat.write_bytes(original)
        self.run_stage()
        self.assertEqual((self.output / 'game' / dat.name).read_bytes()[:len(original)], original)

    def test_source_mutation_during_copy_retains_partial(self):
        original_append = stager.append_group
        changed = False
        def append_and_mutate(*args):
            nonlocal changed
            original_append(*args)
            if not changed:
                changed = True
                p = self.js5 / 'main_file_cache.dat2'
                p.write_bytes(p.read_bytes() + b'changed')
        with mock.patch.object(stager, 'append_group', side_effect=append_and_mutate):
            with self.assertRaisesRegex(ValueError, 'source metadata changed'):
                self.run_stage()
        self.assertTrue((self.output / 'partial.json').exists())
        self.assertFalse((self.output / 'complete.json').exists())

    def test_completion_write_failure_never_publishes_complete(self):
        write = stager.write_json_new
        def fail_pending(path, data):
            if path.name == 'completion-ready.json':
                path.write_bytes(b'partial receipt')
                raise OSError('injected receipt write failure')
            write(path, data)
        with mock.patch.object(stager, 'write_json_new', side_effect=fail_pending):
            with self.assertRaisesRegex(OSError, 'receipt write failure'):
                self.run_stage()
        self.assertTrue((self.output / 'completion-ready.json').exists())
        self.assertFalse((self.output / 'complete.json').exists())

    def test_actual_reference_and_group_version_overflow_stop_before_write(self):
        for mode in ('reference', 'group'):
            root = self.root / ('overflow-' + mode)
            groups = {a: {gid: dict(g) for gid, g in entries.items()} for a, entries in self.groups.items()}
            if mode == 'group':
                groups[2][6]['version'] = 0xFFFFFFFF
            fixture_cache(root, groups, ref_version=0xFFFFFFFF if mode == 'reference' else 10)
            self.game = root
            with self.assertRaisesRegex(ValueError, 'version overflow'):
                self.plan()
            self.assertFalse(self.output.exists())

    def test_final_source_hash_catches_same_size_restored_mtime(self):
        original_append = stager.append_group
        changed = False
        def append_and_mutate(root, *args):
            nonlocal changed
            original_append(root, *args)
            if root.name == 'js5' and not changed:
                changed = True
                p = self.game / 'main_file_cache.dat2'
                info = p.stat(); raw = bytearray(p.read_bytes()); raw[0] ^= 1; p.write_bytes(raw)
                os.utime(p, ns=(info.st_atime_ns, info.st_mtime_ns))
        with mock.patch.object(stager, 'append_group', side_effect=append_and_mutate):
            with self.assertRaisesRegex(ValueError, 'source SHA changed'):
                self.run_stage()
        self.assertFalse((self.output / 'complete.json').exists())

    def test_missing_reference_never_drops_unrelated_indexed_groups(self):
        # Existing unrelated model3 must not disappear behind a newly made table.
        p = self.js5 / 'main_file_cache.idx255'
        raw = bytearray(p.read_bytes()); raw[7 * 6:8 * 6] = bytes(6); p.write_bytes(raw)
        before = {p.name: p.read_bytes() for p in self.js5.iterdir()}
        with self.assertRaisesRegex(ValueError, 'missing reference with existing'):
            self.plan()
        self.assertFalse(self.output.exists())
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.js5.iterdir()})
        # An absent reference is safe only for an actually empty/zero archive index.
        (self.js5 / 'main_file_cache.idx7').write_bytes(bytes(24))
        self.assertEqual(stager.Cache(self.js5).reference(7)['groups'], {})

    def test_model_server_transport_boundary(self):
        self.ns['model'] = 65535; self.sync_location()
        plan = self.plan()
        self.assertEqual(plan['doc']['namespace']['model'], 65535)
        self.assertTrue(stager.summary(plan)['authentic_plan_requires_read_collision_grant'])
        self.assertFalse(stager.summary(plan)['authored_package_namespace_checked'])
        self.ns['model'] = 65536; self.sync_location()
        with self.assertRaisesRegex(ValueError, 'model out of bounds'):
            self.plan()
        self.assertFalse(self.output.exists())

    def test_fsync_barrier_precedes_completion_publication(self):
        synced = set()
        original_fsync, original_rename = os.fsync, os.rename
        def observe_sync(fd):
            synced.add(os.fstat(fd).st_ino)
            return original_fsync(fd)
        def observe_publish(source, target):
            expected = {p.stat().st_ino for child in ('game', 'js5', 'client')
                        for p in (self.output / child).iterdir()}
            self.assertTrue(expected <= synced)
            self.assertTrue(Path(source).read_bytes().endswith(b'\n'))
            return original_rename(source, target)
        with mock.patch.object(stager.os, 'fsync', side_effect=observe_sync), \
                mock.patch.object(stager.os, 'rename', side_effect=observe_publish):
            result = self.run_stage()
        self.assertTrue(result['cache_files_fsynced_before_completion'])

    def test_fsync_failure_has_no_completion(self):
        with mock.patch.object(stager, 'sync_staged_files_and_directories', side_effect=OSError('sync failure')):
            with self.assertRaisesRegex(OSError, 'sync failure'):
                self.run_stage()
        self.assertTrue((self.output / 'completion-ready.json').exists())
        self.assertFalse((self.output / 'complete.json').exists())

    def test_literal_named_reference_and_chunk_group(self):
        # Literal protocol6/version10/named/group6/name12345678/CRCabcdef01/
        # revision7/files2,8/name11111111,22222222. No fixture builder used.
        raw = bytes.fromhex('00 00000024 06 0000000a 01 0001 0006 12345678 '
                            'abcdef01 00000007 0002 0002 0006 11111111 22222222')
        expected = {6: dict(name=0x12345678, crc=0xABCDEF01, version=7,
                           files={2: 0x11111111, 8: 0x22222222})}
        self.assertEqual(independent_reference(raw), (10, expected))
        self.assertEqual(stager.parse_reference(raw[5:])['groups'], expected)
        chunked = bytes.fromhex('41 42 43 78 00000003 fffffffe 01')
        self.assertEqual(stager.split_files(chunked, [2, 8]), {2: b'ABC', 8: b'x'})
        body = bytes.fromhex('00 0000000d 41 42 43 78 00000003 fffffffe 01')
        meta = dict(crc=zlib.crc32(body) & 0xFFFFFFFF, version=7, files={2: 0, 8: 0})
        self.assertEqual(independent_files(body + bytes.fromhex('00000007'), meta), {2: b'ABC', 8: b'x'})

    def test_literal_two_sector_boundary_fixtures(self):
        # 512 literal bytes of 0x61; header fields and idx entries below are hex
        # literals, not calculated with either producer or fixture packing code.
        a512 = bytes.fromhex('''
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
6161616161616161616161616161616161616161616161616161616161616161
''')
        self.assertEqual(len(a512), 512)
        for gid, first, last, entry, payload in (
                (65535, 'ffff000000000207', 'ffff000100000007', '000201000001', a512),
                (65536, '00010000000000000207', '00010000000100000007', '0001ff000001', a512[:-2])):
            root = self.root / ('literal-' + str(gid)); root.mkdir()
            (root / 'main_file_cache.dat2').write_bytes(bytes(520) + bytes.fromhex(first) + payload + bytes.fromhex(last) + b'Z')
            (root / 'main_file_cache.idx7').write_bytes(bytes(gid * 6) + bytes.fromhex(entry))
            self.assertEqual(stager.Cache(root).raw(7, gid), payload + b'Z')
            self.assertEqual(independent_raw(root, 7, gid), payload + b'Z')

    def test_chunk_overflow_and_u24_append_refusals_without_large_files(self):
        root = self.root / 'append-bounds'; root.mkdir()
        data = root / 'main_file_cache.dat2'; data.write_bytes(bytes(520))
        class HugeLengthOnly:
            def __len__(self):
                return 512 * 65536 + 1
            def __getitem__(self, _):
                raise AssertionError('must refuse before slicing/allocating payload')
        # Current u24 stored-size cap already prevents this; isolate the second
        # chunk-count defense without allocating 32MiB or creating a large file.
        with mock.patch.object(stager, 'MAX_STORED', 64 * 1024**2):
            with self.assertRaisesRegex(ValueError, 'sector/u24 bound'):
                stager.append_group(root, 7, 65535, HugeLengthOnly())
        for size in (0xFFFFFF * 520, 0x1000000 * 520):
            with mock.patch.object(Path, 'stat', return_value=mock.Mock(st_size=size)):
                with self.assertRaisesRegex(ValueError, 'sector/u24 bound'):
                    stager.append_group(root, 7, 65535, b'x' * 513)
        self.assertEqual(data.read_bytes(), bytes(520))
        self.assertFalse((root / 'main_file_cache.idx7').exists())


def stat_regular():
    import stat
    return stat.S_IFREG | 0o600


if __name__ == '__main__':
    unittest.main()
